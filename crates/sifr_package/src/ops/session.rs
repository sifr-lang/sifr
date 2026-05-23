use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageSelection,
    CargoPublishOptions, CargoVendorOptions,
};
use crate::cargo::lock_modes::CargoLockMode;
use crate::diag::PackageDiagnostic;
use crate::manifest::package_sections::SifrScript;
use crate::manifest::sifr::SifrManifest;
use crate::ops::plan::{OperationPlan, PackageOperation};
use crate::ops::session_discovery::{find_manifest, session_cargo_id};
use crate::ops::session_targets::{discover_app_targets, AppTarget};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageSession {
    pub workspace_root: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub source_root: Option<PathBuf>,
    pub source_roots: Vec<PathBuf>,
    pub manifest_less_mode: bool,
    pub lock_mode: CargoLockMode,
    pub manifest: Option<SifrManifest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageSessionOptions {
    pub current_dir: PathBuf,
    pub lock_mode: CargoLockMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageRunRequest {
    pub target_or_path: Option<String>,
    pub bin: Option<String>,
    pub script: Option<String>,
    pub app_args: Vec<String>,
    pub script_depth: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageCommandPlan {
    pub operation: OperationPlan,
    pub cargo: Option<CargoCommandPlan>,
    pub run_target: Option<ResolvedRunTarget>,
    pub script_origin: Option<ScriptOrigin>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedRunTarget {
    File(PathBuf),
    App { name: String, path: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptOrigin {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl PackageSession {
    pub fn discover(options: PackageSessionOptions) -> Result<Self, PackageDiagnostic> {
        let manifest_path = find_manifest(&options.current_dir);
        let Some(manifest_path) = manifest_path else {
            return Ok(Self {
                workspace_root: options.current_dir,
                manifest_path: None,
                source_root: None,
                source_roots: Vec::new(),
                manifest_less_mode: true,
                lock_mode: options.lock_mode,
                manifest: None,
            });
        };
        let workspace_root = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| options.current_dir.clone());
        let cargo_id = session_cargo_id(&workspace_root);
        let manifest = SifrManifest::load(&cargo_id, &manifest_path)?;
        let source_roots = manifest
            .source_roots
            .iter()
            .map(|root| workspace_root.join(&root.0))
            .collect::<Vec<_>>();
        let source_root = source_roots.first().cloned();
        Ok(Self {
            workspace_root,
            manifest_path: Some(manifest_path),
            source_root,
            source_roots,
            manifest_less_mode: false,
            lock_mode: options.lock_mode,
            manifest: Some(manifest),
        })
    }

    #[must_use]
    pub fn plan_fetch(&self) -> PackageCommandPlan {
        let mut operation = OperationPlan::read_only(PackageOperation::Fetch, self.lock_mode);
        operation.requires_network = !self.lock_mode.is_network_disallowed();
        PackageCommandPlan {
            operation,
            cargo: Some(CargoCommandPlan::fetch(
                self.workspace_root.clone(),
                self.lock_mode,
            )),
            run_target: None,
            script_origin: None,
        }
    }

    #[must_use]
    pub fn plan_tree(&self, forwarded: &[String]) -> PackageCommandPlan {
        PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Tree, self.lock_mode),
            cargo: Some(CargoCommandPlan::tree(
                self.workspace_root.clone(),
                self.lock_mode,
                forwarded,
            )),
            run_target: None,
            script_origin: None,
        }
    }

    pub fn plan_check(
        &self,
        explicit_file: Option<&Path>,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
    ) -> Result<PackageCommandPlan, PackageDiagnostic> {
        if let Some(file) = explicit_file {
            self.validate_explicit_file(file)?;
            return Ok(PackageCommandPlan {
                operation: explicit_file_operation(PackageOperation::Check, self),
                cargo: None,
                run_target: Some(ResolvedRunTarget::File(file.to_path_buf())),
                script_origin: None,
            });
        }
        Ok(PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Check, self.lock_mode),
            cargo: Some(CargoCommandPlan::check(
                self.workspace_root.clone(),
                self.lock_mode,
                features,
                selection,
                None,
            )),
            run_target: None,
            script_origin: None,
        })
    }

    pub fn plan_package(
        &self,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
        options: &CargoPackageArchiveOptions,
    ) -> PackageCommandPlan {
        PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Package, self.lock_mode),
            cargo: Some(CargoCommandPlan::package_with_options(
                self.workspace_root.clone(),
                self.lock_mode,
                features,
                selection,
                options,
            )),
            run_target: None,
            script_origin: None,
        }
    }

    pub fn plan_publish(
        &self,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
        options: &CargoPublishOptions,
    ) -> PackageCommandPlan {
        PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Publish, self.lock_mode),
            cargo: Some(CargoCommandPlan::publish_with_options(
                self.workspace_root.clone(),
                self.lock_mode,
                features,
                selection,
                options,
            )),
            run_target: None,
            script_origin: None,
        }
    }

    pub fn plan_vendor(
        &self,
        output_dir: &Path,
        options: &CargoVendorOptions,
    ) -> PackageCommandPlan {
        PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Vendor, self.lock_mode),
            cargo: Some(CargoCommandPlan::vendor_with_options(
                self.workspace_root.clone(),
                self.lock_mode,
                output_dir,
                options,
            )),
            run_target: None,
            script_origin: None,
        }
    }

    pub fn plan_run(
        &self,
        request: &PackageRunRequest,
    ) -> Result<PackageCommandPlan, PackageDiagnostic> {
        if let Some(script) = request.script.as_deref() {
            return self.plan_script(script, request.script_depth);
        }
        if let Some(bin) = request.bin.as_deref() {
            let target = self.find_app_target(bin)?;
            return Ok(self.app_target_plan(target, &request.app_args));
        }
        if let Some(target_or_path) = request.target_or_path.as_deref() {
            if is_explicit_sifr_path(target_or_path) {
                let file = self.workspace_root.join(target_or_path);
                self.validate_explicit_file(&file)?;
                return Ok(PackageCommandPlan {
                    operation: explicit_file_operation(PackageOperation::Run, self),
                    cargo: None,
                    run_target: Some(ResolvedRunTarget::File(file)),
                    script_origin: None,
                });
            }
            let app = self.lookup_app_target(target_or_path)?;
            let script = self.lookup_script(target_or_path);
            match (app, script) {
                (Some(_app), Some(_)) => {
                    let candidates = vec![
                        format!("bin:{target_or_path}"),
                        format!("script:{target_or_path}"),
                    ];
                    Err(PackageDiagnostic::run_target_ambiguous(
                        target_or_path,
                        &candidates,
                    ))
                }
                (Some(app), None) => Ok(self.app_target_plan(app, &request.app_args)),
                (None, Some(_)) => self.plan_script(target_or_path, request.script_depth),
                (None, None) => Err(PackageDiagnostic::run_target_ambiguous(target_or_path, &[])),
            }
        } else if let Some(manifest) = &self.manifest {
            if let Some(default_run) = manifest.default_run.as_deref() {
                let target = self.find_app_target(default_run)?;
                Ok(self.app_target_plan(target, &request.app_args))
            } else if let Some(target) = self.default_app_target()? {
                Ok(self.app_target_plan(target, &request.app_args))
            } else {
                Err(PackageDiagnostic::run_target_ambiguous("<default>", &[]))
            }
        } else {
            Err(PackageDiagnostic::run_target_ambiguous("<default>", &[]))
        }
    }

    fn app_target_plan(&self, target: AppTarget, app_args: &[String]) -> PackageCommandPlan {
        PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Run, self.lock_mode),
            cargo: Some(CargoCommandPlan::run(
                self.workspace_root.clone(),
                self.lock_mode,
                &CargoFeatureSelection::default(),
                Some(&target.name),
                app_args,
            )),
            run_target: Some(ResolvedRunTarget::App {
                name: target.name,
                path: target.path,
            }),
            script_origin: None,
        }
    }

    fn plan_script(&self, name: &str, depth: u8) -> Result<PackageCommandPlan, PackageDiagnostic> {
        if depth > 0 {
            return Err(PackageDiagnostic::script_recursion(name));
        }
        let Some(script) = self.lookup_script(name) else {
            return Err(PackageDiagnostic::run_target_ambiguous(name, &[]));
        };
        if script.command == "script"
            || script.args.iter().any(|arg| arg == "--script")
            || script.command == "run"
                && script
                    .args
                    .first()
                    .is_some_and(|arg| self.lookup_script(arg).is_some())
        {
            return Err(PackageDiagnostic::script_recursion(name));
        }
        Ok(PackageCommandPlan {
            operation: OperationPlan::read_only(PackageOperation::Run, self.lock_mode),
            cargo: None,
            run_target: None,
            script_origin: Some(ScriptOrigin {
                name: name.to_string(),
                command: script.command.clone(),
                args: script.args.clone(),
            }),
        })
    }

    fn validate_explicit_file(&self, file: &Path) -> Result<(), PackageDiagnostic> {
        if self.manifest_less_mode {
            return Ok(());
        }
        if self.source_roots.is_empty() {
            return Ok(());
        }
        if self
            .source_roots
            .iter()
            .any(|source_root| path_is_under(file, source_root))
        {
            Ok(())
        } else {
            let source_root = self.source_roots.first().unwrap_or(&self.workspace_root);
            Err(PackageDiagnostic::explicit_file_outside_source_root(
                file,
                source_root,
            ))
        }
    }

    fn lookup_script(&self, name: &str) -> Option<&SifrScript> {
        self.manifest.as_ref()?.scripts.get(name)
    }

    fn find_app_target(&self, name: &str) -> Result<AppTarget, PackageDiagnostic> {
        self.lookup_app_target(name)?
            .ok_or_else(|| PackageDiagnostic::run_target_ambiguous(name, &[]))
    }

    fn lookup_app_target(&self, name: &str) -> Result<Option<AppTarget>, PackageDiagnostic> {
        Ok(self
            .discover_app_targets()?
            .into_iter()
            .find(|target| target.name == name))
    }

    fn default_app_target(&self) -> Result<Option<AppTarget>, PackageDiagnostic> {
        let targets = self.discover_app_targets()?;
        if let Some(main) = targets
            .iter()
            .find(|target| target.path.ends_with("main.sifr"))
        {
            return Ok(Some(main.clone()));
        }
        if targets.len() == 1 {
            Ok(targets.into_iter().next())
        } else {
            Ok(None)
        }
    }

    fn discover_app_targets(&self) -> Result<Vec<AppTarget>, PackageDiagnostic> {
        if self.source_roots.is_empty() {
            return Ok(Vec::new());
        }
        let package_name = self
            .manifest
            .as_ref()
            .map(|manifest| manifest.package_name.0.clone())
            .unwrap_or_else(|| "main".to_string());
        let targets = discover_app_targets(&self.source_roots, &package_name)?;
        Ok(targets)
    }
}

fn explicit_file_operation(operation: PackageOperation, session: &PackageSession) -> OperationPlan {
    if session.manifest_less_mode {
        OperationPlan::manifest_less(operation)
    } else {
        OperationPlan::read_only(operation, session.lock_mode)
    }
}

fn is_explicit_sifr_path(value: &str) -> bool {
    Path::new(value)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("sifr"))
        || value.contains('/')
        || value.contains(std::path::MAIN_SEPARATOR)
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    path.starts_with(root)
}
