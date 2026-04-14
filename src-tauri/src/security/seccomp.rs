use std::path::Path;

use serde::Serialize;
use tracing::info;

use crate::error::Result;
use crate::security::models::SandboxPolicy;

/// Syscalls blocked by the "strict" seccomp profile.
/// These are dangerous operations that container workloads should never need.
const BLOCKED_SYSCALLS: &[&str] = &[
    "mount",
    "umount2",
    "ptrace",
    "kexec_load",
    "kexec_file_load",
    "reboot",
    "swapon",
    "swapoff",
    "init_module",
    "finit_module",
    "delete_module",
    "keyctl",
    "request_key",
    "add_key",
    "pivot_root",
    "syslog",
    "acct",
    "settimeofday",
    "clock_settime",
    "clock_adjtime",
    "adjtimex",
    "create_module",
    "get_kernel_syms",
    "query_module",
    "nfsservctl",
    "personality",
    "uselib",
    "lookup_dcookie",
    "ioperm",
    "iopl",
    "vm86",
    "vm86old",
    "modify_ldt",
];

/// Generate a seccomp profile JSON file for Docker.
/// When `SandboxPolicy.seccomp_profile` is `Strict`, this file is placed
/// alongside docker-compose.yml and referenced via `security_opt`.
pub fn generate_seccomp_profile(_policy: &SandboxPolicy, output_path: &Path) -> Result<()> {
    let profile = SeccompProfileDoc {
        default_action: "SCMP_ACT_ALLOW".to_string(),
        syscalls: vec![SyscallRule {
            names: BLOCKED_SYSCALLS.iter().map(|s| s.to_string()).collect(),
            action: "SCMP_ACT_ERRNO".to_string(),
            args: vec![],
        }],
    };

    let json = serde_json::to_string_pretty(&profile)?;
    std::fs::write(output_path, json)?;

    info!("Generated seccomp profile at {:?}", output_path);
    Ok(())
}

#[derive(Debug, Serialize)]
struct SeccompProfileDoc {
    #[serde(rename = "defaultAction")]
    default_action: String,
    syscalls: Vec<SyscallRule>,
}

#[derive(Debug, Serialize)]
struct SyscallRule {
    names: Vec<String>,
    action: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seccomp_profile() {
        let policy = SandboxPolicy::default();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seccomp-profile.json");

        generate_seccomp_profile(&policy, &path).unwrap();

        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert_eq!(doc["defaultAction"], "SCMP_ACT_ALLOW");
        let syscalls = doc["syscalls"].as_array().unwrap();
        assert!(!syscalls.is_empty());

        let names = syscalls[0]["names"].as_array().unwrap();
        let name_strings: Vec<&str> = names.iter().map(|n| n.as_str().unwrap()).collect();
        assert!(name_strings.contains(&"mount"));
        assert!(name_strings.contains(&"ptrace"));
        assert!(name_strings.contains(&"reboot"));
        assert!(name_strings.contains(&"kexec_load"));
    }
}
