use std::path::Path;
use std::time::Duration;

use flate2::read::GzDecoder;
use reqwest::Client;
use tar::Archive;
use tracing::{info, warn};

use crate::error::{OutClawError, Result};
use crate::instance::Release;

const OPENCLAW_REPO: &str = "openclaw/openclaw";

/// Fetch and extract the full OpenClaw source for a specific release
///
/// Downloads the source tarball from GitHub and extracts it to the cache directory.
/// Returns the path to the extracted source directory (the build context).
///
/// The tarball is cached by tag, so subsequent builds of the same version
/// don't need to re-download.
pub async fn fetch_release_source(
    release: &Release,
    cache_dir: &Path,
    client: &Client,
) -> Result<std::path::PathBuf> {
    // Create source cache directory
    let source_cache_dir = cache_dir.join("source-cache");
    std::fs::create_dir_all(&source_cache_dir)?;

    // The extracted directory will be named like "openclaw-0.42.1" (tag without 'v')
    let tag_normalized = release.tag.trim_start_matches('v');
    let extracted_dir = source_cache_dir.join(format!("openclaw-{}", tag_normalized));

    // Check if already extracted
    if extracted_dir.exists() && extracted_dir.join("package.json").exists() {
        info!("Using cached source for {}", release.tag);
        return Ok(extracted_dir);
    }

    // Download tarball from GitHub
    // URL format: https://github.com/openclaw/openclaw/archive/refs/tags/{tag}.tar.gz
    let tarball_url = format!(
        "https://github.com/{}/archive/refs/tags/{}.tar.gz",
        OPENCLAW_REPO, release.tag
    );

    info!(
        "Fetching source tarball for {} from {}",
        release.tag, tarball_url
    );

    let response = client
        .get(&tarball_url)
        .timeout(Duration::from_secs(120)) // Larger tarball, allow more time
        .send()
        .await
        .map_err(|e| OutClawError::SourceFetch(format!("Network error: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(OutClawError::SourceFetch(format!(
            "GitHub returned status {} for {}",
            status, tarball_url
        )));
    }

    // Download the tarball bytes
    let bytes = response
        .bytes()
        .await
        .map_err(|e| OutClawError::SourceFetch(format!("Failed to read response: {}", e)))?;

    info!("Downloaded {} bytes for {}", bytes.len(), release.tag);

    // Extract tarball
    let cursor = std::io::Cursor::new(&bytes[..]);
    let gz_decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz_decoder);

    // Extract to a temp location first, then move if successful
    let temp_extract_dir = source_cache_dir.join(format!(".extracting-{}", tag_normalized));
    if temp_extract_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_extract_dir);
    }
    std::fs::create_dir_all(&temp_extract_dir)?;

    // Extract the archive
    archive
        .unpack(&temp_extract_dir)
        .map_err(|e| OutClawError::SourceFetch(format!("Failed to extract tarball: {}", e)))?;

    // Find the extracted directory (GitHub creates a single top-level dir)
    let extracted_content: Vec<_> = std::fs::read_dir(&temp_extract_dir)
        .map_err(|e| OutClawError::SourceFetch(format!("Failed to read extracted dir: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();

    if extracted_content.len() != 1 {
        return Err(OutClawError::SourceFetch(format!(
            "Expected single directory in tarball, found {} items",
            extracted_content.len()
        )));
    }

    let actual_extracted = extracted_content[0].path();

    // Verify it looks like the OpenClaw repo
    if !actual_extracted.join("package.json").exists() {
        return Err(OutClawError::SourceFetch(
            "Downloaded source does not contain package.json - not a valid OpenClaw release"
                .to_string(),
        ));
    }

    // Remove old cached version if exists
    if extracted_dir.exists() {
        let _ = std::fs::remove_dir_all(&extracted_dir);
    }

    // Move to final location
    std::fs::rename(&actual_extracted, &extracted_dir).map_err(|e| {
        OutClawError::SourceFetch(format!("Failed to move extracted source: {}", e))
    })?;

    // Clean up temp dir
    let _ = std::fs::remove_dir_all(&temp_extract_dir);

    info!("Extracted source to {:?}", extracted_dir);
    Ok(extracted_dir)
}

/// Get the path to cached source for a release, if it exists
#[allow(dead_code)]
pub fn get_cached_source_path(release: &Release, cache_dir: &Path) -> Option<std::path::PathBuf> {
    let source_cache_dir = cache_dir.join("source-cache");
    let tag_normalized = release.tag.trim_start_matches('v');
    let extracted_dir = source_cache_dir.join(format!("openclaw-{}", tag_normalized));

    if extracted_dir.exists() && extracted_dir.join("package.json").exists() {
        Some(extracted_dir)
    } else {
        None
    }
}

/// Estimate the size of cached sources
#[allow(dead_code)]
pub fn get_cache_size(cache_dir: &Path) -> Result<u64> {
    let source_cache_dir = cache_dir.join("source-cache");
    if !source_cache_dir.exists() {
        return Ok(0);
    }

    fn dir_size(path: &Path) -> u64 {
        std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|entry| {
                        let path = entry.path();
                        if path.is_dir() {
                            dir_size(&path)
                        } else {
                            entry.metadata().map(|m| m.len()).unwrap_or(0)
                        }
                    })
                    .sum()
            })
            .unwrap_or(0)
    }

    Ok(dir_size(&source_cache_dir))
}

/// Clean up old cached sources (keep only the specified number of most recent)
#[allow(dead_code)]
pub fn cleanup_old_cache(cache_dir: &Path, keep_count: usize) -> Result<Vec<String>> {
    let source_cache_dir = cache_dir.join("source-cache");
    if !source_cache_dir.exists() {
        return Ok(vec![]);
    }

    // Get all cached versions with their modification times
    let mut cached: Vec<_> = std::fs::read_dir(&source_cache_dir)
        .map_err(OutClawError::Io)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("openclaw-"))
        .filter_map(|e| {
            let path = e.path();
            let modified = e.metadata().ok()?.modified().ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            Some((name, path, modified))
        })
        .collect();

    // Sort by modification time, newest first
    cached.sort_by(|a, b| b.2.cmp(&a.2));

    // Remove old entries
    let removed: Vec<String> = cached
        .into_iter()
        .enumerate()
        .filter_map(|(i, (name, path, _))| {
            if i >= keep_count {
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    warn!("Failed to remove cached source {}: {}", name, e);
                }
                Some(name)
            } else {
                None
            }
        })
        .collect();

    Ok(removed)
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
    fn test_cache_path_generation() {
        let dir = tempdir().unwrap();
        let release = create_test_release();

        let path = get_cached_source_path(&release, dir.path());
        assert!(path.is_none()); // Not cached yet

        // The expected path format
        let expected = dir.path().join("source-cache/openclaw-0.42.1");
        std::fs::create_dir_all(&expected).unwrap();
        std::fs::write(expected.join("package.json"), "{}").unwrap();

        let path = get_cached_source_path(&release, dir.path());
        assert_eq!(path, Some(expected));
    }

    // Note: fetch_release_source tests would require network access
    // Integration tests should be separate with #[cfg(feature = "integration-tests")]
}
