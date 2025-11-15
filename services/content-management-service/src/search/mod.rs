pub mod client;
pub mod errors;
pub mod service;

pub use client::ElasticsearchClient;
pub use errors::SearchError;
pub use service::{SearchService, UserRole};
