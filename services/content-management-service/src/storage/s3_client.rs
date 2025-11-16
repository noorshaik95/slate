use super::errors::StorageError;
use crate::retry::retry_storage;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// S3Client wrapper for MinIO/S3 operations
#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    /// Creates a new S3Client with MinIO configuration
    #[instrument(skip(access_key, secret_key))]
    pub async fn new(
        endpoint: String,
        region: String,
        access_key: String,
        secret_key: String,
        bucket: String,
    ) -> Result<Self, StorageError> {
        info!("Initializing S3 client with endpoint: {}", endpoint);

        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "content-management-service",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region))
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(s3_config);

        // Verify bucket exists or create it
        match client.head_bucket().bucket(&bucket).send().await {
            Ok(_) => {
                info!("Bucket '{}' exists and is accessible", bucket);
            }
            Err(_) => {
                info!("Bucket '{}' not found, attempting to create", bucket);
                client
                    .create_bucket()
                    .bucket(&bucket)
                    .send()
                    .await
                    .map_err(|e| {
                        StorageError::AwsSdkError(format!("Failed to create bucket: {}", e))
                    })?;
                info!("Bucket '{}' created successfully", bucket);
            }
        }

        Ok(Self { client, bucket })
    }

    /// Uploads an object to S3/MinIO with server-side encryption (with retry)
    #[instrument(skip(self, data))]
    pub async fn put_object(
        &self,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<(), StorageError> {
        debug!(
            "Uploading object: key={}, size={}, content_type={}",
            key,
            data.len(),
            content_type
        );

        let key = key.to_string();
        let content_type = content_type.to_string();
        let client = self.client.clone();
        let bucket = self.bucket.clone();

        retry_storage::<_, _, (), StorageError>("s3_put_object", || {
            let key = key.clone();
            let content_type = content_type.clone();
            let data = data.clone();
            let client = client.clone();
            let bucket = bucket.clone();
            
            async move {
                client
                    .put_object()
                    .bucket(&bucket)
                    .key(&key)
                    .body(ByteStream::from(data))
                    .content_type(&content_type)
                    .server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::Aes256)
                    .send()
                    .await
                    .map_err(|e| StorageError::UploadFailed(e.to_string()))?;
                Ok::<(), StorageError>(())
            }
        })
        .await?;

        info!("Successfully uploaded object: {}", key);
        Ok(())
    }

    /// Downloads an object from S3/MinIO (with retry)
    #[instrument(skip(self))]
    pub async fn get_object(&self, key: &str) -> Result<Bytes, StorageError> {
        debug!("Downloading object: key={}", key);

        let key_str = key.to_string();
        let client = self.client.clone();
        let bucket = self.bucket.clone();

        let data = retry_storage::<_, _, Bytes, StorageError>("s3_get_object", || {
            let key = key_str.clone();
            let client = client.clone();
            let bucket = bucket.clone();
            
            async move {
                let response = client
                    .get_object()
                    .bucket(&bucket)
                    .key(&key)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Failed to get object {}: {}", key, e);
                        StorageError::ObjectNotFound(key.clone())
                    })?;

                let data = response
                    .body
                    .collect()
                    .await
                    .map_err(|e| StorageError::DownloadFailed(e.to_string()))?
                    .into_bytes();

                Ok::<Bytes, StorageError>(data)
            }
        })
        .await?;

        info!("Successfully downloaded object: {} ({} bytes)", key, data.len());
        Ok(data)
    }

    /// Deletes an object from S3/MinIO (with retry)
    #[instrument(skip(self))]
    pub async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        debug!("Deleting object: key={}", key);

        let key_str = key.to_string();
        let client = self.client.clone();
        let bucket = self.bucket.clone();

        retry_storage::<_, _, (), StorageError>("s3_delete_object", || {
            let key = key_str.clone();
            let client = client.clone();
            let bucket = bucket.clone();
            
            async move {
                client
                    .delete_object()
                    .bucket(&bucket)
                    .key(&key)
                    .send()
                    .await
                    .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;
                Ok::<(), StorageError>(())
            }
        })
        .await?;

        info!("Successfully deleted object: {}", key);
        Ok(())
    }

    /// Deletes multiple objects from S3/MinIO
    #[instrument(skip(self))]
    pub async fn delete_objects(&self, keys: Vec<String>) -> Result<(), StorageError> {
        if keys.is_empty() {
            return Ok(());
        }

        debug!("Deleting {} objects", keys.len());

        let objects: Vec<_> = keys
            .iter()
            .map(|key| {
                aws_sdk_s3::types::ObjectIdentifier::builder()
                    .key(key)
                    .build()
                    .expect("Failed to build ObjectIdentifier")
            })
            .collect();

        let delete = aws_sdk_s3::types::Delete::builder()
            .set_objects(Some(objects))
            .build()
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        self.client
            .delete_objects()
            .bucket(&self.bucket)
            .delete(delete)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        info!("Successfully deleted {} objects", keys.len());
        Ok(())
    }

    /// Generates a presigned URL for downloading an object
    #[instrument(skip(self))]
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expiration: Duration,
    ) -> Result<String, StorageError> {
        debug!(
            "Generating presigned URL: key={}, expiration={:?}",
            key, expiration
        );

        let presigning_config = PresigningConfig::expires_in(expiration).map_err(|e| {
            error!("Failed to create presigning config: {}", e);
            StorageError::PresignedUrlFailed(e.to_string())
        })?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                error!("Failed to generate presigned URL for {}: {}", key, e);
                StorageError::PresignedUrlFailed(e.to_string())
            })?;

        let url = presigned_request.uri().to_string();
        info!("Generated presigned URL for: {}", key);
        Ok(url)
    }

    /// Checks if an object exists
    #[instrument(skip(self))]
    pub async fn object_exists(&self, key: &str) -> Result<bool, StorageError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("404") || e.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    Err(StorageError::AwsSdkError(e.to_string()))
                }
            }
        }
    }

    /// Gets the size of an object in bytes
    #[instrument(skip(self))]
    pub async fn get_object_size(&self, key: &str) -> Result<i64, StorageError> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::ObjectNotFound(format!("{}: {}", key, e)))?;

        Ok(response.content_length().unwrap_or(0))
    }

    /// Lists objects with a given prefix
    #[instrument(skip(self))]
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        debug!("Listing objects with prefix: {}", prefix);

        let response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .map_err(|e| StorageError::AwsSdkError(e.to_string()))?;

        let keys: Vec<String> = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();

        info!("Found {} objects with prefix: {}", keys.len(), prefix);
        Ok(keys)
    }

    /// Copies an object from one key to another
    #[instrument(skip(self))]
    pub async fn copy_object(&self, source_key: &str, dest_key: &str) -> Result<(), StorageError> {
        debug!("Copying object: {} -> {}", source_key, dest_key);

        let copy_source = format!("{}/{}", self.bucket, source_key);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(dest_key)
            .server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::Aes256)
            .send()
            .await
            .map_err(|e| StorageError::AwsSdkError(e.to_string()))?;

        info!("Successfully copied object: {} -> {}", source_key, dest_key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::ObjectNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "Object not found: test.txt");

        let err = StorageError::InvalidStorageKey("invalid/key".to_string());
        assert_eq!(err.to_string(), "Invalid storage key: invalid/key");
    }
}
