//! A process-local lock ends with its Rust owner, not an unrelated child's exec.
//! CLOEXEC prevents retention after exec, but a concurrently forked child still
//! shares the open file description before exec. Explicit unlock closes that gap.
use std::{fs::File, io};

pub(super) struct ProcessLease {
    file: File,
}

impl ProcessLease {
    pub(super) fn exclusive(file: File) -> io::Result<Self> {
        file.try_lock().map_err(io::Error::from)?;
        Ok(Self { file })
    }

    pub(super) fn shared(file: File) -> io::Result<Self> {
        file.try_lock_shared().map_err(io::Error::from)?;
        Ok(Self { file })
    }
}

impl Drop for ProcessLease {
    fn drop(&mut self) {
        // This guard never hands its lock to a child. Shared parent ownership
        // uses Arc<ProcessLease>, never independently unlocking File clones.
        // Drop cannot report an unlock error; closing still releases this handle.
        let _ = self.file.unlock();
    }
}

#[cfg(all(test, unix))]
#[path = "process_lease_tests.rs"]
mod tests;
