use super::rust_interop_digest::{digest_file, digest_path, fnv1a64_hex, push_cache_bytes};
use super::rust_interop_probe::PendingRustBridgeProbe;
use super::rust_interop_probe_paths::normalize_cargo_target_dir;
use super::rust_interop_sqlx_offline::sqlx_offline_metadata_digest;
use super::workspace::artifact_cache_root;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, b"ok\n");
}

pub(super) fn probe_cache_key(
    probe: &PendingRustBridgeProbe,
    backend_root: &Path,
    probe_manifest: &str,
    probe_source: &str,
    cache: &mut ProbeCacheKeyCache,
) -> String {
    let mut input = Vec::new();
    for value in [
        "sifr-rust-bridge-probe-cache-v1",
        &toolchain_signature(),
        &probe.backend.cargo_package_id.0,
        &probe.backend.dependency_name,
        &probe.backend.cargo_package_name,
        &probe.backend.cargo_version,
        probe.backend.cargo_source.as_deref().unwrap_or("<path>"),
        &probe.backend.cargo_manifest_path.display().to_string(),
        &probe.path.dotted(),
        probe_manifest,
        probe_source,
        if probe.cargo_resolution.lock_mode == sifr_package::CargoLockMode::Normal {
            "normal"
        } else {
            "lock-constrained"
        },
        &cached_digest_path(backend_root),
        &nearest_lock_digest(backend_root),
        &cached_digest_path(&probe.sysroot_runtime_crate),
        &optional_vendor_identity(
            probe
                .cargo_resolution
                .uses_sysroot_vendor()
                .then_some(probe.sysroot_vendor_dir.as_deref())
                .flatten(),
        ),
    ] {
        push_cache_bytes(&mut input, value);
    }
    if let Some(metadata_digest) = cache.sqlx_metadata_digest(backend_root) {
        push_cache_bytes(&mut input, "sqlx-offline-metadata");
        push_cache_bytes(&mut input, &metadata_digest);
    }
    fnv1a64_hex(&input)
}

fn toolchain_signature() -> String {
    static TOOLCHAIN_SIGNATURE: OnceLock<String> = OnceLock::new();
    TOOLCHAIN_SIGNATURE
        .get_or_init(|| {
            let mut input = Vec::new();
            for value in [
                command_output("cargo", &["-V"]).unwrap_or_else(|| "cargo:unavailable".into()),
                command_output("rustc", &["-Vv"]).unwrap_or_else(|| "rustc:unavailable".into()),
                env_signature("RUSTFLAGS"),
                env_signature("CARGO_BUILD_TARGET"),
                env_signature("RUSTC_WRAPPER"),
            ] {
                push_cache_bytes(&mut input, &value);
            }
            fnv1a64_hex(&input)
        })
        .clone()
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn env_signature(name: &str) -> String {
    env::var(name).map_or_else(
        |_| format!("{name}=<unset>"),
        |value| format!("{name}={value}"),
    )
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
        artifact_cache_root, probe_cache_file_with_env, probe_cache_root, ProbeCacheKeyCache,
        RUST_BRIDGE_PROBE_CACHE_DIR,
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
