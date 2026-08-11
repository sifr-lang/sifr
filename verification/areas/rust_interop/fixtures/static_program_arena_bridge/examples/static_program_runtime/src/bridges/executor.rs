use std::cell::RefCell;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use sifr_runtime::interop::structural::{
    structural_construct, ArenaNode, NodeId, StaticProgramType, StructuralConstruct,
    StructuralContractError, StructuralEdge, StructuralEdgeKind, StructuralEnter, StructuralKind,
    StructuralNodeEdge, StructuralProject, StructuralScalar, StructuralScalarRef,
    StructuralVisitor, VisitControl, STATIC_PROGRAM_FORMAT_VERSION,
    STRUCTURAL_BRIDGE_CONTRACT_VERSION,
};
use sifr_runtime::interop::{Handle, HandleStateError};

const RECORD_IDENTITY: &str = "StaticRecord";
static ACTIVE_DOCUMENTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct StaticProgramError {
    message: String,
}

impl StaticProgramError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StaticProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StaticProgramError {}

impl From<StructuralContractError> for StaticProgramError {
    fn from(error: StructuralContractError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<HandleStateError> for StaticProgramError {
    fn from(error: HandleStateError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug)]
pub struct ValidatedDocument {
    nodes: Vec<ArenaNode>,
}

impl ValidatedDocument {
    fn into_nodes(mut self) -> Vec<ArenaNode> {
        std::mem::take(&mut self.nodes)
    }
}

impl Drop for ValidatedDocument {
    fn drop(&mut self) {
        ACTIVE_DOCUMENTS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn edge(kind: StructuralEdgeKind<'static>, node: u32) -> StructuralNodeEdge<'static> {
    StructuralNodeEdge::new(kind, NodeId::new(node))
}

fn integer(value: i64) -> ArenaNode {
    ArenaNode::scalar(
        StructuralKind::SignedInteger,
        StructuralScalar::SignedInteger {
            value: i128::from(value),
            width: 64,
        },
    )
}

fn string(value: &str) -> ArenaNode {
    ArenaNode::scalar(
        StructuralKind::String,
        StructuralScalar::String(value.to_string()),
    )
}

fn document_nodes() -> Vec<ArenaNode> {
    vec![
        ArenaNode::aggregate(
            StructuralKind::Record,
            Some(RECORD_IDENTITY),
            vec![
                edge(StructuralEdgeKind::RecordField("exact"), 1),
                edge(StructuralEdgeKind::RecordField("fixed"), 2),
                edge(StructuralEdgeKind::RecordField("payload"), 3),
                edge(StructuralEdgeKind::RecordField("tags"), 4),
                edge(StructuralEdgeKind::RecordField("lookup"), 7),
            ],
        ),
        integer(123),
        ArenaNode::scalar(
            StructuralKind::UnsignedInteger,
            StructuralScalar::UnsignedInteger {
                value: 42,
                width: 32,
            },
        ),
        ArenaNode::scalar(
            StructuralKind::Bytes,
            StructuralScalar::Bytes(b"arena-bytes".to_vec()),
        ),
        ArenaNode::aggregate(
            StructuralKind::Sequence,
            None,
            vec![
                edge(StructuralEdgeKind::Index(0), 5),
                edge(StructuralEdgeKind::Index(1), 6),
            ],
        ),
        string("alpha"),
        string("beta"),
        ArenaNode::aggregate(
            StructuralKind::Mapping,
            None,
            vec![
                edge(StructuralEdgeKind::MappingKey(0), 8),
                edge(StructuralEdgeKind::MappingValue(0), 9),
                edge(StructuralEdgeKind::MappingKey(1), 10),
                edge(StructuralEdgeKind::MappingValue(1), 11),
            ],
        ),
        string("left"),
        integer(-7),
        string("right"),
        integer(9_007_199_254_740_991),
    ]
}

pub fn open() -> Result<Handle<ValidatedDocument>, StaticProgramError> {
    ACTIVE_DOCUMENTS.fetch_add(1, Ordering::SeqCst);
    Ok(Handle::new(ValidatedDocument {
        nodes: document_nodes(),
    }))
}

pub fn close(document: Handle<ValidatedDocument>) -> Result<(), StaticProgramError> {
    let _closed = document.into_inner()?;
    Ok(())
}

pub fn execute<T>(document: Handle<ValidatedDocument>, input: &T) -> Result<T, StaticProgramError>
where
    T: StructuralConstruct + StructuralProject + StaticProgramType,
{
    let program = T::static_program();
    let header = program.header();
    program.verify_envelope(
        STATIC_PROGRAM_FORMAT_VERSION,
        header.structural_contract_version(),
        STRUCTURAL_BRIDGE_CONTRACT_VERSION,
        header.identity(),
        T::shape_identity(),
    )?;
    let _input = project(input)?;
    if program.bytes().is_empty() {
        return Err(StaticProgramError::new("static program payload is empty"));
    }
    let nodes = document.into_inner()?.into_nodes();
    let arena = sifr_runtime::interop::structural::StructuralArena::seal(
        T::shape_identity(),
        NodeId::new(0),
        nodes,
    )?;
    let output = structural_construct::<T, _>(arena)?;
    let observed = project(&output)?;
    LAST_OBSERVATION.with_borrow_mut(|slot| *slot = Some(observed.summary()));
    Ok(output)
}

pub fn execute_corrupt<T>(
    document: Handle<ValidatedDocument>,
    _input: &T,
) -> Result<T, StaticProgramError>
where
    T: StructuralConstruct + StructuralProject + StaticProgramType,
{
    let document = document.into_inner()?;
    drop(document);
    let program = T::static_program();
    let header = program.header();
    if let Err(error) = program.verify_envelope(
        STATIC_PROGRAM_FORMAT_VERSION.saturating_add(1),
        header.structural_contract_version(),
        STRUCTURAL_BRIDGE_CONTRACT_VERSION,
        header.identity(),
        T::shape_identity(),
    ) {
        return Err(StaticProgramError::new(format!(
            "{error};active={}",
            ACTIVE_DOCUMENTS.load(Ordering::SeqCst)
        )));
    }
    Err(StaticProgramError::new(
        "corrupt static program was accepted",
    ))
}

thread_local! {
    static LAST_OBSERVATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn observation() -> Result<String, StaticProgramError> {
    LAST_OBSERVATION
        .take()
        .ok_or_else(|| StaticProgramError::new("static program execution was not observed"))
}

#[derive(Default)]
struct Observation {
    integers: Vec<String>,
    fixed: Option<u128>,
    bytes: Option<usize>,
    strings: Vec<String>,
}

impl Observation {
    fn summary(mut self) -> String {
        let integer = self.integers.first().cloned().unwrap_or_default();
        let fixed = self.fixed.unwrap_or_default();
        let bytes = self.bytes.unwrap_or_default();
        let tags = self
            .strings
            .drain(..2.min(self.strings.len()))
            .collect::<Vec<_>>();
        let mut pairs = self
            .strings
            .into_iter()
            .zip(self.integers.into_iter().skip(1))
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>();
        pairs.sort();
        format!(
            "program=sealed;integer={integer};fixed={fixed};bytes={bytes};tags={};lookup={};active={}",
            tags.join(","),
            pairs.join(","),
            ACTIVE_DOCUMENTS.load(Ordering::SeqCst)
        )
    }
}

impl<'value> StructuralVisitor<'value> for Observation {
    type Error = StaticProgramError;

    fn enter(&mut self, _event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
        Ok(VisitControl::Continue)
    }

    fn edge(&mut self, _edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
        match value {
            StructuralScalarRef::SignedInteger { value, width: 64 } => {
                self.integers.push(value.to_string());
            }
            StructuralScalarRef::UnsignedInteger { value, width: 32 } => {
                self.fixed = Some(value);
            }
            StructuralScalarRef::Bytes(value) => self.bytes = Some(value.len()),
            StructuralScalarRef::String(value) => self.strings.push(value.to_string()),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self, _kind: StructuralKind) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn project<T: StructuralProject>(value: &T) -> Result<Observation, StaticProgramError> {
    let mut observation = Observation::default();
    value.structural_project(&mut observation)?;
    Ok(observation)
}

impl From<sifr_runtime::interop::structural::StaticProgramEnvelopeError> for StaticProgramError {
    fn from(error: sifr_runtime::interop::structural::StaticProgramEnvelopeError) -> Self {
        Self::new(error.to_string())
    }
}
