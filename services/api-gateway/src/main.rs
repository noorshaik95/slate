//! API Gateway main entry point.
//!
//! Minimal entry point that delegates to the application builder.

mod app;
mod auth;
mod config;
mod discovery;
mod docs;
mod grpc;
mod handlers;
mod health;
mod middleware;
mod observability;
mod proto;
mod router;
mod security;
mod shared;

use std::process;
use tracing::error;

#[tokio::main]
async fn main() {
    // Initialize the application and run it
    if let Err(e) = app::run().await {
        error!(error = %e, "Application failed to start");
        process::exit(1);
    }
}
