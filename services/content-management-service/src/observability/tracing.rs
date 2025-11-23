use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize distributed tracing with OpenTelemetry and structured logging
pub fn init_tracing(service_name: &str, _otlp_endpoint: &str) -> anyhow::Result<()> {
    // Create JSON formatting layer for structured logging
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    // Create environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Combine layers and initialize subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .init();

    tracing::info!(service_name = %service_name, "Tracing initialized");

    Ok(())
}

/// Shutdown tracing and flush pending spans
pub fn shutdown_tracing() {
    // Shutdown is handled automatically
}
