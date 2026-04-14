use tauri::State;
use tracing::info;

use crate::commands::instances::AppState;
use crate::security::{
    check_security_changes, AuditAction, AuditEntry, ApprovalRequest, SecurityPolicy,
};

/// Get recent audit log entries
#[tauri::command]
pub async fn get_audit_log(
    count: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEntry>, String> {
    let count = count.unwrap_or(100);
    let entries = state.audit_logger.read_recent(count);
    Ok(entries)
}

/// Check if a security policy change requires approval.
/// Returns a list of approval requests for changes that weaken security.
#[tauri::command]
pub async fn check_security_approval(
    instance_id: String,
    proposed_policy: SecurityPolicy,
    state: State<'_, AppState>,
) -> Result<Vec<ApprovalRequest>, String> {
    let config = state
        .instance_manager
        .get(&instance_id)
        .map_err(|e| e.to_string())?;

    let approvals = check_security_changes(&config.security_policy, &proposed_policy);
    Ok(approvals)
}

/// Approve and apply a security policy change
#[tauri::command]
pub async fn approve_security_change(
    instance_id: String,
    approved: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if approved {
        state.audit_logger.log_success(
            AuditAction::SecurityApprovalGranted,
            Some(instance_id.clone()),
            None,
        );
        info!("Security change approved for instance {}", instance_id);
    } else {
        state.audit_logger.log_denied(
            AuditAction::SecurityPolicyChanged,
            Some(instance_id.clone()),
            Some(serde_json::json!({"reason": "User denied security change"})),
        );
        info!("Security change denied for instance {}", instance_id);
    }

    Ok(())
}
