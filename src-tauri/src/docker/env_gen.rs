use tracing::debug;

use crate::error::Result;
use crate::instance::InstanceConfig;

/// Generate .env file content for an instance
pub fn generate_env(config: &InstanceConfig) -> Result<String> {
    let mut lines = Vec::new();

    // Core settings
    lines.push("# OutClaw Instance Configuration".to_string());
    lines.push(format!("# Instance: {} ({})", config.name, config.id));
    lines.push(format!("# Generated: {}", config.updated_at.to_rfc3339()));
    lines.push(String::new());

    // OpenClaw version
    lines.push(format!("OPENCLAW_VERSION={}", config.openclaw_version));

    // Network settings
    lines.push(format!("OPENCLAW_GATEWAY_PORT={}", config.gateway_port));
    lines.push(format!("OPENCLAW_BRIDGE_PORT={}", config.bridge_port));
    lines.push(format!("OPENCLAW_GATEWAY_BIND={}", config.gateway_bind));
    lines.push(format!("OPENCLAW_GATEWAY_TOKEN={}", config.gateway_token));
    lines.push(format!("OPENCLAW_TZ={}", config.timezone));

    // Browser installation
    if config.install_browser {
        lines.push("OPENCLAW_INSTALL_BROWSER=true".to_string());
    }

    // Additional packages
    if !config.apt_packages.is_empty() {
        lines.push(format!("OPENCLAW_DOCKER_APT_PACKAGES={}", config.apt_packages));
    }

    // Extensions
    if !config.extensions.is_empty() {
        lines.push(format!("OPENCLAW_EXTENSIONS={}", config.extensions));
    }

    // Home volume
    if !config.home_volume.is_empty() {
        lines.push(format!("OPENCLAW_HOME_VOLUME={}", config.home_volume));
    }

    // Extra mounts
    if !config.extra_mounts.is_empty() {
        lines.push(format!("OPENCLAW_EXTRA_MOUNTS={}", config.extra_mounts));
    }

    // Insecure WebSocket
    if config.allow_insecure_ws {
        lines.push("OPENCLAW_ALLOW_INSECURE_PRIVATE_WS=true".to_string());
    }

    // Fixed settings managed by OutClaw
    lines.push(String::new());
    lines.push("# Settings managed by OutClaw".to_string());
    lines.push("OPENCLAW_CONFIG_DIR=/home/node/.openclaw".to_string());
    lines.push("OPENCLAW_WORKSPACE_DIR=/home/node/workspace".to_string());

    // V1: No sandbox (container IS the sandbox)
    lines.push("OPENCLAW_SANDBOX=false".to_string());

    let env_content = lines.join("\n");
    debug!("Generated .env file:\n{}", env_content);
    Ok(env_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::instance::GatewayBind;

    fn create_test_config() -> InstanceConfig {
        InstanceConfig {
            id: "ec_test123".to_string(),
            name: "Test Instance".to_string(),
            openclaw_version: "v0.42.1".to_string(),
            container_id: "ct_abc456".to_string(),
            gateway_port: 18789,
            bridge_port: 18790,
            gateway_bind: GatewayBind::Loopback,
            gateway_token: "test_token_123".to_string(),
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
    fn test_generate_env_basic() {
        let config = create_test_config();
        let env = generate_env(&config).unwrap();

        assert!(env.contains("OPENCLAW_VERSION=v0.42.1"));
        assert!(env.contains("OPENCLAW_GATEWAY_PORT=18789"));
        assert!(env.contains("OPENCLAW_BRIDGE_PORT=18790"));
        assert!(env.contains("OPENCLAW_GATEWAY_BIND=loopback"));
        assert!(env.contains("OPENCLAW_GATEWAY_TOKEN=test_token_123"));
        assert!(env.contains("OPENCLAW_TZ=America/Toronto"));
        assert!(env.contains("OPENCLAW_SANDBOX=false"));
    }

    #[test]
    fn test_generate_env_with_options() {
        let mut config = create_test_config();
        config.install_browser = true;
        config.apt_packages = "vim curl".to_string();
        config.extensions = "ext1 ext2".to_string();
        config.allow_insecure_ws = true;

        let env = generate_env(&config).unwrap();

        assert!(env.contains("OPENCLAW_INSTALL_BROWSER=true"));
        assert!(env.contains("OPENCLAW_DOCKER_APT_PACKAGES=vim curl"));
        assert!(env.contains("OPENCLAW_EXTENSIONS=ext1 ext2"));
        assert!(env.contains("OPENCLAW_ALLOW_INSECURE_PRIVATE_WS=true"));
    }

    #[test]
    fn test_generate_env_lan_mode() {
        let mut config = create_test_config();
        config.gateway_bind = GatewayBind::Lan;

        let env = generate_env(&config).unwrap();

        assert!(env.contains("OPENCLAW_GATEWAY_BIND=lan"));
    }
}
