use super::{
    Record, RecordId, References, RustCallbackBackpressure, RustCallbackOverflow,
    RustCallbackShutdown, RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
    RustThreadsafeCallbackContract, sealed,
};
impl References for RustTargetPath {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.segments.references(out);
        self.span.references(out);
    }
}
impl sealed::Sealed for RustTargetPath {}
impl Record for RustTargetPath {
    const KIND: u16 = 112;
}
impl References for RustInteropDecoratorKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Function => {}
            Self::Opaque => {}
            Self::Async => {}
            Self::Callback => {}
            Self::Structural => {}
            Self::ZeroCopy => {}
            Self::View => {}
        }
    }
}
impl sealed::Sealed for RustInteropDecoratorKind {}
impl Record for RustInteropDecoratorKind {
    const KIND: u16 = 108;
}
impl References for RustInteropEffect {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Sync => {}
            Self::Async => {}
            Self::BlockingIo => {}
            Self::CpuHeavy => {}
        }
    }
}
impl sealed::Sealed for RustInteropEffect {}
impl Record for RustInteropEffect {
    const KIND: u16 = 109;
}
impl References for RustInteropAbiRequirements {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.async_boundary.references(out);
        self.opaque_handle.references(out);
        self.zero_copy.references(out);
        self.view.references(out);
    }
}
impl sealed::Sealed for RustInteropAbiRequirements {}
impl Record for RustInteropAbiRequirements {
    const KIND: u16 = 105;
}
impl References for RustInteropArgument {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.value.references(out);
        self.span.references(out);
    }
}
impl sealed::Sealed for RustInteropArgument {}
impl Record for RustInteropArgument {
    const KIND: u16 = 106;
}
impl References for RustInteropValue {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Boolean(v0) => {
                v0.references(out);
            }
            Self::Symbol(v0) => {
                v0.references(out);
            }
            Self::Integer(v0) => {
                v0.references(out);
            }
            Self::IntegerList(v0) => {
                v0.references(out);
            }
            Self::PolicyCall {
                name,
                argument,
                span,
            } => {
                name.references(out);
                argument.references(out);
                span.references(out);
            }
            Self::TargetPath(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for RustInteropValue {}
impl Record for RustInteropValue {
    const KIND: u16 = 110;
}
impl References for RustCallbackBackpressure {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Direct => {}
            Self::Bounded(v0) => {
                v0.references(out);
            }
            Self::Unbounded => {}
        }
    }
}
impl sealed::Sealed for RustCallbackBackpressure {}
impl Record for RustCallbackBackpressure {
    const KIND: u16 = 102;
}
impl References for RustCallbackOverflow {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Error => {}
            Self::DropOldest => {}
            Self::DropNewest => {}
        }
    }
}
impl sealed::Sealed for RustCallbackOverflow {}
impl Record for RustCallbackOverflow {
    const KIND: u16 = 103;
}
impl References for RustCallbackShutdown {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Drain => {}
            Self::Cancel => {}
            Self::DetachForbidden => {}
        }
    }
}
impl sealed::Sealed for RustCallbackShutdown {}
impl Record for RustCallbackShutdown {
    const KIND: u16 = 104;
}
impl References for RustThreadsafeCallbackContract {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.backpressure.references(out);
        self.overflow.references(out);
        self.shutdown.references(out);
    }
}
impl sealed::Sealed for RustThreadsafeCallbackContract {}
impl Record for RustThreadsafeCallbackContract {
    const KIND: u16 = 113;
}
impl References for RustInteropDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.kind.references(out);
        self.target.references(out);
        self.arguments.references(out);
        self.span.references(out);
        self.effect.references(out);
        self.abi_requirements.references(out);
        self.consumes_receiver.references(out);
    }
}
impl sealed::Sealed for RustInteropDeclaration {}
impl Record for RustInteropDeclaration {
    const KIND: u16 = 107;
}
