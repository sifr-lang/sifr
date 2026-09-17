use std::path::{Path, PathBuf};

pub(super) fn normalize_cargo_target_dir(invocation_cwd: &Path, target_dir: PathBuf) -> PathBuf {
    if target_dir.is_absolute() {
        target_dir
    } else {
        invocation_cwd.join(target_dir)
    }
}
