use super::{
    FixedIntType, FunctionType, ParamConvention, ParamMutability, ParamOwnership, PythonArrowKind,
    ReceiverConvention, Record, RecordId, References, StructuralRecordField, StructuralRecordType,
    Type, sealed,
};
impl References for StructuralRecordField {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.ty.references(out);
    }
}
impl sealed::Sealed for StructuralRecordField {}
impl Record for StructuralRecordField {
    const KIND: u16 = 126;
}
impl References for StructuralRecordType {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.canonical_fields.references(out);
        self.source_order.references(out);
    }
}
impl sealed::Sealed for StructuralRecordType {}
impl Record for StructuralRecordType {
    const KIND: u16 = 127;
}
impl References for PythonArrowKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Array => {}
            Self::Schema => {}
            Self::Stream => {}
            Self::DeviceArray => {}
            Self::DeviceStream => {}
        }
    }
}
impl sealed::Sealed for PythonArrowKind {}
impl Record for PythonArrowKind {
    const KIND: u16 = 81;
}
impl References for Type {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Int => {}
            Self::FixedInt(v0) => {
                v0.references(out);
            }
            Self::Float => {}
            Self::Bool => {}
            Self::Str => {}
            Self::Bytes => {}
            Self::None => {}
            Self::Function(v0) => {
                v0.references(out);
            }
            Self::AsyncFunction(v0) => {
                v0.references(out);
            }
            Self::Coroutine(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::Task(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::TaskResult(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::Failure(v0) => {
                v0.references(out);
            }
            Self::TimeoutResult(v0) => {
                v0.references(out);
            }
            Self::Select2(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::BlockingTask(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::JoinSet(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::Awaitable(v0) => {
                v0.references(out);
            }
            Self::AsyncIterator(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::AsyncGenerator(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::PythonBuffer(v0) => {
                v0.references(out);
            }
            Self::PythonArrow(v0) => {
                v0.references(out);
            }
            Self::PythonDlpackTensor(v0) => {
                v0.references(out);
            }
            Self::PythonDlpackStream => {}
            Self::List(v0) => {
                v0.references(out);
            }
            Self::Dict(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::Set(v0) => {
                v0.references(out);
            }
            Self::Tuple(v0) => {
                v0.references(out);
            }
            Self::Template(v0) => {
                v0.references(out);
            }
            Self::StructuralRecord(v0) => {
                v0.references(out);
            }
            Self::Range => {}
            Self::Iterable(v0) => {
                v0.references(out);
            }
            Self::Iterator(v0) => {
                v0.references(out);
            }
            Self::Any => {}
            Self::Never => {}
            Self::Union(v0) => {
                v0.references(out);
            }
            Self::Intersection(v0) => {
                v0.references(out);
            }
            Self::LiteralInt(v0) => {
                v0.references(out);
            }
            Self::LiteralStr(v0) => {
                v0.references(out);
            }
            Self::LiteralBool(v0) => {
                v0.references(out);
            }
            Self::Alias {
                name,
                declaration,
                type_args,
                body,
            } => {
                name.references(out);
                declaration.references(out);
                type_args.references(out);
                body.references(out);
            }
            Self::Unknown => {}
            Self::Result(v0, v1) => {
                v0.references(out);
                v1.references(out);
            }
            Self::Class {
                declaration,
                view,
                type_args,
            } => {
                declaration.references(out);
                view.references(out);
                type_args.references(out);
            }
            Self::Protocol {
                declaration,
                view,
                type_args,
            } => {
                declaration.references(out);
                view.references(out);
                type_args.references(out);
            }
            Self::Newtype {
                declaration,
                view,
                type_args,
            } => {
                declaration.references(out);
                view.references(out);
                type_args.references(out);
            }
            Self::TypeVar { binder, slot } => {
                binder.references(out);
                slot.references(out);
            }
            Self::Callable(v0, v1, v2) => {
                v0.references(out);
                v1.references(out);
                v2.references(out);
            }
            Self::AsyncCallable(v0, v1, v2) => {
                v0.references(out);
                v1.references(out);
                v2.references(out);
            }
            Self::Enum {
                declaration,
                view,
                type_args,
            } => {
                declaration.references(out);
                view.references(out);
                type_args.references(out);
            }
            Self::Decimal => {}
            Self::BigDecimal => {}
        }
    }
}
impl sealed::Sealed for Type {}
impl Record for Type {
    const KIND: u16 = 132;
}
impl References for FixedIntType {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::I8 => {}
            Self::I16 => {}
            Self::I32 => {}
            Self::I64 => {}
            Self::U8 => {}
            Self::U16 => {}
            Self::U32 => {}
            Self::U64 => {}
            Self::ISize => {}
            Self::USize => {}
        }
    }
}
impl sealed::Sealed for FixedIntType {}
impl Record for FixedIntType {
    const KIND: u16 = 24;
}
impl References for FunctionType {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.receiver.references(out);
        self.params.references(out);
        self.return_type.references(out);
    }
}
impl sealed::Sealed for FunctionType {}
impl Record for FunctionType {
    const KIND: u16 = 27;
}
impl References for ReceiverConvention {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::SharedBorrow => {}
            Self::MutableBorrow => {}
            Self::Owned => {}
            Self::OwnedMutable => {}
        }
    }
}
impl sealed::Sealed for ReceiverConvention {}
impl Record for ReceiverConvention {
    const KIND: u16 = 101;
}
impl References for ParamOwnership {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Borrow => {}
            Self::Own => {}
        }
    }
}
impl sealed::Sealed for ParamOwnership {}
impl Record for ParamOwnership {
    const KIND: u16 = 77;
}
impl References for ParamMutability {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Immutable => {}
            Self::Mutable => {}
        }
    }
}
impl sealed::Sealed for ParamMutability {}
impl Record for ParamMutability {
    const KIND: u16 = 76;
}
impl References for ParamConvention {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.ownership.references(out);
        self.mutability.references(out);
    }
}
impl sealed::Sealed for ParamConvention {}
impl Record for ParamConvention {
    const KIND: u16 = 75;
}
