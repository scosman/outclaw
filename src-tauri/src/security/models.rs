use serde::{Deserialize, Serialize};

/// Per-instance security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Container sandbox hardening settings
    #[serde(default)]
    pub sandbox: SandboxPolicy,
    /// Network egress control settings
    #[serde(default)]
    pub network: NetworkPolicy,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            sandbox: SandboxPolicy::default(),
            network: NetworkPolicy::default(),
        }
    }
}

/// Sandbox hardening configuration for Docker containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Drop all Linux capabilities (adds cap_drop: ["ALL"] to compose)
    #[serde(default = "default_true")]
    pub drop_all_capabilities: bool,
    /// Specific capabilities to add back (e.g., ["NET_BIND_SERVICE"])
    #[serde(default)]
    pub added_capabilities: Vec<String>,
    /// Seccomp profile to apply
    #[serde(default)]
    pub seccomp_profile: SeccompProfile,
    /// Prevent gaining new privileges (no-new-privileges flag)
    #[serde(default = "default_true")]
    pub no_new_privileges: bool,
    /// Maximum number of PIDs in the container (None = unlimited)
    #[serde(default = "default_pids_limit")]
    pub pids_limit: Option<i64>,
    /// Memory limit (e.g., "2g", "512m"). None = unlimited
    #[serde(default)]
    pub memory_limit: Option<String>,
    /// CPU limit (e.g., 2.0 = 2 CPUs). None = unlimited
    #[serde(default)]
    pub cpu_limit: Option<f64>,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            drop_all_capabilities: true,
            added_capabilities: vec![],
            seccomp_profile: SeccompProfile::Default,
            no_new_privileges: true,
            pids_limit: Some(256),
            memory_limit: None,
            cpu_limit: None,
        }
    }
}

/// Seccomp profile options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeccompProfile {
    /// Docker's built-in default seccomp profile
    Default,
    /// Custom strict profile blocking additional dangerous syscalls
    Strict,
    /// No seccomp restrictions (requires explicit approval)
    Unconfined,
}

impl std::default::Default for SeccompProfile {
    fn default() -> Self {
        Self::Default
    }
}

impl std::fmt::Display for SeccompProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeccompProfile::Default => write!(f, "default"),
            SeccompProfile::Strict => write!(f, "strict"),
            SeccompProfile::Unconfined => write!(f, "unconfined"),
        }
    }
}

/// Network egress control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Network restriction preset level
    #[serde(default)]
    pub preset: NetworkPreset,
    /// Allowed outbound domains (for Strict/Moderate presets)
    #[serde(default)]
    pub allowed_egress_domains: Vec<String>,
    /// Allowed outbound CIDR ranges (for Strict/Moderate presets)
    #[serde(default)]
    pub allowed_egress_cidrs: Vec<String>,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            preset: NetworkPreset::Permissive,
            allowed_egress_domains: vec![],
            allowed_egress_cidrs: vec![],
        }
    }
}

/// Network restriction preset levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkPreset {
    /// No outbound network access (network_mode: "none")
    Strict,
    /// Restricted outbound: HTTPS and DNS only, plus allowed list
    Moderate,
    /// No restrictions (current default behavior)
    Permissive,
}

impl Default for NetworkPreset {
    fn default() -> Self {
        Self::Permissive
    }
}

impl std::fmt::Display for NetworkPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkPreset::Strict => write!(f, "strict"),
            NetworkPreset::Moderate => write!(f, "moderate"),
            NetworkPreset::Permissive => write!(f, "permissive"),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_pids_limit() -> Option<i64> {
    Some(256)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        assert!(policy.sandbox.drop_all_capabilities);
        assert!(policy.sandbox.no_new_privileges);
        assert_eq!(policy.sandbox.pids_limit, Some(256));
        assert_eq!(policy.sandbox.seccomp_profile, SeccompProfile::Default);
        assert_eq!(policy.network.preset, NetworkPreset::Permissive);
        assert!(policy.sandbox.added_capabilities.is_empty());
    }

    #[test]
    fn test_security_policy_serialization() {
        let policy = SecurityPolicy::default();
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: SecurityPolicy = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.sandbox.drop_all_capabilities,
            policy.sandbox.drop_all_capabilities
        );
        assert_eq!(deserialized.network.preset, policy.network.preset);
    }

    #[test]
    fn test_security_policy_backward_compat() {
        // Empty JSON should deserialize with defaults
        let json = "{}";
        let policy: SecurityPolicy = serde_json::from_str(json).unwrap();
        assert!(policy.sandbox.drop_all_capabilities);
        assert_eq!(policy.network.preset, NetworkPreset::Permissive);
    }

    #[test]
    fn test_network_preset_display() {
        assert_eq!(format!("{}", NetworkPreset::Strict), "strict");
        assert_eq!(format!("{}", NetworkPreset::Moderate), "moderate");
        assert_eq!(format!("{}", NetworkPreset::Permissive), "permissive");
    }

    #[test]
    fn test_seccomp_profile_display() {
        assert_eq!(format!("{}", SeccompProfile::Default), "default");
        assert_eq!(format!("{}", SeccompProfile::Strict), "strict");
        assert_eq!(format!("{}", SeccompProfile::Unconfined), "unconfined");
    }
}
