use crate::cargo::metadata::CargoPackageId;
use crate::diag::{PackageDiagnostic, PackageDiagnosticOrigin};
use sifr_diagnostics::DiagnosticCode;
use std::path::Path;

impl PackageDiagnostic {
    #[must_use]
    pub fn invalid_python_bridge_source(
        cargo_package_id: &CargoPackageId,
        path: &Path,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE,
            message: format!("invalid package-local Python bridge: {}", reason.into()),
            origin: Box::new(PackageDiagnosticOrigin::PythonBridgeSource {
                cargo_package_id: cargo_package_id.clone(),
                path: path.to_path_buf(),
            }),
            help: Some(
                "keep bridge modules under src/python_bridges, use static imports, and remove dynamic import calls"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn python_environment_config(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
        key: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code: DiagnosticCode::PYENV_INVALID_CONFIG,
            message: format!(
                "invalid Python environment configuration in '{}': {}",
                manifest_path.display(),
                reason.into()
            ),
            origin: Box::new(PackageDiagnosticOrigin::SifrManifest {
                cargo_package_id: cargo_package_id.clone(),
                path: manifest_path.to_path_buf(),
                key: Some(key.to_string()),
            }),
            help: Some(
                "configure the root application [python] table with relative paths and static import roots"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub fn python_environment_graph(
        code: DiagnosticCode,
        message: impl Into<String>,
        cargo_package_id: Option<CargoPackageId>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata { cargo_package_id }),
            help: Some(help.into()),
        }
    }

    #[must_use]
    pub fn python_trust_graph(
        code: DiagnosticCode,
        message: impl Into<String>,
        cargo_package_id: Option<CargoPackageId>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata { cargo_package_id }),
            help: Some(help.into()),
        }
    }

    #[must_use]
    pub fn python_environment_probe(
        code: DiagnosticCode,
        interpreter: &Path,
        venv: &Path,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            origin: Box::new(PackageDiagnosticOrigin::CargoMetadata {
                cargo_package_id: None,
            }),
            help: Some(format!(
                "{} Selected interpreter: '{}'; venv: '{}'.",
                help.into(),
                interpreter.display(),
                venv.display()
            )),
        }
    }
}
