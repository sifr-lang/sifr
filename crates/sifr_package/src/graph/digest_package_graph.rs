use super::digest::{GraphDigest, digest_serializable};
use crate::graph::derive::SifrPackageGraph;

#[must_use]
pub fn digest_package_graph(graph: &SifrPackageGraph) -> GraphDigest {
    // The graph and all nested semantic structures are serialized in full.
    // BTree collections keep the encoding independent of discovery order.
    digest_serializable("package-graph", graph)
}
