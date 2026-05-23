use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageSelection,
    CargoPublishOptions, CargoVendorOptions,
};
use crate::cargo::lock_modes::CargoLockMode;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishPlan {
    pub dry_run: bool,
    pub cargo_command: CargoCommandPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendorPlan {
    pub output_dir: PathBuf,
    pub cargo_command: CargoCommandPlan,
}

#[must_use]
pub fn publish_plan(package_root: PathBuf, lock_mode: CargoLockMode, dry_run: bool) -> PublishPlan {
    PublishPlan {
        dry_run,
        cargo_command: CargoCommandPlan::publish(package_root, lock_mode, dry_run),
    }
}

#[must_use]
pub fn package_plan(
    workspace_root: PathBuf,
    lock_mode: CargoLockMode,
    features: &CargoFeatureSelection,
    selection: &CargoPackageSelection,
    options: &CargoPackageArchiveOptions,
) -> CargoCommandPlan {
    CargoCommandPlan::package_with_options(workspace_root, lock_mode, features, selection, options)
}

#[must_use]
pub fn publish_plan_with_options(
    workspace_root: PathBuf,
    lock_mode: CargoLockMode,
    features: &CargoFeatureSelection,
    selection: &CargoPackageSelection,
    options: &CargoPublishOptions,
) -> PublishPlan {
    PublishPlan {
        dry_run: options.dry_run,
        cargo_command: CargoCommandPlan::publish_with_options(
            workspace_root,
            lock_mode,
            features,
            selection,
            options,
        ),
    }
}

#[must_use]
pub fn vendor_plan(
    workspace_root: PathBuf,
    lock_mode: CargoLockMode,
    output_dir: PathBuf,
) -> VendorPlan {
    let cargo_command = CargoCommandPlan::vendor(workspace_root, lock_mode, &output_dir);
    VendorPlan {
        output_dir,
        cargo_command,
    }
}

#[must_use]
pub fn vendor_plan_with_options(
    workspace_root: PathBuf,
    lock_mode: CargoLockMode,
    output_dir: PathBuf,
    options: &CargoVendorOptions,
) -> VendorPlan {
    let cargo_command =
        CargoCommandPlan::vendor_with_options(workspace_root, lock_mode, &output_dir, options);
    VendorPlan {
        output_dir,
        cargo_command,
    }
}
