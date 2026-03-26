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

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn create_command(program: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

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
        let docker_bin =
            std::env::var("OUTCLAW_DOCKER_BIN").unwrap_or_else(|_| "docker".to_string());
        Self { docker_bin }
    }

    /// Check if Docker is available and running
    pub async fn check_available(&self) -> Result<DockerStatus> {
        // Check if docker binary exists
        let docker_check = create_command(&self.docker_bin)
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
        let info_check = create_command(&self.docker_bin)
            .args(["info", "--format", "{{.ServerVersion}}"])
            .output()
            .await;

        let state = match info_check {
            Ok(output) if output.status.success() => DockerState::Running,
            Ok(_) => DockerState::NotRunning,
            Err(_) => DockerState::NotRunning,
        };

        // Check if compose is available
        let compose_check = create_command(&self.docker_bin)
            .args(["compose", "version"])
            .output()
            .await;

        let compose_available = compose_check.map(|o| o.status.success()).unwrap_or(false);

        Ok(DockerStatus {
            state,
            compose_available,
        })
    }

    /// Run docker compose up -d
    pub async fn compose_up(&self, compose_path: &Path, project_name: &str) -> Result<()> {
        info!("Running docker compose up for project {}", project_name);

        let output = create_command(&self.docker_bin)
            .args([
                "compose",
                "-f",
                compose_path.to_str().unwrap(),
                "-p",
                project_name,
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

        let output = create_command(&self.docker_bin)
            .args([
                "compose",
                "-f",
                compose_path.to_str().unwrap(),
                "-p",
                project_name,
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

        let output = create_command(&self.docker_bin)
            .args([
                "compose",
                "-f",
                compose_path.to_str().unwrap(),
                "-p",
                project_name,
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
        self.compose_run_with_entrypoint(compose_path, project_name, service, args, user, None)
            .await
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

        let mut cmd = create_command(&self.docker_bin);
        cmd.args([
            "compose",
            "-f",
            compose_path.to_str().unwrap(),
            "-p",
            project_name,
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

        let mut cmd = create_command(&self.docker_bin);
        cmd.args(["build", "-t", tag, context_path.to_str().unwrap()]);

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
            return Err(OutClawError::DockerCommand(
                "Docker build failed".to_string(),
            ));
        }

        let _ = progress_tx
            .send("Build completed successfully".to_string())
            .await;
        info!("Docker build completed successfully");
        Ok(())
    }

    /// List containers with a label filter
    pub async fn list_containers(&self, label_filter: &str) -> Result<Vec<ContainerInfo>> {
        debug!("Listing containers with label filter: {}", label_filter);

        let output = create_command(&self.docker_bin)
            .args([
                "ps",
                "-a",
                "--filter",
                &format!("label={}", label_filter),
                "--format",
                "json",
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
        debug!(
            "docker ps stdout: {}",
            stdout.chars().take(500).collect::<String>()
        );

        let containers: Vec<ContainerInfo> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| {
                let result: std::result::Result<ContainerInfo, serde_json::Error> =
                    serde_json::from_str(l);
                if let Err(e) = &result {
                    debug!(
                        "Failed to parse container JSON: {:?} | line: {}",
                        e,
                        l.chars().take(100).collect::<String>()
                    );
                }
                result.ok()
            })
            .collect();

        debug!("Found {} containers", containers.len());
        Ok(containers)
    }

    /// Get status of all outclaw instances in a single docker call
    /// Returns a HashMap of instance_id -> (is_running, container_id)
    pub async fn get_outclaw_instance_statuses(
        &self,
    ) -> Result<HashMap<String, (bool, Option<String>)>> {
        let containers = self.list_containers("outclaw.instance").await?;

        let mut statuses: HashMap<String, (bool, Option<String>)> = HashMap::new();

        for container in containers {
            if let Some(instance_id) = container.labels.get("outclaw.instance") {
                let entry = statuses.entry(instance_id.clone()).or_insert((false, None));

                // If any container for this instance is running, mark as running
                if container.is_running() {
                    entry.0 = true;
                }

                // Store the container ID (use the first/primary one)
                if entry.1.is_none() {
                    entry.1 = Some(container.id.clone());
                }
            }
        }

        Ok(statuses)
    }

    /// Inspect a container by name or ID
    #[allow(dead_code)]
    pub async fn inspect_container(&self, container_name: &str) -> Result<ContainerInfo> {
        debug!("Inspecting container: {}", container_name);

        let output = create_command(&self.docker_bin)
            .args(["inspect", "--format", "json", container_name])
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
        let info: ContainerInfo = serde_json::from_str(&stdout).map_err(|e| {
            OutClawError::Serialization(format!("Failed to parse container info: {}", e))
        })?;

        Ok(info)
    }

    /// Remove a Docker image
    pub async fn remove_image(&self, tag: &str) -> Result<()> {
        info!("Removing Docker image: {}", tag);

        let output = create_command(&self.docker_bin)
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

    /// Execute a command in a running container
    /// Returns the stdout output on success
    pub async fn docker_exec(&self, container_name: &str, args: &[&str]) -> Result<String> {
        info!(
            "Executing command in container: {} {:?}",
            container_name, args
        );

        let mut cmd = create_command(&self.docker_bin);
        cmd.args(["exec", container_name]);
        cmd.args(args);

        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker exec failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "docker exec failed: {}",
                stderr
            )));
        }

        debug!("docker exec completed successfully");
        Ok(stdout)
    }

    /// Execute a command in a running container with streaming output
    /// Sends each line of output to the provided sender
    /// Returns Ok(()) if command exits with success, Err otherwise
    pub async fn docker_exec_streaming(
        &self,
        container_name: &str,
        args: &[&str],
        progress_tx: Sender<String>,
    ) -> Result<()> {
        info!(
            "Executing streaming command in container: {} {:?}",
            container_name, args
        );

        let mut cmd = create_command(&self.docker_bin);
        cmd.args(["exec", container_name]);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout).lines();
            let tx = progress_tx.clone();

            tokio::spawn(async move {
                let mut lines = reader;
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("stdout: {}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr).lines();
            let tx = progress_tx.clone();

            tokio::spawn(async move {
                let mut lines = reader;
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("stderr: {}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        let status = child.wait().await?;

        if !status.success() {
            let _ = progress_tx.send("Command failed".to_string()).await;
            return Err(OutClawError::DockerCommand(
                "docker exec command failed".to_string(),
            ));
        }

        debug!("docker exec streaming completed successfully");
        Ok(())
    }

    /// Execute a command in a running container with environment variables
    /// Returns the stdout output on success
    #[allow(dead_code)]
    pub async fn docker_exec_with_env(
        &self,
        container_name: &str,
        args: &[&str],
        env_vars: &[(String, String)],
    ) -> Result<String> {
        info!(
            "Executing command in container with env: {} {:?}",
            container_name, args
        );

        let mut cmd = create_command(&self.docker_bin);
        cmd.args(["exec"]);

        // Add environment variables
        for (key, value) in env_vars {
            cmd.args(["-e", &format!("{}={}", key, value)]);
        }

        cmd.arg(container_name);
        cmd.args(args);

        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("docker exec failed: {}", stderr);
            return Err(OutClawError::DockerCommand(format!(
                "docker exec failed: {}",
                stderr
            )));
        }

        debug!("docker exec completed successfully");
        Ok(stdout)
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
    #[serde(default, rename = "Labels", deserialize_with = "deserialize_labels")]
    pub labels: HashMap<String, String>,
}

/// Custom deserializer for Labels that handles both string (docker ps) and map (docker inspect) formats
fn deserialize_labels<'de, D>(
    deserializer: D,
) -> std::result::Result<HashMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Handle both string and map formats
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Object(map) => {
            // Map format (from docker inspect)
            let mut result = HashMap::new();
            for (k, v) in map {
                if let Some(s) = v.as_str() {
                    result.insert(k, s.to_string());
                }
            }
            Ok(result)
        }
        serde_json::Value::String(s) => {
            // String format (from docker ps): "key1=value1,key2=value2"
            let mut result = HashMap::new();
            if !s.is_empty() {
                // Parse key=value pairs, handling values that may contain commas
                // by looking for ",key=" pattern
                let mut remaining = s.as_str();
                while !remaining.is_empty() {
                    // Find the first '=' to get the key
                    if let Some(eq_pos) = remaining.find('=') {
                        let key = remaining[..eq_pos].to_string();
                        let value_start = eq_pos + 1;

                        // Find the next ",key=" pattern
                        let value_end = find_next_key_start(&remaining[value_start..]);
                        let value = remaining[value_start..value_start + value_end].to_string();

                        result.insert(key, value);
                        remaining = &remaining[value_start + value_end..];
                        // Skip the comma separator
                        if remaining.starts_with(',') {
                            remaining = &remaining[1..];
                        }
                    } else {
                        break;
                    }
                }
            }
            Ok(result)
        }
        _ => Ok(HashMap::new()),
    }
}

/// Find where the next key starts (looks for ",key=" pattern)
fn find_next_key_start(s: &str) -> usize {
    // Look for ",<key>=" pattern where key doesn't contain = or special chars
    for (i, c) in s.char_indices() {
        if c == ',' {
            // Check if what follows looks like a key (letters, numbers, dots, dashes, underscores)
            // followed by '='
            let rest = &s[i + 1..];
            if let Some(eq_pos) = rest.find('=') {
                let potential_key = &rest[..eq_pos];
                // Valid docker label keys contain only alphanumeric, dots, dashes, underscores
                if potential_key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
                {
                    return i;
                }
            }
        }
    }
    s.len()
}

impl ContainerInfo {
    /// Get the container's OutClaw container ID from labels
    #[allow(dead_code)]
    pub fn outclaw_container_id(&self) -> Option<&str> {
        self.labels.get("outclaw.container").map(|s| s.as_str())
    }

    /// Get the container's OutClaw instance ID from labels
    #[allow(dead_code)]
    pub fn outclaw_instance_id(&self) -> Option<&str> {
        self.labels.get("outclaw.instance").map(|s| s.as_str())
    }

    /// Check if the container is running
    pub fn is_running(&self) -> bool {
        self.state.to_lowercase() == "running"
    }
}
