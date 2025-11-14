// Server validation error messages
pub const ERR_EMPTY_HOST: &str = "Server host cannot be empty";
pub const ERR_ZERO_PORT: &str = "Server port must be greater than 0";

// Service validation error messages
pub const ERR_NO_SERVICES: &str = "At least one service must be configured";
pub const ERR_EMPTY_SERVICE_NAME: &str = "Service name cannot be empty for service";
pub const ERR_EMPTY_SERVICE_ENDPOINT: &str = "Service endpoint cannot be empty for service";
pub const ERR_ZERO_SERVICE_TIMEOUT: &str = "Service timeout_ms must be greater than 0 for service";
pub const ERR_ZERO_SERVICE_POOL_SIZE: &str = "Service connection_pool_size must be greater than 0 for service";

// Auth validation error messages
pub const ERR_EMPTY_AUTH_ENDPOINT: &str = "Auth service endpoint cannot be empty";
pub const ERR_ZERO_AUTH_TIMEOUT: &str = "Auth timeout_ms must be greater than 0";

// Rate limit validation error messages
pub const ERR_ZERO_RATE_LIMIT_RPM: &str = "Rate limit requests_per_minute must be greater than 0 when enabled";
pub const ERR_ZERO_RATE_LIMIT_WINDOW: &str = "Rate limit window_seconds must be greater than 0 when enabled";

// Observability validation error messages
pub const ERR_EMPTY_TEMPO_ENDPOINT: &str = "Observability tempo_endpoint cannot be empty";
pub const ERR_EMPTY_OBSERVABILITY_SERVICE_NAME: &str = "Observability service_name cannot be empty";
