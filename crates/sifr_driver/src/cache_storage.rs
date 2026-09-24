//! CLI cache policy, inventory and pruning.
#[cfg(test)]
pub(crate) use sifr_cache_storage::lock_bounded_with_hooks;
pub(crate) use sifr_cache_storage::{
    LEASE_WAIT, LeaseActivity, check_owned, directory, entry_lock, invalid,
    lock_bounded_with_cancel, new_private_file, owner_scope, payload, publish, relative, uid,
};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;
pub fn root() -> PathBuf {
    if let Some(root) = std::env::var_os("SIFR_CACHE_DIR") {
        return PathBuf::from(root);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    if cfg!(target_os = "macos") {
        home.join("Library/Caches/sifr")
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cache"))
            .join("sifr")
    }
}

pub(crate) fn lock_bounded(
    file: &File,
    path: &Path,
    shared: bool,
    limit: Duration,
) -> io::Result<()> {
    lock_bounded_with_cancel(file, path, shared, limit, || {
        if crate::process_signals::cancelled() != 0 {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "cache lease wait cancelled by signal",
            ))
        } else {
            Ok(())
        }
    })
}

pub(crate) fn seal(root: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.uid() != uid() {
        return Err(invalid(format!(
            "foreign-owned staged payload: {}",
            root.display()
        )));
    }
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    fs::set_permissions(
        root,
        fs::Permissions::from_mode(metadata.permissions().mode() & !0o022),
    )?;
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
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}
pub(crate) fn read_write_private_file(path: &Path) -> io::Result<File> {
    check_owned(path)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}
pub(crate) fn sync_stage(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

#[derive(serde::Serialize)]
pub struct CacheInspection {
    pub root: PathBuf,
    pub entries: Vec<CacheEntryInspection>,
    pub protected_roots: Vec<PathBuf>,
    pub owners: Vec<CacheOwnerInspection>,
}
#[derive(serde::Serialize)]
pub struct CacheEntryInspection {
    pub path: PathBuf,
    pub bytes: u64,
    pub protected: bool,
    pub owner: &'static str,
}
#[derive(serde::Serialize)]
pub struct CacheOwnerInspection {
    pub path: PathBuf,
    pub owner: &'static str,
    pub policy: &'static str,
}

fn size(path: &Path) -> io::Result<u64> {
    let meta = fs::symlink_metadata(path)?;
    if meta.uid() != uid() {
        return Err(invalid("foreign-owned cache payload"));
    }
    if meta.file_type().is_symlink() {
        return Ok(meta.len());
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut total = 0;
    for child in fs::read_dir(path)? {
        total += size(&child?.path())?;
    }
    Ok(total)
}

/// Inventory known cache namespaces. Only entries with a matching current
/// worktree owner and an uncontended permanent lease can enter pressure pruning.
pub fn inspect() -> io::Result<CacheInspection> {
    let root = root();
    let artifacts = root.join("native/artifacts");
    let mut entries = Vec::new();
    let mut protected_roots = Vec::new();
    let mut owners = Vec::new();
    let scope = owner_scope()?;
    for (relative, owner, policy) in [
        (
            "native/artifacts",
            "generated artifacts",
            "owned finalized entries and abandoned stages under pressure",
        ),
        (
            "native/artifacts/cargo_resolution",
            "prepared Cargo resolution",
            "owned auxiliary roots under pressure; whole root lease",
        ),
        (
            "native/artifacts/rust_bridge_probes",
            "Rust bridge probe receipts",
            "DX9-F1 lifecycle owner; protected",
        ),
        (
            "native/families",
            "native Cargo families",
            "inactive owned families under pressure; Cargo internals remain whole",
        ),
        (
            "native/publications",
            "native publication leases",
            "permanent lock inodes; no payload pruning",
        ),
        (
            "metadata",
            "development metadata",
            "metadata producer and installed pin owner; protected",
        ),
        (
            "projects",
            "project generations",
            "cache prune-project owns exact workspace",
        ),
        (
            "test-fixtures",
            "native fixture tests",
            "test owner resets scoped fixtures; protected",
        ),
    ] {
        owners.push(CacheOwnerInspection {
            path: root.join(relative),
            owner,
            policy,
        });
    }
    if root.exists() {
        directory(&root)?;
        for child in fs::read_dir(&root)? {
            let child = child?.path();
            if !matches!(
                child.file_name().and_then(|v| v.to_str()),
                Some("native" | "metadata" | "projects" | "test-fixtures")
            ) {
                protected_roots.push(child);
            }
        }
    }
    let native = root.join("native");
    if native.exists() {
        directory(&native)?;
        for child in fs::read_dir(&native)? {
            let child = child?.path();
            if !matches!(
                child.file_name().and_then(|v| v.to_str()),
                Some("artifacts" | "families" | "publications")
            ) {
                protected_roots.push(child);
            }
        }
    }
    if artifacts.exists() {
        directory(&artifacts)?;
        for family in fs::read_dir(&artifacts)? {
            let family = family?.path();
            if family
                .file_name()
                .is_some_and(|name| name == "cargo_resolution")
            {
                if directory(&family).is_err() {
                    protected_roots.push(family);
                    continue;
                }
                for auxiliary in fs::read_dir(&family)? {
                    let path = auxiliary?.path();
                    let Some(key) = path.file_name().and_then(|name| name.to_str()) else {
                        protected_roots.push(path);
                        continue;
                    };
                    if key == ".locks" {
                        continue;
                    }
                    if key.len() != 64
                        || !key.bytes().all(|c| c.is_ascii_hexdigit())
                        || check_owned(&path).is_err()
                        || !path.is_dir()
                    {
                        protected_roots.push(path);
                        continue;
                    }
                    let protected = !resolution_owned_by(&path, key, &scope)
                        || existing_lock(&family, key)
                            .as_ref()
                            .map_or(true, |file| file.try_lock().is_err());
                    let bytes = match size(&path) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            protected_roots.push(path);
                            continue;
                        }
                    };
                    entries.push(CacheEntryInspection {
                        path,
                        bytes,
                        protected,
                        owner: "prepared Cargo resolution",
                    });
                }
                continue;
            }
            // Probe receipts and unknown auxiliary roots have separate owners.
            if !family.join(".locks").is_dir() {
                protected_roots.push(family);
                continue;
            }
            if check_owned(&family).is_err() {
                protected_roots.push(family);
                continue;
            }
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
                        .custom_flags(libc::O_NOFOLLOW)
                        .open(family.join(lock_path))
                });
                let same_scope = artifact_owned_by(&path, &scope);
                let protected =
                    !same_scope || lock.as_ref().map_or(true, |file| file.try_lock().is_err());
                let bytes = match size(&path) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        protected_roots.push(path);
                        continue;
                    }
                };
                entries.push(CacheEntryInspection {
                    bytes,
                    path,
                    protected,
                    owner: "generated artifacts",
                });
            }
        }
    }
    let families = root.join("native/families");
    if families.exists() {
        directory(&families)?;
        for child in fs::read_dir(&families)? {
            let path = child?.path();
            let Some(key) = path.file_name().and_then(|v| v.to_str()) else {
                protected_roots.push(path);
                continue;
            };
            if key == ".locks" {
                continue;
            }
            if key.len() != 64
                || !key.bytes().all(|c| c.is_ascii_hexdigit())
                || check_owned(&path).is_err()
                || !path.is_dir()
            {
                protected_roots.push(path);
                continue;
            }
            let same_scope = family_owned_by(&path, key, &scope);
            let lock = existing_lock(&families, key);
            let protected =
                !same_scope || lock.as_ref().map_or(true, |file| file.try_lock().is_err());
            let bytes = match size(&path) {
                Ok(bytes) => bytes,
                Err(_) => {
                    protected_roots.push(path);
                    continue;
                }
            };
            entries.push(CacheEntryInspection {
                bytes,
                path,
                protected,
                owner: "native Cargo families",
            });
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
        let same_scope = artifact_owned_by(&entry.path, &scope);
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
        owners,
    })
}

fn artifact_owned_by(path: &Path, scope: &Path) -> bool {
    if payload(path, Path::new("artifact_cache.json")).is_err() {
        return false;
    }
    fs::read(path.join("artifact_cache.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value["scope"].as_str().map(PathBuf::from))
        .is_some_and(|owner| owner == scope)
}

fn family_owned_by(path: &Path, key: &str, scope: &Path) -> bool {
    if payload(path, Path::new("native_family.json")).is_err() {
        return false;
    }
    fs::read(path.join("native_family.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|value| {
            value["schema"] == 1
                && value["family"] == key
                && value["owner_scope"]
                    .as_str()
                    .is_some_and(|owner| Path::new(owner) == scope)
        })
}

fn resolution_owned_by(path: &Path, key: &str, scope: &Path) -> bool {
    if payload(path, Path::new("resolution_owner.json")).is_err() {
        return false;
    }
    fs::read(path.join("resolution_owner.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|value| {
            value["schema"] == 1
                && value["key"] == key
                && value["owner_scope"]
                    .as_str()
                    .is_some_and(|owner| Path::new(owner) == scope)
        })
}

fn existing_lock(parent: &Path, key: &str) -> io::Result<File> {
    let path = parent.join(".locks").join(key);
    check_owned(&path)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    if !lock.metadata()?.is_file() {
        return Err(invalid("unsafe cache ownership lock"));
    }
    Ok(lock)
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
        let lock = existing_lock(parent, key)?;
        if lock.try_lock().is_err() {
            entry.protected = true;
            continue;
        }
        check_owned(&entry.path)?;
        let scope = owner_scope()?;
        if entry.owner == "native Cargo families" {
            if !family_owned_by(&entry.path, key, &scope) {
                entry.protected = true;
                continue;
            }
        } else if entry.owner == "prepared Cargo resolution" {
            if !resolution_owned_by(&entry.path, key, &scope) {
                entry.protected = true;
                continue;
            }
        } else {
            if !artifact_owned_by(&entry.path, &scope) {
                entry.protected = true;
                continue;
            }
        }
        if !dry_run {
            fs::remove_dir_all(&entry.path)?;
        }
        reclaimed += entry.bytes;
    }
    Ok(report)
}

/// Available space on the selected destination filesystem.
#[allow(unsafe_code)]
#[allow(
    clippy::useless_conversion,
    reason = "statvfs block counts are u32 on macOS and u64 on Linux"
)]
pub fn available_bytes() -> io::Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let root = root();
    directory(&root)?;
    let path = std::ffi::CString::new(root.as_os_str().as_bytes())
        .map_err(|_| invalid("NUL in cache path"))?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: the C path is terminated and the output points to initialized-size storage.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: statvfs succeeded and initialized the structure.
    let stats = unsafe { stats.assume_init() };
    Ok(u64::from(stats.f_bavail) * stats.f_frsize)
}

pub(crate) fn real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
}
