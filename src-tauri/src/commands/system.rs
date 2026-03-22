use crate::instance::generate_name;

/// Get the system timezone
#[tauri::command]
pub async fn get_system_timezone() -> Result<String, String> {
    // Try to detect system timezone
    #[cfg(target_os = "macos")]
    {
        // On macOS, read from /etc/localtime symlink or systemsetup
        if let Ok(output) = tokio::process::Command::new("systemsetup")
            .args(["-gettimezone"])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Output format: "Time Zone: America/Toronto"
            if let Some(tz) = stdout.strip_prefix("Time Zone: ") {
                return Ok(tz.trim().to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, try /etc/timezone or timedatectl
        if let Ok(output) = tokio::process::Command::new("timedatectl")
            .args(["show", "--property=Timezone", "--value"])
            .output()
            .await
        {
            let tz = String::from_utf8_lossy(&output.stdout);
            let tz = tz.trim();
            if !tz.is_empty() {
                return Ok(tz.to_string());
            }
        }

        // Fallback: read /etc/timezone
        if let Ok(content) = tokio::fs::read_to_string("/etc/timezone").await {
            return Ok(content.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use PowerShell
        if let Ok(output) = tokio::process::Command::new("powershell")
            .args(["-Command", "Get-TimeZone | Select-Object -ExpandProperty Id"])
            .output()
            .await
        {
            let tz = String::from_utf8_lossy(&output.stdout);
            let tz = tz.trim();
            if !tz.is_empty() {
                return Ok(tz.to_string());
            }
        }
    }

    // Default to UTC if detection fails
    Ok("UTC".to_string())
}

/// Generate a random instance name
#[tauri::command]
pub async fn generate_instance_name() -> Result<String, String> {
    // Generate without existing names (caller will handle collisions)
    let name = generate_name(&[]);
    Ok(name)
}

/// Open a URL in the default browser
#[tauri::command]
pub async fn open_in_browser(url: String) -> Result<(), String> {
    // Validate URL scheme for security - only allow http/https
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL: only http:// and https:// URLs are allowed".to_string());
    }

    // Use tauri-plugin-opener
    tauri_plugin_opener::open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;

    Ok(())
}
