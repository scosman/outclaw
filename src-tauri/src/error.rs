use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutClawError {
    #[error("Docker is not running")]
    DockerNotRunning,

    #[error("Docker is not installed")]
    DockerNotInstalled,

    #[error("Docker Compose is not available")]
    DockerComposeNotAvailable,

    #[error("Docker command failed: {0}")]
    DockerCommand(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Instance already exists: {0}")]
    InstanceAlreadyExists(String),

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("Port {0} is out of valid range (1024-65535)")]
    PortOutOfRange(u16),

    #[error("Network error: {0}")]
    Network(String),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("Failed to fetch Dockerfile: {0}")]
    DockerfileFetch(String),

    #[error("Failed to fetch source: {0}")]
    SourceFetch(String),

    #[error("Build failed at stage '{stage}': {message}")]
    BuildFailed { stage: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for OutClawError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<serde_json::Error> for OutClawError {
    fn from(e: serde_json::Error) -> Self {
        OutClawError::Serialization(e.to_string())
    }
}

impl From<serde_yaml::Error> for OutClawError {
    fn from(e: serde_yaml::Error) -> Self {
        OutClawError::Serialization(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, OutClawError>;
