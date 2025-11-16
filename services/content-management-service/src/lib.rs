pub mod config;
pub mod models;
pub mod db;
pub mod content;
pub mod grpc;
pub mod observability;
pub mod storage;
pub mod upload;
pub mod transcoding;
pub mod streaming;
pub mod progress;
pub mod analytics;
pub mod search;
pub mod download;
pub mod error;
pub mod retry;
pub mod circuit_breaker;
pub mod health;
pub mod health_server;

// Include generated protobuf code
pub mod proto {
    pub mod content {
        tonic::include_proto!("content");
    }
}
