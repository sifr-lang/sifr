//! Owned generated-entry storage. Locks are never unlinked: their inode is the lease.
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
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

#[allow(unsafe_code)]
fn uid() -> u32 {
    // SAFETY: geteuid has no arguments, pointers, or preconditions.
    unsafe { libc::geteuid() }
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
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(invalid(format!(
                    "unsafe cache directory {}",
                    current.display()
                )));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let mut builder = fs::DirBuilder::new();
                std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
                match builder.create(&current) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e),
                }
                check_owned(&current)?;
            }
            Err(e) => return Err(e),
        }
    }
    check_owned(path)
}

pub(crate) fn check_owned(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink()
        || meta.uid() != uid()
        || meta.permissions().mode() & 0o022 != 0
    {
        return Err(invalid(format!(
            "cache entry is symlink, foreign-owned, or writable by others: {}",
            path.display()
        )));
    }
    Ok(())
}

/// Seal producer-created payloads without relying on the ambient umask.
/// Symlinks are never followed; required paths still reject them at use.
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

/// Project semantic readers/writers have no mutable native child owner. Keep
/// their leases CLOEXEC so unrelated subprocesses cannot prolong retention.
pub(crate) fn process_entry_lock(parent: &Path, key: &str) -> io::Result<File> {
    open_entry_lock(parent, key, false)
}

fn open_entry_lock(parent: &Path, key: &str, inherit: bool) -> io::Result<File> {
    relative(Path::new(key))?;
    let locks = parent.join(".locks");
    directory(&locks)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(locks.join(key))?;
    let meta = file.metadata()?;
    if meta.uid() != uid() || meta.permissions().mode() & 0o022 != 0 || !meta.is_file() {
        return Err(invalid("unsafe cache ownership lock"));
    }
    if inherit {
        inherit_lease(&file)?;
    }
    Ok(file)
}

#[allow(unsafe_code)]
fn inherit_lease(file: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    // SAFETY: the descriptor is live. Keep the lease in native descendants so
    // a killed writer cannot make its still-running Cargo workspace pruneable.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, 0) } < 0 {
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
                        .custom_flags(libc::O_NOFOLLOW)
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

pub(crate) fn new_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
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
pub(crate) fn publish(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)?;
    let parent = destination
        .parent()
        .ok_or_else(|| invalid("missing publication parent"))?;
    File::open(parent)?.sync_all()
}
pub(crate) fn sync_stage(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

pub(crate) fn real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
}
