use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Persisted instance configuration stored in instance.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    /// Generated instance ID, e.g., "ec_a1b2c3"
    pub id: String,
    /// User-facing name, e.g., "Cosmic Otter"
    pub name: String,
    /// OpenClaw version (GitHub release tag)
    pub openclaw_version: String,
    /// Reference to the container ID
    pub container_id: String,
    /// Gateway port (18789+)
    pub gateway_port: u16,
    /// Bridge port (18790+)
    pub bridge_port: u16,
    /// Network bind mode
    pub gateway_bind: GatewayBind,
    /// Gateway authentication token (32-byte hex)
    pub gateway_token: String,
    /// IANA timezone
    pub timezone: String,
    /// Install browser in container
    pub install_browser: bool,
    /// Space-separated apt packages
    pub apt_packages: String,
    /// Space-separated extensions
    pub extensions: String,
    /// Named volume or host path for home
    pub home_volume: String,
    /// Extra volume mounts (comma-separated source:target[:opts])
    pub extra_mounts: String,
    /// Allow insecure WebSocket
    pub allow_insecure_ws: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl InstanceConfig {
    /// Get the base EasyClaw directory
    ///
    /// # Panics
    /// Panics if the home directory cannot be determined. This should never happen
    /// on properly configured systems, but could occur in unusual environments.
    pub fn easyclaw_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .expect("Cannot determine home directory - please ensure HOME environment variable is set")
            .join(".easyclaw")
    }

    /// Try to get the base EasyClaw directory, returning None if home dir is unavailable
    pub fn try_easyclaw_dir() -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|p| p.join(".easyclaw"))
    }

    /// Get the config directory path for this instance
    pub fn config_path(&self) -> std::path::PathBuf {
        Self::easyclaw_dir()
            .join("instances")
            .join(&self.id)
            .join("config")
    }

    /// Get the workspace directory path for this instance
    pub fn workspace_path(&self) -> std::path::PathBuf {
        Self::easyclaw_dir()
            .join("instances")
            .join(&self.id)
            .join("workspace")
    }

    /// Get the docker directory path for this instance's container
    pub fn docker_path(&self) -> std::path::PathBuf {
        Self::easyclaw_dir()
            .join("docker-containers")
            .join(&self.container_id)
    }

    /// Get the gateway URL
    pub fn gateway_url(&self) -> String {
        format!("http://localhost:{}", self.gateway_port)
    }
}

/// Network bind mode for the gateway
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayBind {
    Loopback,
    Lan,
}

impl Default for GatewayBind {
    fn default() -> Self {
        Self::Loopback
    }
}

impl std::fmt::Display for GatewayBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayBind::Loopback => write!(f, "loopback"),
            GatewayBind::Lan => write!(f, "lan"),
        }
    }
}

/// Settings provided by user when creating/editing an instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceSettings {
    /// User-facing name (generated if not provided)
    #[serde(default)]
    pub name: Option<String>,
    /// OpenClaw version
    #[serde(default = "default_version")]
    pub openclaw_version: String,
    /// Gateway port (auto-assigned if not provided)
    #[serde(default)]
    pub gateway_port: Option<u16>,
    /// Bridge port (auto-assigned if not provided)
    #[serde(default)]
    pub bridge_port: Option<u16>,
    /// Network bind mode
    #[serde(default)]
    pub gateway_bind: GatewayBind,
    /// IANA timezone
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Install browser
    #[serde(default)]
    pub install_browser: bool,
    /// Space-separated apt packages
    #[serde(default)]
    pub apt_packages: String,
    /// Space-separated extensions
    #[serde(default)]
    pub extensions: String,
    /// Named volume or host path
    #[serde(default)]
    pub home_volume: String,
    /// Extra mounts
    #[serde(default)]
    pub extra_mounts: String,
    /// Allow insecure WebSocket
    #[serde(default)]
    pub allow_insecure_ws: bool,
}

fn default_version() -> String {
    "latest".to_string()
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Runtime status of an instance (not persisted)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstanceStatus {
    /// Current state
    pub state: InstanceState,
    /// Docker container ID if running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    /// Error message if in error state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceState {
    Building,
    Running,
    Stopped,
    Error,
    DockerNotRunning,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        Self {
            state: InstanceState::Stopped,
            container_id: None,
            error_message: None,
        }
    }
}

/// Combined config and status for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceWithStatus {
    #[serde(flatten)]
    pub config: InstanceConfig,
    pub status: InstanceStatus,
}

/// Docker availability status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockerStatus {
    pub state: DockerState,
    pub compose_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DockerState {
    Running,
    NotRunning,
    NotInstalled,
}

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release tag, e.g., "v0.42.1"
    pub tag: String,
    /// Release title
    pub name: String,
    /// Publication date
    pub published_at: DateTime<Utc>,
    /// Whether it's a prerelease
    pub prerelease: bool,
    /// Commit SHA for fetching Dockerfile
    pub commit_sha: String,
}

/// App-level state persisted to app-state.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    /// Window position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_position: Option<WindowPosition>,
    /// Window size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_size: Option<WindowSize>,
    /// Last active instance ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_instance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_config_serialization() {
        let config = InstanceConfig {
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
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: InstanceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.gateway_port, deserialized.gateway_port);
    }

    #[test]
    fn test_instance_status_serialization() {
        let status = InstanceStatus {
            state: InstanceState::Running,
            container_id: Some("abc123".to_string()),
            error_message: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: InstanceStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.state, deserialized.state);
        assert_eq!(status.container_id, deserialized.container_id);
    }

    #[test]
    fn test_docker_status_serialization() {
        let status = DockerStatus {
            state: DockerState::Running,
            compose_available: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DockerStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.state, deserialized.state);
        assert!(deserialized.compose_available);
    }

    #[test]
    fn test_gateway_bind_display() {
        assert_eq!(format!("{}", GatewayBind::Loopback), "loopback");
        assert_eq!(format!("{}", GatewayBind::Lan), "lan");
    }
}
