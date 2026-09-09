//! Validate source confinement and pure markers before graph publication.

use crate::cargo::metadata::{CargoPackage, CargoPackageId};
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::SifrManifest;
use crate::source::layout::{MarkerValidation, validate_pure_marker_file};
use sifr_frontend::SourceProvider;
use std::path::Path;

pub(super) fn validate_sql_schema_sources(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    package_root: &Path,
    manifest: &SifrManifest,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    let canonical_root = if manifest.sql.profiles.is_empty() {
        None
    } else {
        Some(provider.canonicalize(package_root).map_err(|error| {
            PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                "sql.profiles",
                format!("cannot resolve package root for schema sources: {error}"),
            )
        })?)
    };
    for (profile_name, profile) in &manifest.sql.profiles {
        for source in &profile.sources {
            let path = package_root.join(source);
            if !provider.is_file(&path) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "schema source '{}' must be a checked-in file inside the package",
                        source.display()
                    ),
                ));
            }
            let canonical_source = provider.canonicalize(&path).map_err(|error| {
                PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "cannot resolve schema source '{}': {error}",
                        source.display()
                    ),
                )
            })?;
            if !canonical_source.starts_with(canonical_root.as_deref().unwrap_or(package_root)) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("sql.profiles.{profile_name}.source"),
                    format!(
                        "schema source '{}' resolves outside the package",
                        source.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_pure_markers(
    package: &CargoPackage,
    cargo_package_id: &CargoPackageId,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    for target in &package.targets {
        if !target.kind.contains("lib") {
            continue;
        }
        match validate_pure_marker_file(&target.src_path, provider) {
            Ok(MarkerValidation::PureMarker) => {}
            Ok(MarkerValidation::NonTrivialRust { reason }) => {
                return Err(PackageDiagnostic::non_trivial_pure_marker(
                    cargo_package_id,
                    target.src_path.clone(),
                    reason,
                ));
            }
            Err(error) => {
                return Err(PackageDiagnostic::non_trivial_pure_marker(
                    cargo_package_id,
                    target.src_path.clone(),
                    format!("marker target could not be read: {error}"),
                ));
            }
        }
    }
    Ok(())
}
