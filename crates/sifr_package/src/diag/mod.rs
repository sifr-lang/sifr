mod package;
mod python;

use crate::cargo::metadata::CargoPackageId;
use crate::cargo::{errors::CargoAction, lock_modes::CargoLockMode};
use crate::graph::derive::SifrPackageId;
use crate::manifest::sifr::ImportRoot;
use sifr_diagnostics::DiagnosticCode;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDiagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub origin: Box<PackageDiagnosticOrigin>,
    pub help: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageDiagnosticOrigin {
    CargoMetadata {
        cargo_package_id: Option<CargoPackageId>,
    },
    CargoManifest {
        cargo_package_id: CargoPackageId,
        path: PathBuf,
        key: Option<String>,
    },
    SifrManifest {
        cargo_package_id: CargoPackageId,
        path: PathBuf,
        key: Option<String>,
    },
    RustMarker {
        cargo_package_id: CargoPackageId,
        path: PathBuf,
    },
    PythonBridgeSource {
        cargo_package_id: CargoPackageId,
        path: PathBuf,
    },
    PackageGraph {
        cargo_package_id: CargoPackageId,
    },
    CargoCommand {
        action: String,
    },
}

impl PackageDiagnostic {
    #[must_use]
    pub fn cargo_metadata_parse(reason: &str) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_METADATA_PARSE,
            message: format!("could not parse Cargo package graph metadata: {reason}"),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(
                "inspect Cargo's selected package graph with the package-management rules metadata adapter"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn cargo_command_failed(action: CargoAction, reason: impl Into<String>) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED,
            message: format!("cargo {} failed: {}", action.as_str(), reason.into()),
            origin: Box::new(PackageDiagnosticOrigin::CargoCommand {
                action: action.as_str().to_string(),
            }),
            help: Some("rerun the printed Cargo command for the full backend error".to_string()),
        }
    }

    #[must_use]
    pub fn source_unavailable_offline(
        cargo_package_id: &CargoPackageId,
        package_path: &Path,
        lock_mode: CargoLockMode,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_SOURCE_UNAVAILABLE_OFFLINE,
            message: format!(
                "package source '{}' is unavailable in {:?} mode",
                package_path.display(),
                lock_mode
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some(
                "run `sifr fetch` without --offline/--frozen before building offline".to_string(),
            ),
        }
    }

    #[must_use]
    pub fn invalid_cargo_sifr_metadata(
        cargo_package_id: &CargoPackageId,
        cargo_package_name: &str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_MISSING_OR_INVALID_CARGO_METADATA,
            message: format!(
                "invalid [package.metadata.sifr] for Cargo package '{cargo_package_name}': {}",
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some("expected `[package.metadata.sifr] manifest = \"sifr.toml\"`".to_string()),
        }
    }

    #[must_use]
    pub fn missing_sifr_manifest(
        cargo_package_id: &CargoPackageId,
        manifest_path: PathBuf,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST,
            message: format!(
                "could not load Sifr manifest '{}': {}",
                manifest_path.display(),
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path,
                key: None,
            }),
            help: Some(
                "make sure `[package.metadata.sifr].manifest` points to a committed sifr.toml file"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn invalid_sifr_manifest(
        cargo_package_id: &CargoPackageId,
        manifest_path: PathBuf,
        key: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let key = key.into();
        Self {
            code: DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST,
            message: format!(
                "invalid sifr.toml key '{key}' at '{}': {}",
                manifest_path.display(),
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path,
                key: Some(key),
            }),
            help: None,
        }
    }

    #[must_use]
    pub fn misplaced_sifr_metadata(
        cargo_package_id: &CargoPackageId,
        cargo_package_name: &str,
        key: impl Into<String>,
    ) -> Self {
        let key = key.into();
        Self {
            code: DiagnosticCode::PACKAGE_UNSUPPORTED_CARGO_SIFR_METADATA,
            message: format!(
                "unsupported Sifr compiler metadata key '{key}' appears in Cargo metadata for '{cargo_package_name}'"
            ),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: Some(cargo_package_id.clone()),
            }),
            help: Some("move compiler semantics to sifr.toml; Cargo metadata is only a discovery hook".to_string()),
        }
    }

    #[must_use]
    pub fn non_trivial_pure_marker(
        cargo_package_id: &CargoPackageId,
        marker_path: PathBuf,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_NON_TRIVIAL_PURE_MARKER,
            message: format!(
                "pure Sifr package marker '{}' contains Rust implementation: {}",
                marker_path.display(),
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::RustMarker {
                cargo_package_id: cargo_package_id.clone(),
                path: marker_path,
            }),
            help: Some("pure Sifr packages must keep Rust marker targets comment-only; declare Rust-backed Sifr when Rust implementation is intentional".to_string()),
        }
    }

    #[must_use]
    pub fn ambiguous_import_root(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
        import_root: &ImportRoot,
        candidates: &[String],
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_AMBIGUOUS_IMPORT_ROOT,
            message: format!(
                "ambiguous package import root '{}' in '{}': {}",
                import_root.0,
                package_id.0,
                candidates.join(", ")
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some(
                "add [package.metadata.sifr.aliases] entries for direct dependencies that export the same import root"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn type_identity_mismatch(
        cargo_package_id: &CargoPackageId,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PACKAGE_TYPE_IDENTITY_MISMATCH,
            message: format!(
                "package type identity mismatch: expected {}, got {}",
                expected.into(),
                actual.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some(
                "types from different resolved package instances are distinct even when their module and type names match"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn undeclared_direct_import(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
        import_path: impl Into<String>,
    ) -> Self {
        let import_path = import_path.into();
        Self {
            code: DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT,
            message: format!(
                "package '{}' imports '{import_path}', which is not in its own sources or direct dependency scope",
                package_id.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some(
                "add a direct Cargo dependency, import through an exported alias, or move the import inside the dependency package"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn private_module_access(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
        import_path: impl Into<String>,
        target_package_id: &SifrPackageId,
    ) -> Self {
        let import_path = import_path.into();
        Self {
            code: DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS,
            message: format!(
                "package '{}' imports private module '{import_path}' from '{}'",
                package_id.0, target_package_id.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some(
                "import only modules exported by the dependency package, or add an explicit public re-export"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn backend_trust_violation(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
        backend_name: impl Into<String>,
    ) -> Self {
        let backend_name = backend_name.into();
        Self {
            code: DiagnosticCode::PACKAGE_BACKEND_TRUST_VIOLATION,
            message: format!(
                "package '{}' depends on untrusted backend crate '{backend_name}'",
                package_id.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some("list intentional backend crates in sifr.toml [trust].native".to_string()),
        }
    }

    #[must_use]
    pub fn trust_non_direct_dependency(
        cargo_package_id: &CargoPackageId,
        package_id: &SifrPackageId,
        backend_name: impl Into<String>,
    ) -> Self {
        let backend_name = backend_name.into();
        Self {
            code: DiagnosticCode::PACKAGE_TRUST_NON_DIRECT_DEPENDENCY,
            message: format!(
                "package '{}' trusts backend crate '{backend_name}', but it is not a direct backend dependency",
                package_id.0
            ),
            origin: Box::new(PackageDiagnosticOrigin::PackageGraph {
                cargo_package_id: cargo_package_id.clone(),
            }),
            help: Some(
                "remove unused trust entries or add the backend crate as a direct Cargo dependency"
                    .to_string(),
            ),
        }
    }
}
