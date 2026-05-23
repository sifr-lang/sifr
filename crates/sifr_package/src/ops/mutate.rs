use crate::ops::plan::{OperationPlan, PackageOperation};

#[must_use]
pub fn manifest_mutation_operation(operation: PackageOperation) -> OperationPlan {
    OperationPlan {
        operation,
        lock_mode: Default::default(),
        mutates_manifests: true,
        mutates_lockfile: true,
        requires_network: false,
        writes_projection: true,
        manifest_less_mode: false,
    }
}
