//! Application builder and initialization.
//!
//! Coordinates the setup of all application components including configuration,
//! telemetry, server, and routing.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::auth::AuthService;
use crate::config::GatewayConfig;
use crate::discovery::RouteDiscoveryService;
use crate::grpc::client::GrpcClientPool;
use crate::middleware::{ClientIpConfig, ClientIpExtractor};
use crate::router::RequestRouter;
use crate::shared::state::AppState;
use common_rust::rate_limit::IpRateLimiter;

mod routes;
mod server;
mod telemetry;

pub use telemetry::{init_telemetry, shutdown_telemetry};

/// Run the API Gateway application.
///
/// This is the main application entry point that:
/// 1. Loads configuration
/// 2. Initializes telemetry (tracing, metrics)
/// 3. Sets up application state
/// 4. Creates and configures the HTTP server
/// 5. Starts the server and handles shutdown
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration
    let config = load_config()?;

    // Initialize telemetry first
    let _telemetry_guard = init_telemetry(&config).await?;

    info!("Starting API Gateway");

    // Create application state
    let app_state = create_app_state(config).await?;

    // Start server (this handles everything including graceful shutdown)
    server::start_server(app_state).await?;

    // Cleanup telemetry
    shutdown_telemetry().await;

    info!("API Gateway shutdown complete");
    Ok(())
}

/// Load application configuration.
fn load_config() -> Result<GatewayConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = std::env::var("GATEWAY_CONFIG_PATH")
        .unwrap_or_else(|_| "config/gateway-config.yaml".to_string());

    info!(config_path = %config_path, "Loading gateway configuration");

    let config = GatewayConfig::load_config(&config_path)
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    info!(
        server_host = %config.server.host,
        server_port = config.server.port,
        "Configuration loaded successfully"
    );

    Ok(config)
}

/// Create application state with all dependencies.
async fn create_app_state(
    config: GatewayConfig,
) -> Result<Arc<AppState>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Initializing application state");

    // Initialize gRPC client pool
    info!("Initializing gRPC client pool");
    let grpc_pool = GrpcClientPool::new(config.services.clone())
        .await
        .map_err(|e| format!("Failed to initialize gRPC client pool: {}", e))?;

    info!(
        services = config.services.len(),
        "gRPC client pool initialized"
    );

    // Initialize authorization service
    info!("Initializing authorization service");
    let auth_service = AuthService::new(config.auth.clone())
        .await
        .map_err(|e| format!("Failed to initialize authorization service: {}", e))?;

    info!("Authorization service initialized");

    // Initialize route discovery and router
    let (router_lock, discovery_service) = initialize_routing(&config, &grpc_pool).await?;

    // Initialize rate limiter if configured
    let rate_limiter = initialize_rate_limiter(&config)?;

    // Initialize client IP extractor
    let client_ip_extractor = initialize_client_ip_extractor(&config)?;

    let app_state = Arc::new(AppState::new(
        config,
        grpc_pool,
        auth_service,
        router_lock,
        rate_limiter,
        client_ip_extractor,
        discovery_service,
    ));

    info!("Application state initialized successfully");
    Ok(app_state)
}

/// Initialize routing with optional discovery service.
async fn initialize_routing(
    config: &GatewayConfig,
    grpc_pool: &GrpcClientPool,
) -> Result<
    (Arc<RwLock<RequestRouter>>, Option<RouteDiscoveryService>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    if config.discovery.enabled {
        initialize_routing_with_discovery(config, grpc_pool).await
    } else {
        initialize_routing_without_discovery()
    }
}

/// Initialize routing with discovery service enabled.
async fn initialize_routing_with_discovery(
    config: &GatewayConfig,
    grpc_pool: &GrpcClientPool,
) -> Result<
    (Arc<RwLock<RequestRouter>>, Option<RouteDiscoveryService>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    info!("Route discovery is enabled, initializing discovery service");

    let discovery_service =
        RouteDiscoveryService::new(Arc::new(grpc_pool.clone()), config.discovery.clone());

    // Perform initial route discovery
    let discovered_routes =
        perform_initial_route_discovery(&discovery_service, &config.services).await;

    // Apply route overrides
    let final_routes =
        discovery_service.apply_overrides(discovered_routes, &config.route_overrides);

    info!(
        total_routes = final_routes.len(),
        "Routes ready (including overrides)"
    );

    let router_lock = Arc::new(RwLock::new(RequestRouter::new(final_routes)));

    Ok((router_lock, Some(discovery_service)))
}

/// Type alias for routing initialization result.
type RoutingInitResult = Result<
    (Arc<RwLock<RequestRouter>>, Option<RouteDiscoveryService>),
    Box<dyn std::error::Error + Send + Sync>,
>;

/// Initialize routing without discovery service.
fn initialize_routing_without_discovery() -> RoutingInitResult {
    info!("Route discovery is disabled, using empty routing table");
    warn!("Gateway will not expose any routes without discovery enabled");

    let router_lock = Arc::new(RwLock::new(RequestRouter::new(Vec::new())));
    Ok((router_lock, None))
}

/// Perform initial route discovery from all services.
async fn perform_initial_route_discovery(
    discovery_service: &RouteDiscoveryService,
    services: &std::collections::HashMap<String, crate::config::ServiceConfig>,
) -> Vec<crate::config::RouteConfig> {
    info!("Performing initial route discovery from all services");

    match discovery_service.discover_routes(services).await {
        Ok(routes) => {
            info!(
                routes = routes.len(),
                "Initial route discovery completed successfully"
            );
            routes
        }
        Err(e) => {
            warn!(error = %e, "Failed to perform initial route discovery");
            warn!("Starting gateway with empty routing table");
            Vec::new()
        }
    }
}

/// Initialize rate limiter if configured.
fn initialize_rate_limiter(
    config: &GatewayConfig,
) -> Result<Option<IpRateLimiter>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(rate_limit_config) = &config.rate_limit {
        if rate_limit_config.enabled {
            info!(
                requests_per_minute = rate_limit_config.requests_per_minute,
                "Initializing rate limiter"
            );
            let limiter = IpRateLimiter::new(rate_limit_config.clone(), 10_000);
            return Ok(Some(limiter));
        }
    }

    info!("Rate limiting disabled");
    Ok(None)
}

/// Initialize client IP extractor.
fn initialize_client_ip_extractor(
    config: &GatewayConfig,
) -> Result<ClientIpExtractor, Box<dyn std::error::Error + Send + Sync>> {
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

    let client_ip_config = ClientIpConfig::new(trusted_proxies.clone());
    let client_ip_extractor = ClientIpExtractor::new(client_ip_config);

    if trusted_proxies.is_empty() {
        info!("No trusted proxies configured, using direct connection IPs only");
    } else {
        info!(
            trusted_proxies = ?trusted_proxies,
            "Client IP extractor configured with trusted proxies"
        );
    }

    Ok(client_ip_extractor)
}
