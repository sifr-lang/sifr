use crate::cargo::metadata::{CargoPackageId, NormalizedCargoMetadata};

#[must_use]
pub fn selected_workspace_members(metadata: &NormalizedCargoMetadata) -> Vec<CargoPackageId> {
    metadata.workspace_members.iter().cloned().collect()
}
