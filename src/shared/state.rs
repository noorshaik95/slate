use prometheus::{HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, Registry};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::AuthService;
use crate::config::GatewayConfig;
use crate::discovery::RouteDiscoveryService;
use crate::grpc::client::GrpcClientPool;
use crate::rate_limit::RateLimiter;
use crate::router::RequestRouter;

#[derive(Clone)]
pub struct AppState {
    pub config: GatewayConfig,
    pub grpc_pool: Arc<GrpcClientPool>,
    pub auth_service: Arc<AuthService>,
    pub router_lock: Arc<RwLock<RequestRouter>>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
    pub registry: Registry,
    pub metrics: GatewayMetrics,
    // For dynamic route updates (used by admin endpoint and periodic refresh)
    pub discovery_service: Option<Arc<RouteDiscoveryService>>,
}

#[derive(Clone)]
pub struct GatewayMetrics {
    pub request_counter: IntCounterVec,
    pub request_duration: HistogramVec,
    pub grpc_call_counter: IntCounterVec,
    pub auth_failure_counter: IntCounter,
    pub rate_limit_counter: IntCounter,
}

impl GatewayMetrics {
    pub fn new(registry: &Registry) -> Self {
        let request_counter = IntCounterVec::new(
            Opts::new("gateway_requests_total", "Total number of requests processed by the gateway")
                .namespace("api_gateway"),
            &["route", "method", "status"],
        )
        .unwrap();

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "gateway_request_duration_seconds",
                "Request duration in seconds",
            )
            .namespace("api_gateway")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["route", "method"],
        )
        .unwrap();

        let grpc_call_counter = IntCounterVec::new(
            Opts::new("gateway_grpc_calls_total", "Total number of gRPC calls to backend services")
                .namespace("api_gateway"),
            &["service", "method", "status"],
        )
        .unwrap();

        let auth_failure_counter = IntCounter::new(
            "gateway_auth_failures_total",
            "Total number of authentication failures",
        )
        .unwrap();

        let rate_limit_counter = IntCounter::new(
            "gateway_rate_limit_exceeded_total",
            "Total number of requests rejected due to rate limiting",
        )
        .unwrap();

        registry.register(Box::new(request_counter.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(grpc_call_counter.clone())).unwrap();
        registry.register(Box::new(auth_failure_counter.clone())).unwrap();
        registry.register(Box::new(rate_limit_counter.clone())).unwrap();

        GatewayMetrics {
            request_counter,
            request_duration,
            grpc_call_counter,
            auth_failure_counter,
            rate_limit_counter,
        }
    }
}

impl AppState {
    /// Create AppState with optional discovery service support
    pub fn new(
        config: GatewayConfig,
        grpc_pool: GrpcClientPool,
        auth_service: AuthService,
        router_lock: Arc<RwLock<RequestRouter>>,
        rate_limiter: Option<RateLimiter>,
        discovery_service: Option<RouteDiscoveryService>,
    ) -> Self {
        let registry = Registry::new();
        let metrics = GatewayMetrics::new(&registry);

        AppState {
            config,
            grpc_pool: Arc::new(grpc_pool),
            auth_service: Arc::new(auth_service),
            router_lock,
            rate_limiter: rate_limiter.map(Arc::new),
            registry,
            metrics,
            discovery_service: discovery_service.map(Arc::new),
        }
    }
}