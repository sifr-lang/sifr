use super::workspace::artifact_cache_root;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub(super) const RUST_BRIDGE_PROBE_TARGET_DIR: &str = "rust_bridge_probe_target";

pub(super) fn probe_cargo_target_dir(invocation_cwd: &Path) -> PathBuf {
    probe_cargo_target_dir_with_env(std::env::var_os("CARGO_TARGET_DIR"), invocation_cwd)
}

pub(super) fn probe_cargo_target_dir_with_env(
    configured: Option<OsString>,
    invocation_cwd: &Path,
) -> PathBuf {
    configured.map_or_else(
        || artifact_cache_root().join(RUST_BRIDGE_PROBE_TARGET_DIR),
        |target_dir| normalize_cargo_target_dir(invocation_cwd, PathBuf::from(target_dir)),
    )
}

pub(super) fn normalize_cargo_target_dir(invocation_cwd: &Path, target_dir: PathBuf) -> PathBuf {
    if target_dir.is_absolute() {
        target_dir
    } else {
        invocation_cwd.join(target_dir)
    }
}
