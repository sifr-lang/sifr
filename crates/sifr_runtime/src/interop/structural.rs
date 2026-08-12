//! Checked construction and allocation-free projection for structural bridges.

use std::fmt;

use crate::SifrInt;

mod arena;
mod implementations;
mod static_program;

pub use arena::{ArenaNode, StructuralArena};
pub use sifr_structural_identity::{
    binary_container, enum_shape, metadata, nominal_record, primitive, recursive_reference,
    refined, static_program_identity, tuple, unary_container, union, NominalField, ShapeIdentity,
    StaticProgramIdentity, ALGORITHM_VERSION, STATIC_PROGRAM_ALGORITHM_VERSION,
};
pub use static_program::{
    StaticProgram, StaticProgramEnvelopeError, StaticProgramHeader, StaticProgramType,
    StaticProgramValue, STATIC_PROGRAM_ENVELOPE_VERSION, STATIC_PROGRAM_FORMAT_VERSION,
    STRUCTURAL_BRIDGE_CONTRACT_VERSION,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(u32);

impl NodeId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConstructToken {
    _private: (),
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralKind {
    None,
    Bool,
    SignedInteger,
    UnsignedInteger,
    ExactInteger,
    Float,
    String,
    Bytes,
    Sequence,
    Tuple,
    Mapping,
    Set,
    FrozenSet,
    Record,
    Enum,
    Optional,
    Union,
    Refined,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum StructuralScalar {
    None,
    Bool(bool),
    SignedInteger { value: i128, width: u16 },
    UnsignedInteger { value: u128, width: u16 },
    ExactInteger(SifrInt),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StructuralScalarRef<'value> {
    None,
    Bool(bool),
    SignedInteger { value: i128, width: u16 },
    UnsignedInteger { value: u128, width: u16 },
    ExactInteger(&'value SifrInt),
    Float(f64),
    String(&'value str),
    Bytes(&'value [u8]),
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuralContractError {
    ShapeMismatch,
    InvalidNode,
    CyclicArena,
    KindMismatch,
    ArityMismatch,
    MemberMismatch,
    AlreadyMoved,
    ScalarMismatch,
}

impl fmt::Display for StructuralContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ShapeMismatch => "structural shape identity mismatch",
            Self::InvalidNode => "invalid structural node",
            Self::CyclicArena => "cyclic structural arena",
            Self::KindMismatch => "structural node kind mismatch",
            Self::ArityMismatch => "structural node arity mismatch",
            Self::MemberMismatch => "structural member identity mismatch",
            Self::AlreadyMoved => "structural scalar was already moved",
            Self::ScalarMismatch => "structural scalar payload mismatch",
        })
    }
}

impl std::error::Error for StructuralContractError {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralEdgeKind<'value> {
    RecordField(&'value str),
    Index(usize),
    MappingKey(usize),
    MappingValue(usize),
    ActiveMember { name: &'value str, index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuralNodeEdge<'source> {
    kind: StructuralEdgeKind<'source>,
    node: NodeId,
}

impl<'source> StructuralNodeEdge<'source> {
    #[must_use]
    pub const fn new(kind: StructuralEdgeKind<'source>, node: NodeId) -> Self {
        Self { kind, node }
    }

    #[must_use]
    pub const fn kind(&self) -> StructuralEdgeKind<'source> {
        self.kind
    }

    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StructuralNodeRef<'source> {
    kind: StructuralKind,
    nominal_identity: Option<&'source str>,
    edges: &'source [StructuralNodeEdge<'source>],
}

impl<'source> StructuralNodeRef<'source> {
    #[must_use]
    pub const fn scalar(kind: StructuralKind) -> Self {
        Self {
            kind,
            nominal_identity: None,
            edges: &[],
        }
    }

    #[must_use]
    pub const fn aggregate(
        kind: StructuralKind,
        nominal_identity: Option<&'source str>,
        edges: &'source [StructuralNodeEdge<'source>],
    ) -> Self {
        Self {
            kind,
            nominal_identity,
            edges,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> StructuralKind {
        self.kind
    }

    #[must_use]
    pub const fn nominal_identity(&self) -> Option<&'source str> {
        self.nominal_identity
    }

    #[must_use]
    pub const fn edges(&self) -> &'source [StructuralNodeEdge<'source>] {
        self.edges
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StructuralEnter<'value> {
    kind: StructuralKind,
    nominal_identity: Option<&'value str>,
    child_count: usize,
}

impl<'value> StructuralEnter<'value> {
    #[must_use]
    pub const fn new(
        kind: StructuralKind,
        nominal_identity: Option<&'value str>,
        child_count: usize,
    ) -> Self {
        Self {
            kind,
            nominal_identity,
            child_count,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> StructuralKind {
        self.kind
    }

    #[must_use]
    pub const fn nominal_identity(&self) -> Option<&'value str> {
        self.nominal_identity
    }

    #[must_use]
    pub const fn child_count(&self) -> usize {
        self.child_count
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StructuralEdge<'value> {
    kind: StructuralEdgeKind<'value>,
}

impl<'value> StructuralEdge<'value> {
    #[must_use]
    pub const fn new(kind: StructuralEdgeKind<'value>) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> StructuralEdgeKind<'value> {
        self.kind
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisitControl {
    Continue,
    SkipChildren,
}

pub trait StructuralSource {
    fn shape_identity(&self) -> ShapeIdentity;
    fn root(&self) -> NodeId;
    fn node(&self, id: NodeId) -> Result<StructuralNodeRef<'_>, StructuralContractError>;
    fn take_scalar(&mut self, id: NodeId) -> Result<StructuralScalar, StructuralContractError>;
}

pub fn structural_construct<T: StructuralConstruct, S: StructuralSource>(
    mut source: S,
) -> Result<T, StructuralContractError> {
    if source.shape_identity() != T::shape_identity() {
        return Err(StructuralContractError::ShapeMismatch);
    }
    let root = source.root();
    T::structural_construct_at(&mut source, root, ConstructToken { _private: () })
}

pub trait StructuralType {
    fn shape_identity() -> ShapeIdentity;
}

pub trait StructuralConstruct: StructuralType + Sized {
    #[doc(hidden)]
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError>;
}

pub trait StructuralVisitor<'value> {
    type Error;

    fn enter(&mut self, event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error>;
    fn edge(&mut self, edge: StructuralEdge<'value>) -> Result<(), Self::Error>;
    fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error>;
    fn exit(&mut self, kind: StructuralKind) -> Result<(), Self::Error>;
}

pub trait StructuralProject: StructuralType {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error>;
}

#[doc(hidden)]
pub fn construct_bytes_at<S: StructuralSource>(
    source: &mut S,
    node: NodeId,
    _token: ConstructToken,
) -> Result<Vec<u8>, StructuralContractError> {
    match checked_scalar(source, node, StructuralKind::Bytes)? {
        StructuralScalar::Bytes(value) => Ok(value),
        _ => Err(StructuralContractError::ScalarMismatch),
    }
}

#[doc(hidden)]
pub fn project_bytes<'value, V: StructuralVisitor<'value>>(
    value: &'value [u8],
    visitor: &mut V,
) -> Result<(), V::Error> {
    visitor.scalar(StructuralScalarRef::Bytes(value))
}

fn checked_scalar<S: StructuralSource>(
    source: &mut S,
    node: NodeId,
    kind: StructuralKind,
) -> Result<StructuralScalar, StructuralContractError> {
    if source.node(node)?.kind() != kind {
        return Err(StructuralContractError::KindMismatch);
    }
    source.take_scalar(node)
}

macro_rules! signed_structural {
    ($ty:ty, $width:expr, $tag:literal) => {
        impl StructuralType for $ty {
            fn shape_identity() -> ShapeIdentity {
                primitive($tag)
            }
        }
        impl StructuralConstruct for $ty {
            fn structural_construct_at<S: StructuralSource>(
                source: &mut S,
                node: NodeId,
                _token: ConstructToken,
            ) -> Result<Self, StructuralContractError> {
                match checked_scalar(source, node, StructuralKind::SignedInteger)? {
                    StructuralScalar::SignedInteger { value, width } if width == $width => {
                        <$ty>::try_from(value).map_err(|_| StructuralContractError::ScalarMismatch)
                    }
                    _ => Err(StructuralContractError::ScalarMismatch),
                }
            }
        }
        impl StructuralProject for $ty {
            fn structural_project<'value, V: StructuralVisitor<'value>>(
                &'value self,
                visitor: &mut V,
            ) -> Result<(), V::Error> {
                visitor.scalar(StructuralScalarRef::SignedInteger {
                    value: i128::from(*self),
                    width: $width,
                })
            }
        }
    };
}

macro_rules! unsigned_structural {
    ($ty:ty, $width:expr, $tag:literal) => {
        impl StructuralType for $ty {
            fn shape_identity() -> ShapeIdentity {
                primitive($tag)
            }
        }
        impl StructuralConstruct for $ty {
            fn structural_construct_at<S: StructuralSource>(
                source: &mut S,
                node: NodeId,
                _token: ConstructToken,
            ) -> Result<Self, StructuralContractError> {
                match checked_scalar(source, node, StructuralKind::UnsignedInteger)? {
                    StructuralScalar::UnsignedInteger { value, width } if width == $width => {
                        <$ty>::try_from(value).map_err(|_| StructuralContractError::ScalarMismatch)
                    }
                    _ => Err(StructuralContractError::ScalarMismatch),
                }
            }
        }
        impl StructuralProject for $ty {
            fn structural_project<'value, V: StructuralVisitor<'value>>(
                &'value self,
                visitor: &mut V,
            ) -> Result<(), V::Error> {
                visitor.scalar(StructuralScalarRef::UnsignedInteger {
                    value: u128::from(*self),
                    width: $width,
                })
            }
        }
    };
}

signed_structural!(i8, 8, "i8");
signed_structural!(i16, 16, "i16");
signed_structural!(i32, 32, "i32");
signed_structural!(i64, 64, "i64");
signed_structural!(i128, 128, "i128");
unsigned_structural!(u8, 8, "u8");
unsigned_structural!(u16, 16, "u16");
unsigned_structural!(u32, 32, "u32");
unsigned_structural!(u64, 64, "u64");
unsigned_structural!(u128, 128, "u128");

impl StructuralType for bool {
    fn shape_identity() -> ShapeIdentity {
        primitive("bool")
    }
}

impl StructuralConstruct for bool {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        _token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        match checked_scalar(source, node, StructuralKind::Bool)? {
            StructuralScalar::Bool(value) => Ok(value),
            _ => Err(StructuralContractError::ScalarMismatch),
        }
    }
}

impl StructuralProject for bool {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.scalar(StructuralScalarRef::Bool(*self))
    }
}

impl StructuralType for f64 {
    fn shape_identity() -> ShapeIdentity {
        primitive("f64")
    }
}

impl StructuralConstruct for f64 {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        _token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        match checked_scalar(source, node, StructuralKind::Float)? {
            StructuralScalar::Float(value) => Ok(value),
            _ => Err(StructuralContractError::ScalarMismatch),
        }
    }
}

impl StructuralProject for f64 {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.scalar(StructuralScalarRef::Float(*self))
    }
}

impl StructuralType for String {
    fn shape_identity() -> ShapeIdentity {
        primitive("str")
    }
}

impl StructuralConstruct for String {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        _token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        match checked_scalar(source, node, StructuralKind::String)? {
            StructuralScalar::String(value) => Ok(value),
            _ => Err(StructuralContractError::ScalarMismatch),
        }
    }
}

impl StructuralProject for String {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.scalar(StructuralScalarRef::String(self))
    }
}

impl StructuralType for SifrInt {
    fn shape_identity() -> ShapeIdentity {
        primitive("int")
    }
}

impl StructuralConstruct for SifrInt {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        _token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        match checked_scalar(source, node, StructuralKind::ExactInteger)? {
            StructuralScalar::ExactInteger(value) => Ok(value),
            _ => Err(StructuralContractError::ScalarMismatch),
        }
    }
}

impl StructuralProject for SifrInt {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.scalar(StructuralScalarRef::ExactInteger(self))
    }
}

impl StructuralType for () {
    fn shape_identity() -> ShapeIdentity {
        primitive("None")
    }
}

impl StructuralConstruct for () {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        _token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        match checked_scalar(source, node, StructuralKind::None)? {
            StructuralScalar::None => Ok(()),
            _ => Err(StructuralContractError::ScalarMismatch),
        }
    }
}

impl StructuralProject for () {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.scalar(StructuralScalarRef::None)
    }
}

impl<T: StructuralType> StructuralType for Vec<T> {
    fn shape_identity() -> ShapeIdentity {
        unary_container("list", T::shape_identity())
    }
}

impl<T: StructuralConstruct> StructuralConstruct for Vec<T> {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        let edges = source.node(node)?;
        if edges.kind() != StructuralKind::Sequence {
            return Err(StructuralContractError::KindMismatch);
        }
        let child_ids = edges
            .edges()
            .iter()
            .enumerate()
            .map(|(index, edge)| {
                if edge.kind() == StructuralEdgeKind::Index(index) {
                    Ok(edge.node())
                } else {
                    Err(StructuralContractError::MemberMismatch)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        child_ids
            .into_iter()
            .map(|child| T::structural_construct_at(source, child, token))
            .collect()
    }
}

impl<T: StructuralProject> StructuralProject for Vec<T> {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        let control = visitor.enter(StructuralEnter::new(
            StructuralKind::Sequence,
            None,
            self.len(),
        ))?;
        if control == VisitControl::Continue {
            for (index, value) in self.iter().enumerate() {
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::Index(index)))?;
                value.structural_project(visitor)?;
            }
        }
        visitor.exit(StructuralKind::Sequence)
    }
}

impl<T: StructuralType> StructuralType for Option<T> {
    fn shape_identity() -> ShapeIdentity {
        unary_container("optional", T::shape_identity())
    }
}

impl<T: StructuralConstruct> StructuralConstruct for Option<T> {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        let description = source.node(node)?;
        if description.kind() != StructuralKind::Optional {
            return Err(StructuralContractError::KindMismatch);
        }
        match description.edges() {
            [] => Ok(None),
            [edge]
                if edge.kind()
                    == StructuralEdgeKind::ActiveMember {
                        name: "some",
                        index: 0,
                    } =>
            {
                let child = edge.node();
                T::structural_construct_at(source, child, token).map(Some)
            }
            _ => Err(StructuralContractError::ArityMismatch),
        }
    }
}

impl<T: StructuralProject> StructuralProject for Option<T> {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        let count = usize::from(self.is_some());
        let control = visitor.enter(StructuralEnter::new(StructuralKind::Optional, None, count))?;
        if control == VisitControl::Continue {
            if let Some(value) = self {
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::ActiveMember {
                    name: "some",
                    index: 0,
                }))?;
                value.structural_project(visitor)?;
            }
        }
        visitor.exit(StructuralKind::Optional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ArenaNode {
        kind: StructuralKind,
        edges: Vec<StructuralNodeEdge<'static>>,
        scalar: Option<StructuralScalar>,
    }

    struct Arena {
        shape: ShapeIdentity,
        nodes: Vec<ArenaNode>,
    }

    impl StructuralSource for Arena {
        fn shape_identity(&self) -> ShapeIdentity {
            self.shape
        }
        fn root(&self) -> NodeId {
            NodeId::new(0)
        }
        fn node(&self, id: NodeId) -> Result<StructuralNodeRef<'_>, StructuralContractError> {
            let node = self
                .nodes
                .get(id.get() as usize)
                .ok_or(StructuralContractError::InvalidNode)?;
            Ok(if node.edges.is_empty() {
                StructuralNodeRef::scalar(node.kind)
            } else {
                StructuralNodeRef::aggregate(node.kind, None, &node.edges)
            })
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

    #[test]
    fn construction_checks_root_identity_before_moving_payloads() {
        let mut wrong = Arena {
            shape: primitive("bytes"),
            nodes: vec![ArenaNode {
                kind: StructuralKind::String,
                edges: vec![],
                scalar: Some(StructuralScalar::String("secret".to_string())),
            }],
        };
        assert_eq!(
            structural_construct::<String, _>(Arena {
                shape: wrong.shape,
                nodes: std::mem::take(&mut wrong.nodes)
            }),
            Err(StructuralContractError::ShapeMismatch)
        );
    }

    #[test]
    fn construction_moves_nested_sequence_scalars_once() {
        let source = Arena {
            shape: <Vec<String> as StructuralType>::shape_identity(),
            nodes: vec![
                ArenaNode {
                    kind: StructuralKind::Sequence,
                    edges: vec![
                        StructuralNodeEdge::new(StructuralEdgeKind::Index(0), NodeId::new(1)),
                        StructuralNodeEdge::new(StructuralEdgeKind::Index(1), NodeId::new(2)),
                    ],
                    scalar: None,
                },
                ArenaNode {
                    kind: StructuralKind::String,
                    edges: vec![],
                    scalar: Some(StructuralScalar::String("a".to_string())),
                },
                ArenaNode {
                    kind: StructuralKind::String,
                    edges: vec![],
                    scalar: Some(StructuralScalar::String("b".to_string())),
                },
            ],
        };
        assert_eq!(
            structural_construct::<Vec<String>, _>(source),
            Ok(vec!["a".to_string(), "b".to_string()])
        );
    }

    struct EventVisitor(Vec<String>);

    impl<'value> StructuralVisitor<'value> for EventVisitor {
        type Error = ();
        fn enter(&mut self, event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
            self.0
                .push(format!("enter:{:?}:{}", event.kind(), event.child_count()));
            Ok(VisitControl::Continue)
        }
        fn edge(&mut self, edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
            self.0.push(format!("edge:{:?}", edge.kind()));
            Ok(())
        }
        fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
            self.0.push(format!("scalar:{value:?}"));
            Ok(())
        }
        fn exit(&mut self, kind: StructuralKind) -> Result<(), Self::Error> {
            self.0.push(format!("exit:{kind:?}"));
            Ok(())
        }
    }

    #[test]
    fn projection_stream_is_balanced_and_allocation_free_in_the_runtime() {
        let mut visitor = EventVisitor(Vec::new());
        vec!["a".to_string(), "b".to_string()]
            .structural_project(&mut visitor)
            .expect("visitor is infallible");
        assert_eq!(
            visitor.0.first().map(String::as_str),
            Some("enter:Sequence:2")
        );
        assert_eq!(visitor.0.last().map(String::as_str), Some("exit:Sequence"));
        assert_eq!(
            visitor
                .0
                .iter()
                .filter(|event| event.starts_with("scalar:"))
                .count(),
            2
        );
    }
}
