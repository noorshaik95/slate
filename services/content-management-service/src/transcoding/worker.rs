use crate::db::repositories::{ResourceRepository, TranscodingJobRepository};
use crate::storage::S3Client;
use crate::transcoding::errors::{Result, TranscodingError};
use crate::transcoding::queue::{TranscodingJobMessage, TranscodingQueue};
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Video transcoder that processes transcoding jobs from the queue
pub struct VideoTranscoder {
    queue: TranscodingQueue,
    s3_client: S3Client,
    job_repository: TranscodingJobRepository,
    resource_repository: ResourceRepository,
    work_dir: PathBuf,
    bucket: String,
}

impl VideoTranscoder {
    /// Creates a new VideoTranscoder
    pub fn new(
        queue: TranscodingQueue,
        s3_client: S3Client,
        job_repository: TranscodingJobRepository,
        resource_repository: ResourceRepository,
        work_dir: PathBuf,
        bucket: String,
    ) -> Self {
        Self {
            queue,
            s3_client,
            job_repository,
            resource_repository,
            work_dir,
            bucket,
        }
    }

    /// Starts the worker loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting video transcoder worker");

        // Ensure work directory exists
        fs::create_dir_all(&self.work_dir).await?;

        loop {
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No job available, wait a bit
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
                Err(e) => {
                    error!("Error processing job: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Processes the next job from the queue
    async fn process_next_job(&mut self) -> Result<bool> {
        // Dequeue with 30 second timeout
        let job_msg = match self.queue.dequeue(30).await? {
            Some(msg) => msg,
            None => return Ok(false),
        };

        info!(
            job_id = %job_msg.job_id,
            resource_id = %job_msg.resource_id,
            "Processing transcoding job"
        );

        // Mark job as processing
        self.queue
            .set_job_status(job_msg.job_id, "processing")
            .await?;

        match self.job_repository.mark_processing(job_msg.job_id).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to mark job as processing: {}", e);
                return Err(TranscodingError::Database(e));
            }
        }

        // Process the job
        let result = self.transcode_video(&job_msg).await;

        match result {
            Ok((hls_manifest, dash_manifest, duration)) => {
                info!(
                    job_id = %job_msg.job_id,
                    duration_seconds = duration,
                    "Transcoding completed successfully"
                );

                // This will be handled in subtask 5.3
                self.handle_success(&job_msg, hls_manifest, dash_manifest, duration)
                    .await?;
            }
            Err(e) => {
                error!(
                    job_id = %job_msg.job_id,
                    error = %e,
                    "Transcoding failed"
                );

                // This will be handled in subtask 5.4
                self.handle_failure(&job_msg, e.to_string()).await?;
            }
        }

        Ok(true)
    }

    /// Transcodes a video to HLS and DASH formats
    async fn transcode_video(
        &self,
        job_msg: &TranscodingJobMessage,
    ) -> Result<(String, String, i32)> {
        let job_dir = self.work_dir.join(job_msg.job_id.to_string());
        fs::create_dir_all(&job_dir).await?;

        // Download original video from S3
        let input_path = job_dir.join("input.mp4");
        self.download_video(&job_msg.storage_key, &input_path)
            .await?;

        // Get video duration
        let duration = self.get_video_duration(&input_path).await?;

        // Transcode to HLS
        let hls_dir = job_dir.join("hls");
        fs::create_dir_all(&hls_dir).await?;
        self.transcode_hls(&input_path, &hls_dir).await?;

        // Transcode to DASH
        let dash_dir = job_dir.join("dash");
        fs::create_dir_all(&dash_dir).await?;
        self.transcode_dash(&input_path, &dash_dir).await?;

        // Upload transcoded files to S3
        let hls_manifest = self
            .upload_transcoded_files(&hls_dir, job_msg.resource_id, "hls")
            .await?;
        let dash_manifest = self
            .upload_transcoded_files(&dash_dir, job_msg.resource_id, "dash")
            .await?;

        // Cleanup
        fs::remove_dir_all(&job_dir).await?;

        Ok((hls_manifest, dash_manifest, duration))
    }

    /// Downloads video from S3
    async fn download_video(&self, storage_key: &str, output_path: &Path) -> Result<()> {
        debug!("Downloading video from S3: {}", storage_key);

        let data = self
            .s3_client
            .get_object(storage_key)
            .await
            .map_err(|e| TranscodingError::DownloadFailed(e.to_string()))?;

        let mut file = fs::File::create(output_path).await?;
        file.write_all(&data).await?;

        info!("Downloaded video to {:?}", output_path);
        Ok(())
    }

    /// Gets video duration using ffprobe
    async fn get_video_duration(&self, input_path: &Path) -> Result<i32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                input_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(TranscodingError::FFmpegFailed(format!(
                "ffprobe failed: {}",
                error
            )));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str
            .trim()
            .parse()
            .map_err(|e| TranscodingError::FFmpegFailed(format!("Invalid duration: {}", e)))?;

        Ok(duration.ceil() as i32)
    }

    /// Transcodes video to HLS format with multiple bitrates
    async fn transcode_hls(&self, input_path: &Path, output_dir: &Path) -> Result<()> {
        info!("Transcoding to HLS format");

        let output = Command::new("ffmpeg")
            .args([
                "-i",
                input_path.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                // 360p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:0",
                "800k",
                "-s:v:0",
                "640x360",
                "-maxrate:v:0",
                "856k",
                "-bufsize:v:0",
                "1200k",
                // 480p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:1",
                "1400k",
                "-s:v:1",
                "854x480",
                "-maxrate:v:1",
                "1498k",
                "-bufsize:v:1",
                "2100k",
                // 720p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:2",
                "2800k",
                "-s:v:2",
                "1280x720",
                "-maxrate:v:2",
                "2996k",
                "-bufsize:v:2",
                "4200k",
                // 1080p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:3",
                "5000k",
                "-s:v:3",
                "1920x1080",
                "-maxrate:v:3",
                "5350k",
                "-bufsize:v:3",
                "7500k",
                // HLS settings
                "-var_stream_map",
                "v:0,a:0 v:1,a:0 v:2,a:0 v:3,a:0",
                "-master_pl_name",
                "master.m3u8",
                "-f",
                "hls",
                "-hls_time",
                "6",
                "-hls_list_size",
                "0",
                "-hls_segment_filename",
                &format!("{}/v%v/segment_%03d.ts", output_dir.to_str().unwrap()),
                &format!("{}/v%v/playlist.m3u8", output_dir.to_str().unwrap()),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(TranscodingError::FFmpegFailed(format!(
                "HLS transcoding failed: {}",
                error
            )));
        }

        info!("HLS transcoding completed");
        Ok(())
    }

    /// Transcodes video to DASH format with multiple bitrates
    async fn transcode_dash(&self, input_path: &Path, output_dir: &Path) -> Result<()> {
        info!("Transcoding to DASH format");

        let output = Command::new("ffmpeg")
            .args([
                "-i",
                input_path.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                // 360p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:0",
                "800k",
                "-s:v:0",
                "640x360",
                // 480p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:1",
                "1400k",
                "-s:v:1",
                "854x480",
                // 720p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:2",
                "2800k",
                "-s:v:2",
                "1280x720",
                // 1080p
                "-map",
                "0:v:0",
                "-map",
                "0:a:0",
                "-b:v:3",
                "5000k",
                "-s:v:3",
                "1920x1080",
                // DASH settings
                "-f",
                "dash",
                "-seg_duration",
                "6",
                "-use_template",
                "1",
                "-use_timeline",
                "1",
                "-init_seg_name",
                "init-$RepresentationID$.m4s",
                "-media_seg_name",
                "chunk-$RepresentationID$-$Number%05d$.m4s",
                &format!("{}/manifest.mpd", output_dir.to_str().unwrap()),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(TranscodingError::FFmpegFailed(format!(
                "DASH transcoding failed: {}",
                error
            )));
        }

        info!("DASH transcoding completed");
        Ok(())
    }

    /// Uploads transcoded files to S3 and returns manifest URL
    async fn upload_transcoded_files(
        &self,
        dir: &Path,
        resource_id: Uuid,
        format: &str,
    ) -> Result<String> {
        info!("Uploading {} files to S3", format);

        let base_key = format!("transcoded/{}/{}", resource_id, format);
        let manifest_name = if format == "hls" {
            "master.m3u8"
        } else {
            "manifest.mpd"
        };

        // Upload all files in directory recursively
        self.upload_directory_recursive(dir, &base_key).await?;

        let manifest_url = format!("{}/{}", base_key, manifest_name);
        info!("Uploaded {} manifest: {}", format, manifest_url);

        Ok(manifest_url)
    }

    /// Recursively uploads directory contents to S3
    fn upload_directory_recursive<'a>(
        &'a self,
        dir: &'a Path,
        base_key: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if path.is_dir() {
                    // Recursively upload subdirectory
                    let sub_key = format!("{}/{}", base_key, file_name_str);
                    self.upload_directory_recursive(&path, &sub_key).await?;
                } else {
                    // Upload file
                    let key = format!("{}/{}", base_key, file_name_str);
                    let data = fs::read(&path).await?;

                    // Determine content type based on file extension
                    let content_type = if file_name_str.ends_with(".m3u8") {
                        "application/vnd.apple.mpegurl"
                    } else if file_name_str.ends_with(".ts") {
                        "video/mp2t"
                    } else if file_name_str.ends_with(".mpd") {
                        "application/dash+xml"
                    } else if file_name_str.ends_with(".m4s") {
                        "video/iso.segment"
                    } else {
                        "application/octet-stream"
                    };

                    self.s3_client
                        .put_object(&key, Bytes::from(data), content_type)
                        .await
                        .map_err(|e| TranscodingError::UploadFailed(e.to_string()))?;

                    debug!("Uploaded file: {}", key);
                }
            }

            Ok(())
        })
    }

    /// Handles successful transcoding
    async fn handle_success(
        &mut self,
        job_msg: &TranscodingJobMessage,
        hls_manifest: String,
        dash_manifest: String,
        duration: i32,
    ) -> Result<()> {
        info!(
            job_id = %job_msg.job_id,
            resource_id = %job_msg.resource_id,
            "Handling successful transcoding"
        );

        // Update resource with manifest URL (using HLS as primary)
        match self
            .resource_repository
            .update_video_metadata(job_msg.resource_id, hls_manifest.clone(), duration)
            .await
        {
            Ok(_) => {
                info!(
                    resource_id = %job_msg.resource_id,
                    hls_manifest = %hls_manifest,
                    dash_manifest = %dash_manifest,
                    duration = duration,
                    "Updated resource with video metadata"
                );
            }
            Err(e) => {
                error!("Failed to update resource metadata: {}", e);
                return Err(TranscodingError::Database(e));
            }
        }

        // Mark transcoding job as completed
        match self.job_repository.mark_completed(job_msg.job_id).await {
            Ok(_) => {
                info!(job_id = %job_msg.job_id, "Marked transcoding job as completed");
            }
            Err(e) => {
                error!("Failed to mark job as completed: {}", e);
                return Err(TranscodingError::Database(e));
            }
        }

        // Update Redis status
        self.queue
            .set_job_status(job_msg.job_id, "completed")
            .await?;

        Ok(())
    }

    /// Handles transcoding failure with retry logic
    async fn handle_failure(
        &mut self,
        job_msg: &TranscodingJobMessage,
        error: String,
    ) -> Result<()> {
        error!(
            job_id = %job_msg.job_id,
            resource_id = %job_msg.resource_id,
            error = %error,
            "Handling transcoding failure"
        );

        // Mark job as failed and increment retry count
        let job = match self
            .job_repository
            .mark_failed(job_msg.job_id, error.clone())
            .await
        {
            Ok(job) => job,
            Err(e) => {
                error!("Failed to mark job as failed: {}", e);
                return Err(TranscodingError::Database(e));
            }
        };

        // Check if we can retry
        if job.can_retry() {
            warn!(
                job_id = %job_msg.job_id,
                retry_count = job.retry_count,
                max_retries = crate::models::TranscodingJob::MAX_RETRIES,
                "Job failed but can be retried"
            );

            // Reset job to pending for retry
            match self.job_repository.reset_for_retry(job_msg.job_id).await {
                Ok(_) => {
                    info!(job_id = %job_msg.job_id, "Reset job for retry");
                }
                Err(e) => {
                    error!("Failed to reset job for retry: {}", e);
                    return Err(TranscodingError::Database(e));
                }
            }

            // Re-enqueue the job with updated retry count
            let retry_msg = TranscodingJobMessage {
                job_id: job_msg.job_id,
                resource_id: job_msg.resource_id,
                storage_key: job_msg.storage_key.clone(),
                retry_count: job.retry_count,
            };

            self.queue.enqueue(retry_msg).await?;
            self.queue
                .set_job_status(job_msg.job_id, "pending_retry")
                .await?;

            info!(
                job_id = %job_msg.job_id,
                retry_count = job.retry_count,
                "Re-enqueued job for retry"
            );
        } else {
            error!(
                job_id = %job_msg.job_id,
                retry_count = job.retry_count,
                "Job failed and exhausted all retries"
            );

            self.queue
                .set_job_status(job_msg.job_id, "failed_permanent")
                .await?;
        }

        Ok(())
    }
}
