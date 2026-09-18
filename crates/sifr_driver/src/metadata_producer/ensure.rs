use super::{Result, production::Inputs, wire};
use sifr_identity::CompilerIdentity;
use std::os::unix::fs::OpenOptionsExt;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub struct PreparedMetadata {
    pub path: PathBuf,
    pub metadata_id: String,
    pub compatibility: wire::Compatibility,
    pub production_seconds: Option<f64>,
    pub load_timings: Option<serde_json::Value>,
    pub store: wire::MetadataStore,
    _lease: File,
}
static SUCCESSES: OnceLock<Mutex<BTreeMap<PathBuf, Arc<PreparedMetadata>>>> = OnceLock::new();
fn fail(error: impl std::fmt::Display) -> wire::MetadataError {
    wire::MetadataError(error.to_string())
}
fn remember(prepared: Arc<PreparedMetadata>) -> Result<Arc<PreparedMetadata>> {
    let mut successes = SUCCESSES
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| fail("metadata success registry poisoned"))?;
    if let Some(existing) = successes.get(&prepared.path) {
        if existing.metadata_id == prepared.metadata_id {
            return Ok(existing.clone());
        }
    }
    successes.insert(prepared.path.clone(), prepared.clone());
    Ok(prepared)
}
fn cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(fail("metadata preparation cancelled"))
    } else {
        Ok(())
    }
}
fn validate(
    path: &Path,
    expected: wire::Compatibility,
    lease: File,
) -> Result<Arc<PreparedMetadata>> {
    let bytes = read_bounded(path)?;
    let metadata_id = sifr_sysroot::sha256_hex(&bytes);
    let store = wire::MetadataStore::open_bytes(bytes, expected, wire::Limits::default())?;
    store.validate_complete()?;
    store.release_unpinned()?;
    Ok(Arc::new(PreparedMetadata {
        path: path.to_owned(),
        metadata_id,
        compatibility: expected,
        production_seconds: None,
        load_timings: None,
        store,
        _lease: lease,
    }))
}
/// Shared write-through entrypoint for CLI development, linked tests and packaging.
/// Only successful immutable owners are memoized. Every failed attempt releases its OS lock.
pub fn ensure_development_metadata(
    identity: &CompilerIdentity,
    source_root: &Path,
    target: &str,
    cache: &Path,
    cancel: &AtomicBool,
) -> Result<Arc<PreparedMetadata>> {
    ensure_with_hook(identity, source_root, target, cache, cancel, |_| Ok(()))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    Captured,
    Owned,
    Waiting,
    Staged,
    Published,
}
pub(super) fn ensure_with_hook(
    identity: &CompilerIdentity,
    source_root: &Path,
    target: &str,
    cache: &Path,
    cancel: &AtomicBool,
    hook: impl Fn(Stage) -> Result<()>,
) -> Result<Arc<PreparedMetadata>> {
    let inputs = Inputs::capture(identity, source_root, target)?;
    hook(Stage::Captured)?;
    let key = sifr_sysroot::sha256_hex(
        &[
            inputs.compatibility.compiler,
            inputs.compatibility.semantic_target,
            inputs.compatibility.stdlib_inputs,
        ]
        .concat(),
    );
    let root = cache.join("metadata");
    crate::cache_storage::directory(&root).map_err(fail)?;
    let path = root.join(format!("{key}.sifrmeta"));
    cancelled(cancel)?;
    if path.exists() {
        crate::cache_storage::payload(
            &root,
            path.file_name()
                .map(Path::new)
                .ok_or_else(|| fail("missing metadata entry name"))?,
        )
        .map_err(fail)?;
    }
    if let Some(prepared) = SUCCESSES
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| fail("metadata success registry poisoned"))?
        .get(&path)
        .cloned()
    {
        if read_bounded(&path)
            .is_ok_and(|bytes| sifr_sysroot::sha256_hex(&bytes) == prepared.metadata_id)
        {
            return Ok(prepared);
        }
    }
    let lock = crate::cache_storage::entry_lock(&root, &key).map_err(fail)?;
    if path.is_file() {
        if let Ok(prepared) = validate(&path, inputs.compatibility, lock.try_clone().map_err(fail)?)
        {
            return remember(prepared);
        }
    }
    loop {
        cancelled(cancel)?;
        match lock.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => {
                hook(Stage::Waiting)?;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(fail(error)),
        }
    }
    // Recheck after ownership; a waiter observes the winner's complete output.
    if let Some(prepared) = SUCCESSES
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| fail("metadata success registry poisoned"))?
        .get(&path)
        .cloned()
    {
        if read_bounded(&path)
            .is_ok_and(|bytes| sifr_sysroot::sha256_hex(&bytes) == prepared.metadata_id)
        {
            return Ok(prepared);
        }
    }
    if path.is_file() {
        if let Ok(prepared) = validate(&path, inputs.compatibility, lock.try_clone().map_err(fail)?)
        {
            lock.unlock().map_err(fail)?;
            return remember(prepared);
        }
    }
    hook(Stage::Owned)?;
    let started = Instant::now();
    let bytes = inputs.produce()?;
    cancelled(cancel)?;
    let current = Inputs::capture(identity, source_root, target)?;
    if current.compatibility != inputs.compatibility {
        return Err(fail(
            "stdlib inputs changed during metadata production; retry preparation",
        ));
    }
    let store = wire::MetadataStore::open(
        std::io::Cursor::new(bytes.clone()),
        inputs.compatibility,
        wire::Limits::default(),
    )?;
    store.validate_complete()?;
    let staging = root.join(format!("{key}.stage"));
    // Exclusive per-key ownership makes an abandoned stage safe to replace.
    if staging.exists() {
        fs::remove_file(&staging).map_err(fail)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&staging)
        .map_err(fail)?;
    file.write_all(&bytes).map_err(fail)?;
    file.sync_all().map_err(fail)?;
    cancelled(cancel)?;
    hook(Stage::Staged)?;
    if Inputs::capture(identity, source_root, target)?.compatibility != inputs.compatibility {
        return Err(fail(
            "stdlib inputs changed during metadata production; retry preparation",
        ));
    }
    cancelled(cancel)?;
    fs::rename(&staging, &path).map_err(fail)?;
    File::open(&root).and_then(|f| f.sync_all()).map_err(fail)?;
    hook(Stage::Published)?;
    lock.unlock().map_err(fail)?;
    let prepared = Arc::new(PreparedMetadata {
        path: path.clone(),
        metadata_id: sifr_sysroot::sha256_hex(&bytes),
        compatibility: inputs.compatibility,
        production_seconds: Some(started.elapsed().as_secs_f64()),
        load_timings: None,
        store,
        _lease: lock,
    });
    remember(prepared)
}
impl PreparedMetadata {
    /// Publish a validated immutable container beside the explicit destination.
    pub fn publish_output(&self, output: &Path) -> Result<()> {
        static NEXT_OUTPUT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent=parent.canonicalize().map_err(|error|fail(format!("metadata output parent {} is unavailable: {error}; select an existing writable directory",parent.display())))?;
        let name = output
            .file_name()
            .ok_or_else(|| fail("metadata output must name a file"))?;
        let output = parent.join(name);
        if output == self.path {
            return Ok(());
        }
        let stage = parent.join(format!(
            ".sifrmeta-{}-{}-{}",
            std::process::id(),
            NEXT_OUTPUT.fetch_add(1, Ordering::Relaxed),
            self.metadata_id
        ));
        let mut owns_stage = false;
        let result = (|| {
            let bytes = read_bounded(&self.path)?;
            if sifr_sysroot::sha256_hex(&bytes) != self.metadata_id {
                return Err(fail(
                    "prepared metadata changed before output publication; retry preparation",
                ));
            }
            let mut file=OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&stage).map_err(|error|fail(format!("cannot create metadata output in {}: {error}; select a writable output directory",parent.display())))?;
            owns_stage = true;
            file.write_all(&bytes).map_err(fail)?;
            file.sync_all().map_err(fail)?;
            fs::rename(&stage, &output).map_err(fail)?;
            File::open(&parent)
                .and_then(|f| f.sync_all())
                .map_err(fail)?;
            Ok(())
        })();
        if result.is_err() && owns_stage {
            let _ = fs::remove_file(stage);
        }
        result
    }
}

/// An explicit test/development override is validated exactly; failure never substitutes another artifact.
pub fn validate_development_metadata(
    identity: &CompilerIdentity,
    source_root: &Path,
    target: &str,
    path: &Path,
) -> Result<Arc<PreparedMetadata>> {
    let inputs = Inputs::capture(identity, source_root, target)?;
    if let Some(prepared) = SUCCESSES
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| fail("metadata success registry poisoned"))?
        .get(path)
        .cloned()
    {
        if prepared.compatibility == inputs.compatibility
            && read_bounded(path)
                .is_ok_and(|bytes| sifr_sysroot::sha256_hex(&bytes) == prepared.metadata_id)
        {
            return Ok(prepared);
        }
    }
    remember(validate(
        path,
        inputs.compatibility,
        File::open(path).map_err(fail)?,
    )?)
}

/// Read at most the wire limit plus one byte, including a concurrent growth race.
pub(crate) fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path).map_err(fail)?;
    let limit = wire::Limits::default().file_bytes;
    if file.metadata().map_err(fail)?.len() > limit {
        return Err(fail("metadata file exceeds bounded container limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).map_err(fail)?;
    if bytes.len() as u64 > limit {
        return Err(fail("metadata file grew beyond bounded container limit"));
    }
    Ok(bytes)
}
/// Open a consumer owner without traversal of unrelated payloads.
pub(crate) fn open_consumer(
    path: &Path,
    compatibility: wire::Compatibility,
    expected_id: Option<&str>,
) -> Result<Arc<PreparedMetadata>> {
    let started = Instant::now();
    let lease = File::open(path).map_err(fail)?;
    let bytes = read_bounded(path)?;
    let read_us = started.elapsed().as_micros();
    let hashing = Instant::now();
    let metadata_id = sifr_sysroot::sha256_hex(&bytes);
    let hash_us = hashing.elapsed().as_micros();
    if expected_id.is_some_and(|id| id != metadata_id) {
        return Err(fail(
            "installed metadata content identity mismatch; reinstall this toolchain",
        ));
    }
    let indexing = Instant::now();
    let input_bytes = bytes.len();
    let store = wire::MetadataStore::open_bytes(bytes, compatibility, wire::Limits::default())?;
    Ok(Arc::new(PreparedMetadata {
        path: path.to_owned(),
        metadata_id,
        compatibility,
        production_seconds: None,
        load_timings: Some(serde_json::json!({
            "read_us": read_us, "hash_us": hash_us,
            "index_us": indexing.elapsed().as_micros().saturating_sub(store.physical_decode_us()),
            "physical_decode_us": store.physical_decode_us(), "input_bytes": input_bytes,
            "total_us": started.elapsed().as_micros()
        })),
        store,
        _lease: lease,
    }))
}

/// Read-only doctor selection; a missing or stale development artifact is not produced.
pub fn development_metadata_path(
    identity: &CompilerIdentity,
    source_root: &Path,
    target: &str,
    cache: &Path,
) -> Result<PathBuf> {
    let inputs = Inputs::capture(identity, source_root, target)?;
    let key = sifr_sysroot::sha256_hex(
        &[
            inputs.compatibility.compiler,
            inputs.compatibility.semantic_target,
            inputs.compatibility.stdlib_inputs,
        ]
        .concat(),
    );
    Ok(cache.join("metadata").join(format!("{key}.sifrmeta")))
}
