use crate::docker::DockerCli;
use crate::instance::{DockerState, DockerStatus};

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
