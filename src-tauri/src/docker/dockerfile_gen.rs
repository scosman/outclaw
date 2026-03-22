use std::path::Path;
use std::time::Duration;

use reqwest::Client;
use tracing::{debug, info, warn};

use crate::error::{EasyClawError, Result};
use crate::instance::Release;

const GITHUB_RAW_URL: &str = "https://raw.githubusercontent.com";
const OPENCLAW_REPO: &str = "openclaw/openclaw";
const DOCKERFILE_PATH: &str = "Dockerfile";

/// Fetch Dockerfile from GitHub for a specific release
///
/// Downloads the Dockerfile from the OpenClaw repository at the release's commit SHA.
/// Caches the result locally for reuse.
pub async fn fetch_dockerfile(
    release: &Release,
    cache_dir: &Path,
    client: &Client,
) -> Result<String> {
    let cache_file = cache_dir.join(format!("Dockerfile-{}", release.tag));

    // Check cache first
    if cache_file.exists() {
        info!("Using cached Dockerfile for {}", release.tag);
        return Ok(std::fs::read_to_string(&cache_file)?);
    }

    // Fetch from GitHub
    info!("Fetching Dockerfile for {} from GitHub", release.tag);

    let url = format!(
        "{}/{}/{}/{}",
        GITHUB_RAW_URL, OPENCLAW_REPO, release.commit_sha, DOCKERFILE_PATH
    );

    debug!("Fetching from: {}", url);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| EasyClawError::DockerfileFetch(format!("Network error: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(EasyClawError::DockerfileFetch(format!(
            "GitHub returned status {} for {}",
            status, url
        )));
    }

    let content = response
        .text()
        .await
        .map_err(|e| EasyClawError::DockerfileFetch(format!("Failed to read response: {}", e)))?;

    // Cache the result
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cache_file, &content)?;
    debug!("Cached Dockerfile to {:?}", cache_file);

    Ok(content)
}

/// Prepare the Docker build context directory
///
/// Writes the Dockerfile and any supporting files to the build context.
pub fn prepare_build_context(
    dockerfile_content: &str,
    docker_dir: &Path,
) -> Result<()> {
    // Ensure directory exists
    std::fs::create_dir_all(docker_dir)?;

    // Write Dockerfile
    let dockerfile_path = docker_dir.join("Dockerfile");
    std::fs::write(&dockerfile_path, dockerfile_content)?;
    debug!("Wrote Dockerfile to {:?}", dockerfile_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    fn create_test_release() -> Release {
        Release {
            tag: "v0.42.1".to_string(),
            name: "Test Release".to_string(),
            published_at: Utc::now(),
            prerelease: false,
            commit_sha: "abc123".to_string(),
        }
    }

    #[test]
    fn test_prepare_build_context() {
        let dir = tempdir().unwrap();
        let docker_dir = dir.path().join("docker");

        let dockerfile = "FROM node:18\nRUN echo 'test'";

        prepare_build_context(dockerfile, &docker_dir).unwrap();

        let written = std::fs::read_to_string(docker_dir.join("Dockerfile")).unwrap();
        assert_eq!(written, dockerfile);
    }

    // Note: fetch_dockerfile tests would require network access or mocking
    // Integration tests should be separate with #[cfg(feature = "integration-tests")]
}
