use super::rust_interop_digest::{digest_file, digest_path, fnv1a64_hex, push_cache_bytes};
use super::rust_interop_probe::{normalize_cargo_target_dir, PendingRustBridgeProbe};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::{env, fs};

pub(super) fn probe_cache_file(cache_key: &str, invocation_cwd: &Path) -> Option<PathBuf> {
    let raw = env::var_os("SIFR_RUST_BRIDGE_PROBE_CACHE_DIR")?;
    let root = normalize_cargo_target_dir(invocation_cwd, PathBuf::from(raw));
    Some(root.join(format!("{cache_key}.ok")))
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
        &cached_digest_path(backend_root),
        &nearest_lock_digest(backend_root),
        &optional_manifest_root_digest(probe.sysroot_runtime_crate_manifest.as_deref()),
        &optional_vendor_identity(probe.sysroot_vendor_dir.as_deref()),
    ] {
        push_cache_bytes(&mut input, value);
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

fn optional_manifest_root_digest(manifest: Option<&Path>) -> String {
    manifest
        .and_then(Path::parent)
        .map_or_else(|| "<no-sysroot-runtime>".to_string(), cached_digest_path)
}

fn optional_vendor_identity(vendor_dir: Option<&Path>) -> String {
    let Some(vendor_dir) = vendor_dir else {
        return "<no-sysroot-vendor>".to_string();
    };
    nearest_lock_digest(vendor_dir)
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
