use thiserror::Error;

/// Error types for conversion operations
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Failed to read request body: {0}")]
    BodyReadError(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(String),

    #[error("Failed to serialize JSON: {0}")]
    JsonSerializeError(String),

    #[error("Invalid UTF-8 in response: {0}")]
    Utf8Error(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl From<serde_json::Error> for ConversionError {
    fn from(err: serde_json::Error) -> Self {
        ConversionError::JsonParseError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ConversionError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ConversionError::Utf8Error(err.to_string())
    }
}
