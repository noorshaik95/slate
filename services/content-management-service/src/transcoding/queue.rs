use crate::transcoding::errors::{Result, TranscodingError};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Represents a transcoding job in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodingJobMessage {
    pub job_id: Uuid,
    pub resource_id: Uuid,
    pub storage_key: String,
    pub retry_count: i32,
}

/// Redis-based job queue for video transcoding
pub struct TranscodingQueue {
    connection: ConnectionManager,
    queue_name: String,
}

impl TranscodingQueue {
    /// Creates a new TranscodingQueue
    pub async fn new(redis_url: &str, queue_name: String) -> Result<Self> {
        let client = redis::Client::open(redis_url).map_err(|e| TranscodingError::Redis(e))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        info!("Connected to Redis queue: {}", queue_name);

        Ok(Self {
            connection,
            queue_name,
        })
    }

    /// Enqueues a transcoding job
    pub async fn enqueue(&mut self, job: TranscodingJobMessage) -> Result<()> {
        let job_json =
            serde_json::to_string(&job).map_err(|e| TranscodingError::Serialization(e))?;

        self.connection
            .rpush::<_, _, ()>(&self.queue_name, job_json)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        info!(
            job_id = %job.job_id,
            resource_id = %job.resource_id,
            "Enqueued transcoding job"
        );

        Ok(())
    }

    /// Dequeues a transcoding job (blocking with timeout)
    pub async fn dequeue(
        &mut self,
        timeout_seconds: usize,
    ) -> Result<Option<TranscodingJobMessage>> {
        let result: Option<(String, String)> = self
            .connection
            .blpop(&self.queue_name, timeout_seconds as f64)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        match result {
            Some((_, job_json)) => {
                let job: TranscodingJobMessage = serde_json::from_str(&job_json).map_err(|e| {
                    error!("Failed to deserialize job: {}", e);
                    TranscodingError::Serialization(e)
                })?;

                debug!(
                    job_id = %job.job_id,
                    resource_id = %job.resource_id,
                    "Dequeued transcoding job"
                );

                Ok(Some(job))
            }
            None => {
                debug!("No jobs available in queue");
                Ok(None)
            }
        }
    }

    /// Gets the current queue size
    pub async fn size(&mut self) -> Result<i64> {
        let size: i64 = self
            .connection
            .llen(&self.queue_name)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        Ok(size)
    }

    /// Tracks job status in Redis (for monitoring)
    pub async fn set_job_status(&mut self, job_id: Uuid, status: &str) -> Result<()> {
        let key = format!("transcoding:status:{}", job_id);

        self.connection
            .set_ex::<_, _, ()>(&key, status, 3600) // Expire after 1 hour
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        debug!(job_id = %job_id, status = status, "Updated job status");

        Ok(())
    }

    /// Gets job status from Redis
    pub async fn get_job_status(&mut self, job_id: Uuid) -> Result<Option<String>> {
        let key = format!("transcoding:status:{}", job_id);

        let status: Option<String> = self
            .connection
            .get(&key)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        Ok(status)
    }

    /// Removes job status from Redis
    pub async fn remove_job_status(&mut self, job_id: Uuid) -> Result<()> {
        let key = format!("transcoding:status:{}", job_id);

        self.connection
            .del::<_, ()>(&key)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        Ok(())
    }

    /// Clears all jobs from the queue (for testing/maintenance)
    pub async fn clear(&mut self) -> Result<i64> {
        let count: i64 = self
            .connection
            .del(&self.queue_name)
            .await
            .map_err(|e| TranscodingError::Redis(e))?;

        warn!("Cleared {} jobs from queue", count);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcoding_job_message_serialization() {
        let job = TranscodingJobMessage {
            job_id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            storage_key: "videos/test.mp4".to_string(),
            retry_count: 0,
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: TranscodingJobMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(job.job_id, deserialized.job_id);
        assert_eq!(job.resource_id, deserialized.resource_id);
        assert_eq!(job.storage_key, deserialized.storage_key);
        assert_eq!(job.retry_count, deserialized.retry_count);
    }
}
