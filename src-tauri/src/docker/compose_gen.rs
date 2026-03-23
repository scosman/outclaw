use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::Result;
use crate::instance::{GatewayBind, InstanceConfig};

/// Generate docker-compose.yml content for an instance
pub fn generate_compose(config: &InstanceConfig) -> Result<String> {
    let project_name = format!("easyclaw-{}", config.container_id);
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
    labels.insert("easyclaw.container".to_string(), config.container_id.clone());
    labels.insert("easyclaw.instance".to_string(), config.id.clone());

    // Get paths
    let config_path = config.config_path();
    let workspace_path = config.workspace_path();

    let compose = DockerCompose {
        version: "3.8",
        services: {
            let mut services = HashMap::new();

            // Gateway service
            services.insert(
                gateway_service,
                Service {
                    image: image_name.clone(),
                    container_name: Some(format!("easyclaw-{}-gateway", config.container_id)),
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
                },
            );

            // CLI service (for running commands)
            services.insert(
                cli_service,
                Service {
                    image: image_name,
                    container_name: Some(format!("easyclaw-{}-cli", config.container_id)),
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
        assert!(compose.contains("easyclaw-ct_abc456-gateway"));
        assert!(compose.contains("easyclaw-ct_abc456-cli"));
        assert!(compose.contains("18789:18789"));
        assert!(compose.contains("18790:18790"));
        assert!(compose.contains("easyclaw.container"));
        assert!(compose.contains("easyclaw.instance"));
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
}
