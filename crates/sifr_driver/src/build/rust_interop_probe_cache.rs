use super::rust_interop_digest::{digest_file, digest_path};
use super::rust_interop_probe::PendingRustBridgeProbe;
use super::rust_interop_probe_paths::normalize_cargo_target_dir;
use super::rust_interop_sqlx_offline::sqlx_offline_metadata_digest;
use super::workspace::artifact_cache_root;
use sifr_identity::IdentityEncoder;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::{env, fs};

const RUST_BRIDGE_PROBE_CACHE_DIR: &str = "rust_bridge_probes";

#[derive(Default)]
pub(super) struct ProbeCacheKeyCache {
    sqlx_metadata_by_backend_root: BTreeMap<PathBuf, Option<String>>,
}

impl ProbeCacheKeyCache {
    fn sqlx_metadata_digest(&mut self, backend_root: &Path) -> Option<String> {
        self.sqlx_metadata_digest_with(backend_root, sqlx_offline_metadata_digest)
    }

    fn sqlx_metadata_digest_with(
        &mut self,
        backend_root: &Path,
        inspect: impl FnOnce(&Path) -> Option<String>,
    ) -> Option<String> {
        self.sqlx_metadata_by_backend_root
            .entry(backend_root.to_path_buf())
            .or_insert_with(|| inspect(backend_root))
            .clone()
    }
}

pub(super) fn probe_cache_file(cache_key: &str, invocation_cwd: &Path) -> PathBuf {
    probe_cache_file_with_env(
        cache_key,
        env::var_os("SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"),
        invocation_cwd,
    )
}

fn probe_cache_file_with_env(
    cache_key: &str,
    configured: Option<OsString>,
    invocation_cwd: &Path,
) -> PathBuf {
    let root = probe_cache_root(configured, invocation_cwd);
    root.join(format!("{cache_key}.ok"))
}

pub(super) fn mark_probe_cache_hit(path: &Path) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let Some(parent) = path.parent() else { return };
    if crate::cache_storage::directory(parent).is_err() {
        return;
    }
    if let Ok(mut file) = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
    {
        let _ = file.write_all(b"ok\n");
    }
}

pub(super) fn probe_cache_key(
    probe: &PendingRustBridgeProbe,
    backend_root: &Path,
    probe_manifest: &str,
    probe_source: &str,
    cache: &mut ProbeCacheKeyCache,
) -> String {
    let mut input = IdentityEncoder::new("rust-bridge-probe-cache-v2");
    for (name, value) in [
        ("package-id", probe.backend.cargo_package_id.0.as_str()),
        ("dependency", probe.backend.dependency_name.as_str()),
        ("cargo-package", probe.backend.cargo_package_name.as_str()),
        ("cargo-version", probe.backend.cargo_version.as_str()),
        (
            "manifest-path",
            &probe.backend.cargo_manifest_path.display().to_string(),
        ),
        ("target-path", &probe.path.dotted()),
        ("manifest", probe_manifest),
        ("source", probe_source),
        ("backend-tree", &cached_digest_path(backend_root)),
        ("nearest-lock", &nearest_lock_digest(backend_root)),
        (
            "sysroot-runtime-tree",
            &cached_digest_path(&probe.sysroot_runtime_crate),
        ),
        (
            "vendor",
            &optional_vendor_identity(
                probe
                    .cargo_resolution
                    .uses_sysroot_vendor()
                    .then_some(probe.sysroot_vendor_dir.as_deref())
                    .flatten(),
            ),
        ),
    ] {
        input.field(name, value.as_bytes());
    }
    input.field(
        "cargo-source-present",
        &[u8::from(probe.backend.cargo_source.is_some())],
    );
    if let Some(source) = &probe.backend.cargo_source {
        input.field("cargo-source", source.as_bytes());
    }
    input.field(
        "lock-mode",
        probe.cargo_resolution.lock_mode.as_str().as_bytes(),
    );
    input.field(
        "vendor-mode",
        format!("{:?}", probe.cargo_resolution.cargo_vendor_mode).as_bytes(),
    );
    input.field(
        "vendor-path-present",
        &[u8::from(probe.sysroot_vendor_dir.is_some())],
    );
    if let Some(path) = &probe.sysroot_vendor_dir {
        input.field("vendor-path", path.to_string_lossy().as_bytes());
    }
    input.field(
        "authority-count",
        &(probe.cargo_resolution.authoritative_locks.len() as u64).to_be_bytes(),
    );
    for path in &probe.cargo_resolution.authoritative_locks {
        input.field("authority-path", path.to_string_lossy().as_bytes());
        let digest = digest_file(path);
        input.field("authority-readable", &[u8::from(digest.is_some())]);
        if let Some(digest) = digest {
            input.field("authority-digest", digest.as_bytes());
        }
    }
    input.field(
        "toolchain-present",
        &[u8::from(probe.cargo_resolution.native_toolchain.is_ok())],
    );
    if let Ok(tools) = &probe.cargo_resolution.native_toolchain {
        input.field("toolchain", tools.identity().as_bytes());
    }
    let seed = probe.cargo_resolution.normal_seed_cache_fragment();
    input.field("seed-present", &[u8::from(seed.is_some())]);
    if let Some(seed) = seed {
        input.field("seed", seed.as_bytes());
    }
    let metadata = cache.sqlx_metadata_digest(backend_root);
    input.field("sqlx-present", &[u8::from(metadata.is_some())]);
    if let Some(metadata) = metadata {
        input.field("sqlx", metadata.as_bytes());
    }
    input.finish()
}

fn cached_digest_path(path: &Path) -> String {
    static DIGESTS: OnceLock<Mutex<BTreeMap<PathBuf, String>>> = OnceLock::new();
    let cache = DIGESTS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(mut digests) = cache.lock() {
        if let Some(digest) = digests.get(path) {
            return digest.clone();
        }
        let digest = digest_path(path);
        digests.insert(path.to_path_buf(), digest.clone());
        return digest;
    }
    digest_path(path)
}

fn optional_vendor_identity(vendor_dir: Option<&Path>) -> String {
    let Some(vendor_dir) = vendor_dir else {
        return "<no-sysroot-vendor>".to_string();
    };
    nearest_lock_digest(vendor_dir)
}

fn probe_cache_root(configured: Option<OsString>, invocation_cwd: &Path) -> PathBuf {
    configured.map_or_else(
        || artifact_cache_root().join(RUST_BRIDGE_PROBE_CACHE_DIR),
        |raw| normalize_cargo_target_dir(invocation_cwd, PathBuf::from(raw)),
    )
}

fn nearest_lock_digest(path: &Path) -> String {
    nearest_ancestor_file(path, "Cargo.lock")
        .and_then(|lock| digest_file(&lock))
        .unwrap_or_else(|| "<no-cargo-lock>".to_string())
}

fn nearest_ancestor_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        let candidate = path.join(file_name);
        if candidate.is_file() {
            return Some(candidate);
        }
        current = path.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        ProbeCacheKeyCache, RUST_BRIDGE_PROBE_CACHE_DIR, artifact_cache_root,
        probe_cache_file_with_env, probe_cache_root,
    };
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn probe_cache_key_cache_inspects_each_backend_sqlx_identity_once() {
        let mut cache = ProbeCacheKeyCache::default();
        let clean_root = Path::new("/backend/clean");
        let sqlx_root = Path::new("/backend/sqlx");
        let mut clean_inspections = 0;
        let mut sqlx_inspections = 0;

        assert_eq!(
            cache.sqlx_metadata_digest_with(clean_root, |_| {
                clean_inspections += 1;
                None
            }),
            None
        );
        assert_eq!(
            cache.sqlx_metadata_digest_with(clean_root, |_| {
                clean_inspections += 1;
                Some("changed".to_string())
            }),
            None
        );
        assert_eq!(
            cache.sqlx_metadata_digest_with(sqlx_root, |_| {
                sqlx_inspections += 1;
                Some("sqlx-digest".to_string())
            }),
            Some("sqlx-digest".to_string())
        );
        assert_eq!(
            cache.sqlx_metadata_digest_with(sqlx_root, |_| {
                sqlx_inspections += 1;
                None
            }),
            Some("sqlx-digest".to_string())
        );
        assert_eq!(clean_inspections, 1);
        assert_eq!(sqlx_inspections, 1);
    }

    #[test]
    fn probe_cache_defaults_to_stable_artifact_cache_subdir() {
        let root = probe_cache_root(None, Path::new("/repo"));

        assert_eq!(
            root,
            artifact_cache_root().join(RUST_BRIDGE_PROBE_CACHE_DIR)
        );
    }

    #[test]
    fn probe_cache_honors_absolute_env_override() {
        let root = probe_cache_root(Some(OsString::from("/tmp/sifr-probes")), Path::new("/repo"));

        assert_eq!(root, PathBuf::from("/tmp/sifr-probes"));
    }

    #[test]
    fn probe_cache_normalizes_relative_env_override_from_invocation_cwd() {
        let root = probe_cache_root(Some(OsString::from("target/probes")), Path::new("/repo"));

        assert_eq!(root, PathBuf::from("/repo/target/probes"));
    }

    #[test]
    fn probe_cache_file_uses_resolved_root() {
        let path = probe_cache_file_with_env("abc123", None, Path::new("/repo"));

        assert_eq!(
            path,
            artifact_cache_root()
                .join(RUST_BRIDGE_PROBE_CACHE_DIR)
                .join("abc123.ok")
        );
    }
}
