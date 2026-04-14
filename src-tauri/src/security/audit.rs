use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Security-relevant actions that are logged
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    InstanceCreated,
    InstanceDeleted,
    InstanceStarted,
    InstanceStopped,
    InstanceRestarted,
    ConfigChanged,
    SecurityPolicyChanged,
    SecurityApprovalGranted,
    ProviderConnected,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    InputValidationFailed,
    SsrfBlocked,
    RateLimitHit,
}

/// Outcome of an audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    Success,
    Denied,
    Error(String),
}

/// A single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub outcome: AuditOutcome,
}

/// Append-only JSONL audit logger
pub struct AuditLogger {
    log_path: PathBuf,
    file: Mutex<Option<fs::File>>,
}

impl AuditLogger {
    /// Create a new audit logger writing to `~/.outclaw/audit.log`
    pub fn new(base_dir: &PathBuf) -> Self {
        let log_path = base_dir.join("audit.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        Self {
            log_path,
            file: Mutex::new(file),
        }
    }

    /// Log an audit entry
    pub fn log(&self, entry: AuditEntry) {
        debug!(
            "Audit: {:?} instance={:?} outcome={:?}",
            entry.action, entry.instance_id, entry.outcome
        );

        let json = match serde_json::to_string(&entry) {
            Ok(j) => j,
            Err(e) => {
                warn!("Failed to serialize audit entry: {}", e);
                return;
            }
        };

        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut file) = *guard {
                if let Err(e) = writeln!(file, "{}", json) {
                    warn!("Failed to write audit log: {}", e);
                    // Try to reopen the file
                    *guard = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&self.log_path)
                        .ok();
                }
            }
        }
    }

    /// Convenience method to log a successful action
    pub fn log_success(
        &self,
        action: AuditAction,
        instance_id: Option<String>,
        details: Option<serde_json::Value>,
    ) {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            action,
            instance_id,
            details,
            outcome: AuditOutcome::Success,
        });
    }

    /// Convenience method to log a denied action
    pub fn log_denied(
        &self,
        action: AuditAction,
        instance_id: Option<String>,
        details: Option<serde_json::Value>,
    ) {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            action,
            instance_id,
            details,
            outcome: AuditOutcome::Denied,
        });
    }

    /// Read recent audit entries (last N lines)
    pub fn read_recent(&self, count: usize) -> Vec<AuditEntry> {
        let content = match fs::read_to_string(&self.log_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        content
            .lines()
            .rev()
            .take(count)
            .filter_map(|line| serde_json::from_str::<AuditEntry>(line).ok())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: AuditAction::InstanceCreated,
            instance_id: Some("ec_test123".to_string()),
            details: Some(serde_json::json!({"name": "Test Instance"})),
            outcome: AuditOutcome::Success,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.action, AuditAction::InstanceCreated));
        assert_eq!(deserialized.instance_id, Some("ec_test123".to_string()));
        assert!(matches!(deserialized.outcome, AuditOutcome::Success));
    }

    #[test]
    fn test_audit_logger_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(&dir.path().to_path_buf());

        logger.log_success(
            AuditAction::InstanceCreated,
            Some("ec_test".to_string()),
            None,
        );
        logger.log_success(
            AuditAction::InstanceStarted,
            Some("ec_test".to_string()),
            None,
        );
        logger.log_denied(
            AuditAction::InputValidationFailed,
            None,
            Some(serde_json::json!({"field": "apt_packages", "value": "vim; rm -rf /"})),
        );

        let entries = logger.read_recent(10);
        assert_eq!(entries.len(), 3);
        assert!(matches!(entries[0].action, AuditAction::InstanceCreated));
        assert!(matches!(entries[2].action, AuditAction::InputValidationFailed));
        assert!(matches!(entries[2].outcome, AuditOutcome::Denied));
    }
}
