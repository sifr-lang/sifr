use super::{
    BindingId, CompilerIntrinsicId, FieldIdentity, HirAsyncWithKind, HirClass, HirClassKind,
    HirCollectionMutation, HirExceptHandler, HirFStringPart, HirFunction, HirImport, HirIteratorOp,
    HirMatchArm, HirModule, HirParam, HirPattern, HirStmt, HirTupleTarget, HirTupleTargetBinding,
    HirWithItem, HirWithItemKind, MethodCallSource, MethodKind, MutableArgumentTarget,
    MutableReceiverTarget, Place, PlaceProjection, PythonRecordExpansion, Record, RecordId,
    References, sealed,
};
impl References for HirModule {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.functions.references(out);
        self.classes.references(out);
        self.imports.references(out);
        self.constants.references(out);
        self.generic_functions.references(out);
        self.type_param_bounds.references(out);
    }
}
impl sealed::Sealed for HirModule {}
impl Record for HirModule {
    const KIND: u16 = 39;
}
impl References for HirImport {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.names.references(out);
        self.aliases.references(out);
    }
}
impl sealed::Sealed for HirImport {}
impl Record for HirImport {
    const KIND: u16 = 36;
}
impl References for HirClassKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Regular => {}
            Self::Protocol => {}
            Self::Enum => {}
            Self::PythonOpaque(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirClassKind {}
impl Record for HirClassKind {
    const KIND: u16 = 30;
}
impl References for HirClass {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.identity.references(out);
        self.fields.references(out);
        self.field_defaults.references(out);
        self.field_default_identities.references(out);
        self.declaration_metadata.references(out);
        self.methods.references(out);
        self.is_hashable.references(out);
        self.is_error_type.references(out);
        self.kind.references(out);
        self.operator_impls.references(out);
        self.newtype_inner.references(out);
        self.implements_protocols.references(out);
        self.parent_class.references(out);
        self.parent_type.references(out);
        self.type_params.references(out);
        self.enum_variants.references(out);
        self.rust_interop.references(out);
    }
}
impl sealed::Sealed for HirClass {}
impl Record for HirClass {
    const KIND: u16 = 29;
}
impl References for MethodKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Regular => {}
            Self::ClassMethod => {}
            Self::StaticMethod => {}
        }
    }
}
impl sealed::Sealed for MethodKind {}
impl Record for MethodKind {
    const KIND: u16 = 69;
}
impl References for BindingId {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.binder.references(out);
        self.slot.references(out);
    }
}
impl sealed::Sealed for BindingId {}
impl Record for BindingId {
    const KIND: u16 = 10;
}
impl References for MethodCallSource {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.call_range.references(out);
        self.receiver_range.references(out);
        self.arg_ranges.references(out);
    }
}
impl sealed::Sealed for MethodCallSource {}
impl Record for MethodCallSource {
    const KIND: u16 = 68;
}
impl References for FieldIdentity {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.declaring_class.references(out);
        self.field.references(out);
    }
}
impl sealed::Sealed for FieldIdentity {}
impl Record for FieldIdentity {
    const KIND: u16 = 23;
}
impl References for PlaceProjection {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Field(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for PlaceProjection {}
impl Record for PlaceProjection {
    const KIND: u16 = 79;
}
impl References for Place {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.root.references(out);
        self.projections.references(out);
    }
}
impl sealed::Sealed for Place {}
impl Record for Place {
    const KIND: u16 = 78;
}
impl References for MutableReceiverTarget {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Place(v0) => {
                v0.references(out);
            }
            Self::OwnedTemporary => {}
            Self::SpecializedIndexedStorage(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for MutableReceiverTarget {}
impl Record for MutableReceiverTarget {
    const KIND: u16 = 72;
}
impl References for MutableArgumentTarget {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Place(v0) => {
                v0.references(out);
            }
            Self::OwnedTemporary => {}
        }
    }
}
impl sealed::Sealed for MutableArgumentTarget {}
impl Record for MutableArgumentTarget {
    const KIND: u16 = 71;
}
impl References for HirFunction {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.params.references(out);
        self.return_type.references(out);
        self.body.references(out);
        self.is_async.references(out);
        self.method_kind.references(out);
        self.receiver.references(out);
        self.decorators.references(out);
        self.rust_interop.references(out);
        self.python_interop.references(out);
        self.compiler_intrinsic.references(out);
        self.type_params.references(out);
    }
}
impl sealed::Sealed for HirFunction {}
impl Record for HirFunction {
    const KIND: u16 = 35;
}
impl References for CompilerIntrinsicId {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::TestAssertEqual => {}
            Self::TestAssertNotEqual => {}
            Self::TestAssertTrue => {}
            Self::TestAssertFalse => {}
            Self::TestAssertAlmostEqual => {}
            Self::TestAssertGreaterThan => {}
            Self::TestAssertLessThan => {}
            Self::OpenBinary => {}
            Self::OpenText => {}
            Self::BytesFromHex => {}
            Self::BytesWithSize => {}
            Self::BytesFromIntegers => {}
            Self::StringEncode => {}
            Self::StringEncodeWithEncoding => {}
            Self::BytesDecode => {}
            Self::BytesDecodeWithEncoding => {}
            Self::TaskCurrentContext => {}
            Self::PythonFromValue => {}
            Self::PythonToValue => {}
            Self::PythonKwarg => {}
        }
    }
}
impl sealed::Sealed for CompilerIntrinsicId {}
impl Record for CompilerIntrinsicId {
    const KIND: u16 = 16;
}
impl References for HirAsyncWithKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::TaskScope => {}
            Self::TaskGroup { context } => {
                context.references(out);
            }
            Self::TaskTimeout { duration } => {
                duration.references(out);
            }
            Self::UserDefined {
                context,
                enter_value_ty,
                enter_error_ty,
                exit_error_ty,
                active_error_ty,
                body_may_raise,
            } => {
                context.references(out);
                enter_value_ty.references(out);
                enter_error_ty.references(out);
                exit_error_ty.references(out);
                active_error_ty.references(out);
                body_may_raise.references(out);
            }
            Self::Python {
                context,
                manager_class,
                entered_type,
                enter_error_type,
                exit_error_type,
                entered_is_opaque_borrow,
                active_error_type,
                body_may_raise,
            } => {
                context.references(out);
                manager_class.references(out);
                entered_type.references(out);
                enter_error_type.references(out);
                exit_error_type.references(out);
                entered_is_opaque_borrow.references(out);
                active_error_type.references(out);
                body_may_raise.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirAsyncWithKind {}
impl Record for HirAsyncWithKind {
    const KIND: u16 = 28;
}
impl References for HirWithItemKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Native {
                has_context_manager_protocol,
            } => {
                has_context_manager_protocol.references(out);
            }
            Self::Python {
                entered_type,
                enter_error_type,
                exit_error_type,
                entered_is_opaque_borrow,
                body_may_raise,
            } => {
                entered_type.references(out);
                enter_error_type.references(out);
                exit_error_type.references(out);
                entered_is_opaque_borrow.references(out);
                body_may_raise.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirWithItemKind {}
impl Record for HirWithItemKind {
    const KIND: u16 = 65;
}
impl References for HirWithItem {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.target.references(out);
        self.context.references(out);
        self.kind.references(out);
    }
}
impl sealed::Sealed for HirWithItem {}
impl Record for HirWithItem {
    const KIND: u16 = 64;
}
impl References for HirParam {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.ty.references(out);
        self.default.references(out);
        self.keyword_only.references(out);
        self.convention.references(out);
    }
}
impl sealed::Sealed for HirParam {}
impl Record for HirParam {
    const KIND: u16 = 40;
}
impl References for HirStmt {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Let {
                name,
                ty,
                value,
                is_mutable,
            } => {
                name.references(out);
                ty.references(out);
                value.references(out);
                is_mutable.references(out);
            }
            Self::Assign { name, value } => {
                name.references(out);
                value.references(out);
            }
            Self::AugAssign { name, op, value } => {
                name.references(out);
                op.references(out);
                value.references(out);
            }
            Self::Return { value } => {
                value.references(out);
            }
            Self::Expr { expr } => {
                expr.references(out);
            }
            Self::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                condition.references(out);
                then_body.references(out);
                elif_clauses.references(out);
                else_body.references(out);
            }
            Self::While {
                condition,
                body,
                else_body,
            } => {
                condition.references(out);
                body.references(out);
                else_body.references(out);
            }
            Self::For {
                target,
                target_ty,
                iter,
                body,
                else_body,
            } => {
                target.references(out);
                target_ty.references(out);
                iter.references(out);
                body.references(out);
                else_body.references(out);
            }
            Self::AsyncFor {
                target,
                target_ty,
                iter,
                iter_error_ty,
                close_error_ty,
                active_error_ty,
                body_may_raise,
                body,
                else_body,
            } => {
                target.references(out);
                target_ty.references(out);
                iter.references(out);
                iter_error_ty.references(out);
                close_error_ty.references(out);
                active_error_ty.references(out);
                body_may_raise.references(out);
                body.references(out);
                else_body.references(out);
            }
            Self::Break => {}
            Self::Continue => {}
            Self::TupleUnpack { targets, value } => {
                targets.references(out);
                value.references(out);
            }
            Self::StarUnpack {
                before,
                star,
                after,
                value,
                failure,
            } => {
                before.references(out);
                star.references(out);
                after.references(out);
                value.references(out);
                failure.references(out);
            }
            Self::Pass => {}
            Self::Assert { test, msg } => {
                test.references(out);
                msg.references(out);
            }
            Self::Raise { value } => {
                value.references(out);
            }
            Self::TryExcept {
                body,
                handlers,
                body_error_types,
            } => {
                body.references(out);
                handlers.references(out);
                body_error_types.references(out);
            }
            Self::TryFinally { body, finalbody } => {
                body.references(out);
                finalbody.references(out);
            }
            Self::FieldAssign {
                object,
                field,
                field_ty,
                value,
            } => {
                object.references(out);
                field.references(out);
                field_ty.references(out);
                value.references(out);
            }
            Self::NestedFieldAssign {
                object,
                field,
                field_ty,
                nested_field,
                nested_field_ty,
                value,
            } => {
                object.references(out);
                field.references(out);
                field_ty.references(out);
                nested_field.references(out);
                nested_field_ty.references(out);
                value.references(out);
            }
            Self::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
                failure,
            } => {
                object.references(out);
                index.references(out);
                value.references(out);
                object_ty.references(out);
                failure.references(out);
            }
            Self::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty,
                outer_failure,
                inner_failure,
                operation,
            } => {
                object.references(out);
                outer_index.references(out);
                inner_index.references(out);
                value.references(out);
                object_ty.references(out);
                outer_failure.references(out);
                inner_failure.references(out);
                operation.references(out);
            }
            Self::AttributeNestedSubscriptAssign {
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
                outer_failure,
                inner_failure,
                operation,
            } => {
                object.references(out);
                field.references(out);
                outer_index.references(out);
                inner_index.references(out);
                value.references(out);
                field_ty.references(out);
                outer_failure.references(out);
                inner_failure.references(out);
                operation.references(out);
            }
            Self::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty,
                failure,
            } => {
                object.references(out);
                index.references(out);
                op.references(out);
                value.references(out);
                object_ty.references(out);
                failure.references(out);
            }
            Self::AttributeAugAssign {
                object,
                field,
                op,
                value,
            } => {
                object.references(out);
                field.references(out);
                op.references(out);
                value.references(out);
            }
            Self::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
                failure,
                operation,
            } => {
                object.references(out);
                field.references(out);
                index.references(out);
                value.references(out);
                field_ty.references(out);
                failure.references(out);
                operation.references(out);
            }
            Self::Delete {
                object,
                index,
                failure,
            } => {
                object.references(out);
                index.references(out);
                failure.references(out);
            }
            Self::Yield { value } => {
                value.references(out);
            }
            Self::With { items, body } => {
                items.references(out);
                body.references(out);
            }
            Self::AsyncWith { kind, target, body } => {
                kind.references(out);
                target.references(out);
                body.references(out);
            }
            Self::NestedFunction {
                func,
                move_captures,
                capture_clones,
            } => {
                func.references(out);
                move_captures.references(out);
                capture_clones.references(out);
            }
            Self::Match {
                subject,
                subject_ty,
                arms,
            } => {
                subject.references(out);
                subject_ty.references(out);
                arms.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirStmt {}
impl Record for HirStmt {
    const KIND: u16 = 54;
}
impl References for HirCollectionMutation {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Assign => {}
            Self::AugAssign(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirCollectionMutation {}
impl Record for HirCollectionMutation {
    const KIND: u16 = 31;
}
impl References for HirMatchArm {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.pattern.references(out);
        self.guard.references(out);
        self.body.references(out);
    }
}
impl sealed::Sealed for HirMatchArm {}
impl Record for HirMatchArm {
    const KIND: u16 = 38;
}
impl References for HirPattern {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Wildcard => {}
            Self::Capture { name, ty } => {
                name.references(out);
                ty.references(out);
            }
            Self::Literal { value } => {
                value.references(out);
            }
            Self::None => {}
            Self::Or { patterns } => {
                patterns.references(out);
            }
            Self::Class {
                class_name,
                class_type,
                fields,
            } => {
                class_name.references(out);
                class_type.references(out);
                fields.references(out);
            }
            Self::Value { path } => {
                path.references(out);
            }
            Self::Tuple { elements } => {
                elements.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirPattern {}
impl Record for HirPattern {
    const KIND: u16 = 41;
}
impl References for HirExceptHandler {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.error_type.references(out);
        self.error_resolved_type.references(out);
        self.name.references(out);
        self.body.references(out);
    }
}
impl sealed::Sealed for HirExceptHandler {}
impl Record for HirExceptHandler {
    const KIND: u16 = 32;
}
impl References for HirFStringPart {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Literal(v0) => {
                v0.references(out);
            }
            Self::Expr(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirFStringPart {}
impl Record for HirFStringPart {
    const KIND: u16 = 34;
}
impl References for HirIteratorOp {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Iter => {}
            Self::Next => {}
            Self::Reversed => {}
            Self::Map => {}
            Self::Filter => {}
            Self::Zip => {}
            Self::Enumerate => {}
        }
    }
}
impl sealed::Sealed for HirIteratorOp {}
impl Record for HirIteratorOp {
    const KIND: u16 = 37;
}
impl References for HirTupleTargetBinding {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Name(v0) => {
                v0.references(out);
            }
            Self::Field { object, field } => {
                object.references(out);
                field.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirTupleTargetBinding {}
impl Record for HirTupleTargetBinding {
    const KIND: u16 = 63;
}
impl References for HirTupleTarget {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.binding.references(out);
        self.ty.references(out);
        self.rebind_existing.references(out);
    }
}
impl sealed::Sealed for HirTupleTarget {}
impl Record for HirTupleTarget {
    const KIND: u16 = 62;
}
impl References for PythonRecordExpansion {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.span.references(out);
        self.fields.references(out);
    }
}
impl sealed::Sealed for PythonRecordExpansion {}
impl Record for PythonRecordExpansion {
    const KIND: u16 = 99;
}
