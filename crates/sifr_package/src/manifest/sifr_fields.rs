use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::{
    validate_relative_path, CompilerRequirement, PythonConfig, RustInteropConfig, SifrEdition,
    TrustPolicy,
};
use std::path::Path;

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
        rust_build_scripts: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.rust-build-scripts",
        )?,
        rust_proc_macros: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.rust-proc-macros",
        )?,
        native_links: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.native-links",
        )?,
        unsafe_rust_bridges: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.unsafe-rust-bridges",
        )?,
        build_env: optional_string_list(cargo_package_id, manifest_path, table, "trust.build-env")?,
        rust_no_panic: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.rust-no-panic",
        )?,
        rust_panic_abort: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.rust-panic-abort",
        )?,
    })
}

pub(super) fn parse_python_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<PythonConfig, PackageDiagnostic> {
    if table.contains_key("allow-imports") {
        return Err(PackageDiagnostic::python_environment_config(
            cargo_package_id,
            manifest_path,
            "python.allow-imports",
            "`[python].allow-imports` was removed; publish requirements with `[python].requires-imports` and authorize them from the root `[trust].python` table",
        ));
    }
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
        requires_imports: optional_import_root_list(
            cargo_package_id,
            manifest_path,
            table,
            "python.requires-imports",
        )?,
    })
}

pub(super) fn parse_rust_interop_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<RustInteropConfig, PackageDiagnostic> {
    if table.contains_key("bridge-version") {
        return Err(PackageDiagnostic::removed_rust_bridge_version(
            cargo_package_id,
            manifest_path.to_path_buf(),
        ));
    }
    Ok(RustInteropConfig {
        bridges: optional_relative_path_list(
            cargo_package_id,
            manifest_path,
            table,
            "rust.bridges",
        )?,
        direct_crate_bindings: optional_bool(
            cargo_package_id,
            manifest_path,
            table,
            "rust.direct-crate-bindings",
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

fn optional_relative_path_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<Vec<std::path::PathBuf>, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    let Some(value) = table.get(local_key) else {
        return Ok(Vec::new());
    };
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            dotted_key,
            "expected a list of relative paths",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            let Some(path) = entry.as_str().filter(|value| !value.is_empty()) else {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    dotted_key,
                    "expected every entry to be a non-empty relative path",
                ));
            };
            validate_relative_path(cargo_package_id, manifest_path, dotted_key, path)
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

fn optional_bool(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<bool, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    let Some(value) = table.get(local_key) else {
        return Ok(false);
    };
    value.as_bool().ok_or_else(|| {
        PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            dotted_key,
            "expected a boolean",
        )
    })
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
