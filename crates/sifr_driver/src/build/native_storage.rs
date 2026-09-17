//! Cargo owns freshness. The family lease covers generated-source mutation,
//! Cargo execution and capture, including same-named root output paths.
use std::fs::File;
use std::path::{Path, PathBuf};

pub(crate) struct NativeFamily {
    _lease: File,
    pub(crate) root: PathBuf,
}

impl NativeFamily {
    pub(crate) fn acquire(
        toolchain: &str,
        source_configuration: &str,
        environment: &str,
        trust: &str,
    ) -> std::io::Result<Self> {
        let mut id = sifr_identity::IdentityEncoder::new("native-family-v1");
        id.field("toolchain", toolchain.as_bytes());
        id.field("sources", source_configuration.as_bytes());
        id.field("environment", environment.as_bytes());
        id.field("trust", trust.as_bytes());
        id.field(
            "owner",
            crate::cache_storage::owner_scope()?
                .as_os_str()
                .as_encoded_bytes(),
        );
        let directory = crate::cache_storage::root().join("native/families");
        crate::cache_storage::directory(&directory)?;
        let key = id.finish();
        let lease = crate::cache_storage::entry_lock(&directory, &key)?;
        lease.lock()?;
        let root = directory.join(&key);
        crate::cache_storage::directory(&root)?;
        let metadata = serde_json::to_vec(&serde_json::json!({
            "schema": 1, "family": key,
            "owner_scope": crate::cache_storage::owner_scope()?,
            "native_toolchain": toolchain,
        }))?;
        write_changed(&root.join("native_family.json"), &metadata)?;
        Ok(Self {
            _lease: lease,
            root,
        })
    }

    pub(crate) fn project(&self, scope: &Path, name: &str) -> std::io::Result<PathBuf> {
        let mut id = sifr_identity::IdentityEncoder::new("native-editable-root-v1");
        id.field("scope", scope.as_os_str().as_encoded_bytes());
        id.field("name", name.as_bytes());
        let path = self.root.join("roots").join(id.finish());
        crate::cache_storage::directory(&path)?;
        Ok(path)
    }

    pub(crate) fn target(&self) -> PathBuf {
        self.root.join("target")
    }
}

pub(crate) fn write_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(());
    }
    std::fs::write(path, bytes)
}

/// Remove stale generated files after rewriting the current inventory. Retain
/// byte-identical files and their mtimes so Cargo can prove no-op freshness.
pub(super) fn remove_stale(
    root: &Path,
    current: &std::collections::BTreeSet<PathBuf>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            remove_stale(&entry.path(), current)?;
        } else if !current.contains(&entry.path()) {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// Inventory captured while holding the family lock. Keep side files beside
/// the independent executable, including Cargo's platform debug bundles.
pub(crate) struct NativeSnapshot {
    pub(crate) identity: String,
    files: Vec<(PathBuf, PathBuf)>,
}

impl NativeSnapshot {
    pub(crate) fn inspect(executable: &Path, destination: &Path) -> std::io::Result<Self> {
        let mut files = vec![(executable.to_path_buf(), destination.to_path_buf())];
        for (source, destination) in [
            (
                executable.with_extension("pdb"),
                destination.with_extension("pdb"),
            ),
            (
                PathBuf::from(format!("{}.dSYM", executable.display())),
                PathBuf::from(format!("{}.dSYM", destination.display())),
            ),
        ] {
            if source.exists() {
                collect_bundle(&source, &destination, &mut files)?;
            }
        }
        let mut identity = sifr_identity::IdentityEncoder::new("final-native-bundle-v1");
        for (source, destination) in &files {
            identity.field("path", destination.as_os_str().as_encoded_bytes());
            identity.field("bytes", &std::fs::read(source)?);
        }
        Ok(Self {
            identity: identity.finish(),
            files,
        })
    }

    pub(crate) fn required(&self) -> Vec<&Path> {
        self.files
            .iter()
            .map(|(_, destination)| destination.as_path())
            .collect()
    }

    pub(crate) fn verify(&self, root: &Path) -> std::io::Result<()> {
        for (source, relative) in &self.files {
            if std::fs::read(source)? != std::fs::read(root.join(relative))? {
                return Err(std::io::Error::other(
                    "finalized native bundle failed content validation",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn capture(&self, stage: &Path) -> std::io::Result<()> {
        for (source, relative) in &self.files {
            let destination = stage.join(relative);
            if let Some(parent) = destination.parent() {
                crate::cache_storage::directory(parent)?;
            }
            std::fs::copy(source, destination)?;
        }
        Ok(())
    }
}

fn collect_bundle(
    source: &Path,
    destination: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::other(
            "native output bundle contains a symlink",
        ));
    }
    if metadata.is_dir() {
        let mut entries = std::fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            collect_bundle(&entry.path(), &destination.join(entry.file_name()), files)?;
        }
    } else if metadata.is_file() {
        files.push((source.to_path_buf(), destination.to_path_buf()));
    } else {
        return Err(std::io::Error::other(
            "native output bundle contains a special file",
        ));
    }
    Ok(())
}
