use sifr_ir::RustTargetPath;
use std::path::Path;

pub(super) fn dependency_features(
    _dependency_name: &str,
    backend_root: &Path,
    path: &RustTargetPath,
) -> Vec<String> {
    let Some(feature) = path.segments.get(1) else {
        return Vec::new();
    };
    if crate_feature_exists(backend_root, feature) {
        return vec![feature.clone()];
    }
    let cargo_feature = feature.replace('_', "-");
    crate_feature_exists(backend_root, &cargo_feature)
        .then_some(cargo_feature)
        .into_iter()
        .collect()
}

/// Return whether `feature` is declared by the probed crate. This deliberately
/// treats undeclared path segments as no feature so sysroot-interop tests can
/// use minimal temp crates and future flat targets can still probe without a
/// feature.
fn crate_feature_exists(backend_root: &Path, feature: &str) -> bool {
    let Ok(manifest) = std::fs::read_to_string(backend_root.join("Cargo.toml")) else {
        return false;
    };
    let Ok(value) = manifest.parse::<toml::Table>() else {
        return false;
    };
    value
        .get("features")
        .and_then(toml::Value::as_table)
        .is_some_and(|features| features.contains_key(feature))
}
