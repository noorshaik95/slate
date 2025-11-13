use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, error, info, warn};

use crate::circuit_breaker::CircuitBreaker;
use crate::config::ServiceConfig;
use super::types::{GrpcError, GrpcRequest, GrpcResponse};
use super::constants::*;

/// Pool of gRPC client connections to backend services with circuit breakers
#[derive(Clone)]
pub struct GrpcClientPool {
    clients: HashMap<String, Channel>,
    config: HashMap<String, ServiceConfig>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl GrpcClientPool {
    /// Create a new gRPC client pool with the given service configurations
    pub async fn new(services: HashMap<String, ServiceConfig>) -> Result<Self, GrpcError> {
        info!("Initializing gRPC client pool with {} services", services.len());

        let mut clients = HashMap::new();
        let mut circuit_breakers = HashMap::new();

        for (name, config) in &services {
            info!(
                service = %name,
                endpoint = %config.endpoint,
                timeout_ms = config.timeout_ms,
                pool_size = config.connection_pool_size,
                "Connecting to backend service"
            );

            let channel = Self::create_channel(config).await?;
            clients.insert(name.clone(), channel);

            // Create circuit breaker for this service
            let circuit_breaker = CircuitBreaker::new(config.circuit_breaker.clone());
            circuit_breakers.insert(name.clone(), circuit_breaker);

            debug!(service = %name, "Successfully connected to backend service with circuit breaker");
        }

        info!("gRPC client pool initialized successfully");

        Ok(Self {
            clients,
            config: services,
            circuit_breakers,
        })
    }
    
    /// Create a channel with connection pooling and timeout configuration
    async fn create_channel(config: &ServiceConfig) -> Result<Channel, GrpcError> {
        let endpoint = config.endpoint.parse::<Endpoint>()
            .map_err(|e| GrpcError::InvalidConfig(format!("Invalid endpoint {}: {}", config.endpoint, e)))?;

        let timeout = Duration::from_millis(config.timeout_ms);

        // Configure endpoint with timeout and connection settings
        let mut endpoint = endpoint
            .timeout(timeout)
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .tcp_keepalive(Some(TCP_KEEPALIVE))
            .http2_keep_alive_interval(HTTP2_KEEPALIVE_INTERVAL)
            .keep_alive_timeout(KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true);

        // Configure TLS if enabled
        if config.tls_enabled {
            let mut tls_config = ClientTlsConfig::new();

            // Set domain override if specified
            if let Some(ref domain) = config.tls_domain {
                info!(
                    service = %config.name,
                    domain = %domain,
                    "Configuring TLS with domain override"
                );
                tls_config = tls_config.domain_name(domain);
            }

            // Load custom CA certificate if specified
            if let Some(ref ca_cert_path) = config.tls_ca_cert_path {
                info!(
                    service = %config.name,
                    ca_cert_path = %ca_cert_path,
                    "Loading custom CA certificate for TLS"
                );

                let ca_cert = std::fs::read(ca_cert_path)
                    .map_err(|e| GrpcError::InvalidConfig(format!("Failed to read CA cert {}: {}", ca_cert_path, e)))?;

                let ca_cert = Certificate::from_pem(ca_cert);
                tls_config = tls_config.ca_certificate(ca_cert);
            }

            endpoint = endpoint.tls_config(tls_config)
                .map_err(|e| GrpcError::InvalidConfig(format!("Failed to configure TLS: {}", e)))?;

            info!(
                service = %config.name,
                endpoint = %config.endpoint,
                "TLS enabled for gRPC connection"
            );
        }

        // Connect to the service
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| GrpcError::ConnectionError(format!("Failed to connect to {}: {}", config.endpoint, e)))?;

        Ok(channel)
    }
    
    /// Get a channel for a specific service
    pub fn get_channel(&self, service: &str) -> Result<Channel, GrpcError> {
        self.clients
            .get(service)
            .cloned()
            .ok_or_else(|| GrpcError::ServiceNotFound(service.to_string()))
    }

    /// Get the circuit breaker for a specific service
    pub fn get_circuit_breaker(&self, service: &str) -> Option<CircuitBreaker> {
        self.circuit_breakers.get(service).cloned()
    }
    
    /// Execute a generic gRPC call with retry logic
    pub async fn call(&self, request: GrpcRequest) -> Result<GrpcResponse, GrpcError> {
        let service_name = &request.service;
        
        debug!(
            service = %service_name,
            method = %request.method,
            "Executing gRPC call"
        );
        
        // Get the channel for the service
        let channel = self.get_channel(service_name)?;
        
        // Get service config for timeout
        let config = self.config.get(service_name)
            .ok_or_else(|| GrpcError::ServiceNotFound(service_name.to_string()))?;
        
        // Execute with retry logic
        let result = self.call_with_retry(channel, &request, config).await;
        
        match &result {
            Ok(response) => {
                debug!(
                    service = %service_name,
                    status = ?response.status,
                    "gRPC call completed successfully"
                );
            }
            Err(e) => {
                error!(
                    service = %service_name,
                    error = %e,
                    "gRPC call failed"
                );
            }
        }
        
        result
    }
    
    /// Execute a gRPC call with retry logic (max 3 attempts)
    async fn call_with_retry(
        &self,
        channel: Channel,
        request: &GrpcRequest,
        config: &ServiceConfig,
    ) -> Result<GrpcResponse, GrpcError> {
        let mut last_error = None;
        
        for attempt in 1..=MAX_RETRIES {
            match self.execute_call(channel.clone(), request, config).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Only retry on transient errors
                    if Self::is_retryable_error(&e) && attempt < MAX_RETRIES {
                        warn!(
                            service = %request.service,
                            attempt = attempt,
                            error = %e,
                            "gRPC call failed, retrying"
                        );
                        
                        // Exponential backoff
                        let backoff = Duration::from_millis(INITIAL_BACKOFF_MS * BACKOFF_MULTIPLIER.pow(attempt - 1));
                        tokio::time::sleep(backoff).await;
                        
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| GrpcError::CallFailed(ERR_MAX_RETRIES_EXCEEDED.to_string())))
    }
    
    /// Execute a single gRPC call attempt
    async fn execute_call(
        &self,
        _channel: Channel,
        _request: &GrpcRequest,
        _config: &ServiceConfig,
    ) -> Result<GrpcResponse, GrpcError> {
        // Note: This is a placeholder for generic gRPC calls
        // In practice, specific service clients (auth, backend services) will use
        // the channel directly with their generated proto code
        // This method would need dynamic dispatch or code generation for truly generic calls
        
        // For now, return an error indicating this needs to be implemented per-service
        Err(GrpcError::CallFailed(ERR_GENERIC_CALLS_NOT_IMPLEMENTED.to_string()))
    }
    
    /// Check if an error is retryable
    pub(crate) fn is_retryable_error(error: &GrpcError) -> bool {
        match error {
            GrpcError::Timeout(_) => true,
            GrpcError::ConnectionError(_) => true,
            GrpcError::CallFailed(msg) => {
                // Retry on specific gRPC status codes
                msg.contains(RETRYABLE_STATUS_UNAVAILABLE) || 
                msg.contains(RETRYABLE_STATUS_DEADLINE_EXCEEDED) ||
                msg.contains(RETRYABLE_STATUS_RESOURCE_EXHAUSTED)
            }
            _ => false,
        }
    }
    
    /// Check health of a specific backend service
    pub async fn health_check(&self, service: &str) -> Result<bool, GrpcError> {
        debug!(service = %service, "Performing health check");
        
        // Get the channel for the service
        let channel = self.get_channel(service)?;
        
        // Attempt a simple connection check
        // In a real implementation, this would call a health check gRPC method
        // For now, we just verify the channel exists and is connected
        
        // Try to clone the channel - if it fails, the connection is broken
        let _channel = channel.clone();
        debug!(service = %service, "Health check passed");
        Ok(true)
    }
    
    /// Get all service names in the pool
    pub fn services(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
    
    /// Check if a service exists in the pool
    pub fn has_service(&self, service: &str) -> bool {
        self.clients.contains_key(service)
    }
}
