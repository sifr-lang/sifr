use std::marker::PhantomData;
use std::rc::Rc;

use crate::interop::GeneratedGlueToken;

use super::{
    ShapeIdentity, StaticProgramType, StructuralArena, StructuralContractError, StructuralEdge,
    StructuralEnter, StructuralKind, StructuralScalarRef, StructuralType, StructuralVisitor,
    VisitControl,
};

use sifr_structural_identity::SlotTableIdentity;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotReceiver {
    None,
    Shared,
    Exclusive,
    Owned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotHandlerSignature {
    input: ShapeIdentity,
    output: ShapeIdentity,
}

impl SlotHandlerSignature {
    #[doc(hidden)]
    #[must_use]
    pub const fn __from_compiler(
        input: ShapeIdentity,
        output: ShapeIdentity,
        _token: GeneratedGlueToken,
    ) -> Self {
        Self { input, output }
    }

    #[must_use]
    pub const fn input(&self) -> ShapeIdentity {
        self.input
    }

    #[must_use]
    pub const fn output(&self) -> ShapeIdentity {
        self.output
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotSignature {
    name: &'static str,
    input: ShapeIdentity,
    output: ShapeIdentity,
    receiver: SlotReceiver,
    handler: Option<SlotHandlerSignature>,
}

impl SlotSignature {
    #[doc(hidden)]
    #[must_use]
    pub const fn __from_compiler(
        name: &'static str,
        input: ShapeIdentity,
        output: ShapeIdentity,
        receiver: SlotReceiver,
        handler: Option<SlotHandlerSignature>,
        _token: GeneratedGlueToken,
    ) -> Self {
        Self {
            name,
            input,
            output,
            receiver,
            handler,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn input(&self) -> ShapeIdentity {
        self.input
    }

    #[must_use]
    pub const fn output(&self) -> ShapeIdentity {
        self.output
    }

    #[must_use]
    pub const fn receiver(&self) -> SlotReceiver {
        self.receiver
    }

    #[must_use]
    pub const fn handler(&self) -> Option<SlotHandlerSignature> {
        self.handler
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotError {
    UnknownSlot,
    Contract(StructuralContractError),
    Slot(String),
}

impl core::fmt::Display for SlotError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownSlot => formatter.write_str("method slot does not exist"),
            Self::Contract(error) => error.fmt(formatter),
            Self::Slot(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SlotError {}

pub trait SlotSink {
    fn enter(
        &mut self,
        event: StructuralEnter<'_>,
    ) -> Result<VisitControl, StructuralContractError>;
    fn edge(&mut self, edge: StructuralEdge<'_>) -> Result<(), StructuralContractError>;
    fn scalar(&mut self, value: StructuralScalarRef<'_>) -> Result<(), StructuralContractError>;
    fn exit(&mut self, kind: StructuralKind) -> Result<(), StructuralContractError>;
}

pub struct SlotSinkVisitor<'sink> {
    sink: &'sink mut dyn SlotSink,
}

impl<'sink> SlotSinkVisitor<'sink> {
    #[must_use]
    pub fn new(sink: &'sink mut dyn SlotSink) -> Self {
        Self { sink }
    }
}

impl<'value> StructuralVisitor<'value> for SlotSinkVisitor<'_> {
    type Error = StructuralContractError;

    fn enter(&mut self, event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
        self.sink.enter(event)
    }

    fn edge(&mut self, edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
        self.sink.edge(edge)
    }

    fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
        self.sink.scalar(value)
    }

    fn exit(&mut self, kind: StructuralKind) -> Result<(), Self::Error> {
        self.sink.exit(kind)
    }
}

/// Canonical context used by a slot table whose selected methods do not
/// declare a caller-owned context parameter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoContext;

impl NoContext {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl StructuralType for NoContext {
    fn shape_identity() -> ShapeIdentity {
        super::primitive("method-slot-no-context")
    }
}

/// Immutable context mode. The generated bridge may borrow the wrapped value,
/// but cannot mutate it through `&mut SharedContext<'_, Context>`.
#[derive(Clone, Copy, Debug)]
pub struct SharedContext<'context, Context: StructuralType + ?Sized> {
    value: &'context Context,
}

impl<'context, Context: StructuralType + ?Sized> SharedContext<'context, Context> {
    #[must_use]
    pub const fn new(value: &'context Context) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn get(&self) -> &'context Context {
        self.value
    }
}

impl<Context: StructuralType + ?Sized> StructuralType for SharedContext<'_, Context> {
    fn shape_identity() -> ShapeIdentity {
        Context::shape_identity()
    }
}

/// A borrowed, current-thread-only inner-validation continuation for wrap slots.
pub struct SlotHandler<'call> {
    inner: &'call dyn Fn(StructuralArena, &mut dyn SlotSink) -> Result<(), SlotError>,
    _current_thread: PhantomData<Rc<()>>,
}

impl<'call> SlotHandler<'call> {
    #[must_use]
    pub fn new(
        inner: &'call dyn Fn(StructuralArena, &mut dyn SlotSink) -> Result<(), SlotError>,
    ) -> Self {
        Self {
            inner,
            _current_thread: PhantomData,
        }
    }

    pub fn call(&self, input: StructuralArena, sink: &mut dyn SlotSink) -> Result<(), SlotError> {
        (self.inner)(input, sink)
    }
}

/// Implemented by compiler-generated concrete types whose static program
/// declares an ordered method-slot table.
pub trait MethodSlotTable<Context: StructuralType>: StaticProgramType {
    fn slot_table_identity() -> SlotTableIdentity;
    fn slot_signatures() -> &'static [SlotSignature];
    fn invoke_slot(
        index: usize,
        input: StructuralArena,
        context: &mut Context,
        handler: Option<&SlotHandler<'_>>,
        sink: &mut dyn SlotSink,
    ) -> Result<(), SlotError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::__generated_glue;
    use crate::interop::structural::{primitive, StructuralProject};

    #[derive(Default)]
    struct RecordingSink {
        scalars: usize,
    }

    impl SlotSink for RecordingSink {
        fn enter(
            &mut self,
            _event: StructuralEnter<'_>,
        ) -> Result<VisitControl, StructuralContractError> {
            Ok(VisitControl::Continue)
        }

        fn edge(&mut self, _edge: StructuralEdge<'_>) -> Result<(), StructuralContractError> {
            Ok(())
        }

        fn scalar(
            &mut self,
            _value: StructuralScalarRef<'_>,
        ) -> Result<(), StructuralContractError> {
            self.scalars += 1;
            Ok(())
        }

        fn exit(&mut self, _kind: StructuralKind) -> Result<(), StructuralContractError> {
            Ok(())
        }
    }

    #[test]
    fn slot_sink_visitor_forwards_structural_events() {
        let mut sink = RecordingSink::default();
        "value"
            .to_string()
            .structural_project(&mut SlotSinkVisitor::new(&mut sink))
            .expect("sink accepts scalar");
        assert_eq!(sink.scalars, 1);
    }

    #[test]
    fn shared_context_delegates_identity_without_mutable_access() {
        let value = "context".to_string();
        let context = SharedContext::new(&value);
        assert_eq!(
            SharedContext::<String>::shape_identity(),
            String::shape_identity()
        );
        assert_eq!(context.get(), "context");
    }

    #[test]
    fn slot_handler_forwards_the_checked_arena_channel() {
        let handler = SlotHandler::new(&|_input, sink| {
            sink.scalar(StructuralScalarRef::String("handled"))
                .map_err(SlotError::Contract)
        });
        let mut sink = RecordingSink::default();
        let arena = StructuralArena::seal(
            String::shape_identity(),
            crate::interop::structural::NodeId::new(0),
            vec![crate::interop::structural::ArenaNode::scalar(
                StructuralKind::String,
                crate::interop::structural::StructuralScalar::String("value".to_string()),
            )],
        )
        .expect("test arena is valid");
        handler.call(arena, &mut sink).expect("handler succeeds");
        assert_eq!(sink.scalars, 1);
    }

    #[test]
    fn slot_signature_preserves_checked_shape_contract() {
        let signature = SlotSignature::__from_compiler(
            "normalize",
            primitive("str"),
            primitive("str"),
            SlotReceiver::None,
            None,
            __generated_glue::token(),
        );
        assert_eq!(signature.name(), "normalize");
        assert_eq!(signature.input(), primitive("str"));
        assert_eq!(signature.output(), primitive("str"));
        assert_eq!(signature.receiver(), SlotReceiver::None);
        assert_eq!(signature.handler(), None);
    }
}
