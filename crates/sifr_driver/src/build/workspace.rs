use crate::diagnostics::RenderedDiagnostic;
use serde::{Deserialize, Serialize};
use sifr_diagnostics::DiagnosticCode;
use std::path::{Path, PathBuf};

const ARTIFACT_CACHE_SCHEMA_VERSION: u32 = 3;
const ARTIFACT_CACHE_METADATA_FILE: &str = "artifact_cache.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArtifactCacheReport {
    namespace: String,
    key: String,
    workspace_root: PathBuf,
    cache_hit: bool,
    miss_reason: Option<String>,
}

impl ArtifactCacheReport {
    pub(crate) const fn cache_hit(&self) -> bool {
        self.cache_hit
    }

    #[cfg(test)]
    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    #[cfg(test)]
    pub(crate) fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub(crate) fn status_line(&self) -> String {
        let mut line = format!(
            "[sifr-artifact-cache] namespace={} key={} cache_hit={} workspace={}",
            self.namespace,
            self.key,
            self.cache_hit,
            self.workspace_root.display()
        );
        if let Some(reason) = &self.miss_reason {
            line.push_str(" miss_reason=");
            line.push_str(reason);
        }
        line
    }
}

pub(crate) struct CachedArtifactEntry {
    _lease: std::sync::Arc<std::fs::File>,
    workspace_root: PathBuf,
    report: ArtifactCacheReport,
}

impl CachedArtifactEntry {
    pub(crate) fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub(crate) fn report(&self) -> &ArtifactCacheReport {
        &self.report
    }
}

pub(crate) struct PendingCachedArtifact {
    lease: std::sync::Arc<std::fs::File>,
    scope: PathBuf,
    native_identity: String,
    final_root: PathBuf,
    staging_root: PathBuf,
    report: ArtifactCacheReport,
}

impl PendingCachedArtifact {
    pub(crate) fn workspace_root(&self) -> &Path {
        &self.staging_root
    }

    pub(crate) fn commit(
        self,
        required_paths: &[&Path],
    ) -> Result<CachedArtifactEntry, Vec<RenderedDiagnostic>> {
        crate::cache_storage::seal(&self.staging_root).map_err(storage_error)?;
        for required_path in required_paths {
            let absolute = self.staging_root.join(required_path);
            if crate::cache_storage::payload(&self.staging_root, required_path).is_err() {
                return Err(vec![crate::diagnostics::diagnostic_with_code(
                    format!(
                        "generated artifact cache staging directory is missing required path '{}'",
                        absolute.display()
                    ),
                    DiagnosticCode::BUILD_ARTIFACT_MISSING,
                )]);
            }
        }

        let metadata = ArtifactCacheMetadata {
            schema_version: ARTIFACT_CACHE_SCHEMA_VERSION,
            namespace: self.report.namespace.clone(),
            key: self.report.key.clone(),
            toolchain_signature: self.native_identity.clone(),
            scope: self.scope.clone(),
            required_paths: required_paths.iter().map(|p| p.to_path_buf()).collect(),
        };
        write_cache_metadata(&self.staging_root, &metadata)?;
        std::fs::File::open(self.staging_root.join(ARTIFACT_CACHE_METADATA_FILE))
            .and_then(|file| file.sync_all())
            .map_err(storage_error)?;
        for path in required_paths {
            let absolute = self.staging_root.join(path);
            if absolute.is_file() {
                std::fs::File::open(absolute)
                    .and_then(|file| file.sync_all())
                    .map_err(storage_error)?;
            }
        }
        crate::cache_storage::sync_stage(&self.staging_root).map_err(storage_error)?;

        match crate::cache_storage::publish(&self.staging_root, &self.final_root) {
            Ok(()) => {
                self.lease.lock_shared().map_err(storage_error)?;
                if !valid_entry(&self.final_root, &metadata, required_paths) {
                    return Err(storage_error(
                        "published cache entry disappeared during lock conversion",
                    ));
                }
                Ok(CachedArtifactEntry {
                    _lease: self.lease.clone(),
                    workspace_root: self.final_root.clone(),
                    report: ArtifactCacheReport {
                        workspace_root: self.final_root.clone(),
                        ..self.report.clone()
                    },
                })
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::DirectoryNotEmpty
                ) =>
            {
                if !valid_entry(&self.final_root, &metadata, required_paths) {
                    return Err(storage_error(
                        "concurrent cache winner is incomplete or incompatible",
                    ));
                }
                self.lease.lock_shared().map_err(storage_error)?;
                if !valid_entry(&self.final_root, &metadata, required_paths) {
                    return Err(storage_error(
                        "published cache entry disappeared during lock conversion",
                    ));
                }
                let _ = std::fs::remove_dir_all(&self.staging_root);
                Ok(CachedArtifactEntry {
                    _lease: self.lease.clone(),
                    workspace_root: self.final_root.clone(),
                    report: ArtifactCacheReport {
                        cache_hit: true,
                        miss_reason: Some("concurrent_populate".to_string()),
                        workspace_root: self.final_root.clone(),
                        ..self.report.clone()
                    },
                })
            }
            Err(error) => Err(vec![crate::diagnostics::diagnostic_with_code(
                format!(
                    "failed to promote generated artifact cache directory '{}' into '{}': {error}",
                    self.staging_root.display(),
                    self.final_root.display()
                ),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]),
        }
    }
}

impl Drop for PendingCachedArtifact {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.staging_root);
    }
}

pub(crate) enum PreparedArtifactCache {
    Hit(CachedArtifactEntry),
    Miss(PendingCachedArtifact),
}

pub(crate) fn prepare_cached_artifact(
    native_identity: &str,
    namespace: &str,
    scope: &Path,
    key_material: &str,
    required_paths: &[&Path],
) -> Result<PreparedArtifactCache, Vec<RenderedDiagnostic>> {
    let scope_path = scope.canonicalize().unwrap_or_else(|_| scope.to_path_buf());
    let mut key = sifr_identity::IdentityEncoder::new("generated-artifact-v2");
    key.field("schema", &ARTIFACT_CACHE_SCHEMA_VERSION.to_le_bytes());
    key.field("namespace", namespace.as_bytes());
    key.field("scope", scope_path.as_os_str().as_encoded_bytes());
    key.field("native-context", native_identity.as_bytes());
    key.field("material", key_material.as_bytes());
    let cache_key = key.finish();
    crate::cache_storage::relative(Path::new(namespace)).map_err(storage_error)?;
    for path in required_paths {
        crate::cache_storage::relative(path).map_err(storage_error)?;
    }
    let cache_root = artifact_cache_root().join(namespace);
    crate::cache_storage::directory(&cache_root).map_err(storage_error)?;
    let lease = std::sync::Arc::new(
        crate::cache_storage::entry_lock(&cache_root, &cache_key).map_err(storage_error)?,
    );
    // Readers lease immutable entries. A miss upgrades to exclusive ownership
    // and revalidates after acquisition before producing a new entry.
    lease.lock_shared().map_err(storage_error)?;

    let final_root = cache_root.join(&cache_key);
    let expected = ArtifactCacheMetadata {
        schema_version: ARTIFACT_CACHE_SCHEMA_VERSION,
        namespace: namespace.to_owned(),
        key: cache_key.clone(),
        toolchain_signature: native_identity.to_owned(),
        scope: crate::cache_storage::owner_scope().map_err(storage_error)?,
        required_paths: required_paths.iter().map(|p| p.to_path_buf()).collect(),
    };
    if !valid_entry(&final_root, &expected, required_paths) {
        lease.unlock().map_err(storage_error)?;
        lease.lock().map_err(storage_error)?;
        // Revalidation below sees a winner that completed during lock acquisition.
    }

    let mut miss_reason = Some("not_found".to_string());
    if std::fs::symlink_metadata(&final_root).is_ok() {
        crate::cache_storage::check_owned(&final_root).map_err(storage_error)?;
        match load_cache_metadata(&final_root) {
            Some(metadata)
                if metadata.schema_version == ARTIFACT_CACHE_SCHEMA_VERSION
                    && metadata.namespace == namespace
                    && metadata.key == cache_key
                    && metadata.toolchain_signature == native_identity =>
            {
                if metadata
                    .required_paths
                    .iter()
                    .all(|relative| crate::cache_storage::payload(&final_root, relative).is_ok())
                    && required_paths.iter().all(|relative| {
                        crate::cache_storage::payload(&final_root, relative).is_ok()
                    })
                {
                    lease.lock_shared().map_err(storage_error)?;
                    if !valid_entry(&final_root, &expected, required_paths) {
                        return Err(storage_error(
                            "cache entry disappeared during reader lock conversion",
                        ));
                    }
                    return Ok(PreparedArtifactCache::Hit(CachedArtifactEntry {
                        _lease: lease,
                        workspace_root: final_root.clone(),
                        report: ArtifactCacheReport {
                            namespace: namespace.to_string(),
                            key: cache_key,
                            workspace_root: final_root,
                            cache_hit: true,
                            miss_reason: None,
                        },
                    }));
                }
                miss_reason = Some("artifact_missing".to_string());
            }
            Some(_) => {
                miss_reason = Some("metadata_mismatch".to_string());
            }
            None => {
                miss_reason = Some("metadata_missing".to_string());
            }
        }
        std::fs::remove_dir_all(&final_root).map_err(storage_error)?;
    }

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging_root = cache_root.join(format!("{cache_key}.stage-{}-{nonce}", std::process::id()));
    crate::cache_storage::directory(&staging_root).map_err(storage_error)?;
    write_cache_metadata(
        &staging_root,
        &ArtifactCacheMetadata {
            schema_version: ARTIFACT_CACHE_SCHEMA_VERSION,
            namespace: namespace.to_owned(),
            key: cache_key.clone(),
            toolchain_signature: native_identity.to_owned(),
            scope: crate::cache_storage::owner_scope().map_err(storage_error)?,
            required_paths: required_paths.iter().map(|p| p.to_path_buf()).collect(),
        },
    )?;
    Ok(PreparedArtifactCache::Miss(PendingCachedArtifact {
        lease,
        scope: crate::cache_storage::owner_scope().map_err(storage_error)?,
        native_identity: native_identity.to_owned(),
        final_root,
        staging_root: staging_root.clone(),
        report: ArtifactCacheReport {
            namespace: namespace.to_string(),
            key: cache_key,
            workspace_root: staging_root,
            cache_hit: false,
            miss_reason,
        },
    }))
}

pub(super) fn artifact_cache_root() -> PathBuf {
    crate::cache_storage::root().join("native/artifacts")
}

#[derive(Deserialize, Serialize)]
struct ArtifactCacheMetadata {
    schema_version: u32,
    namespace: String,
    key: String,
    toolchain_signature: String,
    required_paths: Vec<PathBuf>,
    scope: PathBuf,
}

fn load_cache_metadata(workspace_root: &Path) -> Option<ArtifactCacheMetadata> {
    let metadata_path = workspace_root.join(ARTIFACT_CACHE_METADATA_FILE);
    crate::cache_storage::payload(workspace_root, Path::new(ARTIFACT_CACHE_METADATA_FILE)).ok()?;
    let raw = std::fs::read_to_string(metadata_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_cache_metadata(
    workspace_root: &Path,
    metadata: &ArtifactCacheMetadata,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let content = serde_json::to_string_pretty(metadata).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("failed to serialize generated artifact cache metadata: {error}"),
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })?;
    super::native_storage::write_changed(
        &workspace_root.join(ARTIFACT_CACHE_METADATA_FILE),
        content.as_bytes(),
    )
    .map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("failed to write generated artifact cache metadata: {error}"),
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })
}

fn storage_error(error: impl std::fmt::Display) -> Vec<RenderedDiagnostic> {
    vec![crate::diagnostics::diagnostic_with_code(
        format!("generated cache storage: {error}; select a private absolute SIFR_CACHE_DIR"),
        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
    )]
}

fn valid_entry(root: &Path, expected: &ArtifactCacheMetadata, required: &[&Path]) -> bool {
    let Some(actual) = load_cache_metadata(root) else {
        return false;
    };
    actual.schema_version == expected.schema_version
        && actual.namespace == expected.namespace
        && actual.key == expected.key
        && actual.toolchain_signature == expected.toolchain_signature
        && actual
            .required_paths
            .iter()
            .all(|p| crate::cache_storage::payload(root, p).is_ok())
        && required
            .iter()
            .all(|p| crate::cache_storage::payload(root, p).is_ok())
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) fn create_invocation_workspace(
    prefix: &str,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let base_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir();
    for attempt in 0..8u8 {
        let unique = if attempt == 0 {
            format!("sifr_{}_{}_{}", prefix, std::process::id(), base_nanos)
        } else {
            format!(
                "sifr_{}_{}_{}_{}",
                prefix,
                std::process::id(),
                base_nanos,
                attempt
            )
        };
        let workspace = root.join(unique);
        match std::fs::create_dir(&workspace) {
            Ok(()) => return Ok(workspace),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                let workspace_display = workspace.display();
                return Err(vec![crate::diagnostics::diagnostic_with_code(
                    format!("failed to create invocation workspace '{workspace_display}': {error}"),
                    DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
                )]);
            }
        }
    }
    Err(vec![crate::diagnostics::diagnostic_with_code(
        format!("failed to allocate unique invocation workspace for prefix '{prefix}'"),
        DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
    )])
}
