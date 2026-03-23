use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{OutClawError, Result};
use crate::instance::Release;

const GITHUB_API_URL: &str = "https://api.github.com/repos";
const OPENCLAW_REPO: &str = "openclaw/openclaw";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

/// Client for fetching OpenClaw releases from GitHub
pub struct ReleasesClient {
    cache_path: PathBuf,
    client: Client,
}

impl ReleasesClient {
    /// Create a new releases client
    pub fn new() -> Result<Self> {
        let cache_path = InstanceConfig::outclaw_dir().join("releases-cache.json");

        let client = Client::builder()
            .user_agent("OutClaw/0.1.0")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| OutClawError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { cache_path, client })
    }

    /// Get releases, using cache if fresh
    pub async fn get_releases(&self) -> Result<Vec<Release>> {
        // Try cache first
        if let Some(cached) = self.load_cache()? {
            if cached.is_fresh() {
                info!("Using cached releases ({} items)", cached.releases.len());
                return Ok(cached.releases);
            }
        }

        // Fetch from GitHub
        info!("Fetching releases from GitHub API");
        match self.fetch_from_github().await {
            Ok(releases) => {
                // Cache the result
                self.save_cache(&releases)?;
                Ok(releases)
            }
            Err(e) => {
                // If fetch fails, try to use stale cache
                warn!("Failed to fetch releases: {}", e);
                if let Some(cached) = self.load_cache()? {
                    warn!("Using stale cache ({} items)", cached.releases.len());
                    return Ok(cached.releases);
                }
                Err(e)
            }
        }
    }

    /// Fetch releases from GitHub API
    async fn fetch_from_github(&self) -> Result<Vec<Release>> {
        let url = format!("{}/{}/releases", GITHUB_API_URL, OPENCLAW_REPO);

        debug!("Fetching from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| OutClawError::GitHubApi(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(OutClawError::GitHubApi(format!(
                "GitHub API returned status {}",
                response.status()
            )));
        }

        let github_releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| OutClawError::GitHubApi(format!("Failed to parse response: {}", e)))?;

        // Convert to our Release type
        // Note: We'd need to fetch commit SHA from the tag in a real implementation
        // For now, we'll use the tag name as a placeholder
        let releases: Vec<Release> = github_releases
            .into_iter()
            .map(|gr| Release {
                tag: gr.tag_name,
                name: gr.name.unwrap_or_default(),
                published_at: gr.published_at,
                prerelease: gr.prerelease,
                // TODO: Fetch actual commit SHA from tag
                commit_sha: "main".to_string(),
            })
            .collect();

        info!("Fetched {} releases from GitHub", releases.len());
        Ok(releases)
    }

    /// Load cached releases
    fn load_cache(&self) -> Result<Option<CachedReleases>> {
        if !self.cache_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.cache_path)?;
        let cached: CachedReleases = serde_json::from_str(&content)?;
        Ok(Some(cached))
    }

    /// Save releases to cache
    fn save_cache(&self, releases: &[Release]) -> Result<()> {
        let cached = CachedReleases {
            timestamp_secs: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            releases: releases.to_vec(),
        };

        let content = serde_json::to_string_pretty(&cached)?;

        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.cache_path, content)?;
        debug!("Cached releases to {:?}", self.cache_path);

        Ok(())
    }
}

impl Default for ReleasesClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ReleasesClient")
    }
}

/// Cached releases with timestamp (stored as Unix seconds for serialization)
#[derive(Debug, Serialize, Deserialize)]
struct CachedReleases {
    timestamp_secs: u64,
    releases: Vec<Release>,
}

impl CachedReleases {
    fn timestamp(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.timestamp_secs)
    }

    fn is_fresh(&self) -> bool {
        self.timestamp()
            .elapsed()
            .map(|d| d < CACHE_TTL)
            .unwrap_or(false)
    }
}

/// GitHub API release format
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: DateTime<Utc>,
    prerelease: bool,
}

// Need to import InstanceConfig for path helper
use crate::instance::InstanceConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_releases_freshness() {
        let fresh = CachedReleases {
            timestamp_secs: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            releases: vec![],
        };
        assert!(fresh.is_fresh());

        // Create a stale cache (older than TTL)
        let stale_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3700;
        let stale = CachedReleases {
            timestamp_secs: stale_secs,
            releases: vec![],
        };
        assert!(!stale.is_fresh());
    }
}
