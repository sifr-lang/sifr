use crate::cargo::lock_modes::CargoLockMode;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageMetadata;
use crate::ops::session::PackageSession;
use crate::ops::session_targets::discover_app_targets;
use sifr_frontend::SourceProvider;
use std::path::PathBuf;

impl PackageSession {
    /// Prove that an explicit file belongs only to a source workspace, not a
    /// Cargo package. Virtual-workspace members do not own such a file.
    /// Uncertain ownership remains on the ordinary package-resolution path.
    pub fn standalone_file_in_virtual_workspace(
        cwd: &std::path::Path,
        file: &std::path::Path,
        provider: &mut impl SourceProvider,
    ) -> bool {
        let Some(manifest) = super::session_discovery::find_manifest(cwd, provider) else {
            return false;
        };
        let Some(root) = manifest.parent() else {
            return false;
        };
        let table = |text: &str| text.parse::<toml::Table>().ok();
        let Ok(source) = provider.read_file(&manifest) else {
            return false;
        };
        let Some(config) = table(source.as_str()) else {
            return false;
        };
        // These source-workspace fields carry no package backend, component,
        // Python or dependency authority. Other configurations resolve normally.
        if config
            .keys()
            .any(|key| !matches!(key.as_str(), "package" | "source"))
        {
            return false;
        }
        let Some(package) = config.get("package").and_then(toml::Value::as_table) else {
            return false;
        };
        if package.keys().any(|key| {
            !matches!(
                key.as_str(),
                "name" | "version" | "edition" | "sifr-version"
            )
        }) {
            return false;
        }
        let id = super::session_discovery::session_cargo_id(root);
        let Ok(source_manifest) = crate::SifrManifest::load(&id, &manifest, provider) else {
            return false;
        };
        let Ok(source_root) = provider.canonicalize(&root.join(&source_manifest.source_root.0))
        else {
            return false;
        };
        let Ok(cargo) = provider.read_file(&root.join("Cargo.toml")) else {
            return false;
        };
        let Some(cargo) = table(cargo.as_str()) else {
            return false;
        };
        if cargo.contains_key("package")
            || !cargo.get("workspace").is_some_and(toml::Value::is_table)
        {
            return false;
        }
        let (Ok(root), Ok(file)) = (provider.canonicalize(root), provider.canonicalize(file))
        else {
            return false;
        };
        if !file.starts_with(&root) || !file.starts_with(&source_root) {
            return false;
        }
        let Some(parent) = file.parent() else {
            return false;
        };
        for directory in parent.ancestors() {
            if directory == root {
                return true;
            }
            // A nested Cargo or Sifr manifest may carry actual package/source
            // authority. Do not infer its absence from workspace membership.
            if provider.is_file(&directory.join("Cargo.toml"))
                || provider.is_file(&directory.join("sifr.toml"))
            {
                return false;
            }
        }
        false
    }

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
