use crate::cargo::lock_modes::CargoLockMode;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageMetadata;
use crate::ops::session::PackageSession;
use std::path::PathBuf;

impl PackageSession {
    #[must_use]
    pub fn from_package_metadata(
        workspace_root: PathBuf,
        package: &SifrPackageMetadata,
        lock_mode: CargoLockMode,
    ) -> Self {
        let source_roots = package
            .manifest
            .source_roots
            .iter()
            .map(|root| package.package_root.join(&root.0))
            .collect::<Vec<_>>();
        let source_root = source_roots.first().cloned();
        Self {
            workspace_root,
            manifest_path: Some(package.sifr_manifest.clone()),
            source_root,
            source_roots,
            manifest_less_mode: false,
            lock_mode,
            manifest: Some(package.manifest.clone()),
        }
    }

    pub fn has_default_runnable_app(&self) -> Result<bool, PackageDiagnostic> {
        if let Some(manifest) = &self.manifest {
            if let Some(default_run) = manifest.default_run.as_deref() {
                self.find_app_target(default_run)?;
                return Ok(true);
            }
        }
        self.default_app_target().map(|target| target.is_some())
    }
}
