use serde::{Deserialize, Serialize};

use crate::security::models::{NetworkPreset, SeccompProfile, SecurityPolicy};

/// Types of security changes that require explicit approval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    /// Changing seccomp to unconfined
    SeccompUnconfined,
    /// Changing network from strict/moderate to permissive
    NetworkPermissive,
    /// Disabling capability dropping
    DisableCapDrop,
}

/// A request for user approval of a security-sensitive change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Type of change requiring approval
    pub approval_type: ApprovalType,
    /// Human-readable description of the risk
    pub description: String,
    /// What the current setting is
    pub current_value: String,
    /// What it will change to
    pub new_value: String,
}

/// Result of a security approval check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityApproval {
    /// Whether the change was approved
    pub approved: bool,
    /// Approval requests that need user confirmation (empty if auto-approved)
    pub pending_approvals: Vec<ApprovalRequest>,
}

/// Check if a security policy change requires explicit approval.
/// Returns a list of approval requests for changes that weaken security.
pub fn check_security_changes(
    current: &SecurityPolicy,
    proposed: &SecurityPolicy,
) -> Vec<ApprovalRequest> {
    let mut approvals = Vec::new();

    // Check seccomp changes
    if current.sandbox.seccomp_profile != SeccompProfile::Unconfined
        && proposed.sandbox.seccomp_profile == SeccompProfile::Unconfined
    {
        approvals.push(ApprovalRequest {
            approval_type: ApprovalType::SeccompUnconfined,
            description: "Disabling seccomp removes syscall filtering, allowing the container to make any system call. This significantly reduces sandbox security.".to_string(),
            current_value: format!("{}", current.sandbox.seccomp_profile),
            new_value: "unconfined".to_string(),
        });
    }

    // Check network preset changes (tightening is fine, loosening requires approval)
    if is_stricter_network(current.network.preset, proposed.network.preset) {
        approvals.push(ApprovalRequest {
            approval_type: ApprovalType::NetworkPermissive,
            description: format!(
                "Changing network policy from '{}' to '{}' grants the container broader network access.",
                current.network.preset, proposed.network.preset
            ),
            current_value: format!("{}", current.network.preset),
            new_value: format!("{}", proposed.network.preset),
        });
    }

    // Check capability dropping disabled
    if current.sandbox.drop_all_capabilities && !proposed.sandbox.drop_all_capabilities {
        approvals.push(ApprovalRequest {
            approval_type: ApprovalType::DisableCapDrop,
            description: "Disabling capability dropping allows the container to retain all Linux capabilities, which increases the potential attack surface.".to_string(),
            current_value: "enabled".to_string(),
            new_value: "disabled".to_string(),
        });
    }

    approvals
}

/// Returns true if `proposed` is a less restrictive network preset than `current`
fn is_stricter_network(current: NetworkPreset, proposed: NetworkPreset) -> bool {
    let level = |p: NetworkPreset| -> u8 {
        match p {
            NetworkPreset::Strict => 0,
            NetworkPreset::Moderate => 1,
            NetworkPreset::Permissive => 2,
        }
    };
    level(proposed) > level(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_approvals_for_tightening() {
        let current = SecurityPolicy {
            sandbox: crate::security::models::SandboxPolicy {
                seccomp_profile: SeccompProfile::Unconfined,
                ..Default::default()
            },
            network: crate::security::models::NetworkPolicy {
                preset: NetworkPreset::Permissive,
                ..Default::default()
            },
        };
        let proposed = SecurityPolicy::default(); // Stricter
        let approvals = check_security_changes(&current, &proposed);
        assert!(approvals.is_empty());
    }

    #[test]
    fn test_approval_for_seccomp_unconfined() {
        let current = SecurityPolicy::default();
        let mut proposed = SecurityPolicy::default();
        proposed.sandbox.seccomp_profile = SeccompProfile::Unconfined;

        let approvals = check_security_changes(&current, &proposed);
        assert_eq!(approvals.len(), 1);
        assert!(matches!(
            approvals[0].approval_type,
            ApprovalType::SeccompUnconfined
        ));
    }

    #[test]
    fn test_approval_for_network_loosening() {
        let mut current = SecurityPolicy::default();
        current.network.preset = NetworkPreset::Strict;
        let mut proposed = SecurityPolicy::default();
        proposed.network.preset = NetworkPreset::Permissive;

        let approvals = check_security_changes(&current, &proposed);
        assert_eq!(approvals.len(), 1);
        assert!(matches!(
            approvals[0].approval_type,
            ApprovalType::NetworkPermissive
        ));
    }

    #[test]
    fn test_approval_for_disabling_cap_drop() {
        let current = SecurityPolicy::default(); // drop_all_capabilities = true
        let mut proposed = SecurityPolicy::default();
        proposed.sandbox.drop_all_capabilities = false;

        let approvals = check_security_changes(&current, &proposed);
        assert_eq!(approvals.len(), 1);
        assert!(matches!(
            approvals[0].approval_type,
            ApprovalType::DisableCapDrop
        ));
    }
}
