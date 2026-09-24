use super::rust_interop_digest::{
    digest_lock_file_checked, digest_path_checked, nearest_lock_digest_checked,
};
use super::rust_interop_probe::PendingRustBridgeProbe;
use super::rust_interop_probe_paths::normalize_cargo_target_dir;
use super::rust_interop_sqlx_offline::sqlx_offline_metadata_digest;
use super::workspace::artifact_cache_root;
use sifr_identity::IdentityEncoder;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, fs};

const RUST_BRIDGE_PROBE_CACHE_DIR: &str = "rust_bridge_probes";

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
    let Some(parent) = path.parent() else { return };
    if crate::cache_storage::directory(parent).is_err() {
        return;
    }
    let stage = parent.join(format!(
        ".probe-stage-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let result = (|| -> std::io::Result<()> {
        let mut file = crate::cache_storage::new_private_file(&stage)?;
        file.write_all(b"ok\n")?;
        file.sync_all()?;
        crate::cache_storage::publish(&stage, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(stage);
    }
}

pub(super) fn probe_cache_key(
    probe: &PendingRustBridgeProbe,
    backend_root: &Path,
    probe_manifest: &str,
    probe_source: &str,
) -> Result<String, String> {
    let mut input = IdentityEncoder::new("rust-bridge-probe-cache-v4");
    let backend_tree = digest_path_checked(backend_root).map_err(|error| {
        format!(
            "unreadable Rust probe backend tree '{}': {error}",
            backend_root.display()
        )
    })?;
    let runtime_tree = digest_path_checked(&probe.sysroot_runtime_crate).map_err(|error| {
        format!(
            "unreadable Rust probe runtime tree '{}': {error}",
            probe.sysroot_runtime_crate.display()
        )
    })?;
    let nearest_lock = nearest_lock_digest(backend_root)?;
    let vendor_identity = optional_vendor_identity(
        probe
            .cargo_resolution
            .uses_sysroot_vendor()
            .then_some(probe.sysroot_vendor_dir.as_deref())
            .flatten(),
    )?;
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
        ("backend-tree", &backend_tree),
        ("nearest-lock", &nearest_lock),
        ("sysroot-runtime-tree", &runtime_tree),
        ("vendor", &vendor_identity),
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
        let digest = digest_lock_file_checked(path).map_err(|error| {
            format!(
                "unreadable authoritative Cargo lock '{}': {error}",
                path.display()
            )
        })?;
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
    let metadata = sqlx_offline_metadata_digest(backend_root)?;
    input.field("sqlx-present", &[u8::from(metadata.is_some())]);
    if let Some(metadata) = metadata {
        input.field("sqlx", metadata.as_bytes());
    }
    Ok(input.finish())
}

fn optional_vendor_identity(vendor_dir: Option<&Path>) -> Result<String, String> {
    let Some(vendor_dir) = vendor_dir else {
        return Ok("<no-sysroot-vendor>".to_string());
    };
    nearest_lock_digest(vendor_dir)
}

fn probe_cache_root(configured: Option<OsString>, invocation_cwd: &Path) -> PathBuf {
    configured.map_or_else(
        || artifact_cache_root().join(RUST_BRIDGE_PROBE_CACHE_DIR),
        |raw| normalize_cargo_target_dir(invocation_cwd, PathBuf::from(raw)),
    )
}

fn nearest_lock_digest(path: &Path) -> Result<String, String> {
    nearest_lock_digest_checked(path)
        .map_err(|error| {
            format!(
                "unreadable nearest Cargo lock '{}': {error}",
                path.display()
            )
        })
        .map(|digest| digest.unwrap_or_else(|| "<no-cargo-lock>".to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        RUST_BRIDGE_PROBE_CACHE_DIR, artifact_cache_root, probe_cache_file_with_env,
        probe_cache_root,
    };
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

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
