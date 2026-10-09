use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::path::Path;

/// Only reuse a content digest when the filesystem exposes a change timestamp
/// and file identity. Size/mtime alone miss same-version replacements and edits
/// that restore mtime. Unsupported platforms retain content observation.
#[derive(Default)]
pub(super) struct InterpreterSnapshot {
    cached: Option<(InterpreterStamp, u64)>,
    #[cfg(test)]
    reads: usize,
}

impl InterpreterSnapshot {
    pub(super) fn hash(&mut self, path: &Path, hasher: &mut DefaultHasher, stable: &mut bool) {
        let before = stamp(path);
        if let Some((stamp, digest)) = &self.cached {
            if before.as_ref() == Some(stamp) {
                before.hash(hasher);
                digest.hash(hasher);
                return;
            }
        }
        self.cached = None;
        let mut content = DefaultHasher::new();
        let mut readable = true;
        #[cfg(test)]
        {
            self.reads += 1;
        }
        super::hash_path(path, &mut content, &mut readable, true);
        match std::fs::metadata(path) {
            Ok(metadata) => {
                metadata.len().hash(&mut content);
                metadata.modified().ok().hash(&mut content);
            }
            Err(error) => {
                readable &= error.kind() == std::io::ErrorKind::NotFound;
                error.kind().hash(&mut content);
            }
        }
        let after = stamp(path);
        // A mutation during the read cannot establish a reusable snapshot.
        if before != after {
            readable = false;
        }
        let digest = content.finish();
        if readable {
            if let Some(stamp) = before.clone() {
                self.cached = Some((stamp, digest));
            }
        }
        *stable &= readable;
        before.hash(hasher);
        digest.hash(hasher);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct InterpreterStamp {
    path: std::path::PathBuf,
    resolved: std::path::PathBuf,
    metadata: Vec<FileStamp>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct FileStamp {
    device: u64,
    inode: u64,
    mode: u32,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

#[cfg(unix)]
fn stamp(path: &Path) -> Option<InterpreterStamp> {
    use std::os::unix::fs::MetadataExt as _;
    let mut metadata_stamps = Vec::with_capacity(2);
    // Canonical identity also observes changes through intermediate symlinks.
    let resolved = std::fs::canonicalize(path).ok()?;
    for metadata in [
        std::fs::symlink_metadata(path).ok()?,
        std::fs::metadata(path).ok()?,
    ] {
        if !metadata.is_file() && !metadata.file_type().is_symlink() {
            return None;
        }
        metadata_stamps.push(FileStamp {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            size: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        });
    }
    Some(InterpreterStamp {
        path: path.to_path_buf(),
        resolved,
        metadata: metadata_stamps,
    })
}

#[cfg(not(unix))]
fn stamp(_path: &Path) -> Option<InterpreterStamp> {
    None
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;

    fn fingerprint(cache: &mut InterpreterSnapshot, path: &Path) -> (u64, bool) {
        let mut hasher = DefaultHasher::new();
        let mut stable = true;
        cache.hash(path, &mut hasher, &mut stable);
        (hasher.finish(), stable)
    }

    #[test]
    fn replacement_and_restored_mtime_edits_invalidate_cached_content() {
        let temp = tempfile::tempdir().expect("temporary interpreter");
        let path = temp.path().join("python");
        fs::write(&path, b"same-version-A").expect("interpreter");
        let modified = fs::metadata(&path)
            .expect("metadata")
            .modified()
            .expect("mtime");
        let mut cache = InterpreterSnapshot::default();
        let first = fingerprint(&mut cache, &path);
        assert_eq!(fingerprint(&mut cache, &path), first);
        assert_eq!(cache.reads, 1);
        let replacement = temp.path().join("replacement");
        fs::write(&replacement, b"same-version-A").expect("replacement");
        fs::File::open(&replacement)
            .expect("file")
            .set_modified(modified)
            .expect("restore mtime");
        fs::rename(&replacement, &path).expect("replace interpreter");
        let second = fingerprint(&mut cache, &path);
        assert_ne!(
            first, second,
            "identical bytes still replace interpreter identity"
        );
        fs::write(&path, b"same-version-B").expect("in-place edit");
        fs::File::open(&path)
            .expect("file")
            .set_modified(modified)
            .expect("restore mtime");
        assert_ne!(fingerprint(&mut cache, &path), second);
        assert_eq!(cache.reads, 3);
    }

    #[test]
    fn symlink_target_deletion_and_invalid_file_are_not_cached() {
        let temp = tempfile::tempdir().expect("temporary interpreter");
        let target = temp.path().join("target");
        let link = temp.path().join("python");
        fs::write(&target, b"version-A").expect("target");
        symlink(&target, &link).expect("link");
        let mut cache = InterpreterSnapshot::default();
        let first = fingerprint(&mut cache, &link);
        fs::write(&target, b"version-B").expect("target edit");
        assert_ne!(fingerprint(&mut cache, &link), first);
        let other = temp.path().join("other");
        fs::write(&other, b"version-B").expect("same-version target");
        let before_retarget = fingerprint(&mut cache, &link);
        fs::remove_file(&link).expect("remove link");
        symlink(&other, &link).expect("retarget link");
        assert_ne!(fingerprint(&mut cache, &link), before_retarget);
        fs::remove_file(&other).expect("delete selected target");
        fs::remove_file(&target).expect("delete target");
        assert_ne!(fingerprint(&mut cache, &link), first);
        assert!(cache.cached.is_none());
        fs::create_dir(&other).expect("invalid target");
        assert!(!fingerprint(&mut cache, &link).1);
        assert!(cache.cached.is_none());
    }

    #[test]
    fn populated_interpreter_460_observations_read_contents_once() {
        let temp = tempfile::tempdir().expect("temporary interpreter");
        let path = temp.path().join("python");
        fs::write(&path, vec![42; 32_207_448]).expect("populated interpreter");
        let mut cache = InterpreterSnapshot::default();
        let first = fingerprint(&mut cache, &path);
        let start = std::time::Instant::now();
        for _ in 0..460 {
            assert_eq!(fingerprint(&mut cache, &path), first);
        }
        eprintln!("460 warm interpreter observations: {:?}", start.elapsed());
        assert_eq!(
            cache.reads, 1,
            "warm observations must not reread the executable"
        );
    }
}
