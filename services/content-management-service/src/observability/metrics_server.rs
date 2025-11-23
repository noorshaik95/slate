use crate::observability::metrics::Metrics;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::{error, info};

/// Metrics server state
#[derive(Clone)]
pub struct MetricsState {
    pub metrics: Arc<Metrics>,
}

/// Start the metrics HTTP server on the specified port
pub async fn start_metrics_server(port: u16, metrics: Arc<Metrics>) -> anyhow::Result<()> {
    let state = MetricsState { metrics };

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting metrics server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for the /metrics endpoint
async fn metrics_handler(State(state): State<MetricsState>) -> Response {
    match state.metrics.encode() {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode metrics: {}", e),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_handler() {
        let metrics = Arc::new(Metrics::new().unwrap());
        let state = MetricsState { metrics };

        // Increment a counter to ensure we have some metrics
        state
            .metrics
            .uploads_total
            .with_label_values(&["success"])
            .inc();

        let response = metrics_handler(State(state)).await;
        let status = response.status();

        assert_eq!(status, StatusCode::OK);
    }
}
