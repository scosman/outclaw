use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use tokio::sync::{mpsc, RwLock};

use crate::docker::{generate_compose, generate_env, fetch_release_source, DockerCli};
use crate::github::ReleasesClient;
use crate::instance::{
    InstanceConfig, InstanceManager, InstanceSettings, InstanceStatus, InstanceState,
    InstanceWithStatus, Release,
};

/// Tracks active builds for cancellation support
pub struct BuildTracker {
    active_builds: RwLock<HashMap<String, tokio_util::sync::CancellationToken>>,
}

impl BuildTracker {
    pub fn new() -> Self {
        Self {
            active_builds: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, instance_id: &str) -> tokio_util::sync::CancellationToken {
        let token = tokio_util::sync::CancellationToken::new();
        self.active_builds.write().await.insert(instance_id.to_string(), token.clone());
        token
    }

    pub async fn unregister(&self, instance_id: &str) {
        self.active_builds.write().await.remove(instance_id);
    }

    pub async fn cancel(&self, instance_id: &str) -> bool {
        if let Some(token) = self.active_builds.read().await.get(instance_id) {
            token.cancel();
            true
        } else {
            false
        }
    }
}

impl Default for BuildTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared state for the application
#[derive(Clone)]
pub struct AppState {
    pub instance_manager: Arc<InstanceManager>,
    pub docker_cli: Arc<DockerCli>,
    pub build_tracker: Arc<BuildTracker>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            instance_manager: Arc::new(InstanceManager::new().expect("Failed to create InstanceManager")),
            docker_cli: Arc::new(DockerCli::new()),
            build_tracker: Arc::new(BuildTracker::new()),
        }
    }
}

/// List all instances with their current status
#[tauri::command]
pub async fn list_instances(
    state: State<'_, AppState>,
) -> Result<Vec<InstanceWithStatus>, String> {
    info!("Listing instances");

    let instances = state
        .instance_manager
        .list()
        .map_err(|e| e.to_string())?;

    // Get Docker status
    let docker_status = state
        .docker_cli
        .check_available()
        .await
        .map_err::<String, _>(|e| e.to_string())?;

    // Build status for each instance
    let mut result = Vec::new();
    for config in instances {
        let status = if docker_status.state != crate::instance::DockerState::Running {
            InstanceStatus {
                state: InstanceState::DockerNotRunning,
                container_id: None,
                error_message: None,
            }
        } else {
            // Query Docker for actual container status
            match state.docker_cli.list_containers(&format!("outclaw.instance={}", config.id)).await {
                Ok(containers) => {
                    if let Some(container) = containers.first() {
                        InstanceStatus {
                            state: if container.is_running() {
                                InstanceState::Running
                            } else {
                                InstanceState::Stopped
                            },
                            container_id: Some(container.id.clone()),
                            error_message: None,
                        }
                    } else {
                        InstanceStatus {
                            state: InstanceState::Stopped,
                            container_id: None,
                            error_message: None,
                        }
                    }
                }
                Err(e) => {
                    InstanceStatus {
                        state: InstanceState::Error,
                        container_id: None,
                        error_message: Some(format!("Failed to query Docker: {}", e)),
                    }
                }
            }
        };
        result.push(InstanceWithStatus { config, status });
    }

    info!("Found {} instances", result.len());
    Ok(result)
}

/// Get a single instance by ID
#[tauri::command]
pub async fn get_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<InstanceWithStatus, String> {
    info!("Getting instance {}", id);

    let config = state
        .instance_manager
        .get(&id)
        .map_err(|e| e.to_string())?;

    // Get status (simplified for now)
    let status = InstanceStatus::default();

    Ok(InstanceWithStatus { config, status })
}

/// Create a new instance
#[tauri::command]
pub async fn create_instance(
    settings: InstanceSettings,
    state: State<'_, AppState>,
) -> Result<InstanceConfig, String> {
    info!("Creating instance with settings: {:?}", settings);

    let config = state
        .instance_manager
        .create(settings)
        .map_err(|e| e.to_string())?;

    // Generate Docker files
    let docker_dir = config.docker_path();

    // Generate docker-compose.yml
    let compose = generate_compose(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join("docker-compose.yml"), compose)
        .map_err(|e| e.to_string())?;

    // Generate .env
    let env = generate_env(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join(".env"), env)
        .map_err(|e| e.to_string())?;

    info!("Created instance {} ({})", config.name, config.id);
    Ok(config)
}

/// Update an existing instance
#[tauri::command]
pub async fn update_instance(
    id: String,
    settings: InstanceSettings,
    state: State<'_, AppState>,
) -> Result<InstanceConfig, String> {
    info!("Updating instance {}", id);

    let config = state
        .instance_manager
        .update(&id, settings)
        .map_err(|e| e.to_string())?;

    // Regenerate Docker files
    let docker_dir = config.docker_path();

    let compose = generate_compose(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join("docker-compose.yml"), compose)
        .map_err(|e| e.to_string())?;

    let env = generate_env(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join(".env"), env)
        .map_err(|e| e.to_string())?;

    info!("Updated instance {}", id);
    Ok(config)
}

/// Rename an instance
#[tauri::command]
pub async fn rename_instance(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Renaming instance {} to {}", id, name);

    state
        .instance_manager
        .rename(&id, &name)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete an instance
#[tauri::command]
pub async fn delete_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Deleting instance {}", id);

    // Get config before deletion to clean up Docker resources
    let config = state.instance_manager.get(&id).ok();

    // Stop and remove container if running
    if let Some(ref cfg) = config {
        let docker_dir = cfg.docker_path();
        let compose_path = docker_dir.join("docker-compose.yml");
        let project_name = format!("outclaw-{}", cfg.container_id);

        // Stop container
        let _ = state.docker_cli.compose_down(&compose_path, &project_name).await;

        // Remove image
        let image_name = format!("outclaw-{}:latest", cfg.container_id);
        let _ = state.docker_cli.remove_image(&image_name).await;
    }

    // Delete instance files
    state
        .instance_manager
        .delete(&id)
        .map_err(|e| e.to_string())?;

    info!("Deleted instance {}", id);
    Ok(())
}

/// Start an instance
#[tauri::command]
pub async fn start_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting instance {}", id);

    let config = state
        .instance_manager
        .get(&id)
        .map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);

    state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    info!("Started instance {}", id);
    Ok(())
}

/// Stop an instance
#[tauri::command]
pub async fn stop_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Stopping instance {}", id);

    let config = state
        .instance_manager
        .get(&id)
        .map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);

    state
        .docker_cli
        .compose_stop(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    info!("Stopped instance {}", id);
    Ok(())
}

/// Restart an instance
#[tauri::command]
pub async fn restart_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Restarting instance {}", id);

    stop_instance(id.clone(), state.clone()).await?;
    start_instance(id, state).await?;

    Ok(())
}

/// Build an instance (run full setup pipeline)
#[tauri::command]
pub async fn build_instance(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Building instance {}", id);

    // Register with build tracker for cancellation support
    let cancel_token = state.build_tracker.register(&id).await;

    // Helper to check for cancellation
    let check_cancelled = || {
        if cancel_token.is_cancelled() {
            Err("Build cancelled by user".to_string())
        } else {
            Ok(())
        }
    };

    let config = state
        .instance_manager
        .get(&id)
        .map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);
    let image_name = format!("{}:latest", project_name);

    // Helper to emit progress
    let emit_progress = |stage: &str, log: &str, done: bool, error: Option<&str>| {
        if let Err(e) = app_handle.emit("build-progress", serde_json::json!({
            "id": id,
            "stage": stage,
            "log": log,
            "done": done,
            "error": error
        })) {
            warn!("Failed to emit build progress: {}", e);
        }
    };

    // Helper to emit and return error
    let emit_error = |stage: &str, e: &str| -> String {
        emit_progress(stage, "", true, Some(e));
        e.to_string()
    };

    // ========== Stage 1: Fetch source ==========
    emit_progress("fetching-source", "Fetching OpenClaw source from GitHub...", false, None);

    // Get the release info
    let releases_client = match ReleasesClient::new() {
        Ok(client) => client,
        Err(e) => {
            let err_msg = emit_error("fetching-source", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    };

    let releases = match releases_client.get_releases().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to fetch releases: {}", e);
            emit_progress("fetching-source", "Warning: Could not fetch releases list, proceeding with version tag", false, None);
            vec![]
        }
    };

    // Find the release matching our version
    let release = releases.iter().find(|r| r.tag == config.openclaw_version).cloned()
        .unwrap_or_else(|| Release {
            tag: config.openclaw_version.clone(),
            name: config.openclaw_version.clone(),
            published_at: chrono::Utc::now(),
            prerelease: false,
            commit_sha: config.openclaw_version.clone(), // Use tag as fallback
        });

    // Fetch the full source tarball
    let http_client = match reqwest::Client::builder()
        .user_agent("OutClaw/0.1.0")
        .timeout(std::time::Duration::from_secs(120)) // Longer timeout for tarball
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            let err_msg = emit_error("fetching-source", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    };

    let source_dir = match fetch_release_source(&release, &docker_dir, &http_client).await {
        Ok(path) => {
            emit_progress("fetching-source", &format!("Source ready for {}", release.tag), false, None);
            path
        }
        Err(e) => {
            let err_msg = emit_error("fetching-source", &format!("Failed to fetch source: {}", e));
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    };

    if let Err(e) = check_cancelled() {
        state.build_tracker.unregister(&id).await;
        return Err(e);
    }

    // ========== Stage 2: Generate configuration files ==========
    emit_progress("generating-config", "Generating docker-compose.yml and .env...", false, None);

    let compose = match generate_compose(&config) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = emit_error("generating-config", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    };

    if let Err(e) = std::fs::write(&compose_path, &compose) {
        let err_msg = emit_error("generating-config", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    let env = match generate_env(&config) {
        Ok(e) => e,
        Err(e) => {
            let err_msg = emit_error("generating-config", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    };

    if let Err(e) = std::fs::write(docker_dir.join(".env"), &env) {
        let err_msg = emit_error("generating-config", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    emit_progress("generating-config", "Configuration files generated", false, None);

    if let Err(e) = check_cancelled() {
        state.build_tracker.unregister(&id).await;
        return Err(e);
    }

    // ========== Stage 3: Build Docker image ==========
    emit_progress("building-image", "Building Docker image...", false, None);

    let (tx, mut rx) = mpsc::channel::<String>(100);
    let image_name_clone = image_name.clone();
    let source_dir_clone = source_dir.clone();
    let config_clone = config.clone();

    let build_handle = tokio::spawn(async move {
        let docker = DockerCli::new();
        let mut build_args = HashMap::new();

        // Add build args from config - map to OpenClaw Dockerfile's expected args
        if !config_clone.apt_packages.is_empty() {
            build_args.insert("OPENCLAW_DOCKER_APT_PACKAGES".to_string(), config_clone.apt_packages.clone());
        }
        if !config_clone.extensions.is_empty() {
            build_args.insert("OPENCLAW_EXTENSIONS".to_string(), config_clone.extensions.clone());
        }
        if config_clone.install_browser {
            build_args.insert("OPENCLAW_INSTALL_BROWSER".to_string(), "1".to_string());
        }

        // Build using the source directory as context (contains Dockerfile)
        docker.build(&source_dir_clone, &image_name_clone, &build_args, tx).await
    });

    // Forward build output
    while let Some(line) = rx.recv().await {
        emit_progress("building-image", &line, false, None);
    }

    // Wait for build to complete and check result
    match build_handle.await {
        Ok(Ok(())) => {
            emit_progress("building-image", "Docker image built successfully", false, None);
        }
        Ok(Err(e)) => {
            let err_msg = emit_error("building-image", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
        Err(e) => {
            let err_msg = emit_error("building-image", &e.to_string());
            state.build_tracker.unregister(&id).await;
            return Err(err_msg);
        }
    }

    if let Err(e) = check_cancelled() {
        state.build_tracker.unregister(&id).await;
        return Err(e);
    }

    // ========== Stage 4: Verify directories ==========
    emit_progress("verifying-directories", "Verifying directories...", false, None);

    // Ensure all directories exist
    if let Err(e) = std::fs::create_dir_all(config.config_path()) {
        let err_msg = emit_error("verifying-directories", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    if let Err(e) = std::fs::create_dir_all(config.workspace_path()) {
        let err_msg = emit_error("verifying-directories", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    // These subdirectories are optional - log warnings but don't fail
    if let Err(e) = std::fs::create_dir_all(config.config_path().join("identity")) {
        warn!("Failed to create identity directory: {}", e);
    }
    if let Err(e) = std::fs::create_dir_all(config.config_path().join("agents/main/agent")) {
        warn!("Failed to create agents directory: {}", e);
    }
    if let Err(e) = std::fs::create_dir_all(config.config_path().join("agents/main/sessions")) {
        warn!("Failed to create sessions directory: {}", e);
    }

    emit_progress("verifying-directories", "Directories verified", false, None);

    if let Err(e) = check_cancelled() {
        state.build_tracker.unregister(&id).await;
        return Err(e);
    }

    // ========== Stage 5: Start container ==========
    emit_progress("starting-container", "Starting container...", false, None);

    if let Err(e) = state.docker_cli.compose_up(&compose_path, &project_name).await {
        let err_msg = emit_error("starting-container", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    // Wait a moment for container to be ready
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    emit_progress("starting-container", "Container started", false, None);

    if let Err(e) = check_cancelled() {
        state.build_tracker.unregister(&id).await;
        return Err(e);
    }

    // ========== Stage 6: Run onboarding ==========
    emit_progress("running-onboarding", "Running initial setup...", false, None);

    let onboarding_result = state.docker_cli.compose_run(
        &compose_path,
        &project_name,
        &format!("{}-cli", project_name),
        &["onboard", "--mode", "local", "--no-install-daemon"],
        None,
    ).await;

    match onboarding_result {
        Ok(output) => {
            emit_progress("running-onboarding", &format!("Onboarding output: {}", output.lines().take(3).collect::<Vec<_>>().join("\n")), false, None);
        }
        Err(e) => {
            // Onboarding might fail if already set up - log but continue
            warn!("Onboarding warning (non-fatal): {}", e);
            emit_progress("running-onboarding", "Onboarding completed (may have been run before)", false, None);
        }
    }

    // ========== Stage 7: Fix permissions ==========
    emit_progress("fixing-permissions", "Fixing file permissions...", false, None);

    let perm_result = state.docker_cli.compose_run_with_entrypoint(
        &compose_path,
        &project_name,
        &format!("{}-cli", project_name),
        &["-c", "find /home/node/.openclaw -xdev -exec chown node:node {} +"],
        Some("root"),
        Some("sh"),
    ).await;

    match perm_result {
        Ok(_) => emit_progress("fixing-permissions", "Permissions fixed", false, None),
        Err(e) => {
            warn!("Permission fix warning (non-fatal): {}", e);
            emit_progress("fixing-permissions", "Permissions check completed", false, None);
        }
    }

    // ========== Stage 8: Configure gateway ==========
    emit_progress("configuring-gateway", "Configuring gateway settings...", false, None);

    // Set gateway mode - errors are non-fatal as CLI may not exist
    let gateway_mode = "local";
    if let Err(e) = state.docker_cli.compose_run(
        &compose_path,
        &project_name,
        &format!("{}-cli", project_name),
        &["config", "set", "gateway.mode", gateway_mode],
        None,
    ).await {
        warn!("Gateway mode config warning (non-fatal): {}", e);
    }

    // Set gateway bind
    let bind_mode = match config.gateway_bind {
        crate::instance::GatewayBind::Loopback => "loopback",
        crate::instance::GatewayBind::Lan => "lan",
    };
    if let Err(e) = state.docker_cli.compose_run(
        &compose_path,
        &project_name,
        &format!("{}-cli", project_name),
        &["config", "set", "gateway.bind", bind_mode],
        None,
    ).await {
        warn!("Gateway bind config warning (non-fatal): {}", e);
    }

    emit_progress("configuring-gateway", "Gateway configured", false, None);

    // ========== Stage 9: Restart gateway ==========
    emit_progress("restarting-gateway", "Restarting gateway to apply changes...", false, None);

    // Stop and start to pick up config changes - stop errors are non-fatal
    if let Err(e) = state.docker_cli.compose_stop(&compose_path, &project_name).await {
        warn!("Gateway stop warning (non-fatal): {}", e);
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    if let Err(e) = state.docker_cli.compose_up(&compose_path, &project_name).await {
        let err_msg = emit_error("restarting-gateway", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    emit_progress("restarting-gateway", "Gateway restarted", false, None);

    // ========== Done! ==========
    emit_progress("complete", &format!("Build complete! Gateway running at {}", config.gateway_url()), true, None);

    // Unregister from build tracker
    state.build_tracker.unregister(&id).await;

    info!("Build complete for instance {}", id);
    Ok(())
}

/// Cancel an active build
#[tauri::command]
pub async fn cancel_build(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Cancelling build for instance {}", id);

    let cancelled = state.build_tracker.cancel(&id).await;

    if cancelled {
        info!("Build cancellation signalled for instance {}", id);
        Ok(())
    } else {
        warn!("No active build found for instance {}", id);
        Err(format!("No active build found for instance {}", id))
    }
}
