mod auth;
mod circuit_breaker;
mod config;
mod discovery;
mod grpc;
mod handlers;
mod health;
mod rate_limit;
mod router;
mod shared;


use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::RandomIdGenerator;
use opentelemetry_sdk::Resource;
use prometheus::{Encoder, TextEncoder};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::middleware::auth_middleware;
use crate::auth::AuthService;
use crate::config::GatewayConfig;
use crate::discovery::RouteDiscoveryService;
use crate::grpc::client::GrpcClientPool;
use crate::handlers::gateway::gateway_handler;
use crate::handlers::refresh_routes_handler;
use crate::health::{health_handler, HealthChecker};
use crate::rate_limit::RateLimiter;
use crate::router::RequestRouter;
use crate::shared::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load gateway configuration
    let config_path = std::env::var("GATEWAY_CONFIG_PATH")
        .unwrap_or_else(|_| "config/gateway-config.yaml".to_string());
    
    info!(config_path = %config_path, "Loading gateway configuration");
    
    let config = match GatewayConfig::load_config(&config_path) {
        Ok(cfg) => {
            info!("Gateway configuration loaded successfully");
            cfg
        }
        Err(e) => {
            error!(error = %e, "Failed to load gateway configuration");
            return Err(anyhow::anyhow!("Configuration error: {}", e));
        }
    };

    // Initialize OpenTelemetry with Tempo
    let tempo_endpoint = config.observability.tempo_endpoint.clone();
    let service_name = config.observability.service_name.clone();

    info!(
        tempo_endpoint = %tempo_endpoint,
        service_name = %service_name,
        "Initializing observability"
    );

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&tempo_endpoint)
        .with_timeout(std::time::Duration::from_secs(config.observability.otlp_timeout_secs))
        .build()?;

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(config.observability.max_events_per_span)
        .with_max_attributes_per_span(config.observability.max_attributes_per_span)
        .with_resource(
            Resource::builder_empty()
                .with_attributes([KeyValue::new("service.name", service_name.clone())])
                .build(),
        )
        .build();

    // Create JSON formatter for Loki compatibility
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let tracer = tracer_provider.tracer(service_name.clone());
    global::set_tracer_provider(tracer_provider);

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_gateway=debug".into()),
        )
        .init();

    // Log startup message
    info!(
        version = env!("CARGO_PKG_VERSION"),
        service = %service_name,
        "Starting API Gateway"
    );

    // Initialize gRPC client pool
    info!("Initializing gRPC client pool");
    let grpc_pool = match GrpcClientPool::new(config.services.clone()).await {
        Ok(pool) => {
            info!(services = config.services.len(), "gRPC client pool initialized");
            pool
        }
        Err(e) => {
            error!(error = %e, "Failed to initialize gRPC client pool");
            return Err(anyhow::anyhow!("gRPC pool initialization failed: {}", e));
        }
    };

    // Initialize authorization service
    info!("Initializing authorization service");
    let auth_service = match AuthService::new(config.auth.clone()).await {
        Ok(service) => {
            info!("Authorization service initialized");
            service
        }
        Err(e) => {
            error!(error = %e, "Failed to initialize authorization service");
            return Err(anyhow::anyhow!("Auth service initialization failed: {}", e));
        }
    };

    // Initialize route discovery service (if enabled)
    let (router_lock, discovery_service) = if config.discovery.enabled {
        info!("Route discovery is enabled, initializing discovery service");

        // Create discovery service
        let discovery_service = RouteDiscoveryService::new(
            Arc::new(grpc_pool.clone()),
            config.discovery.clone(),
        );

        // Perform initial route discovery at startup
        info!("Performing initial route discovery from all services");
        let discovered_routes = match discovery_service.discover_routes(&config.services).await {
            Ok(routes) => {
                info!(
                    routes = routes.len(),
                    "Initial route discovery completed successfully"
                );
                routes
            }
            Err(e) => {
                error!(error = %e, "Failed to perform initial route discovery");
                warn!("Starting gateway with empty routing table");
                Vec::new()
            }
        };

        // Apply route overrides
        let final_routes = discovery_service.apply_overrides(
            discovered_routes,
            &config.route_overrides,
        );

        info!(
            total_routes = final_routes.len(),
            "Routes ready (including overrides)"
        );

        // Create router wrapped in Arc<RwLock<>> for thread-safe updates
        let router_lock = Arc::new(RwLock::new(RequestRouter::new(final_routes)));

        (router_lock, Some(discovery_service))
    } else {
        info!("Route discovery is disabled, using empty routing table");
        warn!("Gateway will not expose any routes without discovery enabled");

        // Still use RwLock for consistency, even without dynamic updates
        let router_lock = Arc::new(RwLock::new(RequestRouter::new(Vec::new())));
        (router_lock, None)
    };

    // Initialize rate limiter (if enabled)
    let rate_limiter = if let Some(rate_limit_config) = &config.rate_limit {
        if rate_limit_config.enabled {
            info!(
                requests_per_minute = rate_limit_config.requests_per_minute,
                "Initializing rate limiter"
            );
            Some(RateLimiter::new(rate_limit_config.clone()))
        } else {
            info!("Rate limiting disabled");
            None
        }
    } else {
        info!("Rate limiting not configured");
        None
    };

    // Start rate limiter cleanup task to prevent memory leak
    let cleanup_task_handle = if let Some(ref limiter) = rate_limiter {
        info!("Starting rate limiter cleanup background task");
        let limiter_clone = limiter.clone();
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                limiter_clone.cleanup_expired().await;
                debug!("Rate limiter cleanup completed");
            }
        }))
    } else {
        None
    };

    // Create application state
    info!("Creating application state");
    let shared_state = Arc::new(AppState::new(
        config.clone(),
        grpc_pool.clone(),
        auth_service,
        router_lock.clone(),
        rate_limiter,
        discovery_service,
    ));

    // Initialize health checker
    info!("Initializing health checker");
    let health_checker = Arc::new(HealthChecker::new(shared_state.grpc_pool.clone()));

    // Create auth middleware state
    let auth_middleware_state = auth::middleware::AuthMiddlewareState {
        auth_service: shared_state.auth_service.clone(),
        grpc_pool: shared_state.grpc_pool.clone(),
        router_lock: shared_state.router_lock.clone(),
    };

    // Build Axum router
    info!("Building HTTP router");
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_handler))
        .with_state(health_checker)
        // Metrics endpoint
        .route("/metrics", get(metrics))
        .with_state(shared_state.clone())
        // Admin endpoint for manual route refresh (requires auth)
        .route(
            "/admin/refresh-routes",
            post(refresh_routes_handler)
                .layer(axum::middleware::from_fn_with_state(
                    auth_middleware_state.clone(),
                    auth_middleware,
                )),
        )
        .with_state(shared_state.clone())
        // Gateway handler for all other routes with auth middleware
        .route(
            "/*path",
            any(gateway_handler)
                .layer(axum::middleware::from_fn_with_state(
                    auth_middleware_state,
                    auth_middleware,
                )),
        )
        .with_state(shared_state.clone())
        // Add tracing layer
        .layer(TraceLayer::new_for_http());

    // Start periodic refresh background task (if discovery is enabled)
    let refresh_task_handle = if let (Some(discovery_svc), Some(router_lck)) = 
        (shared_state.discovery_service.clone(), shared_state.router_lock.clone()) 
    {
        info!("Starting periodic refresh background task");
        let services = config.services.clone();
        Some(discovery_svc.start_refresh_task(router_lck, services))
    } else {
        None
    };

    // Bind to address
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));
    
    info!(address = %addr, "Starting HTTP server");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Set up graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    );

    let server_handle = tokio::spawn(async move {
        server
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
            .unwrap();
    });

    info!("API Gateway is ready to accept requests");

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal (Ctrl+C)");
        }
    }

    info!("Initiating graceful shutdown");

    // Send shutdown signal to server
    let _ = tx.send(());

    // Abort periodic refresh task if running
    if let Some(handle) = refresh_task_handle {
        info!("Stopping periodic route refresh task");
        handle.abort();
    }

    // Abort rate limiter cleanup task if running
    if let Some(handle) = cleanup_task_handle {
        info!("Stopping rate limiter cleanup task");
        handle.abort();
    }

    // Wait for server to finish
    let _ = server_handle.await;

    info!("API Gateway shutdown complete");

    Ok(())
}
/// Metrics endpoint handler
#[tracing::instrument(skip(state))]
async fn metrics(State(state): State<Arc<AppState>>) -> Response {
    let metric_families = state.registry.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    
    let (status, body) = match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => match String::from_utf8(buffer) {
            Ok(s) => (StatusCode::OK, s),
            Err(e) => {
                error!(error = %e, "Metrics not UTF-8");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Metrics not UTF-8: {}", e),
                )
            }
        },
        Err(e) => {
            error!(error = %e, "Failed to encode metrics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode metrics: {}", e),
            )
        }
    };
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/plain; version=0.0.4"),
    );
    
    (status, headers, body).into_response()
}
