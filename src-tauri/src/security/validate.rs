use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

use crate::error::{OutClawError, Result};
use crate::instance::InstanceConfig;

/// Sensitive host paths that should never be used as mount sources
const BLOCKED_MOUNT_SOURCES: &[&str] = &[
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/sudoers.d",
    "/root/.ssh",
    "/etc/ssl/private",
    "/proc",
    "/sys",
    "/dev",
    "/boot",
];

/// Validate a URL for SSRF safety.
/// Only allows http/https schemes and blocks private/loopback IP ranges.
pub fn validate_url(url: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // Only allow http and https schemes
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(OutClawError::SsrfBlocked(format!(
            "Only http:// and https:// URLs are allowed, got: '{}'",
            trimmed
        )));
    }

    // Parse the URL to extract the host
    let host = extract_host(trimmed)?;

    // Try to parse as IP address directly
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            return Err(OutClawError::SsrfBlocked(format!(
                "Private/loopback IP addresses are blocked: '{}'",
                ip
            )));
        }
    }

    // Try DNS resolution to catch hostname-based SSRF
    if let Ok(addrs) = format!("{}:80", host).to_socket_addrs() {
        for addr in addrs {
            if is_private_ip(&addr.ip()) {
                return Err(OutClawError::SsrfBlocked(format!(
                    "Hostname '{}' resolves to private/loopback address: '{}'",
                    host,
                    addr.ip()
                )));
            }
        }
    }

    Ok(())
}

/// Extract the host portion from a URL
fn extract_host(url: &str) -> Result<String> {
    // Strip scheme
    let without_scheme = if url.starts_with("https://") {
        &url[8..]
    } else if url.starts_with("http://") {
        &url[7..]
    } else {
        url
    };

    // Strip path, query, fragment
    let host_port = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme);

    // Strip port
    let host = if host_port.starts_with('[') {
        // IPv6 address in brackets
        host_port
            .split(']')
            .next()
            .map(|s| &s[1..])
            .unwrap_or(host_port)
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };

    if host.is_empty() {
        return Err(OutClawError::SsrfBlocked(
            "URL has empty host".to_string(),
        ));
    }

    Ok(host.to_string())
}

/// Check if an IP address is private, loopback, or link-local
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_ipv4(v4),
        IpAddr::V6(v6) => is_private_ipv6(v6),
    }
}

fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();

    // Loopback: 127.0.0.0/8
    if octets[0] == 127 {
        return true;
    }
    // Private: 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }
    // Private: 172.16.0.0/12
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return true;
    }
    // Private: 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }
    // Link-local: 169.254.0.0/16
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }
    // Current network: 0.0.0.0/8
    if octets[0] == 0 {
        return true;
    }

    false
}

fn is_private_ipv6(ip: &Ipv6Addr) -> bool {
    // Loopback: ::1
    if ip == &Ipv6Addr::LOCALHOST {
        return true;
    }
    // Unspecified: ::
    if ip == &Ipv6Addr::UNSPECIFIED {
        return true;
    }

    let segments = ip.segments();
    // Link-local: fe80::/10
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    // Unique local: fc00::/7
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }
    // IPv4-mapped IPv6: ::ffff:0:0/96
    if segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff {
        let v4 = Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            segments[6] as u8,
            (segments[7] >> 8) as u8,
            segments[7] as u8,
        );
        return is_private_ipv4(&v4);
    }

    false
}

/// Validate a mount source path against the blocklist.
pub fn validate_mount_source(path: &str) -> Result<()> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // Reject null bytes
    if trimmed.contains('\0') {
        return Err(OutClawError::InputValidation(
            "Mount source path must not contain null bytes".to_string(),
        ));
    }

    // Reject path traversal
    if trimmed.contains("..") {
        return Err(OutClawError::InputValidation(format!(
            "Path traversal (..) is not allowed in mount source: '{}'",
            trimmed
        )));
    }

    // Check against blocklist
    for blocked in BLOCKED_MOUNT_SOURCES {
        if trimmed == *blocked || trimmed.starts_with(&format!("{}/", blocked)) {
            return Err(OutClawError::InputValidation(format!(
                "Mounting sensitive path is blocked: '{}'",
                trimmed
            )));
        }
    }

    Ok(())
}

/// Validate an instance configuration before applying it to Docker.
/// This is a pre-flight check run before compose/env generation.
pub fn validate_config_lifecycle(config: &InstanceConfig) -> Result<()> {
    // Validate token is present and has sufficient entropy
    if config.gateway_token.len() < 32 {
        return Err(OutClawError::InvalidConfig(
            "Gateway token must be at least 32 characters".to_string(),
        ));
    }

    // Validate port range
    if config.gateway_port < 1024 {
        return Err(OutClawError::PortOutOfRange(config.gateway_port));
    }
    if config.bridge_port < 1024 {
        return Err(OutClawError::PortOutOfRange(config.bridge_port));
    }

    // Validate ports are different
    if config.gateway_port == config.bridge_port {
        return Err(OutClawError::InvalidConfig(
            "Gateway port and bridge port must be different".to_string(),
        ));
    }

    // Validate ID format
    if config.id.is_empty() || config.container_id.is_empty() {
        return Err(OutClawError::InvalidConfig(
            "Instance ID and container ID must not be empty".to_string(),
        ));
    }

    // Validate version is set
    if config.openclaw_version.is_empty() {
        return Err(OutClawError::InvalidConfig(
            "OpenClaw version must be specified".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- URL validation tests ---

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://api.openai.com/v1").is_ok());
        assert!(validate_url("").is_ok());
    }

    #[test]
    fn test_validate_url_blocked_schemes() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("gopher://evil.com").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_blocked_private_ips() {
        assert!(validate_url("http://127.0.0.1").is_err());
        assert!(validate_url("http://10.0.0.1").is_err());
        assert!(validate_url("http://172.16.0.1").is_err());
        assert!(validate_url("http://192.168.1.1").is_err());
        assert!(validate_url("http://169.254.1.1").is_err());
        assert!(validate_url("http://0.0.0.0").is_err());
    }

    #[test]
    fn test_validate_url_blocked_ipv6() {
        assert!(validate_url("http://[::1]").is_err());
    }

    // --- Mount source validation tests ---

    #[test]
    fn test_validate_mount_source_valid() {
        assert!(validate_mount_source("/home/user/data").is_ok());
        assert!(validate_mount_source("").is_ok());
    }

    #[test]
    fn test_validate_mount_source_blocked() {
        assert!(validate_mount_source("/var/run/docker.sock").is_err());
        assert!(validate_mount_source("/proc").is_err());
        assert!(validate_mount_source("/proc/1/mem").is_err());
        assert!(validate_mount_source("/etc/shadow").is_err());
    }

    #[test]
    fn test_validate_mount_source_traversal() {
        assert!(validate_mount_source("/home/user/../etc/shadow").is_err());
    }

    // --- Config lifecycle tests ---

    #[test]
    fn test_validate_config_lifecycle_valid() {
        let config = create_test_config();
        assert!(validate_config_lifecycle(&config).is_ok());
    }

    #[test]
    fn test_validate_config_lifecycle_short_token() {
        let mut config = create_test_config();
        config.gateway_token = "short".to_string();
        assert!(validate_config_lifecycle(&config).is_err());
    }

    #[test]
    fn test_validate_config_lifecycle_same_ports() {
        let mut config = create_test_config();
        config.bridge_port = config.gateway_port;
        assert!(validate_config_lifecycle(&config).is_err());
    }

    fn create_test_config() -> InstanceConfig {
        use chrono::Utc;
        use crate::instance::GatewayBind;

        InstanceConfig {
            id: "ec_test123".to_string(),
            name: "Test Instance".to_string(),
            openclaw_version: "v0.42.1".to_string(),
            container_id: "ct_abc456".to_string(),
            gateway_port: 18789,
            bridge_port: 18790,
            gateway_bind: GatewayBind::Loopback,
            gateway_token: "0123456789abcdef".repeat(4),
            timezone: "America/Toronto".to_string(),
            install_browser: false,
            apt_packages: "".to_string(),
            extensions: "".to_string(),
            home_volume: "".to_string(),
            extra_mounts: "".to_string(),
            allow_insecure_ws: false,
            security_policy: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
