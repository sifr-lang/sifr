use crate::cargo::lock_modes::CargoLockMode;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageMetadata;
use crate::ops::session::PackageSession;
use crate::ops::session_targets::discover_app_targets;
use sifr_frontend::SourceProvider;
use std::path::PathBuf;

impl PackageSession {
    #[must_use]
    pub fn package_id(&self, graph: &crate::SifrPackageGraph) -> Option<crate::SifrPackageId> {
        let manifest_path = self.manifest_path.as_ref()?;
        graph
            .packages
            .values()
            .find(|package| same_path(&package.sifr_manifest, manifest_path))
            .map(|package| package.package_id.clone())
    }

    pub fn from_package_metadata(
        workspace_root: PathBuf,
        package: &SifrPackageMetadata,
        lock_mode: CargoLockMode,
        provider: &mut impl SourceProvider,
    ) -> Self {
        let source_root_path = package.package_root.join(&package.manifest.source_root.0);
        let app_targets = discover_app_targets(
            &source_root_path,
            &package.manifest.package_name.0,
            provider,
        );
        Self {
            workspace_root,
            manifest_path: Some(package.sifr_manifest.clone()),
            source_root: Some(source_root_path),
            manifest_less_mode: false,
            lock_mode,
            manifest: Some(package.manifest.clone()),
            app_targets,
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

    /// Return every runnable application entrypoint in deterministic target order.
    ///
    /// Read-only package inspection uses this instead of default-target selection so
    /// a package with multiple applications is still treated as a final application.
    pub fn runnable_app_paths(&self) -> Result<Vec<PathBuf>, PackageDiagnostic> {
        Ok(self
            .discover_app_targets()?
            .into_iter()
            .map(|target| target.path)
            .collect())
    }
}

fn same_path(left: &std::path::Path, right: &std::path::Path) -> bool {
    left.canonicalize().unwrap_or_else(|_| left.to_path_buf())
        == right.canonicalize().unwrap_or_else(|_| right.to_path_buf())
}
