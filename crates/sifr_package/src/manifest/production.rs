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

pub(super) fn reject_unsupported_layout_fields(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Table,
) -> Result<(), PackageDiagnostic> {
    if value.get("exports").is_some() {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "exports",
            "unsupported field",
        ));
    }
    if value.get("bin").is_some() {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "bin",
            "unsupported field",
        ));
    }
    if value
        .get("source")
        .and_then(toml::Value::as_table)
        .is_some_and(|source| source.contains_key("roots"))
    {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "source.roots",
            "unsupported field; use source.root",
        ));
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
