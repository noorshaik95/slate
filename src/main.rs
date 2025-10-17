mod routes;
mod shared;
mod models;
mod db;

use axum::{extract::State, http::{StatusCode, HeaderMap, HeaderValue}, response::{IntoResponse, Response}, routing::get, Router};
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use prometheus::{Encoder, TextEncoder};
use std::{net::SocketAddr, sync::Arc};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{RandomIdGenerator};
use tracing::{info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;
use crate::shared::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize OpenTelemetry with Tempo
    let otlp_exporter  = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://tempo:4317")
        .with_timeout(std::time::Duration::from_secs(3))
        .build()?;
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "axum-grafana-example")]).build())
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
    let tracer = tracer_provider.tracer("axum-grafana-example");
    global::set_tracer_provider(tracer_provider);

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,axum-grafana-example=debug".into()))
        .init();

    // Log startup message
    info!(version = env!("CARGO_PKG_VERSION"), "Starting service");
    let mongodb_database = db::init().await.unwrap().database("axum-grafana-example");
    let shared_state = Arc::new(AppState::new(mongodb_database));
    let app = Router::new()
        .route("/", get(root))
        .route("/metrics", get(metrics))
        .nest("/user", routes::user::register_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!(address = %addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Set up graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = axum::serve(listener, app.into_make_service());

    tokio::spawn(async move {
        server.with_graceful_shutdown(async {
            rx.await.ok();
        }).await.unwrap();
    });

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    // Send shutdown signal to server
    let _ = tx.send(());

    Ok(())
}


#[tracing::instrument]
async fn root(State(state): State<Arc<AppState>> ) -> impl IntoResponse {
    let timer = state.req_timer.start_timer();
    state.req_counter.inc();
    info!("Handling root request");
    timer.observe_duration();
    "Hello, World!"
}





#[tracing::instrument]
async fn metrics(State(state): State<Arc<AppState>>) -> Response {
    let metric_families = state.registry.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let (status, body) = match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => match String::from_utf8(buffer) {
            Ok(s) => (StatusCode::OK, s),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Metrics not UTF-8: {}", e)),
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode metrics: {}", e)),
    };
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/plain; version=0.0.4"));
    (status, headers, body).into_response()
}
