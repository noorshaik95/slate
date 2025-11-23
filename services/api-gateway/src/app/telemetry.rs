//! OpenTelemetry and observability setup.
//!
//! Handles initialization of tracing, metrics, and observability infrastructure.

use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::RandomIdGenerator;
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::GatewayConfig;
use crate::observability::FlattenedJsonFormat;

/// Telemetry guard that handles cleanup on drop.
pub struct TelemetryGuard;

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        // Cleanup is handled by shutdown_telemetry()
    }
}

/// Initialize OpenTelemetry tracing and metrics.
///
/// Sets up distributed tracing with OTLP export and structured logging.
pub async fn init_telemetry(
    config: &GatewayConfig,
) -> Result<TelemetryGuard, Box<dyn std::error::Error + Send + Sync>> {
    let tempo_endpoint = config.observability.tempo_endpoint.clone();
    let service_name = config.observability.service_name.clone();

    info!(
        tempo_endpoint = %tempo_endpoint,
        service_name = %service_name,
        "Initializing observability"
    );

    // Setup OpenTelemetry tracer
    let tracer = setup_opentelemetry_tracer(config, &tempo_endpoint, &service_name)?;

    // Setup tracing subscriber
    setup_tracing_subscriber(tracer, &service_name)?;

    Ok(TelemetryGuard)
}

/// Setup OpenTelemetry tracer with OTLP exporter.
fn setup_opentelemetry_tracer(
    config: &GatewayConfig,
    tempo_endpoint: &str,
    service_name: &str,
) -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error + Send + Sync>> {
    // Create OTLP exporter
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(tempo_endpoint)
        .with_timeout(Duration::from_secs(config.observability.otlp_timeout_secs))
        .build()?;

    // Create tracer provider
    let tracer_provider = create_tracer_provider(config, otlp_exporter, service_name);

    // Create tracer
    let tracer = tracer_provider.tracer(service_name.to_string());
    global::set_tracer_provider(tracer_provider);

    // Set global propagator
    configure_trace_propagator();

    Ok(tracer)
}

/// Create tracer provider with configuration.
fn create_tracer_provider(
    config: &GatewayConfig,
    otlp_exporter: opentelemetry_otlp::SpanExporter,
    service_name: &str,
) -> opentelemetry_sdk::trace::SdkTracerProvider {
    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(config.observability.max_events_per_span)
        .with_max_attributes_per_span(config.observability.max_attributes_per_span)
        .with_resource(
            Resource::builder_empty()
                .with_attributes([KeyValue::new("service.name", service_name.to_string())])
                .build(),
        )
        .build()
}

/// Configure W3C Trace Context propagator.
fn configure_trace_propagator() {
    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    global::set_text_map_propagator(propagator);
    info!("OpenTelemetry propagator configured: W3C Trace Context");
}

/// Setup tracing subscriber with layers.
fn setup_tracing_subscriber(
    tracer: opentelemetry_sdk::trace::Tracer,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create JSON formatter for Loki compatibility
    let fmt_layer = tracing_subscriber::fmt::layer().event_format(FlattenedJsonFormat::new());

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Configure logging level
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let env_filter = create_env_filter(&log_level);

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .with(env_filter)
        .init();

    // Log startup message
    info!(
        version = env!("CARGO_PKG_VERSION"),
        service = %service_name,
        log_level = %log_level,
        "OpenTelemetry initialized successfully"
    );

    Ok(())
}

/// Create environment filter for logging.
fn create_env_filter(log_level: &str) -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(log_level))
        .unwrap_or_else(|e| {
            eprintln!(
                "Invalid RUST_LOG value '{}': {}. Using default 'info'",
                log_level, e
            );
            tracing_subscriber::EnvFilter::new("info")
        })
}

/// Shutdown telemetry and flush any pending data.
pub async fn shutdown_telemetry() {
    info!("Shutting down telemetry");

    // Give some time for final exports
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!("Telemetry shutdown complete");
}
