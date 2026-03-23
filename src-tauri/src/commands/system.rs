use crate::instance::generate_name;

/// Get the system timezone
#[tauri::command]
pub async fn get_system_timezone() -> Result<String, String> {
    iana_time_zone::get_timezone().map_err(|e| e.to_string())
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
