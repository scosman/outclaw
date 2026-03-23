use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use rand::Rng;
use tracing::{debug, info, warn};

use crate::error::{EasyClawError, Result};
use crate::instance::{
    generate_name, allocate_ports, validate_port,
    InstanceConfig, InstanceSettings,
};

/// Manages instance CRUD operations and directory structure
pub struct InstanceManager {
    base_dir: PathBuf,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .ok_or_else(|| EasyClawError::Other("Cannot determine home directory".to_string()))?
            .join(".easyclaw");

        // Ensure base directory exists
        fs::create_dir_all(&base_dir).map_err(|e| {
            EasyClawError::Io(std::io::Error::other(
                format!("Failed to create .easyclaw directory: {}", e),
            ))
        })?;

        Ok(Self { base_dir })
    }

    /// Get the base directory path
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Get instances directory path
    pub fn instances_dir(&self) -> PathBuf {
        self.base_dir.join("instances")
    }

    /// Get docker containers directory path
    pub fn docker_containers_dir(&self) -> PathBuf {
        self.base_dir.join("docker-containers")
    }

    /// List all instances
    pub fn list(&self) -> Result<Vec<InstanceConfig>> {
        let instances_dir = self.instances_dir();

        if !instances_dir.exists() {
            return Ok(Vec::new());
        }

        let mut instances = Vec::new();

        for entry in fs::read_dir(&instances_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let config_path = path.join("instance.json");
            if !config_path.exists() {
                warn!("Missing instance.json in {:?}", path);
                continue;
            }

            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<InstanceConfig>(&content) {
                    Ok(config) => instances.push(config),
                    Err(e) => {
                        warn!("Failed to parse instance.json in {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read instance.json in {:?}: {}", path, e);
                }
            }
        }

        // Sort by creation date, newest first
        instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(instances)
    }

    /// Get a single instance by ID
    pub fn get(&self, id: &str) -> Result<InstanceConfig> {
        let config_path = self.instances_dir().join(id).join("instance.json");

        if !config_path.exists() {
            return Err(EasyClawError::InstanceNotFound(id.to_string()));
        }

        let content = fs::read_to_string(&config_path)?;
        let config: InstanceConfig = serde_json::from_str(&content)?;

        Ok(config)
    }

    /// Create a new instance
    pub fn create(&self, settings: InstanceSettings) -> Result<InstanceConfig> {
        info!("Creating new instance with settings: {:?}", settings);

        // Get existing instances for name/port collision checks
        let existing = self.list()?;
        let existing_names: Vec<String> = existing.iter().map(|i| i.name.clone()).collect();

        // Generate or validate name
        let name = match settings.name {
            Some(ref n) if !n.is_empty() => {
                if existing_names.contains(n) {
                    return Err(EasyClawError::InstanceAlreadyExists(n.clone()));
                }
                n.clone()
            }
            _ => generate_name(&existing_names),
        };

        // Allocate or validate ports
        let (gateway_port, bridge_port) = match (settings.gateway_port, settings.bridge_port) {
            (Some(gp), Some(bp)) => {
                validate_port(gp, None, &existing)?;
                validate_port(bp, None, &existing)?;
                (gp, bp)
            }
            _ => allocate_ports(&existing)?,
        };

        // Generate IDs
        let instance_id = generate_id("ec_");
        let container_id = generate_id("ct_");

        // Generate gateway token (32 bytes = 64 hex chars)
        let gateway_token = generate_token();

        // Create config
        let now = Utc::now();
        let config = InstanceConfig {
            id: instance_id.clone(),
            name,
            openclaw_version: settings.openclaw_version,
            container_id: container_id.clone(),
            gateway_port,
            bridge_port,
            gateway_bind: settings.gateway_bind,
            gateway_token,
            timezone: settings.timezone,
            install_browser: settings.install_browser,
            apt_packages: settings.apt_packages,
            extensions: settings.extensions,
            home_volume: settings.home_volume,
            extra_mounts: settings.extra_mounts,
            allow_insecure_ws: settings.allow_insecure_ws,
            created_at: now,
            updated_at: now,
        };

        // Create directory structure
        self.create_instance_directories(&config)?;

        // Write instance.json
        let config_path = self.instances_dir().join(&instance_id).join("instance.json");
        let config_content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_content)?;

        info!("Created instance {} with ID {}", config.name, config.id);

        Ok(config)
    }

    /// Update an existing instance
    pub fn update(&self, id: &str, settings: InstanceSettings) -> Result<InstanceConfig> {
        info!("Updating instance {} with settings: {:?}", id, settings);

        let mut config = self.get(id)?;
        let existing = self.list()?;

        // Validate and update name if provided
        if let Some(ref name) = settings.name {
            if !name.is_empty() && name != &config.name {
                let existing_names: Vec<String> = existing
                    .iter()
                    .filter(|i| i.id != id)
                    .map(|i| i.name.clone())
                    .collect();
                if existing_names.contains(name) {
                    return Err(EasyClawError::InstanceAlreadyExists(name.clone()));
                }
                config.name = name.clone();
            }
        }

        // Validate and update ports if provided
        if let Some(gp) = settings.gateway_port {
            validate_port(gp, Some(id), &existing)?;
            config.gateway_port = gp;
        }
        if let Some(bp) = settings.bridge_port {
            validate_port(bp, Some(id), &existing)?;
            config.bridge_port = bp;
        }

        // Update other settings
        config.openclaw_version = settings.openclaw_version;
        config.gateway_bind = settings.gateway_bind;
        config.timezone = settings.timezone;
        config.install_browser = settings.install_browser;
        config.apt_packages = settings.apt_packages;
        config.extensions = settings.extensions;
        config.home_volume = settings.home_volume;
        config.extra_mounts = settings.extra_mounts;
        config.allow_insecure_ws = settings.allow_insecure_ws;
        config.updated_at = Utc::now();

        // Write updated config
        let config_path = self.instances_dir().join(id).join("instance.json");
        let config_content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_content)?;

        info!("Updated instance {}", id);

        Ok(config)
    }

    /// Rename an instance (just the name field)
    pub fn rename(&self, id: &str, new_name: &str) -> Result<()> {
        info!("Renaming instance {} to {}", id, new_name);

        let existing = self.list()?;
        let existing_names: Vec<String> = existing
            .iter()
            .filter(|i| i.id != id)
            .map(|i| i.name.clone())
            .collect();

        if existing_names.contains(&new_name.to_string()) {
            return Err(EasyClawError::InstanceAlreadyExists(new_name.to_string()));
        }

        let mut config = self.get(id)?;
        config.name = new_name.to_string();
        config.updated_at = Utc::now();

        let config_path = self.instances_dir().join(id).join("instance.json");
        let config_content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_content)?;

        info!("Renamed instance {} to {}", id, new_name);

        Ok(())
    }

    /// Delete an instance and all its data
    pub fn delete(&self, id: &str) -> Result<()> {
        info!("Deleting instance {}", id);

        let config = self.get(id)?;
        let instance_dir = self.instances_dir().join(id);
        let container_dir = self.docker_containers_dir().join(&config.container_id);

        // Delete instance directory
        if instance_dir.exists() {
            fs::remove_dir_all(&instance_dir)?;
            debug!("Deleted instance directory: {:?}", instance_dir);
        }

        // Delete container directory
        if container_dir.exists() {
            fs::remove_dir_all(&container_dir)?;
            debug!("Deleted container directory: {:?}", container_dir);
        }

        info!("Deleted instance {}", id);

        Ok(())
    }

    /// Create the directory structure for a new instance
    fn create_instance_directories(&self, config: &InstanceConfig) -> Result<()> {
        let instance_dir = self.instances_dir().join(&config.id);
        let container_dir = self.docker_containers_dir().join(&config.container_id);

        // Instance directories
        let dirs = vec![
            instance_dir.clone(),
            instance_dir.join("config"),
            instance_dir.join("config/identity"),
            instance_dir.join("config/agents"),
            instance_dir.join("config/agents/main"),
            instance_dir.join("config/agents/main/agent"),
            instance_dir.join("config/agents/main/sessions"),
            instance_dir.join("workspace"),
        ];

        for dir in dirs {
            fs::create_dir_all(&dir)?;
            debug!("Created directory: {:?}", dir);
        }

        // Container directory for Docker files
        fs::create_dir_all(&container_dir)?;
        debug!("Created container directory: {:?}", container_dir);

        Ok(())
    }

    /// Get the config directory path for an instance
    pub fn config_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id).join("config")
    }

    /// Get the workspace directory path for an instance
    pub fn workspace_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id).join("workspace")
    }

    /// Get the docker directory path for an instance's container
    pub fn docker_dir(&self, container_id: &str) -> PathBuf {
        self.docker_containers_dir().join(container_id)
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create InstanceManager")
    }
}

/// Generate a random ID with prefix
fn generate_id(prefix: &str) -> String {
    let mut rng = rand::thread_rng();
    let chars: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + idx as u8 - 10) as char
            }
        })
        .collect();
    format!("{}{}", prefix, chars)
}

/// Generate a secure random token (32 bytes = 64 hex chars)
fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(&bytes)
}

// Add hex dependency functionality inline to avoid extra crate
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id("ec_");
        assert!(id.starts_with("ec_"));
        assert_eq!(id.len(), 9); // "ec_" + 6 chars

        let id2 = generate_id("ct_");
        assert!(id2.starts_with("ct_"));
        assert_eq!(id2.len(), 9);
    }

    #[test]
    fn test_generate_token() {
        let token = generate_token();
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars

        // Tokens should be unique
        let token2 = generate_token();
        assert_ne!(token, token2);
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode(&[0x00]), "00");
        assert_eq!(hex::encode(&[0xff]), "ff");
        assert_eq!(hex::encode(&[0x0a, 0xbc]), "0abc");
    }
}
