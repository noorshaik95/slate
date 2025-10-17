use mongodb::sync::Database;
use prometheus::{Histogram, HistogramOpts, IntCounter, Registry};

#[derive(Clone, Debug)]
pub struct AppState {
    pub req_counter: IntCounter,
    pub registry: Registry,
    pub req_timer: Histogram,
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        let registry = Registry::new();
        let req_counter = IntCounter::new("axum_requests_total", "Total requests").unwrap();
        let req_timer = prometheus::Histogram::with_opts(
            HistogramOpts::from(prometheus::Opts::new("axum_request_duration_seconds", "Request duration in seconds"))
        ).unwrap();
        registry.register(Box::new(req_timer.clone())).unwrap();
        registry.register(Box::new(req_counter.clone())).unwrap();
        AppState {
            req_counter,
            registry,
            req_timer,
            db
        }
    }
}