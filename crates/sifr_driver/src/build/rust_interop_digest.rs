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

// Lock digests remain with the separate N02b2 prepared/unchanged-lock batch.
pub(super) fn digest_file(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| fnv1a64_hex(&bytes))
}

pub(super) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
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
    use super::digest_path_checked;
    use std::fs;

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
