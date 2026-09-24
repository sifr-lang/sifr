use sifr_identity::IdentityEncoder;
use std::fs;
use std::io;
use std::path::Path;

/// Required bridge and sysroot roots must be readable in full. Links are not
/// source members: rejecting them prevents escape and traversal cycles.
pub(super) fn digest_path_checked(path: &Path) -> io::Result<String> {
    reject_linked_ancestors(path)?;
    let mut identity = IdentityEncoder::new("rust-bridge-source-tree-v3");
    encode_digest_entries(path, path, &mut identity)?;
    Ok(identity.finish())
}

/// An optional root is absent only on NotFound. Existing files, empty
/// directories, links and read failures remain distinct observations.
pub(super) fn digest_optional_directory_checked(path: &Path) -> io::Result<Option<String>> {
    reject_linked_ancestors(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => digest_path_checked(path).map(Some),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "optional source root is not a directory: {}",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
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

fn reject_linked_ancestors(path: &Path) -> io::Result<()> {
    for ancestor in path.ancestors().skip(1) {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "bridge source parent link is unsupported: {}",
                        ancestor.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn encode_digest_entries(
    root: &Path,
    path: &Path,
    identity: &mut IdentityEncoder,
) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bridge source link is unsupported: {}", path.display()),
        ));
    }
    let relative = path.strip_prefix(root).unwrap_or(path);
    if metadata.is_file() {
        identity.field("kind", b"file");
        identity.field("path", relative.as_os_str().as_encoded_bytes());
        identity.field("contents", &fs::read(path)?);
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported bridge source",
        ));
    }
    identity.field("kind", b"dir");
    identity.field("path", relative.as_os_str().as_encoded_bytes());
    let mut children = fs::read_dir(path)?.collect::<io::Result<Vec<_>>>()?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for entry in children {
        encode_digest_entries(root, &entry.path(), identity)?;
    }
    Ok(())
}

pub(super) fn push_cache_bytes(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0xff);
}

#[cfg(test)]
mod tests {
    use super::{
        digest_file_checked, digest_optional_directory_checked, digest_path_checked,
        nearest_lock_digest_checked,
    };
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
    fn optional_tree_identity_distinguishes_absent_empty_and_replacement() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("optional");
        assert_eq!(digest_optional_directory_checked(&path).unwrap(), None);
        fs::create_dir(&path).unwrap();
        let empty = digest_optional_directory_checked(&path).unwrap().unwrap();
        fs::write(path.join("source.rs"), b"one").unwrap();
        let first = digest_optional_directory_checked(&path).unwrap().unwrap();
        assert_ne!(empty, first);
        fs::write(path.join("source.rs"), b"two").unwrap();
        assert_ne!(
            first,
            digest_optional_directory_checked(&path).unwrap().unwrap()
        );
        fs::remove_dir_all(&path).unwrap();
        assert_eq!(digest_optional_directory_checked(&path).unwrap(), None);
        fs::write(&path, b"replacement").unwrap();
        assert!(digest_optional_directory_checked(&path).is_err());
    }

    #[test]
    fn bridge_tree_identity_orders_membership_and_empty_directories() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        for (root, names) in [
            (first.path(), ["a.rs", "z.rs"]),
            (second.path(), ["z.rs", "a.rs"]),
        ] {
            for name in names {
                fs::write(root.join(name), name).unwrap();
            }
        }
        assert_eq!(
            digest_path_checked(first.path()).unwrap(),
            digest_path_checked(second.path()).unwrap()
        );
        fs::create_dir(first.path().join("empty")).unwrap();
        assert_ne!(
            digest_path_checked(first.path()).unwrap(),
            digest_path_checked(second.path()).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn bridge_tree_rejects_links_and_denied_members() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("outside.rs"), b"outside").unwrap();
        let linked = root.path().join("linked.rs");
        symlink(outside.path().join("outside.rs"), &linked).unwrap();
        assert!(digest_path_checked(root.path()).is_err());
        fs::remove_file(&linked).unwrap();
        symlink(root.path(), &linked).unwrap();
        assert!(digest_path_checked(root.path()).is_err());
        fs::remove_file(&linked).unwrap();
        let denied = root.path().join("denied.rs");
        fs::write(&denied, b"secret").unwrap();
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
        let result = digest_path_checked(root.path());
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(result.is_err());
        symlink(outside.path(), &linked).unwrap();
        assert!(digest_path_checked(&linked).is_err());
        assert!(digest_path_checked(&linked.join("outside.rs")).is_err());
        assert!(digest_optional_directory_checked(&linked.join("missing")).is_err());
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
