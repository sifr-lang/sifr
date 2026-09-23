use sifr_identity::IdentityEncoder;
use std::fs;
use std::io;
use std::path::Path;

/// A bridge authority must be readable in full before it can identify reusable output.
pub(super) fn digest_path_checked(path: &Path) -> io::Result<String> {
    let mut entries = Vec::new();
    collect_digest_entries(path, path, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut identity = IdentityEncoder::new("rust-bridge-source-tree-v2");
    for (relative, file_bytes) in entries {
        identity.field("path", relative.as_bytes());
        identity.field("contents", &file_bytes);
    }
    Ok(identity.finish())
}

/// Probe-only snapshots still run Cargo before accepting a successful probe.
pub(super) fn digest_path(path: &Path) -> String {
    digest_path_checked(path).unwrap_or_else(|error| {
        let mut identity = IdentityEncoder::new("rust-bridge-source-unreadable-v2");
        identity.field("path", normalized_path_string(path).as_bytes());
        identity.field("error", error.kind().to_string().as_bytes());
        identity.finish()
    })
}

/// A missing file is distinct from an empty file. An existing unreadable path
/// cannot authorize reuse of a previously prepared or final artifact.
pub(super) fn digest_file_checked(path: &Path) -> io::Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    let bytes = fs::read(path)?;
    let mut identity = IdentityEncoder::new("cargo-input-file-v2");
    identity.field("contents", &bytes);
    Ok(Some(identity.finish()))
}

pub(super) fn digest_lock_file_checked(path: &Path) -> io::Result<Option<String>> {
    let Some(payload) = digest_file_checked(path)? else {
        return Ok(None);
    };
    let mut identity = IdentityEncoder::new("cargo-lock-authority-v2");
    identity.field("path", path.as_os_str().as_encoded_bytes());
    identity.field("contents", payload.as_bytes());
    Ok(Some(identity.finish()))
}

pub(super) fn nearest_lock_digest_checked(start: &Path) -> io::Result<Option<String>> {
    for ancestor in start.ancestors() {
        let lock = ancestor.join("Cargo.lock");
        match fs::symlink_metadata(&lock) {
            Ok(_) => return digest_lock_file_checked(&lock),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}

pub(super) fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map_or_else(|_| normalized_path_string(path), normalized_path_string)
}

pub(super) fn normalized_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn collect_digest_entries(
    root: &Path,
    path: &Path,
    entries: &mut Vec<(String, Vec<u8>)>,
) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        entries.push((relative_path_string(root, path), fs::read(path)?));
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported bridge source",
        ));
    }
    for entry in fs::read_dir(path)? {
        collect_digest_entries(root, &entry?.path(), entries)?;
    }
    Ok(())
}

pub(super) fn push_cache_bytes(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0xff);
}

#[cfg(test)]
mod tests {
    use super::{digest_file_checked, digest_path_checked, nearest_lock_digest_checked};
    use std::fs;

    #[test]
    fn lock_identity_distinguishes_absent_empty_payload_and_unreadable() {
        let root = tempfile::tempdir().unwrap();
        let lock = root.path().join("Cargo.lock");
        assert_eq!(digest_file_checked(&lock).unwrap(), None);
        fs::write(&lock, "").unwrap();
        let empty = digest_file_checked(&lock).unwrap().unwrap();
        assert_eq!(
            nearest_lock_digest_checked(root.path()).unwrap(),
            Some(super::digest_lock_file_checked(&lock).unwrap().unwrap())
        );
        fs::write(&lock, "version = 4\n").unwrap();
        assert_ne!(empty, digest_file_checked(&lock).unwrap().unwrap());
        #[cfg(unix)]
        {
            fs::remove_file(&lock).unwrap();
            std::os::unix::fs::symlink(root.path().join("missing"), &lock).unwrap();
            assert!(digest_file_checked(&lock).is_err());
            assert!(nearest_lock_digest_checked(root.path()).is_err());
        }
    }

    #[test]
    fn bridge_tree_identity_distinguishes_states_and_payload() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("bridge");
        assert!(digest_path_checked(&source).is_err());
        fs::create_dir(&source).unwrap();
        let empty = digest_path_checked(&source).unwrap();
        fs::write(source.join("a.rs"), b"a").unwrap();
        let first = digest_path_checked(&source).unwrap();
        assert_ne!(empty, first);
        fs::write(source.join("a.rs"), b"b").unwrap();
        assert_ne!(first, digest_path_checked(&source).unwrap());
        fs::remove_file(source.join("a.rs")).unwrap();
        fs::create_dir(source.join("a.rs")).unwrap();
        assert_ne!(first, digest_path_checked(&source).unwrap());
        fs::remove_dir(source.join("a.rs")).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source.join("missing.rs"), source.join("unreadable.rs"))
                .unwrap();
            assert!(digest_path_checked(&source).is_err());
        }
    }
}
