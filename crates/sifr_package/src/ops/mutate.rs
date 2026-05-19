use crate::ops::plan::{OperationPlan, PackageOperation};

#[must_use]
pub fn manifest_mutation_operation(operation: PackageOperation) -> OperationPlan {
    OperationPlan {
        operation,
        lock_mode: Default::default(),
        mutates_manifests: true,
        mutates_lockfile: true,
    }
}
