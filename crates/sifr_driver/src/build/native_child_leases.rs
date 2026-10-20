//! Explicit native child ownership. Unix parent descriptors stay close-on-exec.
//! Windows retains its existing inheritable-handle and owned-Job policy.
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;

pub(crate) fn open(parent: &Path, key: &str) -> io::Result<File> {
    sifr_cache_storage::open_entry_lock(parent, key, cfg!(windows))
}

pub(crate) fn bind(command: &mut Command, lease: &File) -> io::Result<()> {
    #[cfg(unix)]
    return sifr_cache_storage::inherit_lease_for_command(command, lease);
    #[cfg(windows)]
    {
        // The original owner handle already inherits on this platform. Do not
        // substitute Unix descriptor locking semantics for the Windows Job policy.
        let _ = (command, lease);
        Ok(())
    }
}

/// Lifetime-only state; descriptor values are never part of a build identity.
#[derive(Clone, Debug, Default)]
pub(crate) struct NativeChildLeases {
    #[cfg(unix)]
    files: Vec<std::sync::Arc<File>>,
}

impl NativeChildLeases {
    pub(crate) fn with(&self, lease: &File) -> io::Result<Self> {
        #[cfg(unix)]
        {
            let mut next = self.clone();
            next.files.push(std::sync::Arc::new(lease.try_clone()?));
            Ok(next)
        }
        #[cfg(windows)]
        {
            let _ = lease;
            Ok(self.clone())
        }
    }

    pub(crate) fn bind(&self, command: &mut Command) -> io::Result<()> {
        #[cfg(unix)]
        for lease in &self.files {
            bind(command, lease)?;
        }
        #[cfg(windows)]
        let _ = command;
        Ok(())
    }
}

#[cfg(all(test, unix))]
#[path = "native_child_lease_tests.rs"]
mod tests;
