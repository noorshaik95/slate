use super::errors::UploadError;
use super::validator::FileValidator;
use crate::db::repositories::{
    ResourceRepository, TranscodingJobRepository, UploadSessionRepository,
};
use crate::models::{ContentType, Resource, UploadSession, UploadStatus};
use crate::storage::S3Client;
use crate::transcoding::queue::{TranscodingJobMessage, TranscodingQueue};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// UploadHandler manages file uploads with chunked upload support
#[derive(Clone)]
pub struct UploadHandler {
    session_repo: Arc<UploadSessionRepository>,
    resource_repo: Arc<ResourceRepository>,
    s3_client: Arc<S3Client>,
    transcoding_queue: Option<Arc<Mutex<TranscodingQueue>>>,
    transcoding_job_repo: Option<Arc<TranscodingJobRepository>>,
}

impl UploadHandler {
    /// Creates a new UploadHandler
    pub fn new(
        session_repo: Arc<UploadSessionRepository>,
        resource_repo: Arc<ResourceRepository>,
        s3_client: Arc<S3Client>,
    ) -> Self {
        Self {
            session_repo,
            resource_repo,
            s3_client,
            transcoding_queue: None,
            transcoding_job_repo: None,
        }
    }

    /// Sets the transcoding queue and job repository (optional for video transcoding)
    pub fn with_transcoding(
        mut self,
        queue: Arc<Mutex<TranscodingQueue>>,
        job_repo: Arc<TranscodingJobRepository>,
    ) -> Self {
        self.transcoding_queue = Some(queue);
        self.transcoding_job_repo = Some(job_repo);
        self
    }

    /// Initiates a new upload session
    #[instrument(skip(self))]
    pub async fn initiate_upload(
        &self,
        user_id: Uuid,
        lesson_id: Uuid,
        filename: String,
        content_type: String,
        total_size: i64,
    ) -> Result<UploadSession, UploadError> {
        info!(
            "Initiating upload: user={}, lesson={}, filename={}, size={}",
            user_id, lesson_id, filename, total_size
        );

        // Validate inputs
        FileValidator::validate_filename(&filename)?;
        FileValidator::validate_mime_type(&content_type)?;
        FileValidator::validate_file_size(total_size)?;

        // Generate unique storage key
        let session_id = Uuid::new_v4();
        let storage_key = self.generate_storage_key(&session_id, &filename);

        debug!("Generated storage key: {}", storage_key);

        // Create upload session in database
        let session = self
            .session_repo
            .create(
                user_id,
                Some(lesson_id),
                filename,
                content_type,
                total_size,
                UploadSession::DEFAULT_CHUNK_SIZE,
                storage_key,
            )
            .await?;

        info!(
            "Upload session created: id={}, total_chunks={}",
            session.id, session.total_chunks
        );

        Ok(session)
    }

    /// Processes a single chunk upload
    #[instrument(skip(self, chunk_data))]
    pub async fn process_chunk(
        &self,
        session_id: Uuid,
        chunk_index: i32,
        chunk_data: Bytes,
    ) -> Result<ChunkUploadResult, UploadError> {
        debug!(
            "Processing chunk: session={}, index={}, size={}",
            session_id,
            chunk_index,
            chunk_data.len()
        );

        // Retrieve and validate session
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| UploadError::SessionNotFound(session_id.to_string()))?;

        // Validate session is active
        if session.is_expired() {
            warn!("Upload session expired: {}", session_id);
            return Err(UploadError::SessionExpired(session_id.to_string()));
        }

        if session.status != UploadStatus::InProgress {
            return Err(UploadError::SessionNotResumable(format!(
                "Session status is {}",
                session.status
            )));
        }

        // Validate chunk index
        session
            .validate_chunk_index(chunk_index)
            .map_err(|e| UploadError::InvalidChunkIndex(e))?;

        // Verify file header on first chunk
        if chunk_index == 0 {
            FileValidator::verify_file_header(&chunk_data, &session.content_type)?;
            FileValidator::scan_for_malware(&chunk_data).await?;
        }

        // Store chunk in temporary S3 location
        let chunk_key = self.generate_chunk_key(&session.storage_key, chunk_index);
        self.s3_client
            .put_object(&chunk_key, chunk_data, "application/octet-stream")
            .await?;

        // Update uploaded chunks counter
        let updated_session = self
            .session_repo
            .increment_uploaded_chunks(session_id)
            .await?;

        info!(
            "Chunk uploaded: session={}, chunk={}, progress={}/{} ({:.1}%)",
            session_id,
            chunk_index,
            updated_session.uploaded_chunks,
            updated_session.total_chunks,
            updated_session.progress_percentage()
        );

        Ok(ChunkUploadResult {
            uploaded_chunks: updated_session.uploaded_chunks,
            total_chunks: updated_session.total_chunks,
            progress_percentage: updated_session.progress_percentage(),
        })
    }

    /// Completes an upload and assembles chunks
    #[instrument(skip(self))]
    pub async fn complete_upload(
        &self,
        session_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Resource, UploadError> {
        info!("Completing upload: session={}", session_id);

        // Retrieve session
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| UploadError::SessionNotFound(session_id.to_string()))?;

        // Get lesson_id from session
        let lesson_id = session.lesson_id.ok_or_else(|| {
            UploadError::DatabaseError("Upload session missing lesson_id".to_string())
        })?;

        // Validate all chunks uploaded
        if !session.is_complete() {
            return Err(UploadError::InvalidChunkIndex(format!(
                "Not all chunks uploaded: {}/{}",
                session.uploaded_chunks, session.total_chunks
            )));
        }

        // Assemble chunks
        let assembled_data = self.assemble_chunks(&session).await?;

        // Move to permanent storage
        let permanent_key = self.generate_permanent_key(&session.filename);
        self.s3_client
            .put_object(&permanent_key, assembled_data, &session.content_type)
            .await?;

        info!("File assembled and stored: {}", permanent_key);

        // Clean up temporary chunks
        self.cleanup_chunks(&session).await?;

        // Convert MIME type to ContentType enum
        let content_type = Self::mime_to_content_type(&session.content_type)?;

        // Create resource record
        let resource = self
            .resource_repo
            .create(
                lesson_id,
                name,
                description,
                content_type.clone(),
                session.total_size,
                permanent_key.clone(),
                0, // display_order will be set by content manager
            )
            .await?;

        // Mark session as completed
        self.session_repo
            .update_status(session_id, UploadStatus::Completed)
            .await?;

        info!("Upload completed: resource={}", resource.id);

        // Trigger transcoding for video files
        if content_type == ContentType::Video {
            self.trigger_transcoding(resource.id, permanent_key).await?;
        }

        Ok(resource)
    }

    /// Triggers transcoding for a video resource
    async fn trigger_transcoding(
        &self,
        resource_id: Uuid,
        storage_key: String,
    ) -> Result<(), UploadError> {
        // Check if transcoding is configured
        let (queue, job_repo) = match (&self.transcoding_queue, &self.transcoding_job_repo) {
            (Some(q), Some(r)) => (q, r),
            _ => {
                warn!("Transcoding not configured, skipping video transcoding");
                return Ok(());
            }
        };

        info!(
            resource_id = %resource_id,
            storage_key = %storage_key,
            "Triggering video transcoding"
        );

        // Create transcoding job in database
        let job = job_repo
            .create(resource_id)
            .await
            .map_err(|e| UploadError::DatabaseError(e.to_string()))?;

        // Enqueue transcoding job
        let job_msg = TranscodingJobMessage {
            job_id: job.id,
            resource_id,
            storage_key,
            retry_count: 0,
        };

        let mut queue_guard = queue.lock().await;
        queue_guard.enqueue(job_msg).await.map_err(|e| {
            UploadError::DatabaseError(format!("Failed to enqueue transcoding job: {}", e))
        })?;

        info!(
            job_id = %job.id,
            resource_id = %resource_id,
            "Transcoding job created and enqueued"
        );

        Ok(())
    }

    /// Resumes an existing upload session
    #[instrument(skip(self))]
    pub async fn resume_upload(&self, session_id: Uuid) -> Result<UploadSession, UploadError> {
        info!("Resuming upload: session={}", session_id);

        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| UploadError::SessionNotFound(session_id.to_string()))?;

        if !session.is_resumable() {
            return Err(UploadError::SessionNotResumable(format!(
                "Session is not resumable: status={}, expired={}",
                session.status,
                session.is_expired()
            )));
        }

        info!(
            "Upload session resumed: id={}, progress={}/{}",
            session.id, session.uploaded_chunks, session.total_chunks
        );

        Ok(session)
    }

    /// Cancels an upload session
    #[instrument(skip(self))]
    pub async fn cancel_upload(&self, session_id: Uuid) -> Result<(), UploadError> {
        info!("Cancelling upload: session={}", session_id);

        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| UploadError::SessionNotFound(session_id.to_string()))?;

        // Clean up chunks
        self.cleanup_chunks(&session).await?;

        // Mark session as cancelled
        self.session_repo
            .update_status(session_id, UploadStatus::Cancelled)
            .await?;

        info!("Upload cancelled: session={}", session_id);

        Ok(())
    }

    /// Generates a storage key for a new upload
    fn generate_storage_key(&self, session_id: &Uuid, filename: &str) -> String {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        format!("uploads/temp/{}.{}", session_id, extension)
    }

    /// Generates a chunk key for temporary storage
    fn generate_chunk_key(&self, storage_key: &str, chunk_index: i32) -> String {
        format!("{}.chunk.{}", storage_key, chunk_index)
    }

    /// Generates a permanent storage key
    fn generate_permanent_key(&self, filename: &str) -> String {
        let uuid = Uuid::new_v4();
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        format!("content/{}.{}", uuid, extension)
    }

    /// Assembles chunks into a single file
    async fn assemble_chunks(&self, session: &UploadSession) -> Result<Bytes, UploadError> {
        debug!("Assembling {} chunks", session.total_chunks);

        let mut assembled = Vec::with_capacity(session.total_size as usize);

        for chunk_index in 0..session.total_chunks {
            let chunk_key = self.generate_chunk_key(&session.storage_key, chunk_index);
            let chunk_data = self.s3_client.get_object(&chunk_key).await?;
            assembled.extend_from_slice(&chunk_data);
        }

        info!(
            "Assembled {} chunks into {} bytes",
            session.total_chunks,
            assembled.len()
        );

        Ok(Bytes::from(assembled))
    }

    /// Cleans up temporary chunk files
    async fn cleanup_chunks(&self, session: &UploadSession) -> Result<(), UploadError> {
        debug!("Cleaning up chunks for session: {}", session.id);

        let mut chunk_keys = Vec::new();
        for chunk_index in 0..session.total_chunks {
            chunk_keys.push(self.generate_chunk_key(&session.storage_key, chunk_index));
        }

        if !chunk_keys.is_empty() {
            self.s3_client.delete_objects(chunk_keys).await?;
        }

        info!("Cleaned up {} chunks", session.total_chunks);

        Ok(())
    }

    /// Converts MIME type string to ContentType enum
    fn mime_to_content_type(mime_type: &str) -> Result<ContentType, UploadError> {
        if mime_type.starts_with("video/") {
            Ok(ContentType::Video)
        } else if mime_type == "application/pdf" {
            Ok(ContentType::Pdf)
        } else if mime_type
            == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        {
            Ok(ContentType::Docx)
        } else {
            Err(UploadError::InvalidFileType(mime_type.to_string()))
        }
    }
}

/// Result of a chunk upload operation
#[derive(Debug, Clone)]
pub struct ChunkUploadResult {
    pub uploaded_chunks: i32,
    pub total_chunks: i32,
    pub progress_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_storage_key() {
        let handler = create_test_handler();
        let session_id = Uuid::new_v4();
        let key = handler.generate_storage_key(&session_id, "test.mp4");
        assert!(key.starts_with("uploads/temp/"));
        assert!(key.ends_with(".mp4"));
    }

    #[test]
    fn test_generate_chunk_key() {
        let handler = create_test_handler();
        let key = handler.generate_chunk_key("uploads/temp/test.mp4", 0);
        assert_eq!(key, "uploads/temp/test.mp4.chunk.0");
    }

    #[test]
    fn test_generate_permanent_key() {
        let handler = create_test_handler();
        let key = handler.generate_permanent_key("test.mp4");
        assert!(key.starts_with("content/"));
        assert!(key.ends_with(".mp4"));
    }

    fn create_test_handler() -> UploadHandler {
        // This is a minimal handler for testing key generation
        // In real tests, you'd use proper mocks
        use sqlx::PgPool;
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let session_repo = Arc::new(UploadSessionRepository::new(pool.clone()));
        let resource_repo = Arc::new(ResourceRepository::new(pool));

        // Create a dummy S3 client (won't be used in these tests)
        let s3_client = Arc::new(unsafe {
            std::mem::zeroed() // This is just for compilation, not for actual use
        });

        UploadHandler::new(session_repo, resource_repo, s3_client)
    }
}
