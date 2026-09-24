//! CLI cache policy, inventory and pruning.
use crate::windows_storage_security as security;
pub(crate) use sifr_cache_storage::{
    check_owned, directory, entry_lock, invalid, new_private_file, owner_scope, payload, publish,
    relative,
};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
pub fn root() -> PathBuf {
    if let Some(root) = std::env::var_os("SIFR_CACHE_DIR") {
        return PathBuf::from(root);
    }
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("sifr")
}

pub(crate) fn seal(root: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_attributes()
        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
        != 0
    {
        return Err(invalid("reparse-point staged payload"));
    }
    security::seal(root)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(root)? {
            seal(&entry?.path())?;
        }
    }
    Ok(())
}

pub(crate) fn process_entry_lock(parent: &Path, key: &str) -> io::Result<File> {
    sifr_cache_storage::open_entry_lock(parent, key, false)
}
pub(crate) fn read_private_file(path: &Path) -> io::Result<File> {
    security::open_read(path)
}
pub(crate) fn read_write_private_file(path: &Path) -> io::Result<File> {
    check_owned(path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if file.metadata()?.file_attributes()
        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
        != 0
    {
        return Err(invalid("reparse-point storage file"));
    }
    Ok(file)
}
pub(crate) fn sync_stage(path: &Path) -> io::Result<()> {
    // Every staged file is sync_all'd before MoveFileExW WRITE_THROUGH
    // publishes the directory. Verify the stage still has a safe owner.
    check_owned(path)
}

pub struct CacheInspection {
    pub root: PathBuf,
    pub entries: Vec<CacheEntryInspection>,
    pub protected_roots: Vec<PathBuf>,
}
#[derive(serde::Serialize)]
pub struct CacheEntryInspection {
    pub path: PathBuf,
    pub bytes: u64,
    pub protected: bool,
}

fn size(path: &Path) -> io::Result<u64> {
    security::no_reparse(path)?;
    let meta = fs::symlink_metadata(path)?;
    if meta.file_attributes()
        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
        != 0
    {
        return Err(invalid("reparse-point cache payload"));
    }
    // Staging may contain a producer-created child with the token default
    // owner. Acquire its lease and seal it before any destructive pruning.
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut total = 0;
    for child in fs::read_dir(path)? {
        total += size(&child?.path())?;
    }
    Ok(total)
}

fn same_scope(path: &Path, scope: &Path) -> bool {
    let Ok(marker) = read_private_file(&path.join("artifact_cache.json")) else {
        return false;
    };
    if !marker
        .metadata()
        .is_ok_and(|metadata| metadata.len() <= 16 * 1024)
    {
        return false;
    }
    let mut bytes = Vec::new();
    if marker.take(16 * 1024 + 1).read_to_end(&mut bytes).is_err() || bytes.len() > 16 * 1024 {
        return false;
    }
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|value| value["scope"].as_str().map(PathBuf::from))
        .is_some_and(|owner| owner == scope)
}

/// Restrict pruning to completed generated entries and staging in known native
/// namespaces. Cargo families, toolchains, probes, and other worktrees are excluded.
pub fn inspect() -> io::Result<CacheInspection> {
    let root = root().join("native/artifacts");
    let mut entries = Vec::new();
    let mut protected_roots = Vec::new();
    let scope = owner_scope()?;
    if root.exists() {
        directory(&root)?;
        for family in fs::read_dir(&root)? {
            let family = family?.path();
            // Auxiliary Cargo resolution/probe roots have different owners and
            // no finalized-entry lock namespace; never inspect or prune them.
            if !family.join(".locks").is_dir() {
                protected_roots.push(family);
                continue;
            }
            check_owned(&family)?;
            if !family.is_dir() {
                continue;
            }
            for child in fs::read_dir(&family)? {
                let path = child?.path();
                let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                    continue;
                };
                if name.starts_with('.') {
                    continue;
                }
                let key = name.split(".stage-").next().unwrap_or(name);
                if key.len() != 64 || !key.bytes().all(|c| c.is_ascii_hexdigit()) {
                    continue;
                }
                let lock_path = PathBuf::from(".locks").join(key);
                let lock = payload(&family, &lock_path).and_then(|()| {
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .custom_flags(
                            windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT,
                        )
                        .open(family.join(lock_path))
                });
                let same_scope = same_scope(&path, &scope);
                let protected =
                    !same_scope || lock.as_ref().map_or(true, |file| file.try_lock().is_err());
                entries.push(CacheEntryInspection {
                    bytes: size(&path)?,
                    path,
                    protected,
                });
            }
        }
    }
    // Preserve the newest completed candidate in each family even when inactive.
    let mut newest = std::collections::BTreeMap::new();
    for entry in &entries {
        if entry
            .path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(".stage-"))
        {
            continue;
        }
        let same_scope = same_scope(&entry.path, &scope);
        if !same_scope {
            continue;
        }
        if let (Some(parent), Ok(time)) = (
            entry.path.parent(),
            fs::metadata(&entry.path).and_then(|m| m.modified()),
        ) {
            let old = newest
                .entry(parent.to_path_buf())
                .or_insert((time, entry.path.clone()));
            if time > old.0 {
                *old = (time, entry.path.clone());
            }
        }
    }
    for entry in &mut entries {
        if newest.values().any(|(_, path)| path == &entry.path) {
            entry.protected = true;
        }
    }
    Ok(CacheInspection {
        root,
        entries,
        protected_roots,
    })
}

/// Explicit pressure input comes from the resource-policy owner. No target-size
/// threshold and no Cargo-internal deletion. Active readers/writers always win.
pub fn prune(
    reserve_bytes: u64,
    available_bytes: u64,
    dry_run: bool,
) -> io::Result<CacheInspection> {
    let mut report = inspect()?;
    let mut reclaimed = 0;
    for entry in &mut report.entries {
        if entry.protected {
            continue;
        }
        if available_bytes.saturating_add(reclaimed) >= reserve_bytes {
            break;
        }
        let parent = entry
            .path
            .parent()
            .ok_or_else(|| invalid("missing entry parent"))?;
        let name = entry
            .path
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or_else(|| invalid("invalid entry name"))?;
        let key = name.split(".stage-").next().unwrap_or(name);
        let lock = entry_lock(parent, key)?;
        if lock.try_lock().is_err() {
            entry.protected = true;
            continue;
        }
        check_owned(&entry.path)?;
        if !dry_run {
            seal(&entry.path)?;
            fs::remove_dir_all(&entry.path)?;
        }
        reclaimed += entry.bytes;
    }
    Ok(report)
}

/// Available space on the selected destination filesystem.
pub fn available_bytes() -> io::Result<u64> {
    let root = root();
    directory(&root)?;
    security::available_bytes(&root)
}

pub(crate) fn real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| {
        m.is_dir()
            && m.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                == 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_portability_extended_cache_path_publication() {
        use std::io::{Read, Write};
        use std::os::windows::ffi::OsStrExt;

        let temp = tempfile::tempdir().unwrap();
        let deep = temp
            .path()
            .join("a".repeat(80))
            .join("b".repeat(80))
            .join("c".repeat(80));
        assert!(deep.join("stage").as_os_str().encode_wide().count() > 260);
        directory(&deep).unwrap();
        let stage = deep.join("stage");
        directory(&stage).unwrap();
        let mut file = new_private_file(&stage.join("record")).unwrap();
        file.write_all(b"complete").unwrap();
        file.sync_all().unwrap();
        drop(file);

        let winner = deep.join("winner");
        publish(&stage, &winner).unwrap();
        let mut bytes = Vec::new();
        read_private_file(&winner.join("record"))
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"complete");
    }

    #[test]
    fn windows_portability_private_acl_alias_and_lock_identity() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("private");
        directory(&root).unwrap();
        let file = new_private_file(&root.join("owned")).unwrap();
        file.sync_all().unwrap();
        check_owned(&root).unwrap();
        check_owned(&root.join("owned")).unwrap();

        let lock = entry_lock(&root, "lease").unwrap();
        lock.try_lock().unwrap();
        let contender = entry_lock(&root, "lease").unwrap();
        assert!(contender.try_lock().is_err());
        let index = security::file_identity(&lock).unwrap();
        drop(lock);
        contender.try_lock().unwrap();
        assert_eq!(security::file_identity(&contender).unwrap(), index);
        drop(contender);

        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let alias = root.join("junction");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&alias)
            .arg(&outside)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(directory(&alias).is_err());
        assert!(payload(&root, Path::new("junction")).is_err());
        assert!(seal(&alias).is_err());
        assert!(security::no_reparse(&root.join("missing/../owned")).is_err());
        fs::remove_dir(&alias).unwrap();

        security::test_grant_world(&root.join("owned")).unwrap();
        assert!(check_owned(&root.join("owned")).is_err());
        security::seal(&root.join("owned")).unwrap();

        let system_root = PathBuf::from(std::env::var_os("WINDIR").unwrap());
        assert!(system_root.is_dir());
        assert!(
            check_owned(&system_root).is_err(),
            "foreign Windows owner accepted"
        );
    }

    #[test]
    fn windows_portability_pressure_prune_protects_leases_and_winners() {
        let temp = tempfile::tempdir().unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "cache_storage::tests::windows_portability_prune_child",
                "--nocapture",
            ])
            .env("SIFR_WINDOWS_PRUNE_ROOT", temp.path().join("cache"))
            .env("SIFR_CACHE_DIR", temp.path().join("cache"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[test]
    fn windows_portability_prune_child() {
        use std::io::Write;
        let Some(root) = std::env::var_os("SIFR_WINDOWS_PRUNE_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let family = root.join("native/artifacts/family");
        directory(&family).unwrap();
        let scope = owner_scope().unwrap();
        let winner_key = "a".repeat(64);
        let active_key = "b".repeat(64);
        let abandoned_key = "c".repeat(64);
        let winner = family.join(&winner_key);
        let active = family.join(format!("{active_key}.stage-1"));
        let abandoned = family.join(format!("{abandoned_key}.stage-1"));
        for path in [&winner, &active, &abandoned] {
            directory(path).unwrap();
            let bytes = serde_json::to_vec(&serde_json::json!({"scope": scope})).unwrap();
            let mut marker = new_private_file(&path.join("artifact_cache.json")).unwrap();
            marker.write_all(&bytes).unwrap();
            marker.sync_all().unwrap();
        }
        // A killed writer can leave a child before its final recursive seal.
        fs::write(abandoned.join("unfinished"), b"partial").unwrap();
        let winner_lease = entry_lock(&family, &winner_key).unwrap();
        drop(winner_lease);
        let lease = entry_lock(&family, &active_key).unwrap();
        lease.try_lock().unwrap();
        let abandoned_lease = entry_lock(&family, &abandoned_key).unwrap();
        drop(abandoned_lease);
        let report = inspect().unwrap();
        assert!(
            report
                .entries
                .iter()
                .any(|item| item.path == winner && item.protected)
        );
        assert!(
            report
                .entries
                .iter()
                .any(|item| item.path == active && item.protected)
        );
        assert!(
            report
                .entries
                .iter()
                .any(|item| item.path == abandoned && !item.protected)
        );
        prune(u64::MAX, 0, false).unwrap();
        assert!(winner.is_dir());
        assert!(active.is_dir());
        assert!(!abandoned.exists());
        drop(lease);
        prune(u64::MAX, 0, false).unwrap();
        assert!(winner.is_dir());
        assert!(!active.exists());
    }
}
