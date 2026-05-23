use crate::cargo::errors::CargoAction;
use crate::cargo::lock_modes::CargoLockMode;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoCommandPlan {
    pub action: CargoAction,
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoFeatureSelection {
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoPackageSelection {
    pub workspace: bool,
    pub packages: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoPackageArchiveOptions {
    pub list: bool,
    pub no_verify: bool,
    pub no_metadata: bool,
    pub allow_dirty: bool,
    pub exclude_lockfile: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoPublishOptions {
    pub dry_run: bool,
    pub no_verify: bool,
    pub allow_dirty: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoVendorOptions {
    pub sync: Vec<PathBuf>,
    pub no_delete: bool,
    pub respect_source_config: bool,
    pub versioned_dirs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoPackageMutation {
    pub package_spec: String,
    pub rename: Option<String>,
    pub features: Vec<String>,
}

impl CargoCommandPlan {
    #[must_use]
    pub fn metadata(current_dir: PathBuf, lock_mode: CargoLockMode) -> Self {
        Self::new(
            CargoAction::Metadata,
            current_dir,
            lock_mode,
            vec!["metadata", "--format-version", "1"],
        )
    }

    #[must_use]
    pub fn fetch(current_dir: PathBuf, lock_mode: CargoLockMode) -> Self {
        Self::new(CargoAction::Fetch, current_dir, lock_mode, vec!["fetch"])
    }

    #[must_use]
    pub fn check(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
        target: Option<&str>,
    ) -> Self {
        let mut plan = Self::new(CargoAction::Check, current_dir, lock_mode, vec!["check"]);
        plan.push_package_selection_args(selection);
        plan.push_feature_args(features);
        if let Some(target) = target {
            plan.args
                .extend(["--target".to_string(), target.to_string()]);
        }
        plan
    }

    #[must_use]
    pub fn run(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        bin: Option<&str>,
        app_args: &[String],
    ) -> Self {
        let mut plan = Self::new(CargoAction::Run, current_dir, lock_mode, vec!["run"]);
        plan.push_feature_args(features);
        if let Some(bin) = bin {
            plan.args.extend(["--bin".to_string(), bin.to_string()]);
        }
        if !app_args.is_empty() {
            plan.args.push("--".to_string());
            plan.args.extend(app_args.iter().cloned());
        }
        plan
    }

    #[must_use]
    pub fn test(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        test_args: &[String],
    ) -> Self {
        let mut plan = Self::new(CargoAction::Test, current_dir, lock_mode, vec!["test"]);
        plan.push_feature_args(features);
        if !test_args.is_empty() {
            plan.args.push("--".to_string());
            plan.args.extend(test_args.iter().cloned());
        }
        plan
    }

    #[must_use]
    pub fn build(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        target: Option<&str>,
    ) -> Self {
        let mut plan = Self::new(CargoAction::Build, current_dir, lock_mode, vec!["build"]);
        plan.push_feature_args(features);
        if let Some(target) = target {
            plan.args
                .extend(["--target".to_string(), target.to_string()]);
        }
        plan
    }

    #[must_use]
    pub fn tree(current_dir: PathBuf, lock_mode: CargoLockMode, forwarded: &[String]) -> Self {
        let mut plan = Self::new(CargoAction::Tree, current_dir, lock_mode, vec!["tree"]);
        plan.args.extend(forwarded.iter().cloned());
        plan
    }

    #[must_use]
    pub fn package(current_dir: PathBuf, lock_mode: CargoLockMode) -> Self {
        Self::package_with_options(
            current_dir,
            lock_mode,
            &CargoFeatureSelection::default(),
            &CargoPackageSelection::default(),
            &CargoPackageArchiveOptions::default(),
        )
    }

    #[must_use]
    pub fn package_with_options(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
        options: &CargoPackageArchiveOptions,
    ) -> Self {
        let mut plan = Self::new(
            CargoAction::Package,
            current_dir,
            lock_mode,
            vec!["package"],
        );
        plan.push_package_selection_args(selection);
        plan.push_feature_args(features);
        if options.list {
            plan.args.push("--list".to_string());
        }
        if options.no_verify {
            plan.args.push("--no-verify".to_string());
        }
        if options.no_metadata {
            plan.args.push("--no-metadata".to_string());
        }
        if options.allow_dirty {
            plan.args.push("--allow-dirty".to_string());
        }
        if options.exclude_lockfile {
            plan.args.push("--exclude-lockfile".to_string());
        }
        plan
    }

    #[must_use]
    pub fn publish(current_dir: PathBuf, lock_mode: CargoLockMode, dry_run: bool) -> Self {
        Self::publish_with_options(
            current_dir,
            lock_mode,
            &CargoFeatureSelection::default(),
            &CargoPackageSelection::default(),
            &CargoPublishOptions {
                dry_run,
                ..CargoPublishOptions::default()
            },
        )
    }

    #[must_use]
    pub fn publish_with_options(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        features: &CargoFeatureSelection,
        selection: &CargoPackageSelection,
        options: &CargoPublishOptions,
    ) -> Self {
        let mut plan = Self::new(
            CargoAction::Publish,
            current_dir,
            lock_mode,
            vec!["publish"],
        );
        plan.push_package_selection_args(selection);
        plan.push_feature_args(features);
        if options.dry_run {
            plan.args.push("--dry-run".to_string());
        }
        if options.no_verify {
            plan.args.push("--no-verify".to_string());
        }
        if options.allow_dirty {
            plan.args.push("--allow-dirty".to_string());
        }
        plan
    }

    #[must_use]
    pub fn vendor(current_dir: PathBuf, lock_mode: CargoLockMode, output_dir: &Path) -> Self {
        let mut plan = Self::new(CargoAction::Vendor, current_dir, lock_mode, vec!["vendor"]);
        plan.push_vendor_options(&CargoVendorOptions::default());
        plan.args.push(output_dir.display().to_string());
        plan
    }

    #[must_use]
    pub fn vendor_with_options(
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        output_dir: &Path,
        options: &CargoVendorOptions,
    ) -> Self {
        let mut plan = Self::new(CargoAction::Vendor, current_dir, lock_mode, vec!["vendor"]);
        plan.push_vendor_options(options);
        plan.args.push(output_dir.display().to_string());
        plan
    }

    #[must_use]
    pub fn add(current_dir: PathBuf, mutation: &CargoPackageMutation) -> Self {
        let mut plan = Self::without_lock(CargoAction::Add, current_dir, vec!["add"]);
        plan.args.push(mutation.package_spec.clone());
        if let Some(rename) = &mutation.rename {
            plan.args.extend(["--rename".to_string(), rename.clone()]);
        }
        if !mutation.features.is_empty() {
            plan.args.extend([
                "--features".to_string(),
                stable_csv(mutation.features.iter().map(String::as_str)),
            ]);
        }
        plan
    }

    #[must_use]
    pub fn remove(current_dir: PathBuf, dependency_name: &str) -> Self {
        let mut plan = Self::without_lock(CargoAction::Remove, current_dir, vec!["remove"]);
        plan.args.push(dependency_name.to_string());
        plan
    }

    #[must_use]
    pub fn update(current_dir: PathBuf, lock_mode: CargoLockMode, package: Option<&str>) -> Self {
        let mut plan = Self::new(CargoAction::Update, current_dir, lock_mode, vec!["update"]);
        if let Some(package) = package {
            plan.args.extend(["-p".to_string(), package.to_string()]);
        }
        plan
    }

    fn new(
        action: CargoAction,
        current_dir: PathBuf,
        lock_mode: CargoLockMode,
        args: Vec<&str>,
    ) -> Self {
        let mut plan = Self::without_lock(action, current_dir, args);
        if let Some(arg) = lock_mode.cargo_arg() {
            plan.args.push(arg.to_string());
        }
        plan
    }

    fn without_lock(action: CargoAction, current_dir: PathBuf, args: Vec<&str>) -> Self {
        Self {
            action,
            program: "cargo".to_string(),
            args: args.into_iter().map(str::to_string).collect(),
            current_dir,
        }
    }

    fn push_feature_args(&mut self, features: &CargoFeatureSelection) {
        if features.all_features {
            self.args.push("--all-features".to_string());
        }
        if features.no_default_features {
            self.args.push("--no-default-features".to_string());
        }
        if !features.features.is_empty() {
            self.args.extend([
                "--features".to_string(),
                stable_csv(features.features.iter().map(String::as_str)),
            ]);
        }
    }

    fn push_package_selection_args(&mut self, selection: &CargoPackageSelection) {
        if selection.workspace {
            self.args.push("--workspace".to_string());
        }
        for package in &selection.packages {
            self.args.extend(["-p".to_string(), package.clone()]);
        }
        for exclude in &selection.excludes {
            self.args.extend(["--exclude".to_string(), exclude.clone()]);
        }
    }

    fn push_vendor_options(&mut self, options: &CargoVendorOptions) {
        for manifest in &options.sync {
            self.args
                .extend(["--sync".to_string(), manifest.display().to_string()]);
        }
        if options.no_delete {
            self.args.push("--no-delete".to_string());
        }
        if options.respect_source_config {
            self.args.push("--respect-source-config".to_string());
        }
        if options.versioned_dirs {
            self.args.push("--versioned-dirs".to_string());
        }
    }

    pub fn extend_forwarded_args(&mut self, args: &[String]) {
        self.args.extend(args.iter().cloned());
    }
}

fn stable_csv<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.join(",")
}
