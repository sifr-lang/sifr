use super::{
    NodeId, ShapeIdentity, StructuralContractError, StructuralKind, StructuralNodeEdge,
    StructuralNodeRef, StructuralScalar, StructuralSource,
};

/// One owned node in a validated structural source.
#[derive(Debug)]
pub struct ArenaNode {
    kind: StructuralKind,
    nominal_identity: Option<&'static str>,
    edges: Vec<StructuralNodeEdge<'static>>,
    scalar: Option<StructuralScalar>,
}

impl ArenaNode {
    #[must_use]
    pub fn scalar(kind: StructuralKind, scalar: StructuralScalar) -> Self {
        Self {
            kind,
            nominal_identity: None,
            edges: Vec::new(),
            scalar: Some(scalar),
        }
    }

    #[must_use]
    pub fn aggregate(
        kind: StructuralKind,
        nominal_identity: Option<&'static str>,
        edges: Vec<StructuralNodeEdge<'static>>,
    ) -> Self {
        Self {
            kind,
            nominal_identity,
            edges,
            scalar: None,
        }
    }
}

/// A checked, move-only arena that can construct one structural value.
#[derive(Debug)]
pub struct StructuralArena {
    shape: ShapeIdentity,
    root: NodeId,
    nodes: Vec<ArenaNode>,
}

impl StructuralArena {
    pub fn seal(
        shape: ShapeIdentity,
        root: NodeId,
        nodes: Vec<ArenaNode>,
    ) -> Result<Self, StructuralContractError> {
        if nodes.get(root.get() as usize).is_none() {
            return Err(StructuralContractError::InvalidNode);
        }
        for node in &nodes {
            validate_node(node, nodes.len())?;
        }
        validate_acyclic(&nodes)?;
        Ok(Self { shape, root, nodes })
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl StructuralSource for StructuralArena {
    fn shape_identity(&self) -> ShapeIdentity {
        self.shape
    }

    fn root(&self) -> NodeId {
        self.root
    }

    fn node(&self, id: NodeId) -> Result<StructuralNodeRef<'_>, StructuralContractError> {
        let node = self
            .nodes
            .get(id.get() as usize)
            .ok_or(StructuralContractError::InvalidNode)?;
        if node.scalar.is_some() {
            Ok(StructuralNodeRef::scalar(node.kind))
        } else {
            Ok(StructuralNodeRef::aggregate(
                node.kind,
                node.nominal_identity,
                &node.edges,
            ))
        }
    }

    fn take_scalar(&mut self, id: NodeId) -> Result<StructuralScalar, StructuralContractError> {
        self.nodes
            .get_mut(id.get() as usize)
            .ok_or(StructuralContractError::InvalidNode)?
            .scalar
            .take()
            .ok_or(StructuralContractError::AlreadyMoved)
    }
}

fn validate_node(node: &ArenaNode, node_count: usize) -> Result<(), StructuralContractError> {
    let scalar_kind = matches!(
        node.kind,
        StructuralKind::None
            | StructuralKind::Bool
            | StructuralKind::SignedInteger
            | StructuralKind::UnsignedInteger
            | StructuralKind::ExactInteger
            | StructuralKind::Float
            | StructuralKind::String
            | StructuralKind::Bytes
    );
    let scalar_matches = matches!(
        (node.kind, node.scalar.as_ref()),
        (StructuralKind::None, Some(StructuralScalar::None))
            | (StructuralKind::Bool, Some(StructuralScalar::Bool(_)))
            | (
                StructuralKind::SignedInteger,
                Some(StructuralScalar::SignedInteger { .. })
            )
            | (
                StructuralKind::UnsignedInteger,
                Some(StructuralScalar::UnsignedInteger { .. })
            )
            | (
                StructuralKind::ExactInteger,
                Some(StructuralScalar::ExactInteger(_))
            )
            | (StructuralKind::Float, Some(StructuralScalar::Float(_)))
            | (StructuralKind::String, Some(StructuralScalar::String(_)))
            | (StructuralKind::Bytes, Some(StructuralScalar::Bytes(_)))
    );
    if scalar_kind != node.scalar.is_some()
        || (scalar_kind && (!scalar_matches || !node.edges.is_empty()))
    {
        return Err(StructuralContractError::ScalarMismatch);
    }
    if node
        .edges
        .iter()
        .any(|edge| edge.node().get() as usize >= node_count)
    {
        return Err(StructuralContractError::InvalidNode);
    }
    Ok(())
}

fn validate_acyclic(nodes: &[ArenaNode]) -> Result<(), StructuralContractError> {
    let mut state = vec![0_u8; nodes.len()];
    for start in 0..nodes.len() {
        if state[start] != 0 {
            continue;
        }
        state[start] = 1;
        let mut stack = vec![(start, 0_usize)];
        while let Some((node_index, edge_index)) = stack.last_mut() {
            let Some(edge) = nodes[*node_index].edges.get(*edge_index) else {
                state[*node_index] = 2;
                stack.pop();
                continue;
            };
            *edge_index += 1;
            let child = edge.node().get() as usize;
            match state[child] {
                0 => {
                    state[child] = 1;
                    stack.push((child, 0));
                }
                1 => return Err(StructuralContractError::CyclicArena),
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::structural::{primitive, structural_construct, StructuralEdgeKind};
    use crate::SifrInt;

    #[test]
    fn seal_rejects_invalid_root_and_edge_indices() {
        assert!(matches!(
            StructuralArena::seal(primitive("str"), NodeId::new(1), Vec::new()),
            Err(StructuralContractError::InvalidNode)
        ));
        let invalid = ArenaNode::aggregate(
            StructuralKind::Sequence,
            None,
            vec![StructuralNodeEdge::new(
                StructuralEdgeKind::Index(0),
                NodeId::new(9),
            )],
        );
        assert!(matches!(
            StructuralArena::seal(primitive("str"), NodeId::new(0), vec![invalid]),
            Err(StructuralContractError::InvalidNode)
        ));
        let mismatched =
            ArenaNode::scalar(StructuralKind::String, StructuralScalar::Bytes(Vec::new()));
        assert!(matches!(
            StructuralArena::seal(primitive("str"), NodeId::new(0), vec![mismatched]),
            Err(StructuralContractError::ScalarMismatch)
        ));
        let cyclic = ArenaNode::aggregate(
            StructuralKind::Sequence,
            None,
            vec![StructuralNodeEdge::new(
                StructuralEdgeKind::Index(0),
                NodeId::new(0),
            )],
        );
        assert!(matches!(
            StructuralArena::seal(primitive("str"), NodeId::new(0), vec![cyclic]),
            Err(StructuralContractError::CyclicArena)
        ));
    }

    #[test]
    fn sealed_arena_moves_scalar_once() {
        let arena = StructuralArena::seal(
            primitive("str"),
            NodeId::new(0),
            vec![ArenaNode::scalar(
                StructuralKind::String,
                StructuralScalar::String("owned".to_string()),
            )],
        )
        .expect("valid arena");
        assert_eq!(
            structural_construct::<String, _>(arena),
            Ok("owned".to_string())
        );
    }

    #[test]
    fn sealed_arena_preserves_exact_integer_without_narrowing() {
        let exact = SifrInt::parse_decimal(
            "123456789012345678901234567890",
            crate::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .expect("fixture exact integer is valid");
        let arena = StructuralArena::seal(
            primitive("int"),
            NodeId::new(0),
            vec![ArenaNode::scalar(
                StructuralKind::ExactInteger,
                StructuralScalar::ExactInteger(exact.clone()),
            )],
        )
        .expect("valid exact-integer arena");
        assert_eq!(structural_construct::<SifrInt, _>(arena), Ok(exact));
    }
}
