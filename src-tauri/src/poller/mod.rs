#![allow(dead_code)] // Code for future phases

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::{Notify, RwLock};
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info};

use crate::commands::instances::AppState;
use crate::docker::DockerCli;
use crate::instance::{DockerState, InstanceState, InstanceStatus};

/// Background poller for Docker and instance status
pub struct Poller {
    interval: RwLock<Duration>,
    cancel_token: tokio_util::sync::CancellationToken,
    /// Notifies the poller to wake up immediately (e.g., on focus change)
    wake_notify: Notify,
}

impl Poller {
    /// Create a new poller
    pub fn new() -> Self {
        Self {
            interval: RwLock::new(Duration::from_secs(5)),
            cancel_token: tokio_util::sync::CancellationToken::new(),
            wake_notify: Notify::new(),
        }
    }

    /// Start the poller in a background task
    pub fn start(self: Arc<Self>, app_handle: AppHandle, state: Arc<AppState>) {
        let cancel_token = self.cancel_token.clone();

        tokio::spawn(async move {
            let mut last_docker_state: Option<DockerState> = None;
            let mut last_instance_states: HashMap<String, InstanceStatus> = HashMap::new();

            info!("Status poller started");

            loop {
                // Get the current interval
                let current_interval = *self.interval.read().await;
                let deadline = Instant::now() + current_interval;

                // Wait for the interval, cancellation, or wake notification
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Status poller stopped");
                        break;
                    }
                    _ = self.wake_notify.notified() => {
                        // Woken up (e.g., focus changed) - poll immediately
                        debug!("Poller woken up, polling immediately");
                    }
                    _ = sleep_until(deadline) => {
                        // Normal interval elapsed
                    }
                }

                // Poll Docker and instance status
                match state.docker_cli.check_available().await {
                    Ok(docker_status) => {
                        // Emit if changed
                        if last_docker_state != Some(docker_status.state) {
                            debug!("Docker state changed: {:?}", docker_status.state);
                            last_docker_state = Some(docker_status.state);

                            if let Err(e) = app_handle.emit("docker-status-changed", &docker_status)
                            {
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
                    // Single docker call to get all outclaw containers
                    match get_all_instance_statuses(&state.docker_cli).await {
                        Ok(instance_statuses) => {
                            if let Ok(instances) = state.instance_manager.list() {
                                for config in instances {
                                    let status = instance_statuses
                                        .get(&config.id)
                                        .cloned()
                                        .unwrap_or(InstanceStatus {
                                            state: InstanceState::Stopped,
                                            container_id: None,
                                            error_message: None,
                                        });

                                    // Emit if changed
                                    let last_status = last_instance_states.get(&config.id);
                                    if last_status.map(|s| &s.state) != Some(&status.state) {
                                        debug!(
                                            "Instance {} state changed: {:?}",
                                            config.id, status.state
                                        );
                                        last_instance_states
                                            .insert(config.id.clone(), status.clone());

                                        if let Err(e) = app_handle.emit(
                                            "instance-status-changed",
                                            serde_json::json!({
                                                "id": config.id,
                                                "status": status
                                            }),
                                        ) {
                                            error!("Failed to emit instance-status-changed: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to get instance statuses: {}", e);
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

    /// Set the polling interval and wake the poller to apply immediately
    pub fn set_interval(&self, duration: Duration) {
        // Use blocking write since this is called from a Tauri command
        if let Ok(mut interval) = self.interval.try_write() {
            *interval = duration;
            debug!("Poller interval set to {:?}", duration);
        }
        // Wake the poller to apply the new interval immediately
        self.wake_notify.notify_one();
    }
}

impl Default for Poller {
    fn default() -> Self {
        Self::new()
    }
}

/// Default foreground polling interval (5 seconds)
pub const FOREGROUND_INTERVAL: Duration = Duration::from_secs(5);

/// Default background polling interval (30 seconds)
pub const BACKGROUND_INTERVAL: Duration = Duration::from_secs(30);

/// Get status of all outclaw instances in a single docker call
async fn get_all_instance_statuses(
    docker_cli: &DockerCli,
) -> Result<HashMap<String, InstanceStatus>, String> {
    // List all containers with outclaw.instance label (empty filter matches all)
    let containers = docker_cli
        .list_containers("outclaw.instance")
        .await
        .map_err(|e| e.to_string())?;

    let mut statuses: HashMap<String, InstanceStatus> = HashMap::new();

    for container in containers {
        // Get instance ID from label
        if let Some(instance_id) = container.labels.get("outclaw.instance") {
            let entry = statuses
                .entry(instance_id.clone())
                .or_insert(InstanceStatus {
                    state: InstanceState::Stopped,
                    container_id: None,
                    error_message: None,
                });

            // If any container for this instance is running, mark as running
            if container.is_running() {
                entry.state = InstanceState::Running;
            }

            // Store the container ID (use the first/primary one)
            if entry.container_id.is_none() {
                entry.container_id = Some(container.id.clone());
            }
        }
    }

    Ok(statuses)
}

/// Sleep until a specific deadline
async fn sleep_until(deadline: Instant) {
    sleep(deadline.saturating_duration_since(Instant::now())).await;
}
