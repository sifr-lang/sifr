use crate::cargo::lock_modes::CargoLockMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationPlan {
    pub operation: PackageOperation,
    pub lock_mode: CargoLockMode,
    pub mutates_manifests: bool,
    pub mutates_lockfile: bool,
    pub requires_network: bool,
    pub writes_projection: bool,
    pub manifest_less_mode: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageOperation {
    Run,
    Check,
    Test,
    Fetch,
    Tree,
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
        ((self.mutates_manifests || self.writes_projection)
            && matches!(self.lock_mode, CargoLockMode::Frozen))
            || (self.mutates_lockfile && self.lock_mode.is_lock_mutation_disallowed())
            || (self.requires_network && self.lock_mode.is_network_disallowed())
    }
}

impl OperationPlan {
    #[must_use]
    pub const fn read_only(operation: PackageOperation, lock_mode: CargoLockMode) -> Self {
        Self {
            operation,
            lock_mode,
            mutates_manifests: false,
            mutates_lockfile: false,
            requires_network: false,
            writes_projection: false,
            manifest_less_mode: false,
        }
    }

    #[must_use]
    pub const fn manifest_less(operation: PackageOperation) -> Self {
        Self {
            operation,
            lock_mode: CargoLockMode::Normal,
            mutates_manifests: false,
            mutates_lockfile: false,
            requires_network: false,
            writes_projection: false,
            manifest_less_mode: true,
        }
    }
}
