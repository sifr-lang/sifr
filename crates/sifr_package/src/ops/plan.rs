use crate::cargo::lock_modes::CargoLockMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationPlan {
    pub operation: PackageOperation,
    pub lock_mode: CargoLockMode,
    pub mutates_manifests: bool,
    pub mutates_lockfile: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageOperation {
    ReadGraph,
    Add,
    Remove,
    Update,
    Package,
    Publish,
    Vendor,
}

impl OperationPlan {
    #[must_use]
    pub const fn violates_lock_mode(&self) -> bool {
        (self.mutates_manifests && matches!(self.lock_mode, CargoLockMode::Frozen))
            || (self.mutates_lockfile && self.lock_mode.is_lock_mutation_disallowed())
    }
}
