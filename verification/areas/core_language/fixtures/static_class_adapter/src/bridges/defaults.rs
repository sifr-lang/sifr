use core::fmt;

use sifr_runtime::interop::structural::{
    structural_construct, ArenaNode, NodeId, StructuralArena, StructuralConstruct,
    StructuralContractError, StructuralEdgeKind, StructuralKind, StructuralNodeEdge,
    StructuralScalar, StructuralType,
};

#[derive(Debug)]
pub struct ContractError {
    message: String,
}

impl ContractError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ContractError {}

pub fn construct_partial<T>() -> Result<T, ContractError>
where
    T: StructuralConstruct + StructuralType,
{
    if !verify_rejections::<T>()? {
        return Err(ContractError::new(
            "structural construction accepted an invalid partial record",
        ));
    }
    let nominal = T::nominal_identity()
        .ok_or_else(|| ContractError::new("default fixture target must be nominal"))?;
    let root = ArenaNode::aggregate(
        StructuralKind::Record,
        Some(nominal),
        vec![StructuralNodeEdge::new(
            StructuralEdgeKind::RecordField("value"),
            NodeId::new(1),
        )],
    );
    let value = ArenaNode::scalar(
        StructuralKind::SignedInteger,
        StructuralScalar::SignedInteger {
            value: 17,
            width: 64,
        },
    );
    let arena = StructuralArena::seal(T::shape_identity(), NodeId::new(0), vec![root, value])
        .map_err(|error| ContractError::new(error.to_string()))?;
    structural_construct(arena).map_err(|error| ContractError::new(error.to_string()))
}

fn verify_rejections<T>() -> Result<bool, ContractError>
where
    T: StructuralConstruct + StructuralType,
{
    let missing = construct_from_edges::<T>(Vec::new())
        .is_err_and(|error| error == StructuralContractError::ArityMismatch);
    let unknown = construct_from_edges::<T>(vec![StructuralNodeEdge::new(
        StructuralEdgeKind::RecordField("unknown"),
        NodeId::new(1),
    )])
    .is_err_and(|error| error == StructuralContractError::MemberMismatch);
    let duplicate = construct_from_edges::<T>(vec![
        StructuralNodeEdge::new(StructuralEdgeKind::RecordField("value"), NodeId::new(1)),
        StructuralNodeEdge::new(StructuralEdgeKind::RecordField("value"), NodeId::new(1)),
    ])
    .is_err_and(|error| error == StructuralContractError::MemberMismatch);
    Ok(missing && unknown && duplicate)
}

fn construct_from_edges<T>(
    edges: Vec<StructuralNodeEdge<'static>>,
) -> Result<T, StructuralContractError>
where
    T: StructuralConstruct + StructuralType,
{
    let nominal = T::nominal_identity().ok_or(StructuralContractError::MemberMismatch)?;
    let root = ArenaNode::aggregate(StructuralKind::Record, Some(nominal), edges);
    let value = ArenaNode::scalar(
        StructuralKind::SignedInteger,
        StructuralScalar::SignedInteger {
            value: 17,
            width: 64,
        },
    );
    let arena = StructuralArena::seal(T::shape_identity(), NodeId::new(0), vec![root, value])?;
    structural_construct(arena)
}
