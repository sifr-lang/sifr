//! Resolve changed dependency inputs without renaming identical native projects.
//! Cargo still validates every build; this receipt only owns lock preparation.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io;
use std::time::Duration;

#[derive(Debug)]
pub(super) struct Failure {
    pub(super) message: String,
    pub(super) infrastructure: bool,
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self {
            message: format!("native resolution preparation: {error}"),
            infrastructure: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Receipt {
    inputs: String,
}

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn read_owned(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {
            sifr_cache_storage::check_owned(path)?;
            std::fs::read(path).map(Some)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) fn prepare(
    lease: &native_cache::NativeCache,
    group: &BatchGroup,
    root: &Path,
    deadline: Duration,
) -> Result<(), Failure> {
    let started = Instant::now();
    let inputs = group.fingerprint.signature();
    let receipt_path = root.join("resolution.json");
    let previous_receipt = read_owned(&receipt_path)?;
    let previous_lock = read_owned(&root.join("Cargo.lock"))?;
    if let Some(bytes) = &previous_receipt {
        let receipt: Receipt = serde_json::from_slice(bytes).map_err(io::Error::other)?;
        // Cargo owns lock validation and can legitimately update it after a path
        // dependency manifest changes. This receipt does not authenticate a lock
        // or authorize skipping Cargo; it records fresh-resolution input ownership.
        if receipt.inputs == inputs && previous_lock.is_some() {
            return Ok(());
        }
    }

    // Preserve observed prior inputs before replacing them. A failed or interrupted
    // preparation is not evidence that either the old or new lock is current.
    if previous_receipt.is_some() || previous_lock.is_some() {
        let identity =
            serde_json::to_vec(&(&previous_receipt, &previous_lock)).map_err(io::Error::other)?;
        let history = root.join("resolution-history").join(digest(&identity));
        sifr_cache_storage::directory(&history)?;
        if let Some(bytes) = &previous_receipt {
            native_cache::write_changed(&history.join("resolution.json"), bytes)?;
        }
        if let Some(bytes) = &previous_lock {
            native_cache::write_changed(&history.join("Cargo.lock"), bytes)?;
        }
    }

    // A new staging project has no lock, matching fresh project resolution.
    // Its path is retained on failure and cannot publish a partial preparation.
    let mut builder = tempfile::Builder::new();
    builder.prefix("resolution-");
    #[cfg(unix)]
    builder
        .permissions(<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700));
    let staging = builder.tempdir_in(root)?.keep();
    sifr_cache_storage::directory(&staging.join("src"))?;
    for relative in ["Cargo.toml", "src/main.rs"] {
        let bytes = read_owned(&root.join(relative))?
            .ok_or_else(|| io::Error::other(format!("missing native input: {relative}")))?;
        native_cache::write_changed(&staging.join(relative), &bytes)?;
    }
    native_cache::write_changed(&staging.join("inputs.txt"), inputs.as_bytes())?;
    let remaining = deadline.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Err(io::Error::other("native resolution preparation deadline exhausted").into());
    }
    let mut command = Command::new("cargo");
    command
        .args(sifr_driver::sysroot_cargo_config_args(
            &group.dependency_plan,
        ))
        .arg("generate-lockfile")
        .current_dir(&staging);
    lease.bind_cargo_command(&mut command)?;
    let capture = run_capture_bounded(command, remaining);
    native_cache::write_changed(&staging.join("stdout.log"), capture.stdout.as_bytes())?;
    native_cache::write_changed(&staging.join("stderr.log"), capture.stderr.as_bytes())?;
    if !capture.status_ok {
        return Err(Failure {
            message: format!(
                "native dependency resolution failed; retained {}",
                staging.display()
            ),
            infrastructure: !matches!(capture.exit_code, Some(101)),
        });
    }
    let lock = read_owned(&staging.join("Cargo.lock"))?
        .ok_or_else(|| io::Error::other("Cargo resolution produced no lockfile"))?;
    let receipt = serde_json::to_vec(&Receipt { inputs }).map_err(io::Error::other)?;
    native_cache::write_changed(&root.join("Cargo.lock"), &lock)?;
    native_cache::write_changed(&receipt_path, &receipt)?;
    // Only complete successful preparation removes its own staging evidence.
    std::fs::remove_dir_all(staging)?;
    Ok(())
}
