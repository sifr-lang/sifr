//! Owned generated-entry storage. Locks are never unlinked: their inode is the lease.
use crate::windows_storage_security as security;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::{Component, Path, PathBuf};

pub(crate) fn owner_scope() -> io::Result<PathBuf> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    Ok(cwd
        .ancestors()
        .find(|path| path.join(".git").exists())
        .unwrap_or(&cwd)
        .to_path_buf())
}

pub fn root() -> PathBuf {
    if let Some(root) = std::env::var_os("SIFR_CACHE_DIR") {
        return PathBuf::from(root);
    }
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("sifr")
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message.into())
}

pub(crate) fn relative(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid(
            "cache path must contain only normal relative components",
        ));
    }
    Ok(())
}

/// Reject all symlinks, including ancestors; only the selected root and children
/// must be private and owned (system ancestors such as /home may be root-owned).
pub(crate) fn directory(path: &Path) -> io::Result<()> {
    if !path.is_absolute() {
        return Err(invalid("SIFR_CACHE_DIR must be an absolute path"));
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir | Component::CurDir) {
            return Err(invalid("cache path traversal"));
        }
        current.push(part);
        // Drive prefixes are not independently accessible paths.
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() => security::no_reparse(&current)?,
            Ok(_) => {
                return Err(invalid(format!(
                    "unsafe cache directory {}",
                    current.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match security::create_directory(&current) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error),
                }
                check_owned(&current)?;
            }
            Err(error) => return Err(error),
        }
    }
    check_owned(path)
}

pub(crate) fn check_owned(path: &Path) -> io::Result<()> {
    security::check(path)
}

/// Seal producer-created payloads without relying on the ambient umask.
/// Symlinks are never followed; required paths still reject them at use.
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

pub(crate) fn payload(root: &Path, relative_path: &Path) -> io::Result<()> {
    relative(relative_path)?;
    let mut path = root.to_path_buf();
    check_owned(&path)?;
    for part in relative_path.components() {
        path.push(part);
        check_owned(&path)?;
    }
    Ok(())
}

pub(crate) fn entry_lock(parent: &Path, key: &str) -> io::Result<File> {
    open_entry_lock(parent, key, true)
}

/// Project semantic readers/writers do not pass their lock handle to children.
pub(crate) fn process_entry_lock(parent: &Path, key: &str) -> io::Result<File> {
    open_entry_lock(parent, key, false)
}

fn open_entry_lock(parent: &Path, key: &str, inherit: bool) -> io::Result<File> {
    relative(Path::new(key))?;
    let locks = parent.join(".locks");
    directory(&locks)?;
    let path = locks.join(key);
    let file = match security::create_file(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            check_owned(&path)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)?
        }
        Err(error) => return Err(error),
    };
    check_owned(&path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("unsafe cache ownership lock"));
    }
    if inherit {
        inherit_lease(&file)?;
    }
    Ok(file)
}

/// The native child can retain this handle identity; Windows byte-range locks
/// remain process-owned. Job ownership terminates native descendants if their
/// compiler owner dies, before GC can reuse the entry.
#[allow(unsafe_code)]
fn inherit_lease(file: &File) -> io::Result<()> {
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
    // SAFETY: the File retains this live handle for the lifetime of the lease.
    if unsafe {
        SetHandleInformation(
            file.as_raw_handle(),
            HANDLE_FLAG_INHERIT,
            HANDLE_FLAG_INHERIT,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[derive(serde::Serialize)]
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
    let meta = fs::symlink_metadata(path)?;
    check_owned(path)?;
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut total = 0;
    for child in fs::read_dir(path)? {
        total += size(&child?.path())?;
    }
    Ok(total)
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
                let same_scope = fs::read(path.join("artifact_cache.json"))
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                    .and_then(|value| value["scope"].as_str().map(PathBuf::from))
                    .is_some_and(|owner| owner == scope);
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
        let same_scope = fs::read(entry.path.join("artifact_cache.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .and_then(|value| value["scope"].as_str().map(PathBuf::from))
            .is_some_and(|owner| owner == scope);
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

pub(crate) fn new_private_file(path: &Path) -> io::Result<File> {
    security::create_file(path)
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
pub(crate) fn publish(source: &Path, destination: &Path) -> io::Result<()> {
    security::durable_rename(source, destination)
}
pub(crate) fn sync_stage(path: &Path) -> io::Result<()> {
    // Every staged file is sync_all'd before MoveFileExW WRITE_THROUGH
    // publishes the directory. Verify the stage still has a safe owner.
    check_owned(path)
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
        let lease = entry_lock(&family, &active_key).unwrap();
        lease.try_lock().unwrap();
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
