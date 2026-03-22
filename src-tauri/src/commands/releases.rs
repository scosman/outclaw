use crate::error::{EasyClawError, Result};
use crate::github::ReleasesClient;
use crate::instance::Release;

/// Get available OpenClaw releases
#[tauri::command]
pub async fn get_releases() -> std::result::Result<Vec<Release>, String> {
    let client: Result<ReleasesClient> = ReleasesClient::new();
    let client = client.map_err(|e: EasyClawError| e.to_string())?;

    let releases: Result<Vec<Release>> = client.get_releases().await;
    releases.map_err(|e: EasyClawError| e.to_string())
}
