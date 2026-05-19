use crate::cargo::lock_modes::CargoLockMode;
use crate::ops::plan::{OperationPlan, PackageOperation};

#[must_use]
pub const fn read_graph_operation(lock_mode: CargoLockMode) -> OperationPlan {
    OperationPlan {
        operation: PackageOperation::ReadGraph,
        lock_mode,
        mutates_manifests: false,
        mutates_lockfile: false,
    }
}
