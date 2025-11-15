use crate::analytics::{AnalyticsError, AnalyticsEvent, Result};
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use chrono::Duration;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, error, info, warn};

const BATCH_INTERVAL_SECONDS: u64 = 30;
const MAX_EVENT_AGE_HOURS: i64 = 24;
const REDIS_QUEUE_KEY: &str = "analytics:events:queue";
const MAX_BATCH_SIZE: usize = 100;
const RETRY_BACKOFF_BASE_MS: u64 = 1000;
const MAX_RETRY_ATTEMPTS: u32 = 5;

/// Publisher for analytics events with batching and queueing support
pub struct AnalyticsPublisher {
    redis_client: redis::Client,
    event_buffer: Arc<Mutex<Vec<AnalyticsEvent>>>,
    analytics_service_url: String,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl AnalyticsPublisher {
    /// Create a new analytics publisher with circuit breaker
    pub fn new(redis_url: &str, analytics_service_url: &str) -> Result<Self> {
        let redis_client = redis::Client::open(redis_url)
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "analytics_service".to_string(),
            CircuitBreakerConfig::analytics_service(),
        ));

        Ok(Self {
            redis_client,
            event_buffer: Arc::new(Mutex::new(Vec::new())),
            analytics_service_url: analytics_service_url.to_string(),
            circuit_breaker,
        })
    }

    /// Get circuit breaker for monitoring
    pub fn circuit_breaker(&self) -> Arc<CircuitBreaker> {
        self.circuit_breaker.clone()
    }

    /// Publish an event (adds to buffer for batching)
    pub async fn publish_event(&self, event: AnalyticsEvent) -> Result<()> {
        // Check if event is too old
        if event.is_older_than(Duration::hours(MAX_EVENT_AGE_HOURS)) {
            warn!(
                "Discarding event older than {} hours: {:?}",
                MAX_EVENT_AGE_HOURS, event
            );
            return Err(AnalyticsError::EventExpired);
        }

        let mut buffer = self.event_buffer.lock().await;
        buffer.push(event);
        debug!("Event added to buffer, current size: {}", buffer.len());

        Ok(())
    }

    /// Start the background worker that flushes events at regular intervals
    pub async fn start_worker(self: Arc<Self>) {
        info!("Starting analytics publisher worker");
        let mut ticker = interval(TokioDuration::from_secs(BATCH_INTERVAL_SECONDS));

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.flush_batch().await {
                error!("Failed to flush event batch: {}", e);
            }
        }
    }

    /// Flush the current batch of events
    pub async fn flush_batch(&self) -> Result<()> {
        let events = {
            let mut buffer = self.event_buffer.lock().await;
            if buffer.is_empty() {
                debug!("No events to flush");
                return Ok(());
            }
            std::mem::take(&mut *buffer)
        };

        info!("Flushing {} events to analytics service", events.len());

        // Try to send events to analytics service
        match self.send_to_analytics_service(&events).await {
            Ok(_) => {
                info!("Successfully sent {} events to analytics service", events.len());
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to send events to analytics service: {}. Queueing in Redis.",
                    e
                );
                // Queue events in Redis for later retry
                self.queue_events_in_redis(&events).await?;
                Ok(())
            }
        }
    }

    /// Send events to the analytics service via gRPC (with circuit breaker)
    async fn send_to_analytics_service(&self, events: &[AnalyticsEvent]) -> Result<()> {
        if events.is_empty() {
            return Err(AnalyticsError::EmptyBatch);
        }

        let payload = serde_json::to_string(events)?;
        let url = self.analytics_service_url.clone();

        let result = self.circuit_breaker.call(|| async {
            // TODO: Implement actual gRPC client when Analytics Service is available
            // For now, we'll simulate the call with a simple HTTP POST
            let client = reqwest::Client::builder()
                .timeout(TokioDuration::from_secs(10))
                .build()
                .map_err(|e| AnalyticsError::TransmissionError(e.to_string()))?;

            let response = client
                .post(format!("{}/events", url))
                .header("Content-Type", "application/json")
                .body(payload.clone())
                .send()
                .await
                .map_err(|e| AnalyticsError::ServiceUnavailable(e.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(AnalyticsError::TransmissionError(format!(
                    "Analytics service returned status: {}",
                    response.status()
                )))
            }
        }).await;

        match result {
            Ok(()) => Ok(()),
            Err(CircuitBreakerError::CircuitOpen) => {
                warn!("Analytics service circuit breaker is open");
                Err(AnalyticsError::ServiceUnavailable("Circuit breaker open".to_string()))
            }
            Err(CircuitBreakerError::OperationFailed(e)) => Err(e),
        }
    }

    /// Queue events in Redis when analytics service is unavailable
    async fn queue_events_in_redis(&self, events: &[AnalyticsEvent]) -> Result<()> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        for event in events {
            let serialized = serde_json::to_string(event)?;
            conn.rpush::<_, _, ()>(REDIS_QUEUE_KEY, serialized)
                .await
                .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;
        }

        info!("Queued {} events in Redis for retry", events.len());
        Ok(())
    }

    /// Retry sending queued events from Redis
    pub async fn retry_queued_events(&self) -> Result<()> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        // Get queue length
        let queue_len: usize = conn
            .llen(REDIS_QUEUE_KEY)
            .await
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        if queue_len == 0 {
            debug!("No queued events to retry");
            return Ok(());
        }

        info!("Retrying {} queued events", queue_len);

        // Process events in batches
        let batch_size = MAX_BATCH_SIZE.min(queue_len);
        let mut events = Vec::with_capacity(batch_size);
        let mut expired_count = 0;

        for _ in 0..batch_size {
            let serialized: Option<String> = conn
                .lpop(REDIS_QUEUE_KEY, None)
                .await
                .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

            if let Some(data) = serialized {
                match serde_json::from_str::<AnalyticsEvent>(&data) {
                    Ok(event) => {
                        // Check if event is too old
                        if event.is_older_than(Duration::hours(MAX_EVENT_AGE_HOURS)) {
                            expired_count += 1;
                            continue;
                        }
                        events.push(event);
                    }
                    Err(e) => {
                        error!("Failed to deserialize queued event: {}", e);
                    }
                }
            }
        }

        if expired_count > 0 {
            warn!("Discarded {} expired events from queue", expired_count);
        }

        if events.is_empty() {
            return Ok(());
        }

        // Try to send with exponential backoff
        let mut attempt = 0;
        loop {
            match self.send_to_analytics_service(&events).await {
                Ok(_) => {
                    info!("Successfully sent {} queued events", events.len());
                    return Ok(());
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= MAX_RETRY_ATTEMPTS {
                        error!(
                            "Failed to send queued events after {} attempts, re-queueing",
                            MAX_RETRY_ATTEMPTS
                        );
                        // Re-queue events
                        self.queue_events_in_redis(&events).await?;
                        return Err(e);
                    }

                    let backoff_ms = RETRY_BACKOFF_BASE_MS * 2_u64.pow(attempt - 1);
                    warn!(
                        "Retry attempt {} failed: {}. Retrying in {}ms",
                        attempt, e, backoff_ms
                    );
                    tokio::time::sleep(TokioDuration::from_millis(backoff_ms)).await;
                }
            }
        }
    }

    /// Start a background worker for retrying queued events
    pub async fn start_retry_worker(self: Arc<Self>) {
        info!("Starting analytics retry worker");
        let mut ticker = interval(TokioDuration::from_secs(300)); // Retry every 5 minutes

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.retry_queued_events().await {
                error!("Failed to retry queued events: {}", e);
            }
        }
    }

    /// Get the current buffer size (for testing/monitoring)
    pub async fn buffer_size(&self) -> usize {
        self.event_buffer.lock().await.len()
    }

    /// Get the Redis queue size (for monitoring)
    pub async fn queue_size(&self) -> Result<usize> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        let size: usize = conn
            .llen(REDIS_QUEUE_KEY)
            .await
            .map_err(|e| AnalyticsError::QueueError(e.to_string()))?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_publish_event() {
        // This test requires a Redis instance
        // Skip if Redis is not available
        let redis_url = "redis://localhost:6379";
        let publisher = match AnalyticsPublisher::new(redis_url, "http://localhost:8080") {
            Ok(p) => p,
            Err(_) => return, // Skip test if Redis not available
        };

        let event = AnalyticsEvent::VideoPlay(crate::analytics::VideoPlayEvent {
            student_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: None,
        });

        let result = publisher.publish_event(event).await;
        assert!(result.is_ok());
        assert_eq!(publisher.buffer_size().await, 1);
    }

    #[test]
    fn test_event_expiration() {
        let old_event = AnalyticsEvent::VideoPlay(crate::analytics::VideoPlayEvent {
            student_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            timestamp: Utc::now() - Duration::hours(25),
            session_id: None,
        });

        assert!(old_event.is_older_than(Duration::hours(24)));
    }
}
