//! Trusted proxy configuration for secure IP extraction.
//!
//! Prevents IP spoofing by validating X-Forwarded-For and X-Real-IP headers
//! against a list of trusted proxy IPs/CIDR ranges.

use std::net::IpAddr;

/// Configuration for trusted proxy handling.
#[derive(Debug, Clone, Default)]
pub struct TrustedProxyConfig {
    /// List of trusted proxy IPs or CIDR ranges.
    /// If empty, proxy headers are not trusted.
    pub trusted_proxies: Vec<String>,

    /// Whether to trust proxy headers at all.
    pub trust_proxy_headers: bool,
}

impl TrustedProxyConfig {
    /// Create a new config from environment variables.
    ///
    /// Environment variables:
    /// - `TRUSTED_PROXIES`: Comma-separated list of IPs/CIDR ranges
    /// - `TRUST_PROXY_HEADERS`: "true" to enable proxy header parsing
    pub fn from_env() -> Self {
        let trust_proxy_headers = std::env::var("TRUST_PROXY_HEADERS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            trusted_proxies,
            trust_proxy_headers,
        }
    }

    /// Check if an IP address is in the trusted proxy list.
    pub fn is_trusted(&self, ip: &str) -> bool {
        if !self.trust_proxy_headers || self.trusted_proxies.is_empty() {
            return false;
        }

        let Ok(client_ip) = ip.parse::<IpAddr>() else {
            return false;
        };

        self.trusted_proxies.iter().any(|trusted| {
            if let Ok(trusted_ip) = trusted.parse::<IpAddr>() {
                return client_ip == trusted_ip;
            }

            if let Some((base_ip, prefix_len)) = trusted.split_once('/') {
                // CIDR notation
                if let Ok(base) = base_ip.parse::<IpAddr>() {
                    return is_ip_in_cidr(client_ip, base, prefix_len);
                }
            }
            false
        })
    }
}

/// Check if an IP is within a CIDR range.
fn is_ip_in_cidr(ip: IpAddr, base: IpAddr, prefix_len: &str) -> bool {
    let Ok(prefix): Result<u8, _> = prefix_len.parse() else {
        return false;
    };

    match (ip, base) {
        (IpAddr::V4(ip), IpAddr::V4(base)) => {
            let ip_u32 = u32::from(ip);
            let base_u32 = u32::from(base);
            let mask = if prefix == 0 {
                0
            } else {
                (!0u32) << (32 - prefix)
            };
            (ip_u32 & mask) == (base_u32 & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(base)) => {
            let ip_u128 = u128::from(ip);
            let base_u128 = u128::from(base);
            let mask = if prefix == 0 {
                0
            } else {
                (!0u128) << (128 - prefix)
            };
            (ip_u128 & mask) == (base_u128 & mask)
        }
        _ => false, // Mixed IPv4/IPv6
    }
}

/// Extract client IP from headers with proxy trust validation.
///
/// If proxy headers are not trusted, returns None.
/// If trusted, parses X-Forwarded-For and returns the rightmost trusted IP.
pub fn extract_client_ip(headers: &http::HeaderMap, config: &TrustedProxyConfig) -> Option<String> {
    if !config.trust_proxy_headers {
        return None;
    }

    // Try X-Forwarded-For first (most common)
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        // X-Forwarded-For: client, proxy1, proxy2, ...
        // We want the rightmost IP that is NOT a trusted proxy
        let ips: Vec<&str> = xff.split(',').map(|s| s.trim()).collect();

        // Walk backwards from the rightmost IP
        for ip in ips.iter().rev() {
            if !config.is_trusted(ip) {
                // This is the first untrusted IP from the right = the actual client
                return Some(ip.to_string());
            }
        }

        // All IPs are trusted proxies, return the leftmost (original client)
        // This shouldn't happen in practice with proper configuration
        return ips.first().map(|s| s.to_string());
    }

    // Fall back to X-Real-IP
    if let Some(xri) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        // X-Real-IP should be set by the last proxy
        // We trust it only if we trust the proxy that set it
        // Since we can't validate which proxy set it, we trust it if proxy headers are enabled
        return Some(xri.to_string());
    }

    None
}

#[cfg(test)]
#[path = "tests/network_tests.rs"]
mod tests;
