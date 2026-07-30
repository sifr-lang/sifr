use crate::{HirAsyncWithKind, HirExpr, HirStmt, HirTupleTargetBinding};
use sifr_ir::{FlowEffect, FlowExitKind, MutableArgumentTarget};
use sifr_type_system::{OwnershipKind, ReceiverConvention, Type};

pub(super) fn stmt_label(stmt: &HirStmt) -> &'static str {
    match stmt {
        HirStmt::Let { .. } => "let",
        HirStmt::Assign { .. } => "assign",
        HirStmt::AugAssign { .. } => "aug_assign",
        HirStmt::Return { .. } => "return",
        HirStmt::Expr { .. } => "expr",
        HirStmt::If { .. } => "if",
        HirStmt::While { .. } => "while",
        HirStmt::For { .. } => "for",
        HirStmt::AsyncFor { .. } => "async_for",
        HirStmt::Break => "break",
        HirStmt::Continue => "continue",
        HirStmt::TupleUnpack { .. } => "tuple_unpack",
        HirStmt::StarUnpack { .. } => "star_unpack",
        HirStmt::Pass => "pass",
        HirStmt::Assert { .. } => "assert",
        HirStmt::Raise { .. } => "raise",
        HirStmt::TryExcept { .. } => "try_except",
        HirStmt::TryFinally { .. } => "try_finally",
        HirStmt::FieldAssign { .. } => "field_assign",
        HirStmt::NestedFieldAssign { .. } => "nested_field_assign",
        HirStmt::SubscriptAssign { .. } => "subscript_assign",
        HirStmt::NestedSubscriptAssign { .. } => "nested_subscript_assign",
        HirStmt::AttributeNestedSubscriptAssign { .. } => "attribute_nested_subscript_assign",
        HirStmt::SubscriptAugAssign { .. } => "subscript_aug_assign",
        HirStmt::AttributeAugAssign { .. } => "attribute_aug_assign",
        HirStmt::AttributeSubscriptAssign { .. } => "attribute_subscript_assign",
        HirStmt::Delete { .. } => "delete",
        HirStmt::Yield { .. } => "yield",
        HirStmt::With { .. } => "with",
        HirStmt::AsyncWith { .. } => "async_with",
        HirStmt::NestedFunction { .. } => "nested_function",
        HirStmt::Match { .. } => "match",
    }
}

pub(super) fn stmt_effects(stmt: &HirStmt) -> Vec<FlowEffect> {
    let mut effects = Vec::new();
    match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } => {
            effects.push(FlowEffect::Define {
                binding: name.clone(),
                ty: ty.clone(),
            });
            expr_effects(value, &mut effects);
        }
        HirStmt::Assign { name, value } => {
            effects.push(FlowEffect::Assign {
                binding: name.clone(),
            });
            effects.push(FlowEffect::ClearNarrowing {
                binding: name.clone(),
            });
            expr_effects(value, &mut effects);
        }
        HirStmt::AugAssign { name, value, .. } => {
            effects.push(FlowEffect::Mutation {
                target: name.clone(),
                operation: "aug_assign".to_string(),
            });
            expr_effects(value, &mut effects);
        }
        HirStmt::Return { value } => {
            effects.push(FlowEffect::Exit {
                kind: FlowExitKind::Return,
            });
            if let Some(value) = value {
                expr_effects(value, &mut effects);
            }
        }
        HirStmt::Raise { value } => {
            effects.push(FlowEffect::Exit {
                kind: FlowExitKind::Raise,
            });
            expr_effects(value, &mut effects);
        }
        HirStmt::Break => effects.push(FlowEffect::Exit {
            kind: FlowExitKind::Break,
        }),
        HirStmt::Continue => effects.push(FlowEffect::Exit {
            kind: FlowExitKind::Continue,
        }),
        HirStmt::Expr { expr }
        | HirStmt::Assert { test: expr, .. }
        | HirStmt::Yield { value: expr } => expr_effects(expr, &mut effects),
        HirStmt::If { condition, .. } | HirStmt::While { condition, .. } => {
            effects.extend(condition_narrowing_effects(condition, true));
            effects.extend(condition_narrowing_effects(condition, false));
            expr_effects(condition, &mut effects);
        }
        HirStmt::For { target, iter, .. } | HirStmt::AsyncFor { target, iter, .. } => {
            effects.push(FlowEffect::Assign {
                binding: target.clone(),
            });
            expr_effects(iter, &mut effects);
        }
        HirStmt::TupleUnpack { targets, value } => {
            for target in targets {
                match &target.binding {
                    HirTupleTargetBinding::Name(name) => effects.push(FlowEffect::Assign {
                        binding: name.clone(),
                    }),
                    HirTupleTargetBinding::Field { object, field } => {
                        effects.push(FlowEffect::Mutation {
                            target: object.clone(),
                            operation: format!("field {field}"),
                        });
                    }
                }
            }
            expr_effects(value, &mut effects);
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => {
            for (name, _) in before.iter().chain(std::iter::once(star)).chain(after) {
                effects.push(FlowEffect::Assign {
                    binding: name.clone(),
                });
            }
            expr_effects(value, &mut effects);
        }
        HirStmt::FieldAssign {
            object,
            field,
            value,
            ..
        }
        | HirStmt::NestedFieldAssign {
            object,
            nested_field: field,
            value,
            ..
        }
        | HirStmt::AttributeAugAssign {
            object,
            field,
            value,
            ..
        } => {
            effects.push(FlowEffect::Mutation {
                target: object.clone(),
                operation: format!("field {field}"),
            });
            expr_effects(value, &mut effects);
        }
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            ..
        }
        | HirStmt::SubscriptAugAssign {
            object,
            index,
            value,
            ..
        } => {
            effects.push(FlowEffect::Mutation {
                target: object.clone(),
                operation: "subscript".to_string(),
            });
            expr_effects(index, &mut effects);
            expr_effects(value, &mut effects);
        }
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            ..
        } => {
            effects.push(FlowEffect::Mutation {
                target: object.clone(),
                operation: "nested_subscript".to_string(),
            });
            expr_effects(outer_index, &mut effects);
            expr_effects(inner_index, &mut effects);
            expr_effects(value, &mut effects);
        }
        HirStmt::AttributeNestedSubscriptAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            ..
        } => {
            effects.push(FlowEffect::Mutation {
                target: object.clone(),
                operation: format!("field {field} nested_subscript"),
            });
            expr_effects(outer_index, &mut effects);
            expr_effects(inner_index, &mut effects);
            expr_effects(value, &mut effects);
        }
        HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            ..
        } => {
            effects.push(FlowEffect::Mutation {
                target: object.clone(),
                operation: format!("field {field} subscript"),
            });
            expr_effects(index, &mut effects);
            expr_effects(value, &mut effects);
        }
        HirStmt::Delete { object, index } => {
            if let Some(target) = expr_target_name(object) {
                effects.push(FlowEffect::Mutation {
                    target,
                    operation: "delete".to_string(),
                });
            }
            expr_effects(object, &mut effects);
            expr_effects(index, &mut effects);
        }
        HirStmt::With { items, .. } => {
            for item in items {
                effects.push(FlowEffect::Define {
                    binding: item.target.clone(),
                    ty: item.context.ty().clone(),
                });
                expr_effects(&item.context, &mut effects);
            }
        }
        HirStmt::AsyncWith { kind, target, .. } => {
            if let Some(target) = target {
                effects.push(FlowEffect::Define {
                    binding: target.clone(),
                    ty: Type::Unknown,
                });
            }
            match kind {
                HirAsyncWithKind::TaskTimeout { duration } => expr_effects(duration, &mut effects),
                HirAsyncWithKind::UserDefined { context, .. }
                | HirAsyncWithKind::Python { context, .. } => {
                    expr_effects(context, &mut effects);
                }
                HirAsyncWithKind::TaskGroup {
                    context: Some(context),
                } => expr_effects(context, &mut effects),
                HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { context: None } => {}
            }
        }
        HirStmt::Match { subject, .. } => expr_effects(subject, &mut effects),
        HirStmt::Pass
        | HirStmt::TryExcept { .. }
        | HirStmt::TryFinally { .. }
        | HirStmt::NestedFunction { .. } => {}
    }
    effects
}

fn expr_effects(expr: &HirExpr, effects: &mut Vec<FlowEffect>) {
    match expr {
        HirExpr::Name { name, ty, .. } if ty.ownership() == OwnershipKind::Move => {
            effects.push(FlowEffect::Borrow {
                binding: name.clone(),
                mutable: false,
            });
        }
        HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        } => {
            effects.push(FlowEffect::Call {
                callee: func.clone(),
            });
            if func == "anext" {
                if let Some(target) = args.first().and_then(expr_target_name) {
                    effects.push(FlowEffect::Mutation {
                        target,
                        operation: "async iterator next".to_string(),
                    });
                }
            }
            for (index, arg) in args.iter().enumerate() {
                argument_effects(
                    arg,
                    mutable_arg_places.get(index).and_then(Option::as_ref),
                    effects,
                );
            }
        }
        HirExpr::PythonCall { func, args, .. } => {
            effects.push(FlowEffect::Call {
                callee: func.clone(),
            });
            for arg in args {
                expr_effects(arg, effects);
            }
        }
        HirExpr::IntrinsicCall {
            intrinsic, args, ..
        } => {
            effects.push(FlowEffect::Call {
                callee: format!("compiler::{}", intrinsic.declaration_name()),
            });
            for arg in args {
                expr_effects(arg, effects);
            }
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            mutable_arg_places,
            ..
        } => {
            let target = expr_target_name(object).unwrap_or_else(|| "<expr>".to_string());
            effects.push(FlowEffect::Call {
                callee: format!("{target}.{method}"),
            });
            if *receiver_convention == Some(ReceiverConvention::MutableBorrow) {
                effects.push(FlowEffect::Mutation {
                    target,
                    operation: format!("method {method}"),
                });
            }
            expr_effects(object, effects);
            for (index, arg) in args.iter().enumerate() {
                argument_effects(
                    arg,
                    mutable_arg_places.get(index).and_then(Option::as_ref),
                    effects,
                );
            }
        }
        HirExpr::IteratorCall {
            op,
            args,
            mutable_arg_places,
            ..
        } => {
            effects.push(FlowEffect::Call {
                callee: format!("compiler::{op:?}"),
            });
            if *op == crate::hir_nodes::HirIteratorOp::Next {
                if let Some(target) = args.first().and_then(expr_target_name) {
                    effects.push(FlowEffect::Mutation {
                        target,
                        operation: "iterator next".to_string(),
                    });
                }
            }
            for (index, arg) in args.iter().enumerate() {
                argument_effects(
                    arg,
                    mutable_arg_places.get(index).and_then(Option::as_ref),
                    effects,
                );
            }
        }
        HirExpr::BinOp { left, right, .. }
        | HirExpr::ContainsOp {
            element: left,
            collection: right,
            ..
        } => {
            expr_effects(left, effects);
            expr_effects(right, effects);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => expr_effects(operand, effects),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_effects(left, effects);
            for comparator in comparators {
                expr_effects(comparator, effects);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::ConstructorCall { args: values, .. }
        | HirExpr::SuperCall { args: values, .. } => {
            for value in values {
                expr_effects(value, effects);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_effects(condition, effects);
            expr_effects(then_expr, effects);
            expr_effects(else_expr, effects);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            expr_effects(start, effects);
            expr_effects(end, effects);
            if let Some(step) = step {
                expr_effects(step, effects);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                expr_effects(key, effects);
            }
            for value in values {
                expr_effects(value, effects);
            }
        }
        HirExpr::Index { object, index, .. } => {
            expr_effects(object, effects);
            expr_effects(index, effects);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let crate::HirFStringPart::Expr(expr) = part {
                    expr_effects(expr, effects);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            expr_effects(object, effects);
            for bound in [start, stop, step].into_iter().flatten() {
                expr_effects(bound, effects);
            }
        }
        HirExpr::WalrusExpr { name, value, .. } => {
            effects.push(FlowEffect::Assign {
                binding: name.clone(),
            });
            expr_effects(value, effects);
        }
        HirExpr::Lambda { body, .. } => expr_effects(body, effects),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            expr_effects(expr, effects);
            for (_, iter, filter) in generators {
                expr_effects(iter, effects);
                if let Some(filter) = filter {
                    expr_effects(filter, effects);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            expr_effects(key_expr, effects);
            expr_effects(val_expr, effects);
            for (_, iter, filter) in generators {
                expr_effects(iter, effects);
                if let Some(filter) = filter {
                    expr_effects(filter, effects);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            expr_effects(expr, effects);
            expr_effects(iter, effects);
            if let Some(filter) = filter {
                expr_effects(filter, effects);
            }
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn argument_effects(
    argument: &HirExpr,
    target: Option<&MutableArgumentTarget>,
    effects: &mut Vec<FlowEffect>,
) {
    if matches!(target, Some(MutableArgumentTarget::Place(_))) {
        if let Some(binding) = expr_target_name(argument) {
            effects.push(FlowEffect::Borrow {
                binding,
                mutable: true,
            });
            return;
        }
    }
    expr_effects(argument, effects);
}

fn condition_narrowing_effects(condition: &HirExpr, is_true: bool) -> Vec<FlowEffect> {
    match condition {
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if ops.len() == 1 && comparators.len() == 1 => {
            let Some(binding) = expr_target_name(left) else {
                return Vec::new();
            };
            if !matches!(comparators[0], HirExpr::NoneLiteral) {
                return Vec::new();
            }
            let positive_is_none = ops[0] == "is";
            let condition_true_means_none = if is_true {
                positive_is_none
            } else {
                !positive_is_none
            };
            let narrowed_type = if condition_true_means_none {
                Type::None
            } else {
                remove_none_from_type(left.ty())
            };
            vec![FlowEffect::Narrow {
                binding,
                narrowed_type,
                condition: format!("{} None", ops[0]),
                is_true,
            }]
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "not" => {
            condition_narrowing_effects(operand, !is_true)
        }
        HirExpr::BoolOp { op, values, .. }
            if (op == "and" && is_true) || (op == "or" && !is_true) =>
        {
            values
                .iter()
                .flat_map(|value| condition_narrowing_effects(value, is_true))
                .collect()
        }
        HirExpr::Name { name, ty, .. } => vec![FlowEffect::Narrow {
            binding: name.clone(),
            narrowed_type: if is_true {
                remove_none_from_type(ty)
            } else {
                Type::None
            },
            condition: "truthiness".to_string(),
            is_true,
        }],
        _ => Vec::new(),
    }
}

fn remove_none_from_type(ty: &Type) -> Type {
    if let Type::Union(members) = ty.resolve_alias() {
        let narrowed: Vec<Type> = members
            .iter()
            .filter(|member| !matches!(member.resolve_alias(), Type::None))
            .cloned()
            .collect();
        match narrowed.as_slice() {
            [] => Type::Never,
            [only] => only.clone(),
            _ => Type::Union(narrowed),
        }
    } else {
        ty.clone()
    }
}

fn expr_target_name(expr: &HirExpr) -> Option<String> {
    match expr {
        HirExpr::Name { name, .. } => Some(name.clone()),
        HirExpr::FieldAccess { object, field, .. } => {
            expr_target_name(object).map(|base| format!("{base}.{field}"))
        }
        _ => None,
    }
}
