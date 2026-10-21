//! A process-local lock ends with its Rust owner, not an unrelated child's exec.
//! CLOEXEC prevents retention after exec, but a concurrently forked child still
//! shares the open file description before exec. Explicit unlock closes that gap.
use std::{fs::File, io};

/// A nonblocking lock released when its process-local Rust owner is dropped.
///
/// The input file must have unique ownership: do not retain file clones.
/// Share an acquired guard with `Arc<ProcessLease>` when needed. This owner
/// must not be used for locks intentionally retained by child processes.
pub struct ProcessLease {
    file: File,
}

impl ProcessLease {
    /// Acquire an exclusive lock on a uniquely owned file without waiting.
    pub fn exclusive(file: File) -> io::Result<Self> {
        file.try_lock().map_err(io::Error::from)?;
        Ok(Self { file })
    }

    /// Acquire a shared lock on a uniquely owned file without waiting.
    pub fn shared(file: File) -> io::Result<Self> {
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
