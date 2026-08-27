use crate::diagnostics::RenderedDiagnostic;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use sifr_diagnostics::DiagnosticCode;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

const ARTIFACT_CACHE_SCHEMA_VERSION: u32 = 2;
const ARTIFACT_CACHE_ROOT_DIR: &str = "sifr_generated_artifact_cache";
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
    final_root: PathBuf,
    staging_root: PathBuf,
    key_material: String,
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
        for required_path in required_paths {
            let absolute = self.staging_root.join(required_path);
            if !absolute.exists() {
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
            key_material: self.key_material.clone(),
            toolchain_signature: toolchain_signature().to_string(),
        };
        write_cache_metadata(&self.staging_root, &metadata)?;

        match std::fs::rename(&self.staging_root, &self.final_root) {
            Ok(()) => Ok(CachedArtifactEntry {
                workspace_root: self.final_root.clone(),
                report: ArtifactCacheReport {
                    workspace_root: self.final_root.clone(),
                    ..self.report.clone()
                },
            }),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::DirectoryNotEmpty
                ) =>
            {
                let winner = load_cache_metadata(&self.final_root);
                if winner.as_ref().is_some_and(|metadata| {
                    metadata_matches(
                        metadata,
                        &self.report.namespace,
                        &self.report.key,
                        &self.key_material,
                    )
                }) && required_paths
                    .iter()
                    .all(|relative| self.final_root.join(relative).exists())
                {
                    let _ = std::fs::remove_dir_all(&self.staging_root);
                    Ok(CachedArtifactEntry {
                        workspace_root: self.final_root.clone(),
                        report: ArtifactCacheReport {
                            cache_hit: true,
                            miss_reason: Some("concurrent_populate".to_string()),
                            workspace_root: self.final_root.clone(),
                            ..self.report.clone()
                        },
                    })
                } else {
                    Err(vec![crate::diagnostics::diagnostic_with_code(
                        "generated artifact cache concurrent winner did not match the full cache key",
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )])
                }
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
    namespace: &str,
    scope: &Path,
    key_material: &str,
    required_paths: &[&Path],
) -> Result<PreparedArtifactCache, Vec<RenderedDiagnostic>> {
    let scope_path = scope.canonicalize().unwrap_or_else(|_| scope.to_path_buf());
    let full_key_material = artifact_key_material(namespace, &scope_path, key_material);
    let cache_key = deterministic_hash(&full_key_material);
    let cache_root = artifact_cache_root().join(namespace);
    std::fs::create_dir_all(&cache_root).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!(
                "failed to create generated artifact cache root '{}': {error}",
                cache_root.display()
            ),
            DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
        )]
    })?;

    let final_root = cache_root.join(&cache_key);
    let mut miss_reason = Some("not_found".to_string());
    if final_root.is_dir() {
        match load_cache_metadata(&final_root) {
            Some(metadata)
                if metadata_matches(&metadata, namespace, &cache_key, &full_key_material) =>
            {
                if required_paths
                    .iter()
                    .all(|relative| final_root.join(relative).exists())
                {
                    return Ok(PreparedArtifactCache::Hit(CachedArtifactEntry {
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
            Some(metadata) if metadata.schema_version == ARTIFACT_CACHE_SCHEMA_VERSION => {
                return Err(vec![crate::diagnostics::diagnostic_with_code(
                    format!(
                        "generated artifact cache key collision at '{}'; stored full key does not match the requested key",
                        final_root.display()
                    ),
                    DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                )]);
            }
            Some(_) => {
                miss_reason = Some("schema_mismatch".to_string());
            }
            None => {
                miss_reason = Some("metadata_missing".to_string());
            }
        }
        let _ = std::fs::remove_dir_all(&final_root);
    }

    let staging_root = create_invocation_workspace(&format!("{namespace}_cache_stage"))?;
    Ok(PreparedArtifactCache::Miss(PendingCachedArtifact {
        final_root,
        staging_root: staging_root.clone(),
        key_material: full_key_material,
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
    std::env::temp_dir().join(ARTIFACT_CACHE_ROOT_DIR)
}

#[derive(Deserialize, Serialize)]
struct ArtifactCacheMetadata {
    schema_version: u32,
    namespace: String,
    key: String,
    key_material: String,
    toolchain_signature: String,
}

fn metadata_matches(
    metadata: &ArtifactCacheMetadata,
    namespace: &str,
    key: &str,
    key_material: &str,
) -> bool {
    metadata.schema_version == ARTIFACT_CACHE_SCHEMA_VERSION
        && metadata.namespace == namespace
        && metadata.key == key
        && metadata.key_material == key_material
        && metadata.toolchain_signature == toolchain_signature()
}

fn artifact_key_material(namespace: &str, scope: &Path, key_material: &str) -> String {
    let mut material = String::new();
    for (name, value) in [
        ("schema", ARTIFACT_CACHE_SCHEMA_VERSION.to_string()),
        ("namespace", namespace.to_string()),
        ("scope", scope.display().to_string()),
        ("toolchain", toolchain_signature().to_string()),
        ("input", key_material.to_string()),
    ] {
        material.push_str(name);
        material.push('=');
        material.push_str(&value.len().to_string());
        material.push(':');
        material.push_str(&value);
        material.push('\n');
    }
    material
}

fn load_cache_metadata(workspace_root: &Path) -> Option<ArtifactCacheMetadata> {
    let metadata_path = workspace_root.join(ARTIFACT_CACHE_METADATA_FILE);
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
    std::fs::write(workspace_root.join(ARTIFACT_CACHE_METADATA_FILE), content).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("failed to write generated artifact cache metadata: {error}"),
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })
}

fn toolchain_signature() -> &'static str {
    static TOOLCHAIN_SIGNATURE: OnceLock<String> = OnceLock::new();
    TOOLCHAIN_SIGNATURE
        .get_or_init(|| {
            let values = [
                command_signature("cargo", &["-V"]).unwrap_or_else(|| "cargo:unavailable".into()),
                command_signature("rustc", &["-Vv"]).unwrap_or_else(|| "rustc:unavailable".into()),
                env_signature("RUSTFLAGS"),
                env_signature("CARGO_BUILD_TARGET"),
                env_signature("CARGO_TARGET_DIR"),
                env_signature("RUSTC_WRAPPER"),
            ];
            deterministic_hash(&values.join("\0"))
        })
        .as_str()
}

fn command_signature(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn env_signature(name: &str) -> String {
    let value = std::env::var(name).unwrap_or_default();
    format!("{name}={value}")
}

fn deterministic_hash(value: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let bytes = Sha256::digest(value.as_bytes());
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        ARTIFACT_CACHE_SCHEMA_VERSION, ArtifactCacheMetadata, ArtifactCacheReport,
        PendingCachedArtifact, toolchain_signature, write_cache_metadata,
    };
    use std::path::Path;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = format!(
            "sifr_artifact_cache_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn pending_artifact_commit_treats_existing_final_dir_as_concurrent_populate() {
        let root = temp_dir("concurrent_populate");
        let staging_root = root.join("stage");
        let final_root = root.join("final");
        std::fs::create_dir_all(&staging_root).expect("staging should be created");
        std::fs::create_dir_all(&final_root).expect("final should be created");
        std::fs::write(final_root.join("winner"), b"ok").expect("winner file should be written");
        write_cache_metadata(
            &final_root,
            &ArtifactCacheMetadata {
                schema_version: ARTIFACT_CACHE_SCHEMA_VERSION,
                namespace: "test".to_string(),
                key: "key".to_string(),
                key_material: "material".to_string(),
                toolchain_signature: toolchain_signature().to_string(),
            },
        )
        .expect("winner metadata should be written");

        let pending = PendingCachedArtifact {
            final_root: final_root.clone(),
            staging_root: staging_root.clone(),
            key_material: "material".to_string(),
            report: ArtifactCacheReport {
                namespace: "test".to_string(),
                key: "key".to_string(),
                workspace_root: staging_root,
                cache_hit: false,
                miss_reason: Some("not_found".to_string()),
            },
        };

        let entry = pending.commit(&[]).expect("commit should use winner dir");
        let report = entry.report();

        assert!(report.cache_hit);
        assert_eq!(report.workspace_root, final_root);
        assert_eq!(report.miss_reason.as_deref(), Some("concurrent_populate"));
        assert!(!root.join("stage").exists());

        let _ = std::fs::remove_dir_all(Path::new(&root));
    }

    #[test]
    fn pending_artifact_commit_rejects_concurrent_full_key_mismatch() {
        let root = temp_dir("concurrent_mismatch");
        let staging_root = root.join("stage");
        let final_root = root.join("final");
        std::fs::create_dir_all(&staging_root).expect("staging should be created");
        std::fs::create_dir_all(&final_root).expect("final should be created");
        write_cache_metadata(
            &final_root,
            &ArtifactCacheMetadata {
                schema_version: ARTIFACT_CACHE_SCHEMA_VERSION,
                namespace: "test".to_string(),
                key: "key".to_string(),
                key_material: "different-material".to_string(),
                toolchain_signature: toolchain_signature().to_string(),
            },
        )
        .expect("winner metadata should be written");
        let pending = PendingCachedArtifact {
            final_root,
            staging_root: staging_root.clone(),
            key_material: "material".to_string(),
            report: ArtifactCacheReport {
                namespace: "test".to_string(),
                key: "key".to_string(),
                workspace_root: staging_root,
                cache_hit: false,
                miss_reason: Some("not_found".to_string()),
            },
        };

        let diagnostics = match pending.commit(&[]) {
            Err(diagnostics) => diagnostics,
            Ok(_) => panic!("mismatched concurrent winner must fail closed"),
        };
        assert!(diagnostics[0].message.contains("full cache key"));
        let _ = std::fs::remove_dir_all(Path::new(&root));
    }
}
