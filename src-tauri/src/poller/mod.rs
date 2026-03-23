#![allow(dead_code)] // Code for future phases

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};

use crate::docker::DockerCli;
use crate::instance::{DockerState, InstanceState, InstanceStatus};
use crate::commands::instances::AppState;

/// Background poller for Docker and instance status
pub struct Poller {
    interval: RwLock<Duration>,
    cancel_token: tokio_util::sync::CancellationToken,
}

impl Poller {
    /// Create a new poller
    pub fn new() -> Self {
        Self {
            interval: RwLock::new(Duration::from_secs(5)),
            cancel_token: tokio_util::sync::CancellationToken::new(),
        }
    }

    /// Start the poller in a background task
    pub fn start(self: Arc<Self>, app_handle: AppHandle, state: Arc<AppState>) {
        let cancel_token = self.cancel_token.clone();

        tokio::spawn(async move {
            let mut ticker = interval(*self.interval.read().await);
            let mut last_docker_state: Option<DockerState> = None;
            let mut last_instance_states: HashMap<String, InstanceStatus> = HashMap::new();

            info!("Status poller started");

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Status poller stopped");
                        break;
                    }
                    _ = ticker.tick() => {
                        let current_interval = *self.interval.read().await;
                        ticker = interval(current_interval);

                        // Check Docker status
                        match state.docker_cli.check_available().await {
                            Ok(docker_status) => {
                                // Emit if changed
                                if last_docker_state != Some(docker_status.state) {
                                    debug!("Docker state changed: {:?}", docker_status.state);
                                    last_docker_state = Some(docker_status.state);

                                    if let Err(e) = app_handle.emit("docker-status-changed", &docker_status) {
                                        error!("Failed to emit docker-status-changed: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to check Docker status: {}", e);
                            }
                        }

                        // Check instance statuses (only if Docker is running)
                        if last_docker_state == Some(DockerState::Running) {
                            if let Ok(instances) = state.instance_manager.list() {
                                for config in instances {
                                    // Query Docker for container status
                                    let status = get_instance_status(&state.docker_cli, &config.id, &config.container_id).await;

                                    // Emit if changed
                                    let last_status = last_instance_states.get(&config.id);
                                    if last_status.map(|s| &s.state) != Some(&status.state) {
                                        debug!("Instance {} state changed: {:?}", config.id, status.state);
                                        last_instance_states.insert(config.id.clone(), status.clone());

                                        if let Err(e) = app_handle.emit("instance-status-changed", serde_json::json!({
                                            "id": config.id,
                                            "status": status
                                        })) {
                                            error!("Failed to emit instance-status-changed: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Stop the poller
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }

    /// Set the polling interval
    pub async fn set_interval(&self, duration: Duration) {
        *self.interval.write().await = duration;
        debug!("Poller interval set to {:?}", duration);
    }
}

impl Default for Poller {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the status of a single instance
async fn get_instance_status(
    docker_cli: &DockerCli,
    instance_id: &str,
    _container_id: &str,
) -> InstanceStatus {
    // Try to find the container by label
    match docker_cli.list_containers(&format!("easyclaw.instance={}", instance_id)).await {
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
                // Container doesn't exist
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
}
