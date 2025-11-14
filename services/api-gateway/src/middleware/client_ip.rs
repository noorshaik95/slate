use axum::{extract::ConnectInfo, http::Request};
use std::net::{IpAddr, SocketAddr};
use tracing::{debug, warn};

/// Configuration for client IP extraction
#[derive(Clone, Debug)]
pub struct ClientIpConfig {
    /// List of trusted proxy IP addresses
    pub trusted_proxies: Vec<IpAddr>,
}

impl ClientIpConfig {
    /// Create a new client IP configuration
    pub fn new(trusted_proxies: Vec<IpAddr>) -> Self {
        Self { trusted_proxies }
    }

    /// Create a configuration with no trusted proxies (direct connections only)
    pub fn direct_only() -> Self {
        Self {
            trusted_proxies: Vec::new(),
        }
    }
}

/// Extractor for determining the real client IP address
/// 
/// This handles X-Forwarded-For headers securely by only trusting them
/// from configured proxy IPs to prevent IP spoofing attacks.
#[derive(Clone)]
pub struct ClientIpExtractor {
    config: ClientIpConfig,
}

impl ClientIpExtractor {
    /// Create a new client IP extractor with the given configuration
    pub fn new(config: ClientIpConfig) -> Self {
        Self { config }
    }

    /// Extract the real client IP from a request
    /// 
    /// # Security
    /// - Only trusts X-Forwarded-For from configured proxy IPs
    /// - Uses leftmost IP in X-Forwarded-For chain (original client)
    /// - Validates IP format before using
    /// - Falls back to connection remote address
    pub fn extract_client_ip<B>(&self, request: &Request<B>) -> IpAddr {
        // Get connection remote address
        let remote_addr = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip());

        // Check if request is from trusted proxy
        let from_trusted_proxy = remote_addr
            .map(|ip| self.is_trusted_proxy(&ip))
            .unwrap_or(false);

        if !from_trusted_proxy {
            // Not from trusted proxy, use remote address directly
            let ip = remote_addr.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
            debug!(
                remote_addr = %ip,
                from_trusted_proxy = false,
                "Using remote address as client IP"
            );
            return ip;
        }

        // Request is from trusted proxy, check X-Forwarded-For header
        if let Some(xff) = request.headers().get("x-forwarded-for") {
            if let Ok(xff_str) = xff.to_str() {
                // Get leftmost IP (original client) from comma-separated list
                if let Some(first_ip_str) = xff_str.split(',').next() {
                    let trimmed = first_ip_str.trim();
                    
                    // Validate and parse IP
                    match trimmed.parse::<IpAddr>() {
                        Ok(ip) => {
                            debug!(
                                x_forwarded_for = %xff_str,
                                extracted_ip = %ip,
                                remote_addr = ?remote_addr,
                                "Extracted client IP from X-Forwarded-For"
                            );
                            return ip;
                        }
                        Err(e) => {
                            warn!(
                                x_forwarded_for = %xff_str,
                                invalid_ip = %trimmed,
                                error = %e,
                                "Invalid IP in X-Forwarded-For, falling back to remote address"
                            );
                        }
                    }
                }
            }
        }

        // Fallback to remote address
        let ip = remote_addr.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
        debug!(
            remote_addr = %ip,
            "No valid X-Forwarded-For, using remote address"
        );
        ip
    }

    /// Check if an IP address is in the trusted proxy list
    fn is_trusted_proxy(&self, ip: &IpAddr) -> bool {
        self.config.trusted_proxies.contains(ip)
    }

    /// Get the list of trusted proxies
    pub fn trusted_proxies(&self) -> &[IpAddr] {
        &self.config.trusted_proxies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_direct_connection() {
        let config = ClientIpConfig::direct_only();
        let extractor = ClientIpExtractor::new(config);

        // Create a mock request
        let mut request = Request::builder()
            .uri("/")
            .body(())
            .unwrap();

        // Add ConnectInfo extension
        let addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(addr));

        let client_ip = extractor.extract_client_ip(&request);
        assert_eq!(client_ip, "192.168.1.100".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_xff_from_untrusted_proxy() {
        let config = ClientIpConfig::direct_only();
        let extractor = ClientIpExtractor::new(config);

        let mut request = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1, 192.168.1.1")
            .body(())
            .unwrap();

        let addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(addr));

        // Should ignore X-Forwarded-For from untrusted source
        let client_ip = extractor.extract_client_ip(&request);
        assert_eq!(client_ip, "192.168.1.100".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_xff_from_trusted_proxy() {
        let trusted_proxy: IpAddr = "192.168.1.100".parse().unwrap();
        let config = ClientIpConfig::new(vec![trusted_proxy]);
        let extractor = ClientIpExtractor::new(config);

        let mut request = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1, 192.168.1.1")
            .body(())
            .unwrap();

        let addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(addr));

        // Should use leftmost IP from X-Forwarded-For
        let client_ip = extractor.extract_client_ip(&request);
        assert_eq!(client_ip, "10.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_invalid_xff() {
        let trusted_proxy: IpAddr = "192.168.1.100".parse().unwrap();
        let config = ClientIpConfig::new(vec![trusted_proxy]);
        let extractor = ClientIpExtractor::new(config);

        let mut request = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "invalid-ip, 192.168.1.1")
            .body(())
            .unwrap();

        let addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(addr));

        // Should fall back to remote address on invalid IP
        let client_ip = extractor.extract_client_ip(&request);
        assert_eq!(client_ip, "192.168.1.100".parse::<IpAddr>().unwrap());
    }
}
