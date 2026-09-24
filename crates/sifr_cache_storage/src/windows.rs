//! Owned generated-entry storage. Locks are never unlinked: their inode is the lease.
use crate::windows_storage_security as security;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

pub fn owner_scope() -> io::Result<PathBuf> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    Ok(cwd
        .ancestors()
        .find(|path| path.join(".git").exists())
        .unwrap_or(&cwd)
        .to_path_buf())
}

pub fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message.into())
}

pub fn relative(path: &Path) -> io::Result<()> {
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
pub fn directory(path: &Path) -> io::Result<()> {
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

pub fn check_owned(path: &Path) -> io::Result<()> {
    security::check(path)
}

/// Seal producer-created payloads without relying on the ambient umask.
/// Symlinks are never followed; required paths still reject them at use.
pub fn payload(root: &Path, relative_path: &Path) -> io::Result<()> {
    relative(relative_path)?;
    let mut path = root.to_path_buf();
    check_owned(&path)?;
    for part in relative_path.components() {
        path.push(part);
        check_owned(&path)?;
    }
    Ok(())
}

pub fn entry_lock(parent: &Path, key: &str) -> io::Result<File> {
    open_entry_lock(parent, key, true)
}

/// Project semantic readers/writers do not pass their lock handle to children.
pub fn open_entry_lock(parent: &Path, key: &str, inherit: bool) -> io::Result<File> {
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

/// A renewing owner has the full hard limit; an idle one has a shorter wait.
pub const LEASE_IDLE_WAIT: Duration = Duration::from_secs(30);
pub const LEASE_WAIT: Duration = Duration::from_mins(41);

pub struct LeaseActivity {
    stop: std::sync::mpsc::Sender<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl LeaseActivity {
    pub fn start(file: &File) -> io::Result<Self> {
        Self::start_with_interval(file, Duration::from_secs(5))
    }
    pub fn start_with_interval(file: &File, interval: Duration) -> io::Result<Self> {
        let file = file.try_clone()?;
        let (stop, stopped) = std::sync::mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("sifr-cache-lease".into())
            .spawn(move || {
                while stopped.recv_timeout(interval)
                    == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                {
                    if record_lock_owner(&file).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for LeaseActivity {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn record_lock_owner(file: &File) -> io::Result<()> {
    use std::io::{Seek, Write};
    let note = format!(
        "last_exclusive_pid={} scope={} renewed_at_ns={}\n",
        std::process::id(),
        owner_scope()?.display(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    );
    file.set_len(0)?;
    let mut handle = file;
    handle.seek(std::io::SeekFrom::Start(0))?;
    handle.write_all(note.as_bytes())?;
    file.sync_data()
}
fn lease_timeout(file: &File, path: &Path, limit: Duration) -> io::Error {
    use std::io::{Read, Seek};
    let mut note = String::new();
    let owner = file
        .try_clone()
        .and_then(|mut clone| {
            clone.seek(std::io::SeekFrom::Start(0))?;
            clone.take(4096).read_to_string(&mut note)?;
            Ok(note)
        })
        .unwrap_or_else(|_| "owner note unavailable".into());
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!(
            "cache lease wait exceeded {:?}: {}; {} (the recorded parent may have a live child)",
            limit,
            path.display(),
            owner.trim()
        ),
    )
}
pub fn lock_bounded_with_hooks(
    file: &File,
    path: &Path,
    shared: bool,
    hard_limit: Duration,
    idle_limit: Duration,
    mut cancelled: impl FnMut() -> io::Result<()>,
    mut waiting: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    let started = Instant::now();
    let mut idle_since = started;
    let mut last_change = file.metadata().and_then(|meta| meta.modified()).ok();
    loop {
        cancelled()?;
        let attempt = if shared {
            file.try_lock_shared()
        } else {
            file.try_lock()
        };
        match attempt {
            Ok(()) => {
                if !shared {
                    record_lock_owner(file)?;
                }
                return Ok(());
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                let changed = file.metadata().and_then(|meta| meta.modified()).ok();
                if changed.is_some() && changed != last_change {
                    last_change = changed;
                    idle_since = Instant::now();
                }
                let hard_remaining = hard_limit.saturating_sub(started.elapsed());
                let idle_remaining = idle_limit.saturating_sub(idle_since.elapsed());
                if hard_remaining.is_zero() {
                    return Err(lease_timeout(file, path, hard_limit));
                }
                if idle_remaining.is_zero() {
                    return Err(lease_timeout(file, path, idle_limit));
                }
                waiting()?;
                std::thread::sleep(
                    Duration::from_millis(10)
                        .min(hard_remaining)
                        .min(idle_remaining),
                );
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
        }
    }
}

pub fn new_private_file(path: &Path) -> io::Result<File> {
    security::create_file(path)
}
pub fn publish(source: &Path, destination: &Path) -> io::Result<()> {
    security::durable_rename(source, destination)
}
