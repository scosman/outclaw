use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, warn};

use crate::error::{OutClawError, Result};
use crate::instance::{DockerState, DockerStatus};

/// Docker CLI wrapper
pub struct DockerCli {
    pub docker_bin: String,
}

impl Default for DockerCli {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerCli {
    pub fn new() -> Self {
        // Allow override via environment variable
        let docker_bin = std::env::var("OUTCLAW_DOCKER_BIN")
            .unwrap_or_else(|_| "docker".to_string());
        Self { docker_bin }
    }

    /// Check if Docker is available and running
    pub async fn check_available(&self) -> Result<DockerStatus> {
        // Check if docker binary exists
        let docker_check = Command::new(&self.docker_bin)
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
        let info_check = Command::new(&self.docker_bin)
            .args(["info", "--format", "{{.ServerVersion}}"])
            .output()
            .await;

        let state = match info_check {
            Ok(output) if output.status.success() => DockerState::Running,
            Ok(_) => DockerState::NotRunning,
            Err(_) => DockerState::NotRunning,
        };

        // Check if compose is available
        let compose_check = Command::new(&self.docker_bin)
            .args(["compose", "version"])
            .output()
            .await;

        let compose_available = compose_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        Ok(DockerStatus {
            state,
            compose_available,
        })
    }

    /// Run docker compose up -d
    pub async fn compose_up(&self, compose_path: &Path, project_name: &str) -> Result<()> {
        info!("Running docker compose up for project {}", project_name);

        let output = Command::new(&self.docker_bin)
            .args([
                "compose",
                "-f", compose_path.to_str().unwrap(),
                "-p", project_name,
                "up",
                "-d",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker compose up failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "compose up failed: {}",
                stderr
            )));
        }

        debug!("docker compose up completed successfully");
        Ok(())
    }

    /// Run docker compose stop
    pub async fn compose_stop(&self, compose_path: &Path, project_name: &str) -> Result<()> {
        info!("Running docker compose stop for project {}", project_name);

        let output = Command::new(&self.docker_bin)
            .args([
                "compose",
                "-f", compose_path.to_str().unwrap(),
                "-p", project_name,
                "stop",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker compose stop failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "compose stop failed: {}",
                stderr
            )));
        }

        debug!("docker compose stop completed successfully");
        Ok(())
    }

    /// Run docker compose down
    pub async fn compose_down(&self, compose_path: &Path, project_name: &str) -> Result<()> {
        info!("Running docker compose down for project {}", project_name);

        let output = Command::new(&self.docker_bin)
            .args([
                "compose",
                "-f", compose_path.to_str().unwrap(),
                "-p", project_name,
                "down",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker compose down failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "compose down failed: {}",
                stderr
            )));
        }

        debug!("docker compose down completed successfully");
        Ok(())
    }

    /// Run a command in a compose service container
    pub async fn compose_run(
        &self,
        compose_path: &Path,
        project_name: &str,
        service: &str,
        args: &[&str],
        user: Option<&str>,
    ) -> Result<String> {
        self.compose_run_with_entrypoint(compose_path, project_name, service, args, user, None).await
    }

    /// Run a command in a compose service container with optional entrypoint override
    pub async fn compose_run_with_entrypoint(
        &self,
        compose_path: &Path,
        project_name: &str,
        service: &str,
        args: &[&str],
        user: Option<&str>,
        entrypoint: Option<&str>,
    ) -> Result<String> {
        info!("Running docker compose run for service {}", service);

        let mut cmd = Command::new(&self.docker_bin);
        cmd.args([
            "compose",
            "-f", compose_path.to_str().unwrap(),
            "-p", project_name,
            "run",
            "--rm",
        ]);

        if let Some(u) = user {
            cmd.args(["--user", u]);
        }

        if let Some(e) = entrypoint {
            cmd.args(["--entrypoint", e]);
        }

        cmd.arg(service);
        cmd.args(args);

        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker compose run failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "compose run failed: {}",
                stderr
            )));
        }

        debug!("docker compose run completed successfully");
        Ok(stdout)
    }

    /// Build a Docker image with streaming output
    pub async fn build(
        &self,
        context_path: &Path,
        tag: &str,
        build_args: &HashMap<String, String>,
        progress_tx: Sender<String>,
    ) -> Result<()> {
        info!("Building Docker image with tag {}", tag);

        let mut cmd = Command::new(&self.docker_bin);
        cmd.args([
            "build",
            "-t", tag,
            context_path.to_str().unwrap(),
        ]);

        // Add build args
        for (key, value) in build_args {
            cmd.args(["--build-arg", &format!("{}={}", key, value)]);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Stream stderr (Docker sends build output to stderr)
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr).lines();
            let tx = progress_tx.clone();

            tokio::spawn(async move {
                let mut lines = reader;
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("Build: {}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        let status = child.wait().await?;

        if !status.success() {
            let _ = progress_tx.send("Build failed".to_string()).await;
            return Err(OutClawError::DockerCommand("Docker build failed".to_string()));
        }

        let _ = progress_tx.send("Build completed successfully".to_string()).await;
        info!("Docker build completed successfully");
        Ok(())
    }

    /// List containers with a label filter
    pub async fn list_containers(&self, label_filter: &str) -> Result<Vec<ContainerInfo>> {
        debug!("Listing containers with label filter: {}", label_filter);

        let output = Command::new(&self.docker_bin)
            .args([
                "ps",
                "-a",
                "--filter", &format!("label={}", label_filter),
                "--format", "json",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OutClawError::DockerCommand(format!(
                "docker ps failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<ContainerInfo> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();

        debug!("Found {} containers", containers.len());
        Ok(containers)
    }

    /// Inspect a container by name or ID
    pub async fn inspect_container(&self, container_name: &str) -> Result<ContainerInfo> {
        debug!("Inspecting container: {}", container_name);

        let output = Command::new(&self.docker_bin)
            .args([
                "inspect",
                "--format", "json",
                container_name,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OutClawError::DockerCommand(format!(
                "docker inspect failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: ContainerInfo = serde_json::from_str(&stdout)
            .map_err(|e| OutClawError::Serialization(format!("Failed to parse container info: {}", e)))?;

        Ok(info)
    }

    /// Remove a Docker image
    pub async fn remove_image(&self, tag: &str) -> Result<()> {
        info!("Removing Docker image: {}", tag);

        let output = Command::new(&self.docker_bin)
            .args(["rmi", "-f", tag])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to remove image {}: {}", tag, stderr);
            // Don't fail on image removal errors - it might not exist
        }

        debug!("Image removal completed");
        Ok(())
    }
}

/// Container information from docker ps/inspect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(default, rename = "Labels")]
    pub labels: HashMap<String, String>,
}

impl ContainerInfo {
    /// Get the container's OutClaw container ID from labels
    pub fn outclaw_container_id(&self) -> Option<&str> {
        self.labels.get("outclaw.container").map(|s| s.as_str())
    }

    /// Get the container's OutClaw instance ID from labels
    pub fn outclaw_instance_id(&self) -> Option<&str> {
        self.labels.get("outclaw.instance").map(|s| s.as_str())
    }

    /// Check if the container is running
    pub fn is_running(&self) -> bool {
        self.state.to_lowercase() == "running"
    }
}
