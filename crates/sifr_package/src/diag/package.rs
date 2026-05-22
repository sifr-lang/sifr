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
