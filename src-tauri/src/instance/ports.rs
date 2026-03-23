use std::net::TcpListener;

use crate::error::{OutClawError, Result};
use crate::instance::InstanceConfig;

/// Default gateway port
const DEFAULT_GATEWAY_PORT: u16 = 18789;
/// Default bridge port
const DEFAULT_BRIDGE_PORT: u16 = 18790;
/// Minimum allowed port
const MIN_PORT: u16 = 1024;
/// Maximum allowed port
const MAX_PORT: u16 = 65535;

/// Allocate a pair of ports (gateway, bridge) for a new instance
///
/// Starts from default ports and increments until finding an available pair.
/// Checks both against existing instances and OS-level port availability.
///
/// # Arguments
/// * `existing_instances` - List of existing instances to check against
///
/// # Returns
/// A tuple of (gateway_port, bridge_port)
pub fn allocate_ports(existing_instances: &[InstanceConfig]) -> Result<(u16, u16)> {
    let mut gateway_port = DEFAULT_GATEWAY_PORT;
    let mut bridge_port = DEFAULT_BRIDGE_PORT;

    // Collect all ports already in use by existing instances
    let used_gateway_ports: Vec<u16> = existing_instances.iter().map(|i| i.gateway_port).collect();
    let used_bridge_ports: Vec<u16> = existing_instances.iter().map(|i| i.bridge_port).collect();

    // Find available port pair
    for _ in 0..1000 {
        let gateway_available =
            !used_gateway_ports.contains(&gateway_port) && is_port_available(gateway_port);
        let bridge_available =
            !used_bridge_ports.contains(&bridge_port) && is_port_available(bridge_port);

        if gateway_available && bridge_available {
            return Ok((gateway_port, bridge_port));
        }

        // Increment both ports
        gateway_port += 2;
        bridge_port += 2;

        // Wrap around if we exceed max port
        if gateway_port > MAX_PORT - 2 {
            return Err(OutClawError::Other(
                "No available port pairs found".to_string(),
            ));
        }
    }

    Err(OutClawError::Other(
        "Could not allocate ports after 1000 attempts".to_string(),
    ))
}

/// Validate a specific port for use by an instance
///
/// Checks:
/// 1. Port is in valid range (1024-65535)
/// 2. Port is not already used by another instance (excluding self if editing)
/// 3. Port is available at OS level (can bind)
///
/// # Arguments
/// * `port` - The port to validate
/// * `instance_id` - The instance ID (None for new instance, Some for editing existing)
/// * `existing_instances` - List of existing instances to check against
///
/// # Returns
/// Ok(()) if port is valid, Err with specific message otherwise
pub fn validate_port(
    port: u16,
    instance_id: Option<&str>,
    existing_instances: &[InstanceConfig],
) -> Result<()> {
    // Check range - u16 max is 65535, so we only need to check minimum
    if port < MIN_PORT {
        return Err(OutClawError::PortOutOfRange(port));
    }

    // Check against other instances
    for instance in existing_instances {
        // Skip self when editing
        if let Some(id) = instance_id {
            if instance.id == id {
                continue;
            }
        }

        if instance.gateway_port == port || instance.bridge_port == port {
            return Err(OutClawError::PortInUse(port));
        }
    }

    // Check OS-level availability
    if !is_port_available(port) {
        return Err(OutClawError::PortInUse(port));
    }

    Ok(())
}

/// Check if a port is available at the OS level
///
/// Attempts to bind to the port. If successful, the port is available.
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{GatewayBind, InstanceConfig};
    use chrono::Utc;

    fn create_test_instance(id: &str, gateway_port: u16, bridge_port: u16) -> InstanceConfig {
        InstanceConfig {
            id: id.to_string(),
            name: "Test".to_string(),
            openclaw_version: "v0.1.0".to_string(),
            container_id: "ct_test".to_string(),
            gateway_port,
            bridge_port,
            gateway_bind: GatewayBind::Loopback,
            gateway_token: "test_token".to_string(),
            timezone: "UTC".to_string(),
            install_browser: false,
            apt_packages: "".to_string(),
            extensions: "".to_string(),
            home_volume: "".to_string(),
            extra_mounts: "".to_string(),
            allow_insecure_ws: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_allocate_ports_empty() {
        let instances = vec![];
        let (gateway, bridge) = allocate_ports(&instances).unwrap();

        // Should get sequential ports (gateway + bridge)
        assert_eq!(bridge, gateway + 1);
        // Should be in valid range
        assert!(gateway >= MIN_PORT);
    }

    #[test]
    fn test_allocate_ports_with_existing() {
        // Create instance using default ports
        let instances = vec![create_test_instance("ec_1", 18789, 18790)];

        let (gateway, bridge) = allocate_ports(&instances).unwrap();

        // Should skip the used ports (but actual values depend on OS port availability)
        assert_ne!(gateway, 18789);
        assert_ne!(bridge, 18790);
        // Bridge should be gateway + 1
        assert_eq!(bridge, gateway + 1);
        // Should be in valid range
        assert!(gateway >= MIN_PORT);
    }

    #[test]
    fn test_validate_port_valid() {
        let instances = vec![];
        // Use a high port that's unlikely to be in use
        let test_port = 54321;
        let result = validate_port(test_port, None, &instances);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_port_out_of_range() {
        let instances = vec![];

        let result = validate_port(80, None, &instances);
        assert!(matches!(result, Err(OutClawError::PortOutOfRange(80))));

        // Port 65535+ can't be tested directly since u16 max is 65535
        // The validation will catch values above 65535 at compile time
    }

    #[test]
    fn test_validate_port_used_by_instance() {
        let instances = vec![create_test_instance("ec_1", 18789, 18790)];

        let result = validate_port(18789, None, &instances);
        assert!(matches!(result, Err(OutClawError::PortInUse(18789))));
    }

    #[test]
    fn test_validate_port_excludes_self() {
        // Use a high port that's unlikely to be in use
        let test_port = 54321;
        let instances = vec![create_test_instance("ec_1", test_port, test_port + 1)];

        // When instance_id matches, should allow the port (assuming port is free at OS level)
        let result = validate_port(test_port, Some("ec_1"), &instances);
        // This might still fail if port is actually in use by system, but not due to our check
        if let Err(OutClawError::PortInUse(_)) = result {
            panic!("Port should be allowed for same instance");
        }
    }
}
