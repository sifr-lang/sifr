use crate::cargo::metadata::CargoPackageId;
use crate::diag::{PackageDiagnostic, PackageDiagnosticOrigin};
use crate::graph::derive::SifrPackageId;
use crate::manifest::sifr::ImportRoot;
use sifr_diagnostics::DiagnosticCode;
use std::path::Path;

impl PackageDiagnostic {
    #[must_use]
    pub fn manifest_exports_not_production(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_MANIFEST_EXPORTS_NOT_PRODUCTION,
            message: format!(
                "production sifr.toml at '{}' uses [exports].modules",
                manifest_path.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path.to_path_buf(),
                key: Some("exports.modules".to_string()),
            }),
            help: Some(
                "declare public package APIs in src/__init__.sifr instead of [exports].modules"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn manifest_bins_not_production(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_MANIFEST_BIN_TABLES_NOT_PRODUCTION,
            message: format!(
                "production sifr.toml at '{}' uses [[bin]] target tables",
                manifest_path.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path.to_path_buf(),
                key: Some("bin".to_string()),
            }),
            help: Some(
                "use src/main.sifr, src/bin/*.sifr, [package].default-run, [scripts], or sifr run --bin <name>"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn duplicate_public_api_symbol(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
        symbol: impl Into<String>,
    ) -> Self {
        let symbol = symbol.into();
        Self {
            code: DiagnosticCode::PACKAGE_DUPLICATE_PUBLIC_API_SYMBOL,
            message: format!(
                "public API symbol '{symbol}' is exported more than once by '{}'",
                manifest_path.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path.to_path_buf(),
                key: Some("__init__.sifr".to_string()),
            }),
            help: Some("make each public name in __init__.sifr resolve to one origin".to_string()),
        }
    }

    #[must_use]
    pub fn projection_manifest_pointer_drift(
        cargo_package_id: &CargoPackageId,
        path: impl Into<std::path::PathBuf>,
        reason: impl Into<String>,
    ) -> Self {
        let path = path.into();
        Self {
            code: DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT,
            message: format!(
                "Cargo projection manifest pointer drift at '{}': {}",
                path.display(),
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoManifest {
                cargo_package_id: cargo_package_id.clone(),
                path,
                key: Some("package.metadata.sifr.manifest".to_string()),
            }),
            help: Some(
                "run `sifr repair` to regenerate Sifr-owned Cargo projection metadata".to_string(),
            ),
        }
    }

    #[must_use]
    pub fn projection_include_drift(
        cargo_package_id: &CargoPackageId,
        path: impl Into<std::path::PathBuf>,
        required: impl Into<String>,
    ) -> Self {
        let required = required.into();
        Self {
            code: DiagnosticCode::PACKAGE_PROJECTION_INCLUDE_DRIFT,
            message: format!("Cargo projection include rules omit required entry '{required}'"),
            origin: Box::new(PackageDiagnosticOrigin::CargoManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: path.into(),
                key: Some("package.include".to_string()),
            }),
            help: Some("run `sifr repair` to regenerate Sifr-owned include metadata".to_string()),
        }
    }

    #[must_use]
    pub fn projection_pure_marker_missing(
        cargo_package_id: &CargoPackageId,
        marker_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        let marker_path = marker_path.into();
        Self {
            code: DiagnosticCode::PACKAGE_PROJECTION_PURE_MARKER_MISSING,
            message: format!(
                "pure Sifr package marker '{}' is missing",
                marker_path.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::RustMarker {
                cargo_package_id: cargo_package_id.clone(),
                path: marker_path,
            }),
            help: Some(
                "run `sifr repair` to regenerate the pure marker, or configure Rust-backed trust policy"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn selected_rust_only(cargo_package_id: &CargoPackageId, package_name: &str) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_SELECTED_RUST_ONLY,
            message: format!(
                "Cargo package '{package_name}' is Rust-only and cannot be selected as a Sifr source package"
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some("select a package with [package.metadata.sifr], or build Rust-only crates through Cargo".to_string()),
        }
    }

    #[must_use]
    pub fn rust_only_depends_on_sifr(from: &CargoPackageId, to: &CargoPackageId) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_RUST_ONLY_DEPENDS_ON_SIFR,
            message: format!(
                "Rust-only Cargo package '{}' depends on Sifr package '{}'",
                from.0, to.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(from.clone()),
            }),
            help: Some("convert the Rust package into a Rust-backed Sifr package or hide the Sifr dependency behind a supported Cargo feature boundary".to_string()),
        }
    }

    #[must_use]
    pub fn selector_ambiguous(selector: &str, candidates: &[String]) -> Self {
        let suffix = if candidates.is_empty() {
            "no matching package".to_string()
        } else {
            format!("candidates: {}", candidates.join(", "))
        };
        Self {
            code: DiagnosticCode::PACKAGE_SELECTOR_AMBIGUOUS,
            message: format!("package selector '{selector}' is ambiguous or invalid: {suffix}"),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "use an exact Cargo package name, Sifr package name, or package id".to_string(),
            ),
        }
    }

    #[must_use]
    pub fn duplicate_workspace_import_root(import_root: &ImportRoot, packages: &[String]) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_DUPLICATE_WORKSPACE_IMPORT_ROOT,
            message: format!(
                "workspace packages export duplicate import root '{}': {}",
                import_root.0,
                packages.join(", ")
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "select a narrower package set or add aliases before building the workspace"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn changed_file_mapping_failed(path: &Path) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_CHANGED_FILE_MAPPING_FAILED,
            message: format!("changed path '{}' does not map to one Sifr package", path.display()),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some("include a workspace manifest/lockfile change or restrict the changed-file filter to package roots".to_string()),
        }
    }

    #[must_use]
    pub fn outdated_query_unsupported(cargo_package_id: &CargoPackageId, source: &str) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_OUTDATED_QUERY_UNSUPPORTED,
            message: format!(
                "cannot determine newest compatible package version from source '{source}'"
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some(
                "Cargo source metadata is not available for this package source".to_string(),
            ),
        }
    }

    #[must_use]
    pub fn run_target_ambiguous(selector: &str, candidates: &[String]) -> Self {
        let details = if candidates.is_empty() {
            "no runnable package target matched".to_string()
        } else {
            format!("candidates: {}", candidates.join(", "))
        };
        Self {
            code: DiagnosticCode::PACKAGE_RUN_TARGET_AMBIGUOUS,
            message: format!("ambiguous package run target '{selector}': {details}"),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "use `--bin <name>` for app targets or `--script <name>` for scripts".to_string(),
            ),
        }
    }

    #[must_use]
    pub fn invalid_app_target_name(target: &str) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_INVALID_APP_TARGET_NAME,
            message: format!("invalid package app target name '{target}'"),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "use alphanumeric characters, `_`, `-`, and `/` between nested target segments"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn explicit_file_outside_source_root(file: &Path, source_root: &Path) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_EXPLICIT_FILE_OUTSIDE_SOURCE_ROOT,
            message: format!(
                "explicit file '{}' is outside package source root '{}'",
                file.display(),
                source_root.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "run the file from outside the package, or move it under the package source root"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn script_recursion(script: &str) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_SCRIPT_RECURSION,
            message: format!("package script recursion is not allowed for '{script}'"),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "scripts expand to one Sifr command plan and may not call other scripts"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn archive_missing_sifr_source(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_ARCHIVE_MISSING_SIFR_SOURCE,
            message: format!(
                "package '{}' archive contains no .sifr source files",
                package_id.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some(
                "include sifr.toml and the configured Sifr source roots in Cargo package metadata"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn publish_validation_failed(
        cargo_package_id: &CargoPackageId,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_PUBLISH_VALIDATION_FAILED,
            message: format!("package publish validation failed: {}", reason.into()),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some("fix package metadata before publishing or packaging".to_string()),
        }
    }

    #[must_use]
    pub fn include_exclude_omits_source(cargo_package_id: &CargoPackageId, path: &Path) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE,
            message: format!(
                "Cargo package include/exclude rules omit required Sifr file '{}'",
                path.display()
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some(
                "update Cargo.toml include/exclude so sifr.toml and .sifr sources are packaged"
                    .to_string(),
            ),
        }
    }
}
