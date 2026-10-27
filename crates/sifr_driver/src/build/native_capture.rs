//! Publish copied native files only after every inherited writer has closed.
use std::io;
use std::path::Path;

pub(super) fn copy(source: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        unix::stage(source, destination)?.publish(
            destination,
            std::time::Duration::from_secs(30),
            || {
                if crate::process_signals::cancelled() != 0 {
                    Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "native capture publication cancelled",
                    ))
                } else {
                    Ok(())
                }
            },
            || {},
        )
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(source, destination).map(|_| ())
    }
}

#[cfg(unix)]
mod unix {
    use std::fs::{File, TryLockError};
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    pub(super) struct Staging {
        path: PathBuf,
        writer: Option<File>,
    }

    impl Drop for Staging {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    pub(super) fn stage(source: &Path, destination: &Path) -> io::Result<Staging> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut input = File::open(source)?;
        let metadata = input.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native capture source must be a regular file",
            ));
        }
        // create_new protects existing names, including stale files after a
        // crash or PID reuse. Never truncate or reclaim a name we did not create.
        for _ in 0..128 {
            let path = parent.join(format!(
                ".sifr-native-capture-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            let mut writer = match crate::cache_storage::new_private_file(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            };
            let mut staging = Staging { path, writer: None };
            // A raw file lock follows this open file description into a fork.
            // Do NOT explicitly unlock it, or use ProcessLease: publication
            // must wait for inherited writable descriptors as well as ours.
            writer.lock()?;
            io::copy(&mut input, &mut writer)?;
            writer.set_permissions(metadata.permissions())?;
            staging.writer = Some(writer);
            return Ok(staging);
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "native capture staging names exhausted",
        ))
    }

    impl Staging {
        pub(super) fn publish(
            mut self,
            destination: &Path,
            limit: Duration,
            cancelled: impl Fn() -> io::Result<()>,
            mut blocked: impl FnMut(),
        ) -> io::Result<()> {
            let reader = crate::cache_storage::read_private_file(&self.path)?;
            drop(self.writer.take());
            let start = Instant::now();
            loop {
                cancelled()?;
                match reader.try_lock_shared() {
                    Ok(()) => break,
                    Err(TryLockError::Error(error)) => return Err(error),
                    Err(TryLockError::WouldBlock) => blocked(),
                }
                if start.elapsed() >= limit {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "native capture inherited writer did not close before publication deadline",
                    ));
                }
                std::thread::sleep(
                    Duration::from_millis(2).min(limit.saturating_sub(start.elapsed())),
                );
            }
            // Only read-only descriptors can remain now. Replacing the name
            // also avoids modifying an inode that another process is executing.
            std::fs::rename(&self.path, destination)
        }
    }
}

#[cfg(all(test, unix))]
#[path = "native_capture_tests.rs"]
mod tests;
