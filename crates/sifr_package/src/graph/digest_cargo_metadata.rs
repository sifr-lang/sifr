use super::digest::{GraphDigest, digest_serializable};
use crate::cargo::metadata::NormalizedCargoMetadata;

#[must_use]
pub fn digest_graph_inputs(metadata: &NormalizedCargoMetadata) -> GraphDigest {
    // Normalization orders every package, edge, dependency and target. Serialize
    // the complete resolved projection so a new consumed field is bound by default.
    digest_serializable("cargo-metadata", metadata)
}
