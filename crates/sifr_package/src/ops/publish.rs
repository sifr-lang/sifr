use crate::cargo::commands::CargoCommandPlan;
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
