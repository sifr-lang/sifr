use super::{Result, wire};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
    time::Instant,
};
fn fail(error: impl std::fmt::Display) -> wire::MetadataError {
    wire::MetadataError(error.to_string())
}
pub struct PreparedMetadata {
    pub path: PathBuf,
    pub metadata_id: String,
    pub compatibility: wire::Compatibility,
    pub production_seconds: Option<f64>,
    pub load_timings: Option<serde_json::Value>,
    pub store: wire::MetadataStore,
    _lease: File,
}
impl PreparedMetadata {
    /// Accept only a structurally complete store; retain the producer's lease.
    pub fn from_complete_bytes(
        path: PathBuf,
        bytes: Vec<u8>,
        compatibility: wire::Compatibility,
        lease: File,
        production_seconds: Option<f64>,
    ) -> Result<Arc<Self>> {
        let metadata_id = sifr_sysroot::sha256_hex(&bytes);
        let store = wire::MetadataStore::open_bytes(bytes, compatibility, wire::Limits::default())?;
        store.validate_complete()?;
        Ok(Arc::new(Self {
            path,
            metadata_id,
            compatibility,
            production_seconds,
            load_timings: None,
            store,
            _lease: lease,
        }))
    }
}
impl PreparedMetadata {
    /// Publish a validated immutable container beside the explicit destination.
    pub fn publish_output(&self, output: &Path) -> Result<()> {
        static NEXT_OUTPUT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        #[cfg(windows)]
        let parent = {
            let requested = if parent == Path::new(".") {
                std::env::current_dir().map_err(fail)?
            } else if parent.is_absolute() {
                parent.to_path_buf()
            } else {
                std::env::current_dir().map_err(fail)?.join(parent)
            };
            sifr_cache_storage::windows_storage_security::no_reparse(&requested).map_err(fail)?;
            requested
        };
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
            let mut file=sifr_cache_storage::new_private_file(&stage).map_err(|error|fail(format!("cannot create metadata output in {}: {error}; select a writable output directory",parent.display())))?;
            owns_stage = true;
            file.write_all(&bytes).map_err(fail)?;
            file.sync_all().map_err(fail)?;
            sifr_cache_storage::publish(&stage, &output).map_err(fail)?;
            Ok(())
        })();
        if result.is_err() && owns_stage {
            let _ = fs::remove_file(stage);
        }
        result
    }
}

/// Read at most the wire limit plus one byte, including a concurrent growth race.
pub fn read_bounded(path: &Path) -> Result<Vec<u8>> {
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
pub fn open_consumer(
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
