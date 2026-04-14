use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::Result;
use crate::instance::{GatewayBind, InstanceConfig};
use crate::security::{validate_config_lifecycle, NetworkPreset, SeccompProfile};

/// Generate docker-compose.yml content for an instance
pub fn generate_compose(config: &InstanceConfig) -> Result<String> {
    // Pre-flight validation before generating Docker config
    validate_config_lifecycle(config)?;

    let project_name = format!("outclaw-{}", config.container_id);
    let gateway_service = format!("{}-gateway", project_name);
    let cli_service = format!("{}-cli", project_name);
    let image_name = format!("{}:latest", project_name);

    // Determine port binding based on gateway_bind setting
    let gateway_port_binding = match config.gateway_bind {
        GatewayBind::Loopback => format!("127.0.0.1:{}", config.gateway_port),
        GatewayBind::Lan => format!("{}", config.gateway_port),
    };
    let bridge_port_binding = format!("127.0.0.1:{}", config.bridge_port);

    // Build labels
    let mut labels = HashMap::new();
    labels.insert("outclaw.container".to_string(), config.container_id.clone());
    labels.insert("outclaw.instance".to_string(), config.id.clone());

    // Get paths
    let config_path = config.config_path();
    let workspace_path = config.workspace_path();

    // Build security options from policy
    let security = &config.security_policy;
    let cap_drop = if security.sandbox.drop_all_capabilities {
        Some(vec!["ALL".to_string()])
    } else {
        None
    };
    let cap_add = if !security.sandbox.added_capabilities.is_empty() {
        Some(security.sandbox.added_capabilities.clone())
    } else {
        None
    };

    let mut security_opts = Vec::new();
    if security.sandbox.no_new_privileges {
        security_opts.push("no-new-privileges:true".to_string());
    }
    match &security.sandbox.seccomp_profile {
        SeccompProfile::Strict => {
            // Path to the seccomp profile JSON placed alongside docker-compose.yml
            security_opts.push("seccomp=seccomp-profile.json".to_string());
        }
        SeccompProfile::Unconfined => {
            security_opts.push("seccomp=unconfined".to_string());
        }
        SeccompProfile::Default => {
            // Docker's default seccomp is applied automatically
        }
    }
    let security_opt = if security_opts.is_empty() {
        None
    } else {
        Some(security_opts)
    };

    let pids_limit = security.sandbox.pids_limit;

    // Resource limits via deploy config
    let deploy = {
        let has_memory = security.sandbox.memory_limit.is_some();
        let has_cpu = security.sandbox.cpu_limit.is_some();
        if has_memory || has_cpu {
            Some(DeployConfig {
                resources: Some(ResourceConfig {
                    limits: Some(ResourceLimits {
                        memory: security.sandbox.memory_limit.clone(),
                        cpus: security.sandbox.cpu_limit.map(|c| format!("{:.1}", c)),
                    }),
                }),
            })
        } else {
            None
        }
    };

    // Network mode from network policy
    let network_mode = match security.network.preset {
        NetworkPreset::Strict => Some("none".to_string()),
        _ => None,
    };

    let compose = DockerCompose {
        version: "3.8",
        services: {
            let mut services = HashMap::new();

            // Gateway service
            services.insert(
                gateway_service,
                Service {
                    image: image_name.clone(),
                    container_name: Some(format!("outclaw-{}-gateway", config.container_id)),
                    restart: Some("unless-stopped".to_string()),
                    ports: vec![
                        format!("{}:18789", gateway_port_binding),
                        format!("{}:18790", bridge_port_binding),
                    ],
                    volumes: vec![
                        format!("{}:/home/node/.openclaw", config_path.display()),
                        format!("{}:/home/node/workspace", workspace_path.display()),
                    ],
                    environment: vec![
                        "OPENCLAW_CONFIG_DIR=/home/node/.openclaw".to_string(),
                        "OPENCLAW_WORKSPACE_DIR=/home/node/workspace".to_string(),
                        format!("OPENCLAW_GATEWAY_TOKEN={}", config.gateway_token),
                        format!("OPENCLAW_TZ={}", config.timezone),
                    ],
                    labels: Some(labels.clone()),
                    cap_drop: cap_drop.clone(),
                    cap_add: cap_add.clone(),
                    security_opt: security_opt.clone(),
                    pids_limit,
                    network_mode: network_mode.clone(),
                    deploy: deploy.clone(),
                },
            );

            // CLI service (for running commands)
            services.insert(
                cli_service,
                Service {
                    image: image_name,
                    container_name: Some(format!("outclaw-{}-cli", config.container_id)),
                    restart: None,
                    ports: vec![],
                    volumes: vec![
                        format!("{}:/home/node/.openclaw", config_path.display()),
                        format!("{}:/home/node/workspace", workspace_path.display()),
                    ],
                    environment: vec![
                        "OPENCLAW_CONFIG_DIR=/home/node/.openclaw".to_string(),
                        "OPENCLAW_WORKSPACE_DIR=/home/node/workspace".to_string(),
                        format!("OPENCLAW_GATEWAY_TOKEN={}", config.gateway_token),
                        format!("OPENCLAW_TZ={}", config.timezone),
                    ],
                    labels: Some(labels),
                    cap_drop,
                    cap_add,
                    security_opt,
                    pids_limit,
                    network_mode,
                    deploy,
                },
            );

            services
        },
    };

    let yaml = serde_yaml::to_string(&compose)?;
    debug!("Generated compose file:\n{}", yaml);
    Ok(yaml)
}

/// Docker Compose structure
#[derive(Debug, Serialize, Deserialize)]
struct DockerCompose {
    version: &'static str,
    services: HashMap<String, Service>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Service {
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    restart: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volumes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    environment: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cap_drop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cap_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    security_opt: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pids_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deploy: Option<DeployConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct DeployConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<ResourceConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ResourceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    limits: Option<ResourceLimits>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ResourceLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpus: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_config() -> InstanceConfig {
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

    #[test]
    fn test_generate_compose() {
        let config = create_test_config();
        let compose = generate_compose(&config).unwrap();

        // Check key elements - version may be quoted differently by serde_yaml
        assert!(compose.contains("version:") && compose.contains("3.8"));
        assert!(compose.contains("outclaw-ct_abc456-gateway"));
        assert!(compose.contains("outclaw-ct_abc456-cli"));
        assert!(compose.contains("18789:18789"));
        assert!(compose.contains("18790:18790"));
        assert!(compose.contains("outclaw.container"));
        assert!(compose.contains("outclaw.instance"));
    }

    #[test]
    fn test_compose_lan_binding() {
        let mut config = create_test_config();
        config.gateway_bind = GatewayBind::Lan;

        let compose = generate_compose(&config).unwrap();

        // Should not have 127.0.0.1 prefix for gateway port in LAN mode
        assert!(compose.contains("18789:18789"));
        assert!(!compose.contains("127.0.0.1:18789:18789"));
    }

    #[test]
    fn test_compose_default_security_drops_caps() {
        let config = create_test_config();
        let compose = generate_compose(&config).unwrap();

        // Default policy should drop all capabilities
        assert!(compose.contains("cap_drop"));
        assert!(compose.contains("ALL"));
        // Default policy should set no-new-privileges
        assert!(compose.contains("no-new-privileges:true"));
        // Default policy should set pids_limit
        assert!(compose.contains("pids_limit: 256"));
    }

    #[test]
    fn test_compose_strict_network() {
        let mut config = create_test_config();
        config.security_policy.network.preset = crate::security::NetworkPreset::Strict;

        let compose = generate_compose(&config).unwrap();
        assert!(compose.contains("network_mode"));
        assert!(compose.contains("none"));
    }

    #[test]
    fn test_compose_resource_limits() {
        let mut config = create_test_config();
        config.security_policy.sandbox.memory_limit = Some("2g".to_string());
        config.security_policy.sandbox.cpu_limit = Some(2.0);

        let compose = generate_compose(&config).unwrap();
        assert!(compose.contains("memory:") && compose.contains("2g"));
        assert!(compose.contains("cpus:") && compose.contains("2.0"));
    }
}
