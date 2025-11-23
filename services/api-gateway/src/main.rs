mod auth;
mod config;
mod discovery;
mod docs;
mod grpc;
mod handlers;
mod health;
mod middleware;
mod observability;
mod proto;
mod router;
mod security;
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
use crate::health::{health_handler, liveness_handler, readiness_handler, HealthChecker};
use crate::middleware::{body_limit_middleware, BodyLimitConfig};
use crate::router::RequestRouter;
use crate::shared::state::AppState;
use common_rust::rate_limit::IpRateLimiter;

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
        .with_timeout(std::time::Duration::from_secs(
            config.observability.otlp_timeout_secs,
        ))
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

    // Create JSON formatter for Loki compatibility with flattened trace_id
    // Uses custom formatter to place trace_id at root level instead of nested in fields
    let fmt_layer =
        tracing_subscriber::fmt::layer().event_format(observability::FlattenedJsonFormat::new());

    let tracer = tracer_provider.tracer(service_name.clone());
    global::set_tracer_provider(tracer_provider);

    // Set global propagator to W3C Trace Context (CRITICAL for trace propagation!)
    use opentelemetry::propagation::TextMapPropagator;
    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    global::set_text_map_propagator(propagator);

    info!("OpenTelemetry propagator configured: W3C Trace Context");

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Configure logging level from RUST_LOG environment variable
    // Default to "info" for all modules (production-friendly)
    // Supports per-module configuration: RUST_LOG="info,api_gateway=debug,tower_http=warn"
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(&log_level))
        .unwrap_or_else(|e| {
            eprintln!(
                "Invalid RUST_LOG value '{}': {}. Using default 'info'",
                log_level, e
            );
            tracing_subscriber::EnvFilter::new("info")
        });

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .with(env_filter)
        .init();

    // Log startup message with active log level
    info!(
        version = env!("CARGO_PKG_VERSION"),
        service = %service_name,
        log_level = %log_level,
        "Starting API Gateway"
    );

    // Initialize gRPC client pool
    info!("Initializing gRPC client pool");
    let grpc_pool = match GrpcClientPool::new(config.services.clone()).await {
        Ok(pool) => {
            info!(
                services = config.services.len(),
                "gRPC client pool initialized"
            );
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
        let discovery_service =
            RouteDiscoveryService::new(Arc::new(grpc_pool.clone()), config.discovery.clone());

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
        let final_routes =
            discovery_service.apply_overrides(discovered_routes, &config.route_overrides);

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
            Some(IpRateLimiter::new(rate_limit_config.clone(), 10_000))
        } else {
            info!("Rate limiting disabled");
            None
        }
    } else {
        info!("Rate limiting not configured");
        None
    };

    // Initialize body size limit configuration
    info!("Configuring request body size limits");
    let default_body_limit = std::env::var("MAX_REQUEST_BODY_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1024 * 1024); // Default: 1MB

    let upload_body_limit = std::env::var("MAX_UPLOAD_BODY_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10 * 1024 * 1024); // Default: 10MB

    let upload_paths = std::env::var("UPLOAD_PATHS")
        .ok()
        .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
        .unwrap_or_else(|| vec!["/upload".to_string(), "/api/upload".to_string()]);

    let body_limit_config =
        BodyLimitConfig::new(default_body_limit, upload_body_limit, upload_paths.clone());

    info!(
        default_limit_bytes = default_body_limit,
        upload_limit_bytes = upload_body_limit,
        upload_paths = ?upload_paths,
        "Body size limits configured"
    );

    // Initialize client IP extractor for X-Forwarded-For handling
    info!("Configuring client IP extraction");
    let trusted_proxies_str =
        std::env::var("TRUSTED_PROXIES").unwrap_or_else(|_| config.trusted_proxies.join(","));

    let trusted_proxies: Vec<std::net::IpAddr> = if !trusted_proxies_str.is_empty() {
        trusted_proxies_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                match trimmed.parse() {
                    Ok(ip) => Some(ip),
                    Err(e) => {
                        warn!(
                            proxy_ip = %trimmed,
                            error = %e,
                            "Failed to parse trusted proxy IP, skipping"
                        );
                        None
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let client_ip_config = middleware::ClientIpConfig::new(trusted_proxies.clone());
    let client_ip_extractor = middleware::ClientIpExtractor::new(client_ip_config);

    if trusted_proxies.is_empty() {
        info!("No trusted proxies configured, using direct connection IPs only");
    } else {
        info!(
            trusted_proxies = ?trusted_proxies,
            "Client IP extractor configured with trusted proxies"
        );
    }

    // Create application state
    info!("Creating application state");
    let shared_state = Arc::new(AppState::new(
        config.clone(),
        grpc_pool.clone(),
        auth_service,
        router_lock.clone(),
        rate_limiter.clone(),
        client_ip_extractor,
        discovery_service,
    ));

    // Start rate limiter cleanup task to prevent memory leak
    // Runs every 1 minute (optimized from 5 minutes) for better memory management
    let cleanup_task_handle = if let Some(ref limiter) = rate_limiter {
        info!("Starting rate limiter cleanup background task (interval: 1 minute)");
        let limiter_clone = limiter.clone();
        let metrics_clone = shared_state.metrics.clone();
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // 1 minute
            loop {
                interval.tick().await;

                // Get tracked clients count before cleanup
                let tracked_before = limiter_clone.tracked_clients_count().await;

                // Perform cleanup and get eviction count
                let evicted_count = limiter_clone.cleanup_expired().await;

                // Get tracked clients count after cleanup
                let tracked_after = limiter_clone.tracked_clients_count().await;

                // Update Prometheus metrics
                metrics_clone
                    .rate_limiter_tracked_clients
                    .set(tracked_after as i64);
                metrics_clone
                    .rate_limiter_evictions_total
                    .inc_by(evicted_count as u64);

                info!(
                    tracked_clients = tracked_after,
                    evicted_entries = evicted_count,
                    "Rate limiter cleanup completed"
                );

                debug!(
                    tracked_before = tracked_before,
                    tracked_after = tracked_after,
                    evicted = evicted_count,
                    "Rate limiter cleanup details"
                );
            }
        }))
    } else {
        None
    };

    // Initialize health checker
    info!("Initializing health checker");
    let health_checker = Arc::new(HealthChecker::new(shared_state.grpc_pool.clone()));

    // Create auth middleware state
    let public_routes: Vec<(String, String)> = config
        .auth
        .public_routes
        .iter()
        .map(|r| (r.path.clone(), r.method.clone()))
        .collect();

    let auth_middleware_state = auth::middleware::AuthMiddlewareState {
        auth_service: shared_state.auth_service.clone(),
        grpc_pool: shared_state.grpc_pool.clone(),
        router_lock: shared_state.router_lock.clone(),
        public_routes,
    };

    // Build Axum router
    info!("Building HTTP router");

    // Create health check router
    let health_router = Router::new()
        .route("/health", get(health_handler))
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        .with_state(health_checker);

    // Create metrics router
    let metrics_router = Router::new()
        .route("/metrics", get(metrics))
        .with_state(shared_state.clone());

    // Create admin router
    let admin_router = Router::new()
        .route(
            "/admin/refresh-routes",
            post(refresh_routes_handler).layer(axum::middleware::from_fn_with_state(
                auth_middleware_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(shared_state.clone());

    // Create gateway router with body limit and auth middleware
    let gateway_router = Router::new()
        .route(
            "/*path",
            any(gateway_handler)
                .layer(axum::middleware::from_fn_with_state(
                    auth_middleware_state,
                    auth_middleware,
                ))
                .layer(axum::middleware::from_fn(move |req, next| {
                    body_limit_middleware(body_limit_config.clone(), req, next)
                })),
        )
        .with_state(shared_state.clone());

    // Merge all routers
    let mut app = Router::new()
        // API Documentation endpoints (no auth required)
        .merge(docs::create_docs_router())
        .merge(health_router)
        .merge(metrics_router)
        .merge(admin_router)
        .merge(gateway_router)
        // Add tracing layer
        .layer(TraceLayer::new_for_http());

    // Add CORS middleware if enabled
    if let Some(cors_config) = &config.cors {
        if cors_config.enabled {
            info!("Configuring CORS middleware");

            // Determine dev mode from environment or config
            let dev_mode = std::env::var("DEV_MODE")
                .or_else(|_| std::env::var("ENVIRONMENT"))
                .map(|v| {
                    let lower = v.to_lowercase();
                    lower == "development" || lower == "dev" || lower == "true"
                })
                .unwrap_or(false);

            // Read allowed origins from environment or use config
            let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
                .ok()
                .map(|s| {
                    s.split(',')
                        .map(|origin| origin.trim().to_string())
                        .filter(|origin| !origin.is_empty())
                        .collect()
                })
                .unwrap_or_else(|| cors_config.allowed_origins.clone());

            // Create CORS configuration
            let cors_middleware_config =
                middleware::CorsConfig::new(allowed_origins.clone(), dev_mode);

            // Log configuration
            if dev_mode {
                warn!("CORS: Running in development mode - allowing all origins");
            } else if allowed_origins.is_empty() {
                warn!(
                    "CORS: No allowed origins configured - will reject all cross-origin requests"
                );
            } else if allowed_origins.contains(&"*".to_string()) {
                warn!("CORS: Wildcard origin configured - consider restricting in production");
            } else {
                info!(
                    allowed_origins = ?allowed_origins,
                    "CORS: Configured with specific allowed origins"
                );
            }

            // Build and apply CORS layer
            let cors_layer = cors_middleware_config.build_layer();
            app = app.layer(cors_layer);

            info!("CORS middleware configured successfully");
        } else {
            info!("CORS middleware disabled in configuration");
        }
    } else {
        info!("CORS configuration not found, CORS middleware not applied");
    }

    // Start periodic refresh background task (if discovery is enabled)
    let refresh_task_handle = if let Some(discovery_svc) = shared_state.discovery_service.clone() {
        info!("Starting periodic refresh background task");
        let services = config.services.clone();
        let router_lck = shared_state.router_lock.clone();
        let route_overrides = config.route_overrides.clone();
        Some(discovery_svc.start_refresh_task(router_lck, services, route_overrides))
    } else {
        None
    };

    // Bind to address
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    info!(address = %addr, "Starting HTTP server");
    info!("API Documentation available at: http://{}/docs", addr);
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
            .expect("Server failed to start or encountered an error during execution");
    });

    info!("API Gateway is ready to accept requests");

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal (Ctrl+C)");
        }
    }

    let shutdown_start = std::time::Instant::now();
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

    // Close all gRPC connections
    info!("Closing gRPC connection pools");
    let closed_count = grpc_pool.close().await;
    info!(closed_connections = closed_count, "gRPC connections closed");

    let total_shutdown_duration = shutdown_start.elapsed();
    info!(
        total_shutdown_duration_ms = total_shutdown_duration.as_millis(),
        "API Gateway shutdown complete"
    );

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
