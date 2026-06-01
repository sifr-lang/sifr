use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::{CompilerRequirement, ImportRoot, SifrEdition, TrustPolicy};
use std::path::Path;

pub(super) fn parse_exports(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Value,
) -> Result<Vec<ImportRoot>, PackageDiagnostic> {
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "exports.modules",
            "expected a list of import roots",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            let Some(export) = entry.as_str().filter(|value| !value.is_empty()) else {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    "exports.modules",
                    "expected every export to be a non-empty string",
                ));
            };
            if !export.split('.').all(valid_identifier) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    "exports.modules",
                    format!("`{export}` is not a valid dotted import root"),
                ));
            }
            Ok(ImportRoot(export.to_string()))
        })
        .collect()
}

pub(super) fn parse_trust(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<TrustPolicy, PackageDiagnostic> {
    Ok(TrustPolicy {
        native: optional_string_list(cargo_package_id, manifest_path, table, "trust.native")?,
        build_scripts: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.build-scripts",
        )?,
        proc_macros: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.proc-macros",
        )?,
    })
}

fn optional_string_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<Vec<String>, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    let Some(value) = table.get(local_key) else {
        return Ok(Vec::new());
    };
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            dotted_key,
            "expected a list of strings",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    PackageDiagnostic::invalid_sifr_manifest(
                        cargo_package_id,
                        manifest_path.to_path_buf(),
                        dotted_key,
                        "expected every entry to be a non-empty string",
                    )
                })
        })
        .collect()
}

pub(super) fn validate_edition(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    edition: &SifrEdition,
) -> Result<(), PackageDiagnostic> {
    if edition.0 == "2026" {
        Ok(())
    } else {
        Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "package.edition",
            format!("unsupported Sifr edition `{}`", edition.0),
        ))
    }
}

pub(super) fn validate_compiler_requirement(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    requirement: &CompilerRequirement,
) -> Result<(), PackageDiagnostic> {
    if requirement.0.contains("0.3") || requirement.0 == "*" {
        Ok(())
    } else {
        Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "package.sifr-version",
            format!(
                "compiler requirement `{}` does not match this milestone compiler compatibility window",
                requirement.0
            ),
        ))
    }
}

fn valid_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
