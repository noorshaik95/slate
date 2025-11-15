pub mod errors;
pub mod manager;

pub use errors::DownloadError;
pub use manager::DownloadManager;

pub type Result<T> = std::result::Result<T, DownloadError>;
