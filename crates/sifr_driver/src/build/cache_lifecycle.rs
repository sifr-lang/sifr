use super::workspace::artifact_cache_root;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const DEFAULT_CACHE_SCAN_NODE_LIMIT: usize = 1_000_000;
const SHARED_CACHE_DIRECTORIES: &[&str] = &["rust_bridge_probe_target"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArtifactCacheStatus {
    pub root: PathBuf,
    pub entries: usize,
    pub bytes: u64,
    pub oldest_entry_age_seconds: Option<u64>,
    pub scanned_nodes: usize,
    pub scan_complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheCleanPolicy {
    pub remove_all: bool,
    pub max_age: Option<Duration>,
    pub max_bytes: Option<u64>,
    pub scan_node_limit: usize,
    pub dry_run: bool,
}

impl Default for ArtifactCacheCleanPolicy {
    fn default() -> Self {
        Self {
            remove_all: false,
            max_age: None,
            max_bytes: None,
            scan_node_limit: DEFAULT_CACHE_SCAN_NODE_LIMIT,
            dry_run: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArtifactCacheCleanReport {
    pub root: PathBuf,
    pub removed_all: bool,
    pub removed_entries: usize,
    pub reclaimed_bytes: u64,
    pub remaining_entries: usize,
    pub remaining_bytes: u64,
    pub scan_complete: bool,
    pub dry_run: bool,
}

pub fn artifact_cache_status() -> io::Result<ArtifactCacheStatus> {
    artifact_cache_status_with_limit(DEFAULT_CACHE_SCAN_NODE_LIMIT)
}

pub fn artifact_cache_status_with_limit(scan_node_limit: usize) -> io::Result<ArtifactCacheStatus> {
    cache_status_at(&artifact_cache_root(), scan_node_limit)
}

pub fn clean_artifact_cache(
    policy: &ArtifactCacheCleanPolicy,
) -> io::Result<ArtifactCacheCleanReport> {
    clean_cache_at(&artifact_cache_root(), policy)
}

#[derive(Clone, Debug)]
struct CacheEntry {
    path: PathBuf,
    bytes: u64,
    modified: SystemTime,
}

struct CacheScan {
    entries: Vec<CacheEntry>,
    scanned_nodes: usize,
    complete: bool,
}

fn cache_status_at(root: &Path, scan_node_limit: usize) -> io::Result<ArtifactCacheStatus> {
    let scan = scan_cache(root, scan_node_limit)?;
    let now = SystemTime::now();
    Ok(ArtifactCacheStatus {
        root: root.to_path_buf(),
        entries: scan.entries.len(),
        bytes: scan.entries.iter().map(|entry| entry.bytes).sum(),
        oldest_entry_age_seconds: scan
            .entries
            .iter()
            .filter_map(|entry| now.duration_since(entry.modified).ok())
            .max()
            .map(|age| age.as_secs()),
        scanned_nodes: scan.scanned_nodes,
        scan_complete: scan.complete,
    })
}

fn clean_cache_at(
    root: &Path,
    policy: &ArtifactCacheCleanPolicy,
) -> io::Result<ArtifactCacheCleanReport> {
    if !policy.remove_all && policy.max_age.is_none() && policy.max_bytes.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache clean requires --all, --max-age-days, or --max-size-mib",
        ));
    }
    if policy.scan_node_limit == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache scan node limit must be positive",
        ));
    }
    if policy.remove_all {
        let status = cache_status_at(root, policy.scan_node_limit)?;
        if fs::symlink_metadata(root).is_ok() && !policy.dry_run {
            remove_path(root)?;
        }
        return Ok(ArtifactCacheCleanReport {
            root: root.to_path_buf(),
            removed_all: true,
            removed_entries: status.entries,
            reclaimed_bytes: status.bytes,
            remaining_entries: if policy.dry_run { status.entries } else { 0 },
            remaining_bytes: if policy.dry_run { status.bytes } else { 0 },
            scan_complete: status.scan_complete,
            dry_run: policy.dry_run,
        });
    }

    let scan = scan_cache(root, policy.scan_node_limit)?;
    if !scan.complete {
        return Err(io::Error::other(format!(
            "cache scan exceeded its {}-node limit; raise --scan-node-limit, use --all, or remove entries before policy cleanup",
            policy.scan_node_limit
        )));
    }
    let now = SystemTime::now();
    let mut selected = BTreeSet::new();
    if let Some(max_age) = policy.max_age {
        for (index, entry) in scan.entries.iter().enumerate() {
            if now
                .duration_since(entry.modified)
                .is_ok_and(|age| age > max_age)
            {
                selected.insert(index);
            }
        }
    }
    if let Some(max_bytes) = policy.max_bytes {
        let mut remaining_bytes = scan
            .entries
            .iter()
            .enumerate()
            .filter(|(index, _)| !selected.contains(index))
            .map(|(_, entry)| entry.bytes)
            .sum::<u64>();
        let mut oldest_first = scan
            .entries
            .iter()
            .enumerate()
            .filter(|(index, _)| !selected.contains(index))
            .map(|(index, entry)| (index, entry.modified))
            .collect::<Vec<_>>();
        oldest_first.sort_by_key(|(_, modified)| *modified);
        for (index, _) in oldest_first {
            if remaining_bytes <= max_bytes {
                break;
            }
            remaining_bytes = remaining_bytes.saturating_sub(scan.entries[index].bytes);
            selected.insert(index);
        }
    }

    let removed_entries = selected.len();
    let reclaimed_bytes = selected
        .iter()
        .map(|index| scan.entries[*index].bytes)
        .sum::<u64>();
    if !policy.dry_run {
        for index in &selected {
            remove_path(&scan.entries[*index].path)?;
        }
        remove_empty_directories(root)?;
    }
    let total_bytes = scan.entries.iter().map(|entry| entry.bytes).sum::<u64>();
    Ok(ArtifactCacheCleanReport {
        root: root.to_path_buf(),
        removed_all: false,
        removed_entries,
        reclaimed_bytes,
        remaining_entries: if policy.dry_run {
            scan.entries.len()
        } else {
            scan.entries.len().saturating_sub(removed_entries)
        },
        remaining_bytes: if policy.dry_run {
            total_bytes
        } else {
            total_bytes.saturating_sub(reclaimed_bytes)
        },
        scan_complete: true,
        dry_run: policy.dry_run,
    })
}

fn scan_cache(root: &Path, scan_node_limit: usize) -> io::Result<CacheScan> {
    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(CacheScan {
                entries: Vec::new(),
                scanned_nodes: 0,
                complete: true,
            });
        }
        Err(error) => return Err(error),
    };
    if root_metadata.file_type().is_symlink() {
        return Ok(CacheScan {
            entries: vec![CacheEntry {
                path: root.to_path_buf(),
                bytes: root_metadata.len(),
                modified: root_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            }],
            scanned_nodes: 1,
            complete: true,
        });
    }
    let mut candidates = Vec::new();
    for path in sorted_children(root)? {
        let metadata = fs::symlink_metadata(&path)?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if metadata.file_type().is_symlink()
            || !metadata.is_dir()
            || SHARED_CACHE_DIRECTORIES.contains(&name)
        {
            candidates.push(path);
            continue;
        }
        let children = sorted_children(&path)?;
        if children.is_empty() {
            candidates.push(path);
        } else {
            candidates.extend(children);
        }
    }

    let mut entries = Vec::new();
    let mut budget = scan_node_limit;
    let mut scanned_nodes = 0;
    for path in candidates {
        match measure_entry(&path, &mut budget, &mut scanned_nodes)? {
            Some(entry) => entries.push(entry),
            None => {
                return Ok(CacheScan {
                    entries,
                    scanned_nodes,
                    complete: false,
                });
            }
        }
    }
    Ok(CacheScan {
        entries,
        scanned_nodes,
        complete: true,
    })
}

fn measure_entry(
    path: &Path,
    budget: &mut usize,
    scanned_nodes: &mut usize,
) -> io::Result<Option<CacheEntry>> {
    let mut pending = vec![path.to_path_buf()];
    let mut bytes = 0_u64;
    let mut modified = SystemTime::UNIX_EPOCH;
    while let Some(current) = pending.pop() {
        if *budget == 0 {
            return Ok(None);
        }
        *budget -= 1;
        *scanned_nodes += 1;
        let metadata = fs::symlink_metadata(&current)?;
        modified = modified.max(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        if metadata.file_type().is_symlink() {
            bytes = bytes.saturating_add(metadata.len());
        } else if metadata.is_dir() {
            pending.extend(sorted_children(&current)?);
        } else {
            bytes = bytes.saturating_add(metadata.len());
        }
    }
    Ok(Some(CacheEntry {
        path: path.to_path_buf(),
        bytes,
        modified,
    }))
}

fn sorted_children(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn remove_path(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn remove_empty_directories(root: &Path) -> io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for child in sorted_children(root)? {
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.is_dir()
            && !metadata.file_type().is_symlink()
            && sorted_children(&child)?.is_empty()
        {
            fs::remove_dir(child)?;
        }
    }
    if sorted_children(root)?.is_empty() {
        fs::remove_dir(root)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ArtifactCacheCleanPolicy, cache_status_at, clean_cache_at};
    use std::fs;

    #[test]
    fn status_and_size_cleanup_use_atomic_namespace_entries() {
        let root = tempfile::tempdir().expect("cache root should be created");
        let first = root.path().join("project").join("first");
        let second = root.path().join("project").join("second");
        fs::create_dir_all(&first).expect("first entry should be created");
        fs::create_dir_all(&second).expect("second entry should be created");
        fs::write(first.join("artifact"), vec![1_u8; 16]).expect("artifact should be written");
        fs::write(second.join("artifact"), vec![2_u8; 32]).expect("artifact should be written");

        let status = cache_status_at(root.path(), 100).expect("status should succeed");
        assert_eq!(status.entries, 2);
        assert_eq!(status.bytes, 48);
        assert!(status.scan_complete);

        let report = clean_cache_at(
            root.path(),
            &ArtifactCacheCleanPolicy {
                max_bytes: Some(20),
                ..ArtifactCacheCleanPolicy::default()
            },
        )
        .expect("cleanup should succeed");
        assert!(report.removed_entries >= 1);
        assert!(report.remaining_bytes <= 20);
    }

    #[test]
    fn dry_run_reports_selection_without_removing_entries() {
        let root = tempfile::tempdir().expect("cache root should be created");
        let entry = root.path().join("project").join("entry");
        fs::create_dir_all(&entry).expect("entry should be created");
        fs::write(entry.join("artifact"), vec![1_u8; 8]).expect("artifact should be written");

        let report = clean_cache_at(
            root.path(),
            &ArtifactCacheCleanPolicy {
                max_bytes: Some(0),
                dry_run: true,
                ..ArtifactCacheCleanPolicy::default()
            },
        )
        .expect("dry run should succeed");
        assert_eq!(report.removed_entries, 1);
        assert_eq!(report.remaining_entries, 1);
        assert!(entry.exists());
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_does_not_follow_a_symlinked_cache_root() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("test root should be created");
        let target = directory.path().join("outside");
        let cache_link = directory.path().join("cache");
        fs::create_dir(&target).expect("outside directory should be created");
        fs::write(target.join("keep"), b"keep").expect("outside file should be written");
        symlink(&target, &cache_link).expect("cache symlink should be created");

        clean_cache_at(
            &cache_link,
            &ArtifactCacheCleanPolicy {
                remove_all: true,
                ..ArtifactCacheCleanPolicy::default()
            },
        )
        .expect("symlink cleanup should succeed");

        assert!(!cache_link.exists());
        assert_eq!(
            fs::read(target.join("keep")).expect("outside file should remain"),
            b"keep"
        );
    }
}
