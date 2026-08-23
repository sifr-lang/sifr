use super::{ShapeNode, canonical_node};

pub(super) fn canonical_sequence(kind: &str, elements: &[ShapeNode]) -> String {
    format!(
        "{kind}[{}]",
        elements
            .iter()
            .map(canonical_node)
            .collect::<Vec<_>>()
            .join(",")
    )
}
