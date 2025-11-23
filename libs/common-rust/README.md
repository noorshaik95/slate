# Common Rust Library

Shared utilities for Rust microservices in the Slate platform.

## Features

- **Circuit Breaker**: Fault tolerance for external service calls
- **Rate Limiting**: Request throttling with sliding window algorithm
- **Health Checks**: Service health monitoring and readiness checks
- **Retry Logic**: Exponential backoff retry with configurable presets
- **Error Handling**: Standardized error responses for HTTP and gRPC
- **Observability**: Tracing, logging, and distributed trace propagation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
common-rust = { path = "../../libs/common-rust", features = ["full"] }
```

### Feature Flags

- `default`: Includes `observability` feature
- `observability`: OpenTelemetry tracing and logging utilities
- `grpc`: gRPC interceptors and trace propagation
- `http`: HTTP utilities and Axum integration
- `full`: All features enabled

## Quick Start

### Circuit Breaker

```rust
use common_rust::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

let config = CircuitBreakerConfig {
    failure_threshold: 5,
    success_threshold: 2,
    timeout_seconds: 30,
};

let breaker = CircuitBreaker::with_name("my-service".to_string(), config);

match breaker.call(|| async { external_service_call().await }).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) => eprintln!("Circuit breaker error: {}", e),
}
```

### Rate Limiting

```rust
use common_rust::rate_limit::{IpRateLimiter, RateLimitConfig};
use std::net::IpAddr;

let config = RateLimitConfig {
    enabled: true,
    requests_per_minute: 60,
    window_seconds: 60,
};

let limiter = IpRateLimiter::new(config, 10_000);
let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

match limiter.check_rate_limit(client_ip).await {
    Ok(()) => println!("Request allowed"),
    Err(e) => eprintln!("Rate limit exceeded: {}", e),
}
```

### Health Checks

```rust
use common_rust::health::{HealthChecker, HealthStatus};

let mut checker = HealthChecker::new("my-service");

// Liveness check (always healthy if service is running)
let liveness = checker.liveness().await;

// Readiness check (checks all components)
let readiness = checker.readiness().await;
```

### Retry Logic

```rust
use common_rust::retry::{retry_operation, OperationType};

let result = retry_operation(
    OperationType::Database,
    || async { database_query().await }
).await;
```

### Error Responses

```rust
use common_rust::error::ErrorResponse;

let error = ErrorResponse::new(
    "VALIDATION_ERROR",
    "Invalid input parameters",
    "trace-id-123"
);

// Convert to HTTP response (with http feature)
let http_response = error.to_http_response(400);

// Convert to gRPC status (with grpc feature)
let grpc_status = error.to_grpc_status(tonic::Code::InvalidArgument);
```

## Documentation

Run `cargo doc --open` to view full API documentation.

## License

Internal use only - Slate platform.
