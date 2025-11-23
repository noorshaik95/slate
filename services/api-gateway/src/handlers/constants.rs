/// System paths that should skip gateway processing
pub const SYSTEM_PATH_HEALTH: &str = "/health";
pub const SYSTEM_PATH_METRICS: &str = "/metrics";

/// Maximum request body size in bytes (10 MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Error codes for gateway errors
pub const ERR_CODE_ROUTE_NOT_FOUND: &str = "ROUTE_NOT_FOUND";
pub const ERR_CODE_SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";
pub const ERR_CODE_RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";
pub const ERR_CODE_CONVERSION_ERROR: &str = "CONVERSION_ERROR";
pub const ERR_CODE_BACKEND_ERROR: &str = "BACKEND_ERROR";
pub const ERR_CODE_TIMEOUT: &str = "TIMEOUT";
pub const ERR_CODE_NOT_FOUND: &str = "NOT_FOUND";
pub const ERR_CODE_INTERNAL_ERROR: &str = "INTERNAL_ERROR";

/// Special metadata keys for auth context
pub const METADATA_AUTH_USER_ID: &str = "_auth_user_id";
pub const METADATA_AUTH_ROLES: &str = "_auth_roles";
#[allow(dead_code)]
pub const METADATA_TRACE: &str = "_trace_metadata";

/// Error messages
pub const ERR_MSG_READ_BODY: &str = "Failed to read request body";
pub const ERR_MSG_INVALID_JSON: &str = "Invalid JSON body";
pub const ERR_MSG_SERIALIZE_PAYLOAD: &str = "Failed to serialize payload";
pub const ERR_MSG_PARSE_GRPC_RESPONSE: &str = "Failed to parse gRPC response";
#[allow(dead_code)]
pub const ERR_MSG_SERIALIZE_MOCK: &str = "Failed to serialize mock response";
#[allow(dead_code)]
pub const ERR_MSG_PLACEHOLDER_GRPC: &str =
    "Using placeholder gRPC call - actual implementation needed";
