
use std::env;
use mongodb::sync::Client;

pub async fn init() -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // Runs the blocking sync client creation on a threadpool for blocking tasks.
    // The double `?` unwraps the JoinError and the mongodb::error::Error respectively.
    let client = tokio::task::spawn_blocking(move || Client::with_uri_str(&uri)).await??;
    Ok(client)
}