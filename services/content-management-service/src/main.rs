mod config;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::Config::load()?;

    println!("Content Management Service");
    println!("Server: {}:{}", config.server.host, config.server.port);
    println!("gRPC Port: {}", config.server.grpc_port);
    println!("Metrics Port: {}", config.server.metrics_port);
    println!("Database: {}", mask_password(&config.database.url));
    println!("S3 Endpoint: {}", config.s3.endpoint);
    println!("ElasticSearch: {}", config.elasticsearch.url);
    println!("Redis: {}", config.redis.url);
    println!("Service Name: {}", config.observability.service_name);

    Ok(())
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}
