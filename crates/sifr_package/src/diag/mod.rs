use crate::cargo::metadata::CargoPackageId;
use sifr_diagnostics::DiagnosticCode;
use std::path::PathBuf;

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
                "inspect Cargo's selected package graph with the Phase 37 metadata adapter"
                    .to_string(),
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
}
