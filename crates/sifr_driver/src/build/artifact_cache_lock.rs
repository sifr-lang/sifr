use fs4::fs_std::FileExt as _;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

pub(super) struct ArtifactCacheLease {
    _file: File,
}

pub(super) fn acquire_shared(root: &Path) -> io::Result<ArtifactCacheLease> {
    let file = open_lock_file(root)?;
    file.lock_shared()?;
    Ok(ArtifactCacheLease { _file: file })
}

pub(super) fn try_acquire_exclusive(root: &Path) -> io::Result<ArtifactCacheLease> {
    let file = open_lock_file(root)?;
    if !file.try_lock_exclusive()? {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "artifact cache is in use; retry cache clean after active builds and tests finish",
        ));
    }
    Ok(ArtifactCacheLease { _file: file })
}

fn open_lock_file(root: &Path) -> io::Result<File> {
    let path = lock_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
}

fn lock_path(root: &Path) -> PathBuf {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sifr-artifact-cache");
    root.with_file_name(format!(".{name}.lock"))
}
