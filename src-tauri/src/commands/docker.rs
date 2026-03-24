use tauri::State;

use crate::docker::DockerCli;
use crate::instance::{DockerState, DockerStatus};
use crate::poller::{BACKGROUND_INTERVAL, FOREGROUND_INTERVAL};
use crate::PollerState;

/// Check Docker availability
#[tauri::command]
pub async fn check_docker() -> Result<DockerStatus, String> {
    let docker = DockerCli::new();

    // Check if docker binary exists
    let docker_check = tokio::process::Command::new(&docker.docker_bin)
        .arg("--version")
        .output()
        .await;

    if docker_check.is_err() {
        return Ok(DockerStatus {
            state: DockerState::NotInstalled,
            compose_available: false,
        });
    }

    // Check if Docker daemon is running
    let info_check = tokio::process::Command::new(&docker.docker_bin)
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .await;

    let state = match info_check {
        Ok(output) if output.status.success() => DockerState::Running,
        _ => DockerState::NotRunning,
    };

    // Check if compose is available
    let compose_check = tokio::process::Command::new(&docker.docker_bin)
        .args(["compose", "version"])
        .output()
        .await;

    let compose_available = compose_check.map(|o| o.status.success()).unwrap_or(false);

    Ok(DockerStatus {
        state,
        compose_available,
    })
}

/// Set the poller interval based on window focus state
/// focused: true = foreground (5s), false = background (30s)
#[tauri::command]
pub fn set_poller_interval(focused: bool, state: State<'_, PollerState>) -> Result<(), String> {
    let interval = if focused {
        FOREGROUND_INTERVAL
    } else {
        BACKGROUND_INTERVAL
    };

    state.poller.set_interval(interval);

    tracing::debug!(
        "Poller interval set to {}s (window {})",
        interval.as_secs(),
        if focused { "focused" } else { "blurred" }
    );

    Ok(())
}
