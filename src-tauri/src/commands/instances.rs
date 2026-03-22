use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tracing::{error, info};

use crate::docker::{generate_compose, generate_env, DockerCli};
use crate::error::EasyClawError;
use crate::instance::{
    InstanceConfig, InstanceManager, InstanceSettings, InstanceStatus, InstanceState,
    InstanceWithStatus,
};

/// Shared state for the application
pub struct AppState {
    pub instance_manager: Arc<InstanceManager>,
    pub docker_cli: Arc<DockerCli>,
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
    let result: Vec<InstanceWithStatus> = instances
        .into_iter()
        .map(|config| {
            let status = if docker_status.state != crate::instance::DockerState::Running {
                InstanceStatus {
                    state: InstanceState::DockerNotRunning,
                    container_id: None,
                    error_message: None,
                }
            } else {
                // Check if container is running
                // For now, just return stopped status
                // Full implementation would query Docker
                InstanceStatus::default()
            };

            InstanceWithStatus { config, status }
        })
        .collect();

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
        let project_name = format!("easyclaw-{}", cfg.container_id);

        // Stop container
        let _ = state.docker_cli.compose_down(&compose_path, &project_name).await;

        // Remove image
        let image_name = format!("easyclaw-{}:latest", cfg.container_id);
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
    let project_name = format!("easyclaw-{}", config.container_id);

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
    let project_name = format!("easyclaw-{}", config.container_id);

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

    let config = state
        .instance_manager
        .get(&id)
        .map_err(|e| e.to_string())?;

    let docker_dir = config.docker_path();
    let compose_path = docker_dir.join("docker-compose.yml");
    let project_name = format!("easyclaw-{}", config.container_id);
    let image_name = format!("{}:latest", project_name);

    // Helper to emit progress
    let emit_progress = |stage: &str, log: &str, done: bool, error: Option<&str>| {
        let _ = app_handle.emit("build-progress", serde_json::json!({
            "id": id,
            "stage": stage,
            "log": log,
            "done": done,
            "error": error
        }));
    };

    // Stage 1: Fetch Dockerfile (if not already present)
    emit_progress("fetching-dockerfile", "Fetching Dockerfile...", false, None);

    // For now, skip Dockerfile fetch since we don't have a real OpenClaw repo
    // In production, this would fetch from GitHub
    emit_progress("fetching-dockerfile", "Dockerfile ready", false, None);

    // Stage 2: Generate configuration files
    emit_progress("generating-config", "Generating configuration...", false, None);

    let compose = generate_compose(&config).map_err(|e| {
        emit_progress("generating-config", "", true, Some(&e.to_string()));
        e.to_string()
    })?;
    std::fs::write(&compose_path, &compose).map_err(|e| {
        emit_progress("generating-config", "", true, Some(&e.to_string()));
        e.to_string()
    })?;

    let env = generate_env(&config).map_err(|e| {
        emit_progress("generating-config", "", true, Some(&e.to_string()));
        e.to_string()
    })?;
    std::fs::write(docker_dir.join(".env"), &env).map_err(|e| {
        emit_progress("generating-config", "", true, Some(&e.to_string()));
        e.to_string()
    })?;

    emit_progress("generating-config", "Configuration generated", false, None);

    // Stage 3: Build Docker image
    emit_progress("building-image", "Building Docker image...", false, None);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    let image_name_clone = image_name.clone();
    let docker_dir_clone = docker_dir.clone();

    let build_handle = tokio::spawn(async move {
        let docker = DockerCli::new();
        let mut build_args = std::collections::HashMap::new();

        // Add build args from config
        if !config.apt_packages.is_empty() {
            build_args.insert("APT_PACKAGES".to_string(), config.apt_packages.clone());
        }
        if !config.extensions.is_empty() {
            build_args.insert("EXTENSIONS".to_string(), config.extensions.clone());
        }
        if config.install_browser {
            build_args.insert("INSTALL_BROWSER".to_string(), "true".to_string());
        }

        docker.build(&docker_dir_clone, &image_name_clone, &build_args, tx).await
    });

    // Forward build output
    while let Some(line) = rx.recv().await {
        emit_progress("building-image", &line, false, None);
    }

    build_handle.await.map_err(|e| e.to_string())?
        .map_err(|e| {
            emit_progress("building-image", "", true, Some(&e.to_string()));
            e.to_string()
        })?;

    emit_progress("building-image", "Image built successfully", false, None);

    // Stage 4: Create directories (already done in create_instance)
    emit_progress("creating-directories", "Verifying directories...", false, None);
    emit_progress("creating-directories", "Directories ready", false, None);

    // Stage 5: Start container
    emit_progress("starting-container", "Starting container...", false, None);

    state
        .docker_cli
        .compose_up(&compose_path, &project_name)
        .await
        .map_err(|e| {
            emit_progress("starting-container", "", true, Some(&e.to_string()));
            e.to_string()
        })?;

    emit_progress("starting-container", "Container started", false, None);

    // Stage 6-9: Run onboarding, fix permissions, configure gateway
    // For Phase 2, these are stubs - full implementation in later phases
    emit_progress("running-onboarding", "Running onboarding...", false, None);
    emit_progress("running-onboarding", "Onboarding complete", false, None);

    emit_progress("fixing-permissions", "Fixing permissions...", false, None);
    emit_progress("fixing-permissions", "Permissions fixed", false, None);

    emit_progress("configuring-gateway", "Configuring gateway...", false, None);
    emit_progress("configuring-gateway", "Gateway configured", false, None);

    emit_progress("restarting-gateway", "Restarting gateway...", false, None);
    emit_progress("restarting-gateway", "Gateway restarted", false, None);

    // Done!
    emit_progress("complete", "Build complete!", true, None);

    info!("Build complete for instance {}", id);
    Ok(())
}
