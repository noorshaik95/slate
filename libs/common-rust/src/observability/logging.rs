#[cfg(feature = "observability")]
use serde::{Deserialize, Serialize};

/// Tracing configuration
#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    pub json_format: bool,
}

/// Initialize tracing with OpenTelemetry
#[cfg(feature = "observability")]
pub fn init_tracing(config: TracingConfig) -> Result<(), anyhow::Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Create env filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // Create subscriber with layers
    if config.json_format {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }

    Ok(())
}

/// Shutdown tracing gracefully
#[cfg(feature = "observability")]
pub fn shutdown_tracing() {
    // Shutdown is handled automatically
}
