use super::{HirExpr, Record, RecordId, References, sealed};
impl References for HirExpr {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::IntLiteral(v0) => {
                v0.references(out);
            }
            Self::LargeIntLiteral(v0) => {
                v0.references(out);
            }
            Self::FloatLiteral(v0) => {
                v0.references(out);
            }
            Self::StringLiteral(v0) => {
                v0.references(out);
            }
            Self::BoolLiteral(v0) => {
                v0.references(out);
            }
            Self::NoneLiteral => {}
            Self::Name {
                name,
                binding_id,
                ty,
            } => {
                name.references(out);
                binding_id.references(out);
                ty.references(out);
            }
            Self::BinOp {
                left,
                op,
                right,
                ty,
            } => {
                left.references(out);
                op.references(out);
                right.references(out);
                ty.references(out);
            }
            Self::UnaryOp { op, operand, ty } => {
                op.references(out);
                operand.references(out);
                ty.references(out);
            }
            Self::Compare {
                left,
                ops,
                comparators,
                ty,
            } => {
                left.references(out);
                ops.references(out);
                comparators.references(out);
                ty.references(out);
            }
            Self::BoolOp { op, values, ty } => {
                op.references(out);
                values.references(out);
                ty.references(out);
            }
            Self::Call {
                func,
                args,
                mutable_arg_places,
                ty,
            } => {
                func.references(out);
                args.references(out);
                mutable_arg_places.references(out);
                ty.references(out);
            }
            Self::GenericCall {
                func,
                type_args,
                args,
                mutable_arg_places,
                ty,
            } => {
                func.references(out);
                type_args.references(out);
                args.references(out);
                mutable_arg_places.references(out);
                ty.references(out);
            }
            Self::PythonCall {
                func,
                args,
                provided_arguments,
                record_expansions,
                ty,
            } => {
                func.references(out);
                args.references(out);
                provided_arguments.references(out);
                record_expansions.references(out);
                ty.references(out);
            }
            Self::IntrinsicCall {
                intrinsic,
                args,
                ty,
                call_range,
                arg_ranges,
            } => {
                intrinsic.references(out);
                args.references(out);
                ty.references(out);
                call_range.references(out);
                arg_ranges.references(out);
            }
            Self::Await { value, ty } => {
                value.references(out);
                ty.references(out);
            }
            Self::IteratorCall {
                op,
                args,
                mutable_arg_places,
                ty,
            } => {
                op.references(out);
                args.references(out);
                mutable_arg_places.references(out);
                ty.references(out);
            }
            Self::IfExpr {
                condition,
                then_expr,
                else_expr,
                ty,
            } => {
                condition.references(out);
                then_expr.references(out);
                else_expr.references(out);
                ty.references(out);
            }
            Self::RangeLiteral {
                start,
                end,
                step,
                ty,
            } => {
                start.references(out);
                end.references(out);
                step.references(out);
                ty.references(out);
            }
            Self::ListLiteral { elements, ty } => {
                elements.references(out);
                ty.references(out);
            }
            Self::SetLiteral { elements, ty } => {
                elements.references(out);
                ty.references(out);
            }
            Self::DictLiteral { keys, values, ty } => {
                keys.references(out);
                values.references(out);
                ty.references(out);
            }
            Self::TupleLiteral { elements, ty } => {
                elements.references(out);
                ty.references(out);
            }
            Self::Index { object, index, ty } => {
                object.references(out);
                index.references(out);
                ty.references(out);
            }
            Self::MethodCall {
                object,
                method,
                args,
                receiver_convention,
                receiver_target,
                mutable_arg_places,
                source,
                ty,
            } => {
                object.references(out);
                method.references(out);
                args.references(out);
                receiver_convention.references(out);
                receiver_target.references(out);
                mutable_arg_places.references(out);
                source.references(out);
                ty.references(out);
            }
            Self::ContainsOp {
                element,
                collection,
                ty,
            } => {
                element.references(out);
                collection.references(out);
                ty.references(out);
            }
            Self::FString { parts, ty } => {
                parts.references(out);
                ty.references(out);
            }
            Self::TemplateString(v0) => {
                v0.references(out);
            }
            Self::Slice {
                object,
                start,
                stop,
                step,
                ty,
            } => {
                object.references(out);
                start.references(out);
                stop.references(out);
                step.references(out);
                ty.references(out);
            }
            Self::WalrusExpr { name, value, ty } => {
                name.references(out);
                value.references(out);
                ty.references(out);
            }
            Self::FieldAccess { object, field, ty } => {
                object.references(out);
                field.references(out);
                ty.references(out);
            }
            Self::StructuralRecordProject { source, fields, ty } => {
                source.references(out);
                fields.references(out);
                ty.references(out);
            }
            Self::ConstructorCall {
                class_name,
                args,
                ty,
            } => {
                class_name.references(out);
                args.references(out);
                ty.references(out);
            }
            Self::QuestionMark { expr, ty } => {
                expr.references(out);
                ty.references(out);
            }
            Self::OkWrap { value, ty } => {
                value.references(out);
                ty.references(out);
            }
            Self::ErrWrap { value, ty } => {
                value.references(out);
                ty.references(out);
            }
            Self::SuperCall {
                parent_class,
                parent_type,
                method,
                args,
                ty,
            } => {
                parent_class.references(out);
                parent_type.references(out);
                method.references(out);
                args.references(out);
                ty.references(out);
            }
            Self::Lambda { params, body, ty } => {
                params.references(out);
                body.references(out);
                ty.references(out);
            }
            Self::ListComp {
                expr,
                generators,
                ty,
            } => {
                expr.references(out);
                generators.references(out);
                ty.references(out);
            }
            Self::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } => {
                key_expr.references(out);
                val_expr.references(out);
                generators.references(out);
                ty.references(out);
            }
            Self::SetComp {
                expr,
                generators,
                ty,
            } => {
                expr.references(out);
                generators.references(out);
                ty.references(out);
            }
            Self::GeneratorExpr {
                expr,
                var,
                iter,
                filter,
                ty,
            } => {
                expr.references(out);
                var.references(out);
                iter.references(out);
                filter.references(out);
                ty.references(out);
            }
            Self::EnumVariant {
                enum_name,
                variant,
                ty,
            } => {
                enum_name.references(out);
                variant.references(out);
                ty.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirExpr {}
impl Record for HirExpr {
    const KIND: u16 = 33;
}
