use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::{
    validate_relative_path, CompilerRequirement, ImportRoot, PythonConfig, SifrEdition, TrustPolicy,
};
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
        python: optional_import_root_list(cargo_package_id, manifest_path, table, "trust.python")?,
        python_native: optional_import_root_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.python-native",
        )?,
    })
}

pub(super) fn parse_python_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<PythonConfig, PackageDiagnostic> {
    Ok(PythonConfig {
        venv: optional_relative_path(cargo_package_id, manifest_path, table, "python.venv")?,
        pyproject: optional_relative_path(
            cargo_package_id,
            manifest_path,
            table,
            "python.pyproject",
        )?,
        lock: optional_relative_path(cargo_package_id, manifest_path, table, "python.lock")?,
        interpreter: optional_relative_path(
            cargo_package_id,
            manifest_path,
            table,
            "python.interpreter",
        )?,
        allow_imports: optional_import_root_list(
            cargo_package_id,
            manifest_path,
            table,
            "python.allow-imports",
        )?,
        requires_imports: optional_import_root_list(
            cargo_package_id,
            manifest_path,
            table,
            "python.requires-imports",
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

fn optional_import_root_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<Vec<String>, PackageDiagnostic> {
    optional_string_list(cargo_package_id, manifest_path, table, dotted_key)?
        .into_iter()
        .map(|root| {
            if root == "*" || root.split('.').all(valid_identifier) {
                Ok(root)
            } else {
                Err(PackageDiagnostic::python_environment_config(
                    cargo_package_id,
                    manifest_path,
                    dotted_key,
                    format!("`{root}` is not a valid Python import root"),
                ))
            }
        })
        .collect()
}

fn optional_relative_path(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<Option<std::path::PathBuf>, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    let Some(value) = table.get(local_key) else {
        return Ok(None);
    };
    let Some(path) = value.as_str().filter(|value| !value.is_empty()) else {
        return Err(PackageDiagnostic::python_environment_config(
            cargo_package_id,
            manifest_path,
            dotted_key,
            "expected a non-empty relative path",
        ));
    };
    validate_relative_path(cargo_package_id, manifest_path, dotted_key, path)
        .map(Some)
        .map_err(|diagnostic| {
            PackageDiagnostic::python_environment_config(
                cargo_package_id,
                manifest_path,
                dotted_key,
                diagnostic.message,
            )
        })
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
                "compiler requirement `{}` does not match this compiler compatibility window",
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
