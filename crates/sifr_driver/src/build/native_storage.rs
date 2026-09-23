//! Cargo owns freshness. The family lease covers generated-source mutation,
//! Cargo execution and capture, including same-named root output paths.
use std::fs::File;
use std::io::{Read, Write};
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

/// Caller-owned output roots can be shared by otherwise incompatible families.
/// Serialize their mutation and publication independently of Cargo context.
pub(crate) fn publication_lock(path: &Path) -> std::io::Result<File> {
    #[cfg(windows)]
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    #[cfg(windows)]
    crate::windows_storage_security::no_reparse(&path)?;
    std::fs::create_dir_all(&path)?;
    #[cfg(windows)]
    crate::windows_storage_security::no_reparse(&path)?;
    let path = path.canonicalize()?;
    let mut id = sifr_identity::IdentityEncoder::new("native-publication-v1");
    id.field("path", path.as_os_str().as_encoded_bytes());
    let directory = crate::cache_storage::root().join("native/publications");
    crate::cache_storage::directory(&directory)?;
    let lease = crate::cache_storage::entry_lock(&directory, &id.finish())?;
    lease.lock()?;
    Ok(lease)
}

pub(crate) fn write_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = match crate::cache_storage::read_write_private_file(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            crate::cache_storage::new_private_file(path)?
        }
        Err(error) => return Err(error),
    };
    let mut current = Vec::new();
    file.read_to_end(&mut current)?;
    if current == bytes {
        return Ok(());
    }
    file.set_len(0)?;
    std::io::Seek::rewind(&mut file)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Remove stale generated files after rewriting the current inventory. Retain
/// byte-identical files and their mtimes so Cargo can prove no-op freshness.
pub(crate) fn remove_stale(
    root: &Path,
    current: &std::collections::BTreeSet<PathBuf>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        crate::cache_storage::check_owned(&entry.path())?;
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
        Self::from_files(files)
    }

    fn from_files(files: Vec<(PathBuf, PathBuf)>) -> std::io::Result<Self> {
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

    pub(crate) fn with_runtime(
        self,
        libraries: &[PathBuf],
        destination: &Path,
    ) -> std::io::Result<Self> {
        let mut files = self.files;
        for source in libraries {
            let name = source
                .file_name()
                .ok_or_else(|| std::io::Error::other("runtime library has no filename"))?;
            let target = destination.parent().unwrap_or(Path::new("")).join(name);
            if let Some((previous, _)) = files.iter().find(|(_, path)| path == &target) {
                if std::fs::read(previous)? != std::fs::read(source)? {
                    return Err(std::io::Error::other(
                        "conflicting native runtime library basenames",
                    ));
                }
            } else {
                files.push((source.clone(), target));
            }
        }
        Self::from_files(files)
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
                std::fs::create_dir_all(parent)?;
            }
            if std::fs::read(&destination).ok().as_deref()
                != Some(std::fs::read(source)?.as_slice())
            {
                std::fs::copy(source, destination)?;
            }
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
    #[cfg(windows)]
    crate::windows_storage_security::no_reparse(source)?;
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

/// Runtime libraries produced inside Cargo storage must travel with the root.
/// System/external library search paths retain their explicit environment
/// contract; proc-macro shared objects are compiler inputs, not runtime files.
pub(crate) fn runtime_libraries(stdout: &[u8], target: &Path) -> std::io::Result<Vec<PathBuf>> {
    let target = target.canonicalize()?;
    let mut libraries = std::collections::BTreeSet::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if event["reason"] == "build-script-executed" {
            for path in event["linked_paths"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
            {
                let path = Path::new(path.split_once('=').map_or(path, |(_, path)| path));
                if !path.starts_with(&target) || !path.is_dir() {
                    continue;
                }
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    if is_runtime_library(&entry.path()) {
                        let canonical = entry.path().canonicalize()?;
                        if !canonical.starts_with(&target) {
                            return Err(std::io::Error::other(
                                "native runtime library escapes Cargo storage",
                            ));
                        }
                        libraries.insert(entry.path());
                    }
                }
            }
        }
        if event["reason"] == "compiler-artifact"
            && event["target"]["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "dylib" || kind == "cdylib"))
        {
            for path in event["filenames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
            {
                let path = PathBuf::from(path);
                if path.starts_with(&target) && is_runtime_library(&path) {
                    libraries.insert(path);
                }
            }
        }
    }
    Ok(libraries.into_iter().collect())
}

fn is_runtime_library(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            path.extension().is_some_and(|extension| {
                extension.eq_ignore_ascii_case("dylib") || extension.eq_ignore_ascii_case("so")
            }) || name.contains(".so.")
        })
}

pub(crate) fn loader_build_script(python: Option<String>) -> String {
    let source = python.unwrap_or_else(|| "fn main() {\n}\n".to_owned());
    source.replacen(
        "fn main() {",
        r#"fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path"),
        Ok("linux") => println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN"),
        _ => {}
    }
"#,
        1,
    )
}
