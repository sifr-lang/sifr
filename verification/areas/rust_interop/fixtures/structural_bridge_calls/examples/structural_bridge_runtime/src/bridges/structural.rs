use std::cell::Cell;
use std::fmt;

use sifr_runtime::interop::structural::{
    metadata, nominal_record, primitive, structural_construct, MappedValue, NodeId, NominalField,
    ShapeIdentity, StructuralConstruct, StructuralContractError, StructuralEdge,
    StructuralEdgeKind, StructuralEnter, StructuralKind, StructuralMapping, StructuralNodeEdge,
    StructuralNodeRef, StructuralProject, StructuralScalar, StructuralScalarRef, StructuralSource,
    StructuralVisitor, VisitControl,
};
use sifr_runtime::interop::{CallScopedCallbackBridge, Handle, HandleStateError};

const NESTED_IDENTITY: &str = "main.NestedValue";
const LEAF_IDENTITY: &str = "main.Leaf";
const BOXED_IDENTITY: &str = "main.Boxed";
const STATUS_IDENTITY: &str = "main.Status";
const SUM_IDENTITY: &str = "main.SumPayload";
const TOKEN_IDENTITY: &str = "main.MappedToken";
const JSON_VALUE_IDENTITY: &str = "sifr.json.JsonValue";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Token {
    value: String,
}

impl Drop for Token {
    fn drop(&mut self) {
        TOKEN_DROPS.with(|count| count.set(count.get().saturating_add(1)));
    }
}

thread_local! {
    static TOKEN_DROPS: Cell<u32> = const { Cell::new(0) };
}

pub struct TokenMapping;

impl StructuralMapping<Token> for TokenMapping {
    fn shape_identity() -> ShapeIdentity {
        nominal_record(
            TOKEN_IDENTITY,
            &[],
            &[NominalField {
                name: "value",
                identity: primitive("str"),
                required: true,
                default_identity: None,
            }],
            metadata(&[]),
        )
    }

    fn nominal_identity() -> Option<&'static str> {
        Some(TOKEN_IDENTITY)
    }

    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: sifr_runtime::interop::structural::ConstructToken,
    ) -> Result<Token, StructuralContractError> {
        let description = source.node(node)?;
        if description.kind() != StructuralKind::Record
            || description.nominal_identity() != Some(TOKEN_IDENTITY)
        {
            return Err(StructuralContractError::KindMismatch);
        }
        let [value_edge] = description.edges() else {
            return Err(StructuralContractError::ArityMismatch);
        };
        if value_edge.kind() != StructuralEdgeKind::RecordField("value") {
            return Err(StructuralContractError::MemberMismatch);
        }
        let value_node = value_edge.node();
        let value = String::structural_construct_at(source, value_node, token)?;
        if value == "panic" {
            panic!("deliberate mapped-value panic");
        }
        if value.is_empty() || value == "invalid" {
            return Err(StructuralContractError::ScalarMismatch);
        }
        Ok(Token { value })
    }

    fn structural_project<'value, V: StructuralVisitor<'value>>(
        value: &'value Token,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        let control = visitor.enter(StructuralEnter::new(
            StructuralKind::Record,
            Some(TOKEN_IDENTITY),
            1,
        ))?;
        if control == VisitControl::Continue {
            visitor.edge(StructuralEdge::new(StructuralEdgeKind::RecordField(
                "value",
            )))?;
            value.value.structural_project(visitor)?;
        }
        visitor.exit(StructuralKind::Record)
    }
}

#[derive(Debug)]
pub struct StructuralBridgeError {
    message: String,
}

impl StructuralBridgeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StructuralBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StructuralBridgeError {}

impl From<StructuralContractError> for StructuralBridgeError {
    fn from(error: StructuralContractError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<HandleStateError> for StructuralBridgeError {
    fn from(error: HandleStateError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug)]
struct ArenaNode {
    kind: StructuralKind,
    nominal_identity: Option<&'static str>,
    edges: Vec<StructuralNodeEdge<'static>>,
    scalar: Option<StructuralScalar>,
}

#[derive(Debug)]
pub struct StructuralArena {
    nodes: Vec<ArenaNode>,
}

struct ArenaSource {
    shape: ShapeIdentity,
    nodes: Vec<ArenaNode>,
}

impl StructuralSource for ArenaSource {
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
        if node.edges.is_empty() && node.scalar.is_some() {
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

fn edge(kind: StructuralEdgeKind<'static>, node: u32) -> StructuralNodeEdge<'static> {
    StructuralNodeEdge::new(kind, NodeId::new(node))
}

fn record(nominal_identity: &'static str, fields: &[(&'static str, u32)]) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::Record,
        nominal_identity: Some(nominal_identity),
        edges: fields
            .iter()
            .map(|(name, node)| edge(StructuralEdgeKind::RecordField(name), *node))
            .collect(),
        scalar: None,
    }
}

fn string(value: &str) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::String,
        nominal_identity: None,
        edges: Vec::new(),
        scalar: Some(StructuralScalar::String(value.to_string())),
    }
}

fn signed_integer(value: i64) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::SignedInteger,
        nominal_identity: None,
        edges: Vec::new(),
        scalar: Some(StructuralScalar::SignedInteger {
            value: i128::from(value),
            width: 64,
        }),
    }
}

fn sequence(children: &[u32]) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::Sequence,
        nominal_identity: None,
        edges: children
            .iter()
            .enumerate()
            .map(|(index, node)| edge(StructuralEdgeKind::Index(index), *node))
            .collect(),
        scalar: None,
    }
}

fn optional(child: Option<u32>) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::Optional,
        nominal_identity: None,
        edges: child
            .map(|node| {
                vec![edge(
                    StructuralEdgeKind::ActiveMember {
                        name: "some",
                        index: 0,
                    },
                    node,
                )]
            })
            .unwrap_or_default(),
        scalar: None,
    }
}

fn union(member: usize, child: u32) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::Union,
        nominal_identity: None,
        edges: vec![edge(
            StructuralEdgeKind::ActiveMember {
                name: "member",
                index: member,
            },
            child,
        )],
        scalar: None,
    }
}

fn enumeration(name: &'static str, index: usize, child: u32) -> ArenaNode {
    ArenaNode {
        kind: StructuralKind::Enum,
        nominal_identity: Some(STATUS_IDENTITY),
        edges: vec![edge(
            StructuralEdgeKind::ActiveMember { name, index },
            child,
        )],
        scalar: None,
    }
}

fn structural_nodes() -> Vec<ArenaNode> {
    vec![
        record(
            NESTED_IDENTITY,
            &[("label", 1), ("leaves", 2), ("payload", 7), ("child", 9)],
        ),
        string("root"),
        sequence(&[3, 5]),
        record(LEAF_IDENTITY, &[("value", 4)]),
        string("a"),
        record(LEAF_IDENTITY, &[("value", 6)]),
        string("b"),
        record(BOXED_IDENTITY, &[("value", 8)]),
        string("boxed"),
        optional(Some(10)),
        record(
            NESTED_IDENTITY,
            &[
                ("label", 11),
                ("leaves", 12),
                ("payload", 13),
                ("child", 15),
            ],
        ),
        string("tail"),
        sequence(&[]),
        record(BOXED_IDENTITY, &[("value", 14)]),
        string("tail-box"),
        optional(None),
    ]
}

fn sum_nodes() -> Vec<ArenaNode> {
    vec![
        record(SUM_IDENTITY, &[("choice", 1), ("status", 3)]),
        union(1, 2),
        string("sum"),
        enumeration("WAITING", 1, 4),
        signed_integer(5),
    ]
}

fn token_nodes(value: &str) -> Vec<ArenaNode> {
    vec![record(TOKEN_IDENTITY, &[("value", 1)]), string(value)]
}

fn structural_output_nodes() -> Vec<ArenaNode> {
    vec![
        ArenaNode {
            kind: StructuralKind::Mapping,
            nominal_identity: None,
            edges: vec![
                edge(StructuralEdgeKind::MappingKey(0), 1),
                edge(StructuralEdgeKind::MappingValue(0), 2),
            ],
            scalar: None,
        },
        string("name"),
        record(
            JSON_VALUE_IDENTITY,
            &[
                ("kind", 3),
                ("bool_value", 4),
                ("int_value", 5),
                ("float_value", 6),
                ("str_value", 7),
                ("array_items", 9),
                ("object_items", 10),
            ],
        ),
        string("str"),
        optional(None),
        optional(None),
        optional(None),
        optional(Some(8)),
        string("mapped-output"),
        sequence(&[]),
        sequence(&[]),
    ]
}

pub fn open() -> Result<Handle<StructuralArena>, StructuralBridgeError> {
    Ok(Handle::new(StructuralArena {
        nodes: structural_nodes(),
    }))
}

pub fn open_sum() -> Result<Handle<StructuralArena>, StructuralBridgeError> {
    Ok(Handle::new(StructuralArena { nodes: sum_nodes() }))
}

pub fn open_union() -> Result<Handle<StructuralArena>, StructuralBridgeError> {
    Ok(Handle::new(StructuralArena {
        nodes: vec![union(1, 1), string("sum")],
    }))
}

pub fn open_enum() -> Result<Handle<StructuralArena>, StructuralBridgeError> {
    Ok(Handle::new(StructuralArena {
        nodes: vec![enumeration("WAITING", 1, 1), signed_integer(5)],
    }))
}

pub fn close(source: Handle<StructuralArena>) -> Result<(), StructuralBridgeError> {
    let _closed = source.into_inner()?;
    Ok(())
}

fn into_source<T: StructuralProject>(
    source: Handle<StructuralArena>,
) -> Result<ArenaSource, StructuralBridgeError> {
    let source = source.into_inner()?;
    Ok(ArenaSource {
        shape: T::shape_identity(),
        nodes: source.nodes,
    })
}

#[derive(Default)]
struct SummaryVisitor {
    records: usize,
    sequences: usize,
    optionals: usize,
    strings: Vec<String>,
}

impl<'value> StructuralVisitor<'value> for SummaryVisitor {
    type Error = StructuralBridgeError;

    fn enter(&mut self, event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
        match event.kind() {
            StructuralKind::Record => self.records += 1,
            StructuralKind::Sequence => self.sequences += 1,
            StructuralKind::Optional => self.optionals += 1,
            _ => {}
        }
        Ok(VisitControl::Continue)
    }

    fn edge(&mut self, _edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
        if let StructuralScalarRef::String(value) = value {
            self.strings.push(value.to_string());
        }
        Ok(())
    }

    fn exit(&mut self, _kind: StructuralKind) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn project<T: StructuralProject>(value: &T) -> Result<SummaryVisitor, StructuralBridgeError> {
    let mut visitor = SummaryVisitor::default();
    value.structural_project(&mut visitor)?;
    Ok(visitor)
}

pub fn observe<T>(value: &T) -> Result<String, StructuralBridgeError>
where
    T: StructuralConstruct + StructuralProject,
{
    let summary = project(value)?;
    Ok(format!(
        "records={};sequences={};optionals={};strings={}",
        summary.records,
        summary.sequences,
        summary.optionals,
        summary.strings.join(",")
    ))
}

pub fn roundtrip<T>(source: Handle<StructuralArena>, value: &T) -> Result<T, StructuralBridgeError>
where
    T: StructuralConstruct + StructuralProject,
{
    let _projection = project(value)?;
    let mismatch_probe = ArenaSource {
        shape: primitive("__sifr_deliberate_shape_mismatch__"),
        nodes: structural_nodes(),
    };
    if !matches!(
        structural_construct::<T, _>(mismatch_probe),
        Err(StructuralContractError::ShapeMismatch)
    ) {
        return Err(StructuralBridgeError::new(
            "structural construction accepted a mismatched root identity",
        ));
    }
    structural_construct::<T, _>(into_source::<T>(source)?).map_err(|error| {
        StructuralBridgeError::new(format!(
            "{} construction failed: {error}",
            std::any::type_name::<T>()
        ))
    })
}

pub fn construct_mapped<T>(value: &str) -> Result<T, StructuralBridgeError>
where
    T: StructuralConstruct + StructuralProject,
{
    structural_construct::<T, _>(ArenaSource {
        shape: T::shape_identity(),
        nodes: token_nodes(value),
    })
    .map_err(Into::into)
}

pub fn structural_output<T>() -> Result<T, StructuralBridgeError>
where
    T: StructuralConstruct + StructuralProject,
{
    structural_construct::<T, _>(ArenaSource {
        shape: T::shape_identity(),
        nodes: structural_output_nodes(),
    })
    .map_err(Into::into)
}

pub fn consume_mapped(
    value: MappedValue<Token, TokenMapping>,
) -> Result<String, StructuralBridgeError> {
    let token = value.into_inner()?;
    let observed = token.value.clone();
    drop(token);
    Ok(format!(
        "{observed};drops={}",
        TOKEN_DROPS.with(Cell::get)
    ))
}

pub fn mapped_value(
    value: &MappedValue<Token, TokenMapping>,
) -> Result<String, StructuralBridgeError> {
    Ok(value.inner_ref()?.value.clone())
}

pub fn token_drop_count() -> Result<u32, StructuralBridgeError> {
    Ok(TOKEN_DROPS.with(Cell::get))
}

pub fn transform<T>(
    source: Handle<StructuralArena>,
    value: &T,
    callback: CallScopedCallbackBridge<'_, (T,), Result<T, String>>,
) -> Result<T, StructuralBridgeError>
where
    T: StructuralConstruct + StructuralProject,
{
    let _input_projection = project(value)?;
    let input = structural_construct::<T, _>(into_source::<T>(source)?)?;
    let output = callback
        .call((input,))
        .map_err(StructuralBridgeError::new)?;
    let _output_projection = project(&output)?;
    Ok(output)
}
