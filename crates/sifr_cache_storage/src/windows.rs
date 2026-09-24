//! Owned generated-entry storage. Locks are never unlinked: their inode is the lease.
use crate::windows_storage_security as security;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

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

/// Preserve the Windows metadata wait contract: poll the OS lease until it is
/// released, and let the caller interrupt each attempt. The Windows file time
/// is not a reliable renewal signal while another handle remains open.
pub fn lock_with_hooks(
    file: &File,
    mut cancelled: impl FnMut() -> io::Result<()>,
    mut waiting: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    loop {
        cancelled()?;
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => {
                waiting()?;
                std::thread::sleep(Duration::from_millis(10));
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
