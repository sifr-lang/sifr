use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::{validate_relative_path, PackageSourceRoot};
use std::path::{Path, PathBuf};

pub(super) fn parse_source_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    source_table: Option<&toml::Table>,
) -> Result<PackageSourceRoot, PackageDiagnostic> {
    match source_table.and_then(|source| source.get("root")) {
        Some(root) => {
            parse_source_root(cargo_package_id, manifest_path, root).map(PackageSourceRoot)
        }
        None => Ok(PackageSourceRoot(PathBuf::from("src"))),
    }
}

pub(super) fn validate_manifest_shape(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Table,
) -> Result<(), PackageDiagnostic> {
    for field in ["exports", "bin"] {
        if value.get(field).is_some() {
            return Err(PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                field,
                "unsupported field",
            ));
        }
    }
    if let Some(source) = value.get("source").and_then(toml::Value::as_table) {
        if let Some(field) = source.keys().find(|field| field.as_str() != "root") {
            return Err(PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                format!("source.{field}"),
                "unsupported source field",
            ));
        }
    }
    Ok(())
}

fn parse_source_root(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Value,
) -> Result<PathBuf, PackageDiagnostic> {
    let Some(source_root) = value.as_str() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "source.root",
            "expected a relative path",
        ));
    };
    validate_relative_path(cargo_package_id, manifest_path, "source.root", source_root)
}
