use super::{
    PythonArrowDeclaration, PythonArrowSchemaMode, PythonBufferAccess, PythonBufferDeclaration,
    PythonBufferLayout, PythonCallbackConcurrency, PythonCallbackDeclaration,
    PythonCallbackDispatch, PythonCallbackLifetime, PythonCleanupPolicy, PythonDlpackDeclaration,
    PythonDlpackDevice, PythonDlpackStreamMode, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonInteropParameter, PythonParameterKind,
    PythonTargetPath, Record, RecordId, References, sealed,
};
impl References for PythonTargetPath {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.segments.references(out);
        self.span.references(out);
    }
}
impl sealed::Sealed for PythonTargetPath {}
impl Record for PythonTargetPath {
    const KIND: u16 = 100;
}
impl References for PythonInteropDecoratorKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Function => {}
            Self::Coroutine => {}
            Self::Opaque => {}
            Self::Attribute => {}
            Self::Item => {}
            Self::ContextEnter => {}
            Self::ContextExit => {}
            Self::ContextAsyncEnter => {}
            Self::ContextAsyncExit => {}
            Self::Callback => {}
            Self::Buffer => {}
            Self::Arrow => {}
            Self::Dlpack => {}
            Self::DlpackStream => {}
        }
    }
}
impl sealed::Sealed for PythonInteropDecoratorKind {}
impl Record for PythonInteropDecoratorKind {
    const KIND: u16 = 95;
}
impl References for PythonBufferAccess {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Read => {}
            Self::Write => {}
        }
    }
}
impl sealed::Sealed for PythonBufferAccess {}
impl Record for PythonBufferAccess {
    const KIND: u16 = 83;
}
impl References for PythonBufferLayout {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Any => {}
            Self::CContiguous => {}
            Self::FContiguous => {}
        }
    }
}
impl sealed::Sealed for PythonBufferLayout {}
impl Record for PythonBufferLayout {
    const KIND: u16 = 85;
}
impl References for PythonBufferDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.element_type.references(out);
        self.access.references(out);
        self.layout.references(out);
    }
}
impl sealed::Sealed for PythonBufferDeclaration {}
impl Record for PythonBufferDeclaration {
    const KIND: u16 = 84;
}
impl References for PythonArrowSchemaMode {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Omitted => {}
            Self::Parameter { name, span } => {
                name.references(out);
                span.references(out);
            }
        }
    }
}
impl sealed::Sealed for PythonArrowSchemaMode {}
impl Record for PythonArrowSchemaMode {
    const KIND: u16 = 82;
}
impl References for PythonArrowDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.kind.references(out);
        self.schema.references(out);
    }
}
impl sealed::Sealed for PythonArrowDeclaration {}
impl Record for PythonArrowDeclaration {
    const KIND: u16 = 80;
}
impl References for PythonDlpackDevice {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Cpu => {}
            Self::Cuda => {}
            Self::Any => {}
        }
    }
}
impl sealed::Sealed for PythonDlpackDevice {}
impl Record for PythonDlpackDevice {
    const KIND: u16 = 92;
}
impl References for PythonDlpackStreamMode {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::None => {}
            Self::Parameter { name, span } => {
                name.references(out);
                span.references(out);
            }
        }
    }
}
impl sealed::Sealed for PythonDlpackStreamMode {}
impl Record for PythonDlpackStreamMode {
    const KIND: u16 = 93;
}
impl References for PythonDlpackDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.device.references(out);
        self.stream.references(out);
        self.element_type.references(out);
    }
}
impl sealed::Sealed for PythonDlpackDeclaration {}
impl Record for PythonDlpackDeclaration {
    const KIND: u16 = 91;
}
impl References for PythonInteropEffect {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::BlockingIo => {}
            Self::Async => {}
        }
    }
}
impl sealed::Sealed for PythonInteropEffect {}
impl Record for PythonInteropEffect {
    const KIND: u16 = 96;
}
impl References for PythonCleanupPolicy {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Drop => {}
            Self::Close => {}
            Self::AsyncClose => {}
            Self::Context => {}
            Self::AsyncContext => {}
        }
    }
}
impl sealed::Sealed for PythonCleanupPolicy {}
impl Record for PythonCleanupPolicy {
    const KIND: u16 = 90;
}
impl References for PythonParameterKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Positional => {}
            Self::KeywordOnly => {}
            Self::PositionalVariadic => {}
            Self::KeywordVariadic => {}
        }
    }
}
impl sealed::Sealed for PythonParameterKind {}
impl Record for PythonParameterKind {
    const KIND: u16 = 98;
}
impl References for PythonCallbackLifetime {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Call => {}
            Self::Result => {}
            Self::Receiver => {}
        }
    }
}
impl sealed::Sealed for PythonCallbackLifetime {}
impl Record for PythonCallbackLifetime {
    const KIND: u16 = 89;
}
impl References for PythonCallbackDispatch {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Current => {}
            Self::Foreign => {}
            Self::Asyncio => {}
        }
    }
}
impl sealed::Sealed for PythonCallbackDispatch {}
impl Record for PythonCallbackDispatch {
    const KIND: u16 = 88;
}
impl References for PythonCallbackConcurrency {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Serial => {}
            Self::Parallel => {}
        }
    }
}
impl sealed::Sealed for PythonCallbackConcurrency {}
impl Record for PythonCallbackConcurrency {
    const KIND: u16 = 86;
}
impl References for PythonCallbackDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.parameter_name.references(out);
        self.span.references(out);
        self.lifetime.references(out);
        self.dispatch.references(out);
        self.concurrency.references(out);
        self.argument_types.references(out);
        self.argument_conventions.references(out);
        self.success_type.references(out);
        self.handler_error_type.references(out);
        self.is_async.references(out);
        self.owner_class.references(out);
        self.owner_cleanup.references(out);
    }
}
impl sealed::Sealed for PythonCallbackDeclaration {}
impl Record for PythonCallbackDeclaration {
    const KIND: u16 = 87;
}
impl References for PythonInteropParameter {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.kind.references(out);
        self.has_default.references(out);
        self.omit_when_absent.references(out);
        self.span.references(out);
    }
}
impl sealed::Sealed for PythonInteropParameter {}
impl Record for PythonInteropParameter {
    const KIND: u16 = 97;
}
impl References for PythonInteropDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.kind.references(out);
        self.target.references(out);
        self.span.references(out);
        self.effect.references(out);
        self.cleanup.references(out);
        self.consumes_receiver.references(out);
        self.parameters.references(out);
        self.required_import_root.references(out);
        self.callbacks.references(out);
        self.buffer.references(out);
        self.arrow.references(out);
        self.dlpack.references(out);
    }
}
impl sealed::Sealed for PythonInteropDeclaration {}
impl Record for PythonInteropDeclaration {
    const KIND: u16 = 94;
}
