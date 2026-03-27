use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;
use tracing::warn;

use crate::error::{OutClawError, Result};

/// Extract a gzipped tarball to `dest`, handling symlinks and hardlinks
/// as file/directory copies for Windows compatibility.
pub fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<()> {
    let gz_decoder = GzDecoder::new(std::io::Cursor::new(bytes));
    let archive = Archive::new(gz_decoder);
    extract_archive(archive, dest)
}

fn extract_archive<R: std::io::Read>(mut archive: Archive<R>, dest: &Path) -> Result<()> {
    // Symlinks and hardlinks can't be created on Windows without Developer Mode,
    // so we collect them and resolve as file copies after extracting regular entries.
    let mut deferred_links: Vec<DeferredLink> = Vec::new();

    for entry_result in archive
        .entries()
        .map_err(|e| OutClawError::SourceFetch(format!("Failed to read tarball entries: {}", e)))?
    {
        let mut entry = entry_result
            .map_err(|e| OutClawError::SourceFetch(format!("Bad tarball entry: {}", e)))?;

        let entry_type = entry.header().entry_type();
        match entry_type {
            tar::EntryType::Regular | tar::EntryType::Directory | tar::EntryType::GNUSparse => {
                let path_str = entry_path_string(&entry);
                entry.unpack_in(dest).map_err(|e| {
                    OutClawError::SourceFetch(format!(
                        "Failed to extract tarball entry '{}': {}",
                        path_str, e
                    ))
                })?;
            }
            tar::EntryType::Symlink | tar::EntryType::Link => {
                if let (Ok(path), Ok(Some(target))) = (entry.path(), entry.link_name()) {
                    let kind = if entry_type == tar::EntryType::Link {
                        LinkKind::Hard
                    } else {
                        LinkKind::Sym
                    };
                    deferred_links.push(DeferredLink {
                        link_path: path.to_path_buf(),
                        target: target.to_path_buf(),
                        kind,
                    });
                }
            }
            _ => {
                warn!(
                    "Skipping unsupported tarball entry '{}' (type {:?})",
                    entry_path_string(&entry),
                    entry_type
                );
            }
        }
    }

    resolve_deferred_links(&deferred_links, dest)?;
    Ok(())
}

enum LinkKind {
    Sym,
    Hard,
}

struct DeferredLink {
    link_path: PathBuf,
    target: PathBuf,
    kind: LinkKind,
}

/// Normalize a path by resolving `.` and `..` components purely lexically
/// (no filesystem access). Returns None if the path escapes the root (more
/// `..` components than depth).
fn normalize_path(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}

/// Check that `path` stays within `dest` after normalization.
fn is_within(path: &Path, dest: &Path) -> bool {
    match normalize_path(path) {
        Some(normalized) => normalized.starts_with(dest),
        None => false,
    }
}

/// Resolve collected symlinks/hardlinks by copying the target to the link location.
///
/// Runs up to `links.len()` passes to handle chains (symlink -> symlink -> file).
/// Each pass resolves links whose targets now exist on disk. Links that still
/// can't be resolved after all passes are skipped with a warning.
fn resolve_deferred_links(links: &[DeferredLink], dest: &Path) -> Result<()> {
    let mut pending: Vec<usize> = (0..links.len()).collect();
    let max_passes = links.len().max(1);

    for _ in 0..max_passes {
        let mut still_pending = Vec::new();

        for &idx in &pending {
            let link = &links[idx];
            let full_link = dest.join(&link.link_path);

            let resolved = match link.kind {
                LinkKind::Hard => dest.join(&link.target),
                LinkKind::Sym => full_link.parent().unwrap_or(dest).join(&link.target),
            };

            if !is_within(&full_link, dest) || !is_within(&resolved, dest) {
                warn!(
                    "Link escapes destination, skipping: '{}' -> '{}'",
                    link.link_path.display(),
                    link.target.display()
                );
                continue;
            }

            if resolved.is_dir() {
                copy_dir_recursive(&resolved, &full_link)?;
            } else if resolved.is_file() {
                if let Some(parent) = full_link.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&resolved, &full_link).map_err(|e| {
                    OutClawError::SourceFetch(format!(
                        "Failed to copy link target '{}' -> '{}': {}",
                        link.target.display(),
                        link.link_path.display(),
                        e
                    ))
                })?;
            } else {
                still_pending.push(idx);
            }
        }

        if still_pending.is_empty() || still_pending.len() == pending.len() {
            // All resolved, or no progress — stop iterating.
            for &idx in &still_pending {
                let link = &links[idx];
                warn!(
                    "Link target not found, skipping: '{}' -> '{}'",
                    link.link_path.display(),
                    link.target.display()
                );
            }
            break;
        }

        pending = still_pending;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src).map_err(|e| {
        OutClawError::SourceFetch(format!("Failed to read dir '{}': {}", src.display(), e))
    })? {
        let entry = entry.map_err(OutClawError::Io)?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().map_err(OutClawError::Io)?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path).map_err(|e| {
                OutClawError::SourceFetch(format!(
                    "Failed to copy '{}': {}",
                    entry.path().display(),
                    e
                ))
            })?;
        }
    }
    Ok(())
}

fn entry_path_string<R: std::io::Read>(entry: &tar::Entry<R>) -> String {
    entry
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    /// Build a .tar.gz in memory from a list of entry descriptors.
    fn build_tar_gz(entries: &[TestEntry]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            for e in entries {
                match e {
                    TestEntry::File { path, contents } => {
                        let mut header = tar::Header::new_gnu();
                        header.set_size(contents.len() as u64);
                        header.set_mode(0o644);
                        header.set_entry_type(tar::EntryType::Regular);
                        header.set_cksum();
                        builder
                            .append_data(&mut header, path, contents.as_bytes())
                            .unwrap();
                    }
                    TestEntry::Dir { path } => {
                        let mut header = tar::Header::new_gnu();
                        header.set_size(0);
                        header.set_mode(0o755);
                        header.set_entry_type(tar::EntryType::Directory);
                        header.set_cksum();
                        builder
                            .append_data(&mut header, path, &[] as &[u8])
                            .unwrap();
                    }
                    TestEntry::Symlink { path, target } => {
                        let mut header = tar::Header::new_gnu();
                        header.set_size(0);
                        header.set_mode(0o777);
                        header.set_entry_type(tar::EntryType::Symlink);
                        header.set_cksum();
                        builder.append_link(&mut header, path, target).unwrap();
                    }
                    TestEntry::Hardlink { path, target } => {
                        let mut header = tar::Header::new_gnu();
                        header.set_size(0);
                        header.set_mode(0o644);
                        header.set_entry_type(tar::EntryType::Link);
                        header.set_cksum();
                        builder.append_link(&mut header, path, target).unwrap();
                    }
                }
            }
            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        let mut encoder = flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::fast());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
        gz_bytes
    }

    enum TestEntry {
        File {
            path: &'static str,
            contents: &'static str,
        },
        Dir {
            path: &'static str,
        },
        Symlink {
            path: &'static str,
            target: &'static str,
        },
        Hardlink {
            path: &'static str,
            target: &'static str,
        },
    }

    #[test]
    fn extracts_regular_files_and_dirs() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::File {
                path: "proj/README.md",
                contents: "hello",
            },
            TestEntry::Dir { path: "proj/src/" },
            TestEntry::File {
                path: "proj/src/main.rs",
                contents: "fn main() {}",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/README.md")).unwrap(),
            "hello"
        );
        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/src/main.rs")).unwrap(),
            "fn main() {}"
        );
        assert!(dest.path().join("proj/src").is_dir());
    }

    #[test]
    fn resolves_symlink_as_file_copy() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::Dir { path: "proj/docs/" },
            TestEntry::File {
                path: "proj/docs/CLAUDE.md",
                contents: "symlink target content",
            },
            TestEntry::Symlink {
                path: "proj/CLAUDE.md",
                target: "docs/CLAUDE.md",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        let content = std::fs::read_to_string(dest.path().join("proj/CLAUDE.md")).unwrap();
        assert_eq!(content, "symlink target content");
        // Should be a regular file, not a symlink
        let meta = std::fs::symlink_metadata(dest.path().join("proj/CLAUDE.md")).unwrap();
        assert!(meta.is_file());
    }

    #[test]
    fn resolves_hardlink_as_file_copy() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::File {
                path: "proj/original.txt",
                contents: "hardlink content",
            },
            TestEntry::Hardlink {
                path: "proj/linked.txt",
                target: "proj/original.txt",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/linked.txt")).unwrap(),
            "hardlink content"
        );
    }

    #[test]
    fn resolves_symlink_to_directory() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::Dir {
                path: "proj/real_dir/",
            },
            TestEntry::File {
                path: "proj/real_dir/file.txt",
                contents: "inside dir",
            },
            TestEntry::Symlink {
                path: "proj/alias_dir",
                target: "real_dir",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert!(dest.path().join("proj/alias_dir").is_dir());
        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/alias_dir/file.txt")).unwrap(),
            "inside dir"
        );
    }

    #[test]
    fn skips_symlink_with_missing_target() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::File {
                path: "proj/real.txt",
                contents: "ok",
            },
            TestEntry::Symlink {
                path: "proj/dangling.txt",
                target: "nonexistent.txt",
            },
        ]);

        let dest = tempdir().unwrap();
        // Should not error — dangling symlink is skipped with a warning
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert!(dest.path().join("proj/real.txt").is_file());
        assert!(!dest.path().join("proj/dangling.txt").exists());
    }

    #[test]
    fn resolves_symlink_with_parent_traversal() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::Dir { path: "proj/a/" },
            TestEntry::Dir { path: "proj/b/" },
            TestEntry::File {
                path: "proj/a/data.txt",
                contents: "traversed",
            },
            TestEntry::Symlink {
                path: "proj/b/link.txt",
                target: "../a/data.txt",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/b/link.txt")).unwrap(),
            "traversed"
        );
    }

    #[test]
    fn resolves_chained_symlinks() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::File {
                path: "proj/real.txt",
                contents: "chained",
            },
            TestEntry::Symlink {
                path: "proj/link_a.txt",
                target: "real.txt",
            },
            TestEntry::Symlink {
                path: "proj/link_b.txt",
                target: "link_a.txt",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/link_a.txt")).unwrap(),
            "chained"
        );
        assert_eq!(
            std::fs::read_to_string(dest.path().join("proj/link_b.txt")).unwrap(),
            "chained"
        );
    }

    #[test]
    fn rejects_symlink_escaping_dest() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::Symlink {
                path: "proj/escape.txt",
                target: "../../../../etc/passwd",
            },
        ]);

        let dest = tempdir().unwrap();
        // Should succeed (skips the malicious link) without writing outside dest
        extract_tar_gz(&gz, dest.path()).unwrap();
        assert!(!dest.path().join("proj/escape.txt").exists());
    }

    #[test]
    fn rejects_hardlink_escaping_dest() {
        let gz = build_tar_gz(&[
            TestEntry::Dir { path: "proj/" },
            TestEntry::Hardlink {
                path: "proj/escape.txt",
                target: "../../../../etc/passwd",
            },
        ]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();
        assert!(!dest.path().join("proj/escape.txt").exists());
    }

    #[test]
    fn is_within_rejects_escaping_paths() {
        let dest = Path::new("/tmp/extract");
        assert!(!is_within(Path::new("/tmp/extract/../outside"), dest));
        assert!(!is_within(Path::new("/tmp/other/file"), dest));
        assert!(!is_within(Path::new("/etc/passwd"), dest));
        assert!(is_within(Path::new("/tmp/extract/proj/file.txt"), dest));
        assert!(is_within(Path::new("/tmp/extract/a/../a/file.txt"), dest));
    }

    #[test]
    fn normalize_path_resolves_parent_components() {
        assert_eq!(
            normalize_path(Path::new("/a/b/../c")),
            Some(PathBuf::from("/a/c"))
        );
        assert_eq!(
            normalize_path(Path::new("a/b/./c")),
            Some(PathBuf::from("a/b/c"))
        );
    }

    #[test]
    fn normalize_path_rejects_escape() {
        assert_eq!(normalize_path(Path::new("a/../../outside")), None);
        assert_eq!(normalize_path(Path::new("../escape")), None);
    }

    #[test]
    fn empty_archive_succeeds() {
        let gz = build_tar_gz(&[]);

        let dest = tempdir().unwrap();
        extract_tar_gz(&gz, dest.path()).unwrap();
    }
}
