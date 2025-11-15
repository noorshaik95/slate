use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to upload object: {0}")]
    UploadFailed(String),

    #[error("Failed to download object: {0}")]
    DownloadFailed(String),

    #[error("Failed to delete object: {0}")]
    DeleteFailed(String),

    #[error("Failed to generate presigned URL: {0}")]
    PresignedUrlFailed(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Invalid storage key: {0}")]
    InvalidStorageKey(String),

    #[error("AWS SDK error: {0}")]
    AwsSdkError(String),
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>>
    for StorageError
{
    fn from(
        err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>,
    ) -> Self {
        StorageError::UploadFailed(err.to_string())
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>
    for StorageError
{
    fn from(
        err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    ) -> Self {
        StorageError::DownloadFailed(err.to_string())
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::delete_object::DeleteObjectError>>
    for StorageError
{
    fn from(
        err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::delete_object::DeleteObjectError>,
    ) -> Self {
        StorageError::DeleteFailed(err.to_string())
    }
}
