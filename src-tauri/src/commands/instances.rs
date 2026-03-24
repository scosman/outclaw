use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::docker::{fetch_release_source, generate_compose, generate_env, DockerCli};
use crate::github::ReleasesClient;
use crate::instance::{
    InstanceConfig, InstanceManager, InstanceSettings, InstanceState, InstanceStatus,
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
        self.active_builds
            .write()
            .await
            .insert(instance_id.to_string(), token.clone());
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
            instance_manager: Arc::new(
                InstanceManager::new().expect("Failed to create InstanceManager"),
            ),
            docker_cli: Arc::new(DockerCli::new()),
            build_tracker: Arc::new(BuildTracker::new()),
        }
    }
}

/// List all instances with their current status
#[tauri::command]
pub async fn list_instances(state: State<'_, AppState>) -> Result<Vec<InstanceWithStatus>, String> {
    info!("Listing instances");

    let instances = state.instance_manager.list().map_err(|e| e.to_string())?;

    // Get Docker status
    let docker_status = state
        .docker_cli
        .check_available()
        .await
        .map_err::<String, _>(|e| e.to_string())?;

    // Build status for each instance
    let mut result = Vec::new();

    if docker_status.state != crate::instance::DockerState::Running {
        // Docker not running - all instances show docker-not-running
        for config in instances {
            let status = InstanceStatus {
                state: InstanceState::DockerNotRunning,
                container_id: None,
                error_message: None,
            };
            result.push(InstanceWithStatus { config, status });
        }
    } else {
        // Single docker call to get all instance statuses
        let instance_statuses = state
            .docker_cli
            .get_outclaw_instance_statuses()
            .await
            .map_err(|e| e.to_string())?;

        for config in instances {
            let status = instance_statuses
                .get(&config.id)
                .map(|(running, container_id)| InstanceStatus {
                    state: if *running {
                        InstanceState::Running
                    } else {
                        InstanceState::Stopped
                    },
                    container_id: container_id.clone(),
                    error_message: None,
                })
                .unwrap_or(InstanceStatus {
                    state: InstanceState::Stopped,
                    container_id: None,
                    error_message: None,
                });
            result.push(InstanceWithStatus { config, status });
        }
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

    let config = state.instance_manager.get(&id).map_err(|e| e.to_string())?;

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
    std::fs::write(docker_dir.join("docker-compose.yml"), compose).map_err(|e| e.to_string())?;

    // Generate .env
    let env = generate_env(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join(".env"), env).map_err(|e| e.to_string())?;

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
    std::fs::write(docker_dir.join("docker-compose.yml"), compose).map_err(|e| e.to_string())?;

    let env = generate_env(&config).map_err(|e| e.to_string())?;
    std::fs::write(docker_dir.join(".env"), env).map_err(|e| e.to_string())?;

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
pub async fn delete_instance(id: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("Deleting instance {}", id);

    // Get config before deletion to clean up Docker resources
    let config = state.instance_manager.get(&id);
    if config.is_err() {
        warn!(
            "Could not get config for instance {} during delete: {:?}",
            id,
            config.as_ref().err()
        );
    }
    let config = config.ok();

    // Stop and remove container if running
    if let Some(ref cfg) = config {
        let docker_dir = cfg.docker_path();
        let compose_path = docker_dir.join("docker-compose.yml");
        let project_name = format!("outclaw-{}", cfg.container_id);

        // Stop container
        if let Err(e) = state
            .docker_cli
            .compose_down(&compose_path, &project_name)
            .await
        {
            warn!("Failed to stop containers for instance {}: {}", id, e);
        }

        // Remove image
        let image_name = format!("outclaw-{}:latest", cfg.container_id);
        if let Err(e) = state.docker_cli.remove_image(&image_name).await {
            warn!("Failed to remove image for instance {}: {}", id, e);
        }
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
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting instance {}", id);

    let config = state.instance_manager.get(&id).map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);

    state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    info!("Started instance {}", id);

    // Emit updated status immediately
    emit_instance_status(&id, &app_handle, &state).await;

    Ok(())
}

/// Stop an instance
#[tauri::command]
pub async fn stop_instance(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Stopping instance {}", id);

    let config = state.instance_manager.get(&id).map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);

    state
        .docker_cli
        .compose_stop(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    info!("Stopped instance {}", id);

    // Emit updated status immediately
    emit_instance_status(&id, &app_handle, &state).await;

    Ok(())
}

/// Restart an instance
#[tauri::command]
pub async fn restart_instance(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Restarting instance {}", id);

    let config = state.instance_manager.get(&id).map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);

    state
        .docker_cli
        .compose_down(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
        .map_err(|e| e.to_string())?;

    info!("Restarted instance {}", id);

    // Emit updated status immediately
    emit_instance_status(&id, &app_handle, &state).await;

    Ok(())
}

/// Emit instance status to frontend
async fn emit_instance_status(id: &str, app_handle: &AppHandle, state: &AppState) {
    // Get the current status from Docker
    let status = match state.docker_cli.get_outclaw_instance_statuses().await {
        Ok(statuses) => statuses
            .get(id)
            .map(|(running, container_id)| InstanceStatus {
                state: if *running {
                    InstanceState::Running
                } else {
                    InstanceState::Stopped
                },
                container_id: container_id.clone(),
                error_message: None,
            })
            .unwrap_or(InstanceStatus {
                state: InstanceState::Stopped,
                container_id: None,
                error_message: None,
            }),
        Err(_) => InstanceStatus {
            state: InstanceState::Stopped,
            container_id: None,
            error_message: None,
        },
    };

    if let Err(e) = app_handle.emit(
        "instance-status-changed",
        serde_json::json!({
            "id": id,
            "status": status
        }),
    ) {
        warn!("Failed to emit instance-status-changed: {}", e);
    }
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

    let config = state.instance_manager.get(&id).map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("outclaw-{}", config.container_id);
    let image_name = format!("{}:latest", project_name);

    // Helper to emit progress
    let emit_progress = |stage: &str, log: &str, done: bool, error: Option<&str>| {
        if let Err(e) = app_handle.emit(
            "build-progress",
            serde_json::json!({
                "id": id,
                "stage": stage,
                "log": log,
                "done": done,
                "error": error
            }),
        ) {
            warn!("Failed to emit build progress: {}", e);
        }
    };

    // Helper to emit and return error
    let emit_error = |stage: &str, e: &str| -> String {
        emit_progress(stage, "", true, Some(e));
        e.to_string()
    };

    // ========== Stage 1: Fetch source ==========
    emit_progress(
        "fetching-source",
        "Fetching OpenClaw source from GitHub...",
        false,
        None,
    );

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
            emit_progress(
                "fetching-source",
                "Warning: Could not fetch releases list, proceeding with version tag",
                false,
                None,
            );
            vec![]
        }
    };

    // Find the release matching our version
    let release = releases
        .iter()
        .find(|r| r.tag == config.openclaw_version)
        .cloned()
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
            emit_progress(
                "fetching-source",
                &format!("Source ready for {}", release.tag),
                false,
                None,
            );
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
    emit_progress(
        "generating-config",
        "Generating docker-compose.yml and .env...",
        false,
        None,
    );

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

    emit_progress(
        "generating-config",
        "Configuration files generated",
        false,
        None,
    );

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
            build_args.insert(
                "OPENCLAW_DOCKER_APT_PACKAGES".to_string(),
                config_clone.apt_packages.clone(),
            );
        }
        if !config_clone.extensions.is_empty() {
            build_args.insert(
                "OPENCLAW_EXTENSIONS".to_string(),
                config_clone.extensions.clone(),
            );
        }
        if config_clone.install_browser {
            build_args.insert("OPENCLAW_INSTALL_BROWSER".to_string(), "1".to_string());
        }

        // Build using the source directory as context (contains Dockerfile)
        docker
            .build(&source_dir_clone, &image_name_clone, &build_args, tx)
            .await
    });

    // Forward build output
    while let Some(line) = rx.recv().await {
        emit_progress("building-image", &line, false, None);
    }

    // Wait for build to complete and check result
    match build_handle.await {
        Ok(Ok(())) => {
            emit_progress(
                "building-image",
                "Docker image built successfully",
                false,
                None,
            );
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
    emit_progress(
        "verifying-directories",
        "Verifying directories...",
        false,
        None,
    );

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

    if let Err(e) = state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
    {
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
    emit_progress(
        "running-onboarding",
        "Running initial setup...",
        false,
        None,
    );

    let onboarding_result = state
        .docker_cli
        .compose_run(
            &compose_path,
            &project_name,
            &format!("{}-cli", project_name),
            &["onboard", "--mode", "local", "--no-install-daemon"],
            None,
        )
        .await;

    match onboarding_result {
        Ok(output) => {
            emit_progress(
                "running-onboarding",
                &format!(
                    "Onboarding output: {}",
                    output.lines().take(3).collect::<Vec<_>>().join("\n")
                ),
                false,
                None,
            );
        }
        Err(e) => {
            // Onboarding might fail if already set up - log but continue
            warn!("Onboarding warning (non-fatal): {}", e);
            emit_progress(
                "running-onboarding",
                "Onboarding completed (may have been run before)",
                false,
                None,
            );
        }
    }

    // ========== Stage 7: Fix permissions ==========
    emit_progress(
        "fixing-permissions",
        "Fixing file permissions...",
        false,
        None,
    );

    let perm_result = state
        .docker_cli
        .compose_run_with_entrypoint(
            &compose_path,
            &project_name,
            &format!("{}-cli", project_name),
            &[
                "-c",
                "find /home/node/.openclaw -xdev -exec chown node:node {} +",
            ],
            Some("root"),
            Some("sh"),
        )
        .await;

    match perm_result {
        Ok(_) => emit_progress("fixing-permissions", "Permissions fixed", false, None),
        Err(e) => {
            warn!("Permission fix warning (non-fatal): {}", e);
            emit_progress(
                "fixing-permissions",
                "Permissions check completed",
                false,
                None,
            );
        }
    }

    // ========== Stage 8: Configure gateway ==========
    emit_progress(
        "configuring-gateway",
        "Configuring gateway settings...",
        false,
        None,
    );

    // Set gateway mode - errors are non-fatal as CLI may not exist
    let gateway_mode = "local";
    if let Err(e) = state
        .docker_cli
        .compose_run(
            &compose_path,
            &project_name,
            &format!("{}-cli", project_name),
            &["config", "set", "gateway.mode", gateway_mode],
            None,
        )
        .await
    {
        warn!("Gateway mode config warning (non-fatal): {}", e);
    }

    // Set gateway bind
    let bind_mode = match config.gateway_bind {
        crate::instance::GatewayBind::Loopback => "loopback",
        crate::instance::GatewayBind::Lan => "lan",
    };
    if let Err(e) = state
        .docker_cli
        .compose_run(
            &compose_path,
            &project_name,
            &format!("{}-cli", project_name),
            &["config", "set", "gateway.bind", bind_mode],
            None,
        )
        .await
    {
        warn!("Gateway bind config warning (non-fatal): {}", e);
    }

    emit_progress("configuring-gateway", "Gateway configured", false, None);

    // ========== Stage 9: Restart gateway ==========
    emit_progress(
        "restarting-gateway",
        "Restarting gateway to apply changes...",
        false,
        None,
    );

    // Stop and start to pick up config changes - stop errors are non-fatal
    if let Err(e) = state
        .docker_cli
        .compose_stop(&compose_path, &project_name)
        .await
    {
        warn!("Gateway stop warning (non-fatal): {}", e);
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    if let Err(e) = state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
    {
        let err_msg = emit_error("restarting-gateway", &e.to_string());
        state.build_tracker.unregister(&id).await;
        return Err(err_msg);
    }

    emit_progress("restarting-gateway", "Gateway restarted", false, None);

    // ========== Done! ==========
    emit_progress(
        "complete",
        &format!(
            "Build complete! Gateway running at {}",
            config.gateway_url()
        ),
        true,
        None,
    );

    // Unregister from build tracker
    state.build_tracker.unregister(&id).await;

    info!("Build complete for instance {}", id);
    Ok(())
}

/// Cancel an active build
#[tauri::command]
pub async fn cancel_build(id: String, state: State<'_, AppState>) -> Result<(), String> {
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

/// Connect a provider to an instance
/// Runs the onboard command inside the gateway container with the provided credentials
#[tauri::command]
pub async fn connect_provider(
    instance_id: String,
    auth_choice: String,
    fields: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!(
        "Connecting provider {} for instance {}",
        auth_choice, instance_id
    );

    // Get the instance config to find the container ID
    let config = state
        .instance_manager
        .get(&instance_id)
        .map_err(|e| e.to_string())?;

    // Build the container name: outclaw-{containerId}-gateway
    let container_name = format!("outclaw-{}-gateway", config.container_id);

    // Build the onboard command arguments
    let mut args = vec![
        "openclaw".to_string(),
        "onboard".to_string(),
        "--non-interactive".to_string(),
        "--accept-risk".to_string(),
        "--auth-choice".to_string(),
        auth_choice.clone(),
    ];

    // Add field flags
    for (field_name, value) in &fields {
        if !value.is_empty() {
            args.push(format!("--{}", field_name));
            args.push(value.clone());
        }
    }

    // Log only safe info - container name, auth choice, and field count (not values which may contain API keys)
    info!(
        "Running docker exec in {} for provider {} with {} field(s)",
        container_name,
        auth_choice,
        fields.len()
    );

    // Validate field names to prevent unexpected behavior
    let field_name_pattern = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
    for field_name in fields.keys() {
        if !field_name_pattern.is_match(field_name) {
            return Err(format!(
                "Invalid field name: '{}'. Field names must match pattern ^[a-z0-9-]+$",
                field_name
            ));
        }
    }

    // Execute the onboard command in the container
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let _onboard_result = state
        .docker_cli
        .docker_exec(&container_name, &args_ref)
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to connect provider: {}", e);
            warn!("{}", err_msg);
            err_msg
        })?;

    // Wait for credential restart to complete before testing connection
    sleep(Duration::from_secs(5)).await;

    // Validate the connection by sending a test message (check exit code, not JSON)
    let test_args = [
        "openclaw",
        "agent",
        "--message",
        "Testing connection. Reply with just the word 'OK'",
        "--local",
        "--agent",
        "main",
    ];
    let _ = state
        .docker_cli
        .docker_exec(&container_name, &test_args)
        .await
        .map_err(|_| {
            let err_msg =
                "Provider connection test failed: ensure your API key is valid and try again."
                    .to_string();
            warn!("{}", err_msg);
            err_msg
        })?;

    info!(
        "Provider {} connected and validated successfully for instance {}",
        auth_choice, instance_id
    );

    Ok(())
}

/// Connect WhatsApp channel to an instance
/// Installs the channel and runs login with streaming QR code output
#[tauri::command]
pub async fn connect_whatsapp(
    instance_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Connecting WhatsApp for instance {}", instance_id);

    // Get the instance config to find the container ID
    let config = state
        .instance_manager
        .get(&instance_id)
        .map_err(|e| e.to_string())?;

    // Build the container name: outclaw-{containerId}-gateway
    let container_name = format!("outclaw-{}-gateway", config.container_id);

    // Helper to emit progress
    let emit_progress = |log: &str, done: bool, error: Option<&str>| {
        if let Err(e) = app_handle.emit(
            "whatsapp-progress",
            serde_json::json!({
                "id": instance_id,
                "log": log,
                "done": done,
                "error": error
            }),
        ) {
            warn!("Failed to emit whatsapp progress: {}", e);
        }
    };

    // Create channel for streaming output
    let (tx, mut rx) = mpsc::channel::<String>(100);
    let container_name_clone = container_name.clone();

    // Run channel add first (installs WhatsApp)
    emit_progress("Installing WhatsApp channel...", false, None);

    let add_result = state
        .docker_cli
        .docker_exec(
            &container_name,
            &["openclaw", "channels", "add", "--channel", "whatsapp"],
        )
        .await;

    match add_result {
        Ok(output) => {
            // Channel add succeeded (or was already installed)
            if output.contains("already installed") || output.contains("already exists") {
                emit_progress("WhatsApp channel already installed", false, None);
            } else {
                emit_progress(
                    "WhatsApp channel installed, waiting for restart...",
                    false,
                    None,
                );
            }
        }
        Err(e) => {
            // Non-fatal - channel might already be installed
            warn!("Channel add warning (non-fatal): {}", e);
            emit_progress(
                "WhatsApp channel ready, waiting for restart...",
                false,
                None,
            );
        }
    }

    // Wait 5 seconds for the gateway to restart after channel installation
    emit_progress("Waiting for gateway to restart...", false, None);
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Now run the login command with streaming output
    emit_progress("Starting WhatsApp login...", false, None);

    // Spawn the streaming exec in a separate task
    let tx_clone = tx.clone();
    let handle = tokio::spawn(async move {
        let docker = DockerCli::new();
        docker
            .docker_exec_streaming(
                &container_name_clone,
                &["openclaw", "channels", "login", "--channel", "whatsapp"],
                tx_clone,
            )
            .await
    });

    // Drop the original sender so the channel closes when the task finishes
    drop(tx);

    // Forward output to frontend
    while let Some(line) = rx.recv().await {
        emit_progress(&line, false, None);
    }

    // Wait for command to complete
    match handle.await {
        Ok(Ok(())) => {
            emit_progress("WhatsApp connected successfully!", true, None);
            info!(
                "WhatsApp connected successfully for instance {}",
                instance_id
            );
            Ok(())
        }
        Ok(Err(e)) => {
            let err_msg = format!("WhatsApp connection failed: {}", e);
            emit_progress(&err_msg, true, Some(&err_msg));
            Err(err_msg)
        }
        Err(e) => {
            let err_msg = format!("WhatsApp connection task failed: {}", e);
            emit_progress(&err_msg, true, Some(&err_msg));
            Err(err_msg)
        }
    }
}

/// Add Telegram channel to an instance with bot token
/// Runs: openclaw channels add --channel telegram --token "TOKEN"
#[tauri::command]
pub async fn add_telegram_channel(
    instance_id: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Adding Telegram channel for instance {}", instance_id);

    // Validate token format: numeric_id:alphanumeric_string
    let token_pattern = regex::Regex::new(r"^\d+:[A-Za-z0-9_-]+$").unwrap();
    if !token_pattern.is_match(&token) {
        return Err("Invalid token format. Expected: 1234567890:ABCdef...".to_string());
    }

    // Get the instance config to find the container ID
    let config = state
        .instance_manager
        .get(&instance_id)
        .map_err(|e| e.to_string())?;

    // Build the container name: outclaw-{containerId}-gateway
    let container_name = format!("outclaw-{}-gateway", config.container_id);

    // Execute the channels add command
    let args = [
        "openclaw",
        "channels",
        "add",
        "--channel",
        "telegram",
        "--token",
        &token,
    ];

    info!(
        "Running docker exec to add Telegram channel for instance {}",
        instance_id
    );

    state
        .docker_cli
        .docker_exec(&container_name, &args)
        .await
        .map_err(|e| {
            // Sanitize error message - don't leak the token
            let err_str = e.to_string();
            let sanitized = err_str.replace(&token, "***TOKEN***");
            warn!("Failed to add Telegram channel: {}", sanitized);
            sanitized
        })?;

    info!(
        "Telegram channel added successfully for instance {}",
        instance_id
    );
    Ok(())
}

/// Approve Telegram pairing code
/// Runs: openclaw pairing approve telegram PAIRING_CODE
#[tauri::command]
pub async fn approve_telegram_pairing(
    instance_id: String,
    pairing_code: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Approving Telegram pairing for instance {}", instance_id);

    // Validate pairing code format: alphanumeric with optional dashes/underscores
    let code_pattern = regex::Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    if !code_pattern.is_match(&pairing_code) {
        return Err("Invalid pairing code format".to_string());
    }

    // Get the instance config to find the container ID
    let config = state
        .instance_manager
        .get(&instance_id)
        .map_err(|e| e.to_string())?;

    // Build the container name: outclaw-{containerId}-gateway
    let container_name = format!("outclaw-{}-gateway", config.container_id);

    // Execute the pairing approve command
    let args = ["openclaw", "pairing", "approve", "telegram", &pairing_code];

    info!(
        "Running docker exec to approve Telegram pairing for instance {}",
        instance_id
    );

    state
        .docker_cli
        .docker_exec(&container_name, &args)
        .await
        .map_err(|e| {
            let err_msg = e.to_string();
            warn!("Failed to approve Telegram pairing: {}", err_msg);
            err_msg
        })?;

    info!(
        "Telegram pairing approved successfully for instance {}",
        instance_id
    );
    Ok(())
}
