//! Keep expression cloning owned by the IR crate.
//!
//! Deriving Clone places an inline hint on this large recursive implementation.
//! Downstream compiler crates then generate repeated copies of it. An ordinary
//! non-generic method preserves deep cloning without exporting that inline body.
//! Exhaustive patterns keep new variants and fields a compile-time obligation.

use crate::HirExpr;

impl Clone for HirExpr {
    fn clone(&self) -> Self {
        match self {
            Self::IntLiteral(field0) => Self::IntLiteral(*field0),
            Self::LargeIntLiteral(field0) => Self::LargeIntLiteral(field0.clone()),
            Self::FloatLiteral(field0) => Self::FloatLiteral(*field0),
            Self::StringLiteral(field0) => Self::StringLiteral(field0.clone()),
            Self::BoolLiteral(field0) => Self::BoolLiteral(*field0),
            Self::NoneLiteral => Self::NoneLiteral,
            Self::Name {
                name,
                binding_id,
                ty,
            } => Self::Name {
                name: name.clone(),
                binding_id: *binding_id,
                ty: ty.clone(),
            },
            Self::BinOp {
                left,
                op,
                right,
                ty,
            } => Self::BinOp {
                left: left.clone(),
                op: op.clone(),
                right: right.clone(),
                ty: ty.clone(),
            },
            Self::UnaryOp { op, operand, ty } => Self::UnaryOp {
                op: op.clone(),
                operand: operand.clone(),
                ty: ty.clone(),
            },
            Self::Compare {
                left,
                ops,
                comparators,
                ty,
            } => Self::Compare {
                left: left.clone(),
                ops: ops.clone(),
                comparators: comparators.clone(),
                ty: ty.clone(),
            },
            Self::BoolOp { op, values, ty } => Self::BoolOp {
                op: op.clone(),
                values: values.clone(),
                ty: ty.clone(),
            },
            Self::Call {
                func,
                args,
                mutable_arg_places,
                ty,
            } => Self::Call {
                func: func.clone(),
                args: args.clone(),
                mutable_arg_places: mutable_arg_places.clone(),
                ty: ty.clone(),
            },
            Self::GenericCall {
                func,
                type_args,
                args,
                mutable_arg_places,
                ty,
            } => Self::GenericCall {
                func: func.clone(),
                type_args: type_args.clone(),
                args: args.clone(),
                mutable_arg_places: mutable_arg_places.clone(),
                ty: ty.clone(),
            },
            Self::PythonCall {
                func,
                args,
                provided_arguments,
                record_expansions,
                ty,
            } => Self::PythonCall {
                func: func.clone(),
                args: args.clone(),
                provided_arguments: provided_arguments.clone(),
                record_expansions: record_expansions.clone(),
                ty: ty.clone(),
            },
            Self::IntrinsicCall {
                intrinsic,
                args,
                ty,
                call_range,
                arg_ranges,
            } => Self::IntrinsicCall {
                intrinsic: *intrinsic,
                args: args.clone(),
                ty: ty.clone(),
                call_range: *call_range,
                arg_ranges: arg_ranges.clone(),
            },
            Self::Await { value, ty } => Self::Await {
                value: value.clone(),
                ty: ty.clone(),
            },
            Self::IteratorCall {
                op,
                args,
                mutable_arg_places,
                ty,
            } => Self::IteratorCall {
                op: op.clone(),
                args: args.clone(),
                mutable_arg_places: mutable_arg_places.clone(),
                ty: ty.clone(),
            },
            Self::IfExpr {
                condition,
                then_expr,
                else_expr,
                ty,
            } => Self::IfExpr {
                condition: condition.clone(),
                then_expr: then_expr.clone(),
                else_expr: else_expr.clone(),
                ty: ty.clone(),
            },
            Self::RangeLiteral {
                start,
                end,
                step,
                ty,
            } => Self::RangeLiteral {
                start: start.clone(),
                end: end.clone(),
                step: step.clone(),
                ty: ty.clone(),
            },
            Self::ListLiteral { elements, ty } => Self::ListLiteral {
                elements: elements.clone(),
                ty: ty.clone(),
            },
            Self::SetLiteral { elements, ty } => Self::SetLiteral {
                elements: elements.clone(),
                ty: ty.clone(),
            },
            Self::DictLiteral { keys, values, ty } => Self::DictLiteral {
                keys: keys.clone(),
                values: values.clone(),
                ty: ty.clone(),
            },
            Self::TupleLiteral { elements, ty } => Self::TupleLiteral {
                elements: elements.clone(),
                ty: ty.clone(),
            },
            Self::Index { object, index, ty } => Self::Index {
                object: object.clone(),
                index: index.clone(),
                ty: ty.clone(),
            },
            Self::MethodCall {
                object,
                method,
                args,
                authority,
                receiver_convention,
                receiver_target,
                mutable_arg_places,
                source,
                ty,
            } => Self::MethodCall {
                object: object.clone(),
                method: method.clone(),
                args: args.clone(),
                authority: authority.clone(),
                receiver_convention: *receiver_convention,
                receiver_target: receiver_target.clone(),
                mutable_arg_places: mutable_arg_places.clone(),
                source: source.clone(),
                ty: ty.clone(),
            },
            Self::ContainsOp {
                element,
                collection,
                ty,
            } => Self::ContainsOp {
                element: element.clone(),
                collection: collection.clone(),
                ty: ty.clone(),
            },
            Self::FString { parts, ty } => Self::FString {
                parts: parts.clone(),
                ty: ty.clone(),
            },
            Self::TemplateString(field0) => Self::TemplateString(field0.clone()),
            Self::Slice {
                object,
                start,
                stop,
                step,
                ty,
            } => Self::Slice {
                object: object.clone(),
                start: start.clone(),
                stop: stop.clone(),
                step: step.clone(),
                ty: ty.clone(),
            },
            Self::WalrusExpr { name, value, ty } => Self::WalrusExpr {
                name: name.clone(),
                value: value.clone(),
                ty: ty.clone(),
            },
            Self::FieldAccess { object, field, ty } => Self::FieldAccess {
                object: object.clone(),
                field: field.clone(),
                ty: ty.clone(),
            },
            Self::StructuralRecordProject { source, fields, ty } => Self::StructuralRecordProject {
                source: source.clone(),
                fields: fields.clone(),
                ty: ty.clone(),
            },
            Self::ConstructorCall {
                class_name,
                args,
                ty,
            } => Self::ConstructorCall {
                class_name: class_name.clone(),
                args: args.clone(),
                ty: ty.clone(),
            },
            Self::QuestionMark { expr, ty } => Self::QuestionMark {
                expr: expr.clone(),
                ty: ty.clone(),
            },
            Self::OkWrap { value, ty } => Self::OkWrap {
                value: value.clone(),
                ty: ty.clone(),
            },
            Self::ErrWrap { value, ty } => Self::ErrWrap {
                value: value.clone(),
                ty: ty.clone(),
            },
            Self::SuperCall {
                parent_class,
                parent_type,
                method,
                args,
                ty,
            } => Self::SuperCall {
                parent_class: parent_class.clone(),
                parent_type: parent_type.clone(),
                method: method.clone(),
                args: args.clone(),
                ty: ty.clone(),
            },
            Self::Lambda { params, body, ty } => Self::Lambda {
                params: params.clone(),
                body: body.clone(),
                ty: ty.clone(),
            },
            Self::ListComp {
                expr,
                generators,
                ty,
            } => Self::ListComp {
                expr: expr.clone(),
                generators: generators.clone(),
                ty: ty.clone(),
            },
            Self::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } => Self::DictComp {
                key_expr: key_expr.clone(),
                val_expr: val_expr.clone(),
                generators: generators.clone(),
                ty: ty.clone(),
            },
            Self::SetComp {
                expr,
                generators,
                ty,
            } => Self::SetComp {
                expr: expr.clone(),
                generators: generators.clone(),
                ty: ty.clone(),
            },
            Self::GeneratorExpr {
                expr,
                var,
                iter,
                filter,
                ty,
            } => Self::GeneratorExpr {
                expr: expr.clone(),
                var: var.clone(),
                iter: iter.clone(),
                filter: filter.clone(),
                ty: ty.clone(),
            },
            Self::EnumVariant {
                enum_name,
                variant,
                ty,
            } => Self::EnumVariant {
                enum_name: enum_name.clone(),
                variant: variant.clone(),
                ty: ty.clone(),
            },
        }
    }
}
