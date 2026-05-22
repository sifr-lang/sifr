use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::{parse_source_roots, validate_relative_path, PackageSourceRoot};
use std::path::{Path, PathBuf};

pub(super) fn parse_source_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    source_table: Option<&toml::Table>,
) -> Result<Vec<PackageSourceRoot>, PackageDiagnostic> {
    match source_table {
        Some(source) if source.contains_key("root") => source
            .get("root")
            .map(|root| parse_source_root(cargo_package_id, manifest_path, root))
            .transpose()
            .map(|root| {
                vec![PackageSourceRoot(
                    root.unwrap_or_else(|| PathBuf::from("src")),
                )]
            }),
        Some(source) if source.contains_key("roots") => source
            .get("roots")
            .map(|roots| parse_source_roots(cargo_package_id, manifest_path, roots))
            .transpose()
            .map(|roots| roots.unwrap_or_else(|| vec![PackageSourceRoot(PathBuf::from("sifr"))])),
        _ => Ok(vec![PackageSourceRoot(PathBuf::from("src"))]),
    }
}

pub(super) fn reject_production_manifest_exports(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Table,
) -> Result<(), PackageDiagnostic> {
    if value
        .get("exports")
        .and_then(toml::Value::as_table)
        .and_then(|exports| exports.get("modules"))
        .is_some()
    {
        return Err(PackageDiagnostic::manifest_exports_not_production(
            cargo_package_id,
            manifest_path,
        ));
    }
    Ok(())
}

pub(super) fn reject_production_manifest_bins(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Table,
) -> Result<(), PackageDiagnostic> {
    if value.get("bin").is_some() {
        return Err(PackageDiagnostic::manifest_bins_not_production(
            cargo_package_id,
            manifest_path,
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
