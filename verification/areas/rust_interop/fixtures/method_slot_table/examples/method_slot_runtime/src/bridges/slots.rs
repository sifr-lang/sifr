use std::fmt;

use sifr_runtime::interop::structural::{
    ArenaNode, MethodSlotTable, NodeId, SharedContext, SlotError, SlotSink, StructuralArena,
    StructuralContractError, StructuralEdgeKind, StructuralKind, StructuralNodeEdge,
    StructuralScalar, StructuralScalarRef, StructuralType,
};

#[derive(Debug)]
pub struct SlotFailure {
    message: String,
}

impl SlotFailure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SlotFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SlotFailure {}

impl From<StructuralContractError> for SlotFailure {
    fn from(error: StructuralContractError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<SlotError> for SlotFailure {
    fn from(error: SlotError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Default)]
struct StringSink {
    value: Option<String>,
}

impl SlotSink for StringSink {
    fn enter(
        &mut self,
        _event: sifr_runtime::interop::structural::StructuralEnter<'_>,
    ) -> Result<sifr_runtime::interop::structural::VisitControl, StructuralContractError> {
        Ok(sifr_runtime::interop::structural::VisitControl::Continue)
    }

    fn edge(
        &mut self,
        _edge: sifr_runtime::interop::structural::StructuralEdge<'_>,
    ) -> Result<(), StructuralContractError> {
        Ok(())
    }

    fn scalar(&mut self, value: StructuralScalarRef<'_>) -> Result<(), StructuralContractError> {
        let StructuralScalarRef::String(value) = value else {
            return Err(StructuralContractError::ScalarMismatch);
        };
        self.value = Some(value.to_string());
        Ok(())
    }

    fn exit(&mut self, _kind: StructuralKind) -> Result<(), StructuralContractError> {
        Ok(())
    }
}

pub fn invoke<Context, T>(
    _model: &T,
    value: String,
    context: &mut Context,
) -> Result<String, SlotFailure>
where
    T: sifr_runtime::interop::structural::StructuralConstruct
        + sifr_runtime::interop::structural::StructuralProject
        + sifr_runtime::interop::structural::StaticProgramType
        + MethodSlotTable<Context>,
    Context: StructuralType,
{
    let arena = StructuralArena::seal(
        String::shape_identity(),
        NodeId::new(0),
        vec![ArenaNode::scalar(
            StructuralKind::String,
            StructuralScalar::String(value),
        )],
    )?;
    let mut sink = StringSink::default();
    T::invoke_slot(0, arena, context, None, &mut sink)?;
    sink.value
        .ok_or_else(|| SlotFailure::new("method slot emitted no string output"))
}

pub fn invoke_shared<Context, T>(
    _model: &T,
    value: String,
    context: &Context,
) -> Result<String, SlotFailure>
where
    T: sifr_runtime::interop::structural::StructuralConstruct
        + sifr_runtime::interop::structural::StructuralProject
        + sifr_runtime::interop::structural::StaticProgramType
        + for<'context> MethodSlotTable<SharedContext<'context, Context>>,
    Context: StructuralType,
{
    let arena = StructuralArena::seal(
        String::shape_identity(),
        NodeId::new(0),
        vec![ArenaNode::scalar(
            StructuralKind::String,
            StructuralScalar::String(value),
        )],
    )?;
    let mut sink = StringSink::default();
    let mut shared = SharedContext::new(context);
    T::invoke_slot(0, arena, &mut shared, None, &mut sink)?;
    sink.value
        .ok_or_else(|| SlotFailure::new("method slot emitted no string output"))
}

pub fn invoke_receiver<Context, T>(_model: &T, context: &mut Context) -> Result<String, SlotFailure>
where
    T: sifr_runtime::interop::structural::StructuralConstruct
        + sifr_runtime::interop::structural::StructuralProject
        + sifr_runtime::interop::structural::StaticProgramType
        + MethodSlotTable<Context>,
    Context: StructuralType,
{
    let arena = StructuralArena::seal(
        T::shape_identity(),
        NodeId::new(0),
        vec![
            ArenaNode::aggregate(
                StructuralKind::Record,
                Some("main.Record"),
                vec![StructuralNodeEdge::new(
                    StructuralEdgeKind::RecordField("value"),
                    NodeId::new(1),
                )],
            ),
            ArenaNode::scalar(
                StructuralKind::String,
                StructuralScalar::String("input".to_string()),
            ),
        ],
    )?;
    let mut sink = StringSink::default();
    T::invoke_slot(1, arena, context, None, &mut sink)?;
    sink.value
        .ok_or_else(|| SlotFailure::new("receiver slot emitted no string output"))
}
