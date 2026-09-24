//! Owned generated-entry storage. Locks are never unlinked: their inode is the lease.
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{FileExt, MetadataExt, OpenOptionsExt, PermissionsExt};
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

#[allow(unsafe_code)]
pub fn uid() -> u32 {
    // SAFETY: geteuid has no arguments, pointers, or preconditions.
    unsafe { libc::geteuid() }
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

pub fn check_owned(path: &Path) -> io::Result<()> {
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

/// Project semantic readers/writers have no mutable native child owner. Keep
/// their leases CLOEXEC so unrelated subprocesses cannot prolong retention.
pub fn open_entry_lock(parent: &Path, key: &str, inherit: bool) -> io::Result<File> {
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

/// A short idle deadline detects an owner that stops renewing its note. A
/// separate hard deadline remains bounded even if a live owner cannot finish.
/// Cargo subprocesses have a 40-minute safety deadline, so their waiters get
/// one additional minute to observe release and capture.
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

/// Poll cancellation before each attempt; a timeout never breaks another owner's lease.
pub fn lock_bounded_with_cancel(
    file: &File,
    path: &Path,
    shared: bool,
    limit: Duration,
    cancelled: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    lock_bounded_with_hooks(
        file,
        path,
        shared,
        limit,
        LEASE_IDLE_WAIT.min(limit),
        cancelled,
        || Ok(()),
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

fn record_lock_owner(file: &File) -> io::Result<()> {
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
    std::io::Seek::seek(&mut &*file, std::io::SeekFrom::Start(0))?;
    std::io::Write::write_all(&mut &*file, note.as_bytes())?;
    file.sync_data()
}

fn lease_timeout(file: &File, path: &Path, limit: Duration) -> io::Error {
    let mut bytes = [0_u8; 4096];
    let note = file
        .read_at(&mut bytes, 0)
        .ok()
        .and_then(|count| std::str::from_utf8(&bytes[..count]).ok().map(str::to_owned))
        .unwrap_or_else(|| "owner note unavailable".into());
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!(
            "cache lease wait exceeded {:?}: {}; {} (the recorded parent may have a live child)",
            limit,
            path.display(),
            note.trim()
        ),
    )
}

pub fn new_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}
pub fn publish(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)?;
    let parent = destination
        .parent()
        .ok_or_else(|| invalid("missing publication parent"))?;
    File::open(parent)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn private_file_and_atomic_publication() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        directory(&root).unwrap();
        let stage = root.join("metadata.stage");
        let output = root.join("metadata.sifrmeta");
        let mut file = new_private_file(&stage).unwrap();
        file.write_all(b"complete metadata").unwrap();
        file.sync_all().unwrap();
        assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);
        publish(&stage, &output).unwrap();
        assert!(!stage.exists());
        assert_eq!(fs::read(&output).unwrap(), b"complete metadata");
        payload(&root, Path::new("metadata.sifrmeta")).unwrap();
    }

    #[test]
    fn idle_and_live_lease_waits_remain_bounded() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        directory(&root).unwrap();
        let owner = entry_lock(&root, "metadata-key").unwrap();
        owner.try_lock().unwrap();
        let waiter = entry_lock(&root, "metadata-key").unwrap();
        let lock_path = root.join(".locks/metadata-key");
        let idle = lock_bounded_with_hooks(
            &waiter,
            &lock_path,
            false,
            Duration::from_millis(250),
            Duration::from_millis(40),
            || Ok(()),
            || Ok(()),
        );
        let idle_error = idle.unwrap_err();
        assert_eq!(idle_error.kind(), io::ErrorKind::TimedOut);
        assert!(idle_error.to_string().contains("40ms"));
        let activity =
            LeaseActivity::start_with_interval(&owner, Duration::from_millis(10)).unwrap();
        let live = lock_bounded_with_hooks(
            &waiter,
            &lock_path,
            false,
            Duration::from_millis(140),
            Duration::from_millis(40),
            || Ok(()),
            || Ok(()),
        );
        let live_error = live.unwrap_err();
        assert_eq!(live_error.kind(), io::ErrorKind::TimedOut);
        assert!(live_error.to_string().contains("140ms"));
        assert!(
            lock_path.exists(),
            "timeout must preserve the owner's lock inode"
        );
        drop(activity);
        owner.unlock().unwrap();
        lock_bounded_with_hooks(
            &waiter,
            &lock_path,
            false,
            Duration::from_millis(100),
            Duration::from_millis(40),
            || Ok(()),
            || Ok(()),
        )
        .unwrap();
        waiter.unlock().unwrap();
    }

    #[test]
    fn cancelled_wait_preserves_owner() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        directory(&root).unwrap();
        let owner = entry_lock(&root, "metadata-key").unwrap();
        owner.try_lock().unwrap();
        let waiter = entry_lock(&root, "metadata-key").unwrap();
        let mut polls = 0;
        let error = lock_bounded_with_hooks(
            &waiter,
            &root.join(".locks/metadata-key"),
            false,
            Duration::from_secs(1),
            Duration::from_secs(1),
            || {
                polls += 1;
                if polls > 1 {
                    Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"))
                } else {
                    Ok(())
                }
            },
            || Ok(()),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert!(root.join(".locks/metadata-key").exists());
        owner.unlock().unwrap();
    }
}
