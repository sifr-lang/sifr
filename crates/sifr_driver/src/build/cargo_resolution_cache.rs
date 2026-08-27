use super::cargo_resolution::{PREPARED_LOCK_NONCE, cargo_resolution_error};
use super::rust_interop_digest::{CacheIdentity, digest_file};
use crate::diagnostics::RenderedDiagnostic;
use std::path::Path;
use std::sync::atomic::Ordering;

const PREPARED_LOCK_METADATA_FILE: &str = "cache_identity.json";

pub(super) fn cache_prepared_lock(
    source_lock: &Path,
    prepared_lock: &Path,
    identity: &CacheIdentity,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if prepared_lock_is_valid(prepared_lock, identity)? {
        return Ok(());
    }
    let nonce = PREPARED_LOCK_NONCE.fetch_add(1, Ordering::Relaxed);
    let Some(final_root) = prepared_lock.parent() else {
        return Err(vec![cargo_resolution_error(
            "prepared Cargo lock cache path has no parent",
        )]);
    };
    let temporary_root = final_root.with_extension(format!("tmp-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&temporary_root).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to create prepared Cargo lock staging directory '{}': {error}",
            temporary_root.display()
        ))]
    })?;
    let temporary_lock = temporary_root.join("Cargo.lock");
    if let Err(diagnostics) = stage_prepared_lock(source_lock, &temporary_lock, identity) {
        let _ = std::fs::remove_dir_all(&temporary_root);
        return Err(diagnostics);
    }
    if let Err(error) = std::fs::rename(&temporary_root, final_root) {
        let _ = std::fs::remove_dir_all(&temporary_root);
        if !prepared_lock_is_valid(prepared_lock, identity)? {
            return Err(vec![cargo_resolution_error(format!(
                "failed to publish prepared Cargo lockfile '{}': {error}",
                prepared_lock.display()
            ))]);
        }
    }
    Ok(())
}

fn stage_prepared_lock(
    source_lock: &Path,
    temporary_lock: &Path,
    identity: &CacheIdentity,
) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::copy(source_lock, temporary_lock).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to stage prepared Cargo lockfile '{}': {error}",
            temporary_lock.display()
        ))]
    })?;
    let lock_digest = digest_file(temporary_lock).ok_or_else(|| {
        vec![cargo_resolution_error(format!(
            "failed to digest prepared Cargo lock staging file '{}'",
            temporary_lock.display()
        ))]
    })?;
    let metadata = PreparedLockMetadata {
        schema_version: 2,
        identity: identity.clone(),
        lock_digest,
    };
    let metadata_raw = serde_json::to_vec_pretty(&metadata).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to serialize prepared Cargo lock cache identity: {error}"
        ))]
    })?;
    let metadata_path = temporary_lock
        .parent()
        .map(|root| root.join(PREPARED_LOCK_METADATA_FILE))
        .ok_or_else(|| {
            vec![cargo_resolution_error(
                "prepared lock staging path has no parent",
            )]
        })?;
    std::fs::write(metadata_path, metadata_raw).map_err(|error| {
        vec![cargo_resolution_error(format!(
            "failed to write prepared Cargo lock cache identity: {error}"
        ))]
    })
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PreparedLockMetadata {
    schema_version: u32,
    identity: CacheIdentity,
    lock_digest: String,
}

pub(super) fn prepared_lock_is_valid(
    prepared_lock: &Path,
    identity: &CacheIdentity,
) -> Result<bool, Vec<RenderedDiagnostic>> {
    if !prepared_lock.is_file() {
        if let Some(root) = prepared_lock.parent()
            && root.exists()
        {
            let _ = std::fs::remove_dir_all(root);
        }
        return Ok(false);
    }
    let metadata = prepared_lock
        .parent()
        .map(|root| root.join(PREPARED_LOCK_METADATA_FILE))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<PreparedLockMetadata>(&raw).ok());
    let Some(metadata) = metadata else {
        if let Some(root) = prepared_lock.parent() {
            let _ = std::fs::remove_dir_all(root);
        }
        return Ok(false);
    };
    if metadata.schema_version != 2 || !metadata.identity.matches(identity) {
        return Err(vec![cargo_resolution_error(format!(
            "prepared Cargo lock cache key collision at '{}'; stored full key does not match",
            prepared_lock.display()
        ))]);
    }
    if digest_file(prepared_lock).as_deref() == Some(metadata.lock_digest.as_str()) {
        return Ok(true);
    }
    if let Some(root) = prepared_lock.parent() {
        let _ = std::fs::remove_dir_all(root);
    }
    Ok(false)
}
