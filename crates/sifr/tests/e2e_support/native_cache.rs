//! Cargo validates every invocation; these receipts never authorize a build skip.
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub(crate) struct NativeCache {
    // Only explicitly bound native children inherit this lease on Unix.
    _lease: File,
    pub(crate) root: PathBuf,
}

impl NativeCache {
    pub(crate) fn bind_command(&self, command: &mut std::process::Command) -> io::Result<()> {
        #[cfg(unix)]
        sifr_cache_storage::inherit_lease_for_command(command, &self._lease)?;
        // Windows retains its existing inherited-handle and Job ownership policy.
        #[cfg(windows)]
        let _ = command;
        Ok(())
    }

    pub(crate) fn acquire(requested: &Path) -> io::Result<Self> {
        let absolute = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            std::env::current_dir()?.join(requested)
        };
        // A new namespace leaves legacy per-group targets and evidence untouched.
        let root = absolute.join("cargo-v3");
        sifr_cache_storage::directory(&root)?;
        #[cfg(unix)]
        let lease = sifr_cache_storage::open_entry_lock(&root, "suite", false)?;
        #[cfg(windows)]
        let lease = sifr_cache_storage::entry_lock(&root, "suite")?;
        lease.try_lock().map_err(|error| io::Error::other(format!(
            "cache is busy or cannot be locked ({}): {error}; choose a separate --cache-dir or wait for its owner", root.display()
        )))?;
        let owner = sifr_cache_storage::owner_scope()?;
        let owner_bytes = owner.as_os_str().as_encoded_bytes();
        let owner_path = root.join("owner");
        if owner_path.exists() {
            sifr_cache_storage::check_owned(&owner_path)?;
            if std::fs::read(&owner_path)? != owner_bytes {
                return Err(io::Error::other(
                    "native cache belongs to another source checkout",
                ));
            }
        } else {
            if std::fs::read_dir(&root)?
                .any(|entry| entry.is_ok_and(|entry| entry.file_name() != ".locks"))
            {
                return Err(io::Error::other(
                    "nonempty native cache has no owner record",
                ));
            }
            write_changed(&owner_path, owner_bytes)?;
        }
        Ok(Self {
            _lease: lease,
            root,
        })
    }
}

/// Compare bytes, not timestamps. Atomic replacement also makes interrupted
/// writes recoverable without truncating a previously complete generated file.
pub(crate) fn write_changed(path: &Path, bytes: &[u8]) -> io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {
            sifr_cache_storage::check_owned(path)?;
            if std::fs::read(path)? == bytes {
                return Ok(());
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing generated parent"))?;
    sifr_cache_storage::directory(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct NativeArtifact {
    pub(crate) path: PathBuf,
    pub(crate) fresh: bool,
}

/// Select exactly the requested package's binary from a completed Cargo stream.
/// Validate byte integrity on fresh output; Cargo freshness itself is timestamp
/// based and is not advertised as a content checksum for dependency sources.
pub(crate) fn select_artifact(
    status_ok: bool,
    output: &str,
    project: &Path,
    target: &Path,
    package: &str,
) -> Result<NativeArtifact, String> {
    if !status_ok {
        return Err("Cargo failed; no previous executable is eligible".into());
    }
    let manifest = project
        .join("Cargo.toml")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let target = target.canonicalize().map_err(|e| e.to_string())?;
    let mut selected = None;
    let mut finished = false;
    for line in output.lines() {
        let message: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("invalid Cargo JSON: {e}"))?;
        match message["reason"].as_str() {
            Some("build-finished") => finished = message["success"] == true,
            Some("compiler-artifact")
                if message["target"]["name"] == package
                    && message["target"]["kind"]
                        .as_array()
                        .is_some_and(|k| k.iter().any(|v| v == "bin"))
                    && message["profile"]["test"] == false =>
            {
                let actual_manifest = message["manifest_path"]
                    .as_str()
                    .map(Path::new)
                    .and_then(|p| p.canonicalize().ok());
                let expected_id = format!("{package}@0.1.0");
                if actual_manifest.as_ref() != Some(&manifest)
                    || message["package_id"]
                        .as_str()
                        .and_then(|id| id.rsplit('#').next())
                        != Some(&expected_id)
                {
                    return Err("Cargo artifact package/manifest identity mismatch".into());
                }
                let path = message["executable"]
                    .as_str()
                    .ok_or("Cargo artifact has no executable")?;
                let path = Path::new(path)
                    .canonicalize()
                    .map_err(|e| format!("missing executable: {e}"))?;
                if !path.starts_with(&target) || !path.is_file() {
                    return Err("Cargo executable is outside the owned target or not a file".into());
                }

                let fresh = message["fresh"]
                    .as_bool()
                    .ok_or("Cargo artifact lacks freshness status")?;
                if selected.replace(NativeArtifact { path, fresh }).is_some() {
                    return Err("multiple matching Cargo executables".into());
                }
            }
            _ => {}
        }
    }
    if !finished {
        return Err("Cargo did not report a successful completed build".into());
    }
    let artifact = selected.ok_or("Cargo did not report the expected executable")?;
    let path_id = hex_digest(&Sha256::digest(
        artifact.path.as_os_str().as_encoded_bytes(),
    ));
    let digest = artifact_digest(&artifact.path).map_err(|e| e.to_string())?;
    // Multiple feature/profile configurations can legitimately publish different
    // previously validated bytes at the same Cargo executable path.
    let receipt = project.join(format!("artifact-{path_id}-{digest}.sha256"));
    if artifact.fresh && std::fs::read(&receipt).ok().as_deref() != Some(digest.as_bytes()) {
        return Err(format!(
            "fresh artifact integrity mismatch or missing receipt: {}; clean this generated Cargo package in the owned native target and retry",
            artifact.path.display()
        ));
    }
    write_changed(&receipt, digest.as_bytes()).map_err(|e| e.to_string())?;
    Ok(artifact)
}

fn artifact_digest(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex_digest(&digest.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
struct RetirementContext {
    config: Vec<String>,
    profile: String,
    target: Option<String>,
}

/// Cargo removes the retired package's outputs, never its dependencies. Retain
/// each observed profile/target, but derive package and target-dir from the owned
/// group at retirement rather than accepting executable commands from a record.
pub(crate) fn record_retirement(
    project: &Path,
    target: &Path,
    config: &[String],
    artifact: &Path,
) -> io::Result<()> {
    let target = target.canonicalize()?;
    let relative = artifact.strip_prefix(&target).map_err(io::Error::other)?;
    let parts = relative.iter().collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return Err(io::Error::other("unexpected Cargo executable layout"));
    }
    let profile = match parts[parts.len() - 2].to_str() {
        Some("debug") => "dev",
        Some("release") => "release",
        _ => return Err(io::Error::other("unexpected generated application profile")),
    };
    let context = RetirementContext {
        config: config.to_vec(),
        profile: profile.into(),
        target: if parts.len() == 3 {
            Some(
                parts[0]
                    .to_str()
                    .ok_or_else(|| io::Error::other("non-UTF8 target"))?
                    .into(),
            )
        } else {
            None
        },
    };
    let path = project.join("retirement.json");
    let mut contexts: Vec<RetirementContext> = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };
    if !contexts.contains(&context) {
        contexts.push(context);
    }
    write_changed(&path, &serde_json::to_vec(&contexts)?)
}

pub(crate) fn retirement_commands(
    project: &Path,
    target: &Path,
    package: &str,
) -> io::Result<Vec<Vec<String>>> {
    sifr_cache_storage::check_owned(&project.join("retirement.json"))?;
    let contexts: Vec<RetirementContext> =
        serde_json::from_slice(&std::fs::read(project.join("retirement.json"))?)?;
    if contexts.is_empty() {
        return Err(io::Error::other("missing package retirement contexts"));
    }
    let target = target.canonicalize()?;
    contexts
        .into_iter()
        .map(|context| {
            let config = &context.config;
            let valid_config = config.is_empty()
                || (config.len() == 4
                    && config[0] == "--config"
                    && config[1] == "source.crates-io.replace-with=\"sifr-vendor\""
                    && config[2] == "--config"
                    && config[3].starts_with("source.sifr-vendor.directory="));
            if !valid_config || !matches!(context.profile.as_str(), "dev" | "release") {
                return Err(io::Error::other("invalid native retirement configuration"));
            }
            let mut args = context.config;
            args.extend([
                "clean".into(),
                "--frozen".into(),
                "--package".into(),
                package.into(),
                "--target-dir".into(),
                target.to_string_lossy().into_owned(),
                "--profile".into(),
                context.profile,
            ]);
            if let Some(triple) = context.target {
                if triple.is_empty()
                    || !triple
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
                {
                    return Err(io::Error::other("invalid native retirement target"));
                }
                args.extend(["--target".into(), triple]);
            }
            Ok(args)
        })
        .collect()
}
