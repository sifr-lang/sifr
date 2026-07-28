use crate::{
    HirAsyncWithKind, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt, HirWithItemKind,
};
use sifr_type_system::Type;

/// Transform every stored type in a function, including nested expression and
/// statement metadata. This keeps identity rewrites from stopping at public
/// signatures while stale local views remain in code-generation HIR.
pub fn transform_hir_function_types<F>(function: &mut HirFunction, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    for param in &mut function.params {
        transform_type(&mut param.ty, transform);
        if let Some(default) = &mut param.default {
            transform_expr(default, transform);
        }
    }
    transform_type(&mut function.return_type, transform);
    transform_stmts(&mut function.body, transform);
}

fn transform_type<F>(ty: &mut Type, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    transform(ty);
}

fn transform_expr<F>(expr: &mut HirExpr, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    match expr {
        HirExpr::Name { ty, .. } | HirExpr::EnumVariant { ty, .. } => {
            transform_type(ty, transform);
        }
        HirExpr::BinOp {
            left, right, ty, ..
        } => {
            transform_expr(left, transform);
            transform_expr(right, transform);
            transform_type(ty, transform);
        }
        HirExpr::UnaryOp { operand, ty, .. }
        | HirExpr::Await {
            value: operand, ty, ..
        }
        | HirExpr::WalrusExpr {
            value: operand, ty, ..
        }
        | HirExpr::FieldAccess {
            object: operand,
            ty,
            ..
        }
        | HirExpr::QuestionMark {
            expr: operand, ty, ..
        }
        | HirExpr::OkWrap {
            value: operand, ty, ..
        }
        | HirExpr::ErrWrap {
            value: operand, ty, ..
        } => {
            transform_expr(operand, transform);
            transform_type(ty, transform);
        }
        HirExpr::Compare {
            left,
            comparators,
            ty,
            ..
        } => {
            transform_expr(left, transform);
            transform_exprs(comparators, transform);
            transform_type(ty, transform);
        }
        HirExpr::BoolOp { values, ty, .. }
        | HirExpr::IntrinsicCall {
            args: values, ty, ..
        }
        | HirExpr::ListLiteral {
            elements: values,
            ty,
        }
        | HirExpr::SetLiteral {
            elements: values,
            ty,
        }
        | HirExpr::TupleLiteral {
            elements: values,
            ty,
        }
        | HirExpr::Call {
            args: values, ty, ..
        }
        | HirExpr::IteratorCall {
            args: values, ty, ..
        }
        | HirExpr::ConstructorCall {
            args: values, ty, ..
        } => {
            transform_exprs(values, transform);
            transform_type(ty, transform);
        }
        HirExpr::SuperCall {
            args,
            parent_type,
            ty,
            ..
        } => {
            transform_exprs(args, transform);
            transform_type(parent_type, transform);
            transform_type(ty, transform);
        }
        HirExpr::PythonCall { args, ty, .. } => {
            transform_exprs(args, transform);
            transform_type(ty, transform);
        }
        HirExpr::MethodCall {
            object, args, ty, ..
        } => {
            transform_expr(object, transform);
            transform_exprs(args, transform);
            transform_type(ty, transform);
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ty,
        } => {
            transform_expr(condition, transform);
            transform_expr(then_expr, transform);
            transform_expr(else_expr, transform);
            transform_type(ty, transform);
        }
        HirExpr::RangeLiteral {
            start,
            end,
            step,
            ty,
        } => {
            transform_expr(start, transform);
            transform_expr(end, transform);
            if let Some(step) = step {
                transform_expr(step, transform);
            }
            transform_type(ty, transform);
        }
        HirExpr::DictLiteral {
            keys, values, ty, ..
        } => {
            transform_exprs(keys, transform);
            transform_exprs(values, transform);
            transform_type(ty, transform);
        }
        HirExpr::Index {
            object, index, ty, ..
        } => {
            transform_expr(object, transform);
            transform_expr(index, transform);
            transform_type(ty, transform);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ty,
        } => {
            transform_expr(element, transform);
            transform_expr(collection, transform);
            transform_type(ty, transform);
        }
        HirExpr::FString { parts, ty } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    transform_expr(expr, transform);
                }
            }
            transform_type(ty, transform);
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ty,
        } => {
            transform_expr(object, transform);
            for bound in [start, stop, step].into_iter().flatten() {
                transform_expr(bound, transform);
            }
            transform_type(ty, transform);
        }
        HirExpr::Lambda {
            params, body, ty, ..
        } => {
            for param in params {
                transform_type(&mut param.ty, transform);
                if let Some(default) = &mut param.default {
                    transform_expr(default, transform);
                }
            }
            transform_expr(body, transform);
            transform_type(ty, transform);
        }
        HirExpr::ListComp {
            expr,
            generators,
            ty,
        }
        | HirExpr::SetComp {
            expr,
            generators,
            ty,
        } => {
            transform_expr(expr, transform);
            transform_generators(generators, transform);
            transform_type(ty, transform);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ty,
        } => {
            transform_expr(key_expr, transform);
            transform_expr(val_expr, transform);
            transform_generators(generators, transform);
            transform_type(ty, transform);
        }
        HirExpr::GeneratorExpr {
            expr,
            iter,
            filter,
            ty,
            ..
        } => {
            transform_expr(expr, transform);
            transform_expr(iter, transform);
            if let Some(filter) = filter {
                transform_expr(filter, transform);
            }
            transform_type(ty, transform);
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral => {}
    }
}

fn transform_exprs<F>(expressions: &mut [HirExpr], transform: &mut F)
where
    F: FnMut(&mut Type),
{
    for expression in expressions {
        transform_expr(expression, transform);
    }
}

fn transform_generators<F>(generators: &mut [(String, HirExpr, Option<HirExpr>)], transform: &mut F)
where
    F: FnMut(&mut Type),
{
    for (_, iter, filter) in generators {
        transform_expr(iter, transform);
        if let Some(filter) = filter {
            transform_expr(filter, transform);
        }
    }
}

fn transform_stmts<F>(statements: &mut [HirStmt], transform: &mut F)
where
    F: FnMut(&mut Type),
{
    for statement in statements {
        transform_stmt(statement, transform);
    }
}

#[allow(clippy::too_many_lines)]
fn transform_stmt<F>(statement: &mut HirStmt, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    match statement {
        HirStmt::Let { ty, value, .. } => {
            transform_type(ty, transform);
            transform_expr(value, transform);
        }
        HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => transform_expr(value, transform),
        HirStmt::Return { value } => {
            if let Some(value) = value {
                transform_expr(value, transform);
            }
        }
        HirStmt::Expr { expr } => transform_expr(expr, transform),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            transform_expr(condition, transform);
            transform_stmts(then_body, transform);
            for (condition, body) in elif_clauses {
                transform_expr(condition, transform);
                transform_stmts(body, transform);
            }
            if let Some(body) = else_body {
                transform_stmts(body, transform);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            transform_expr(condition, transform);
            transform_stmts(body, transform);
            if let Some(body) = else_body {
                transform_stmts(body, transform);
            }
        }
        HirStmt::For {
            target_ty,
            iter,
            body,
            else_body,
            ..
        } => {
            transform_type(target_ty, transform);
            transform_expr(iter, transform);
            transform_stmts(body, transform);
            if let Some(body) = else_body {
                transform_stmts(body, transform);
            }
        }
        HirStmt::AsyncFor {
            target_ty,
            iter,
            iter_error_ty,
            close_error_ty,
            body,
            else_body,
            ..
        } => {
            transform_type(target_ty, transform);
            transform_type(iter_error_ty, transform);
            if let Some(ty) = close_error_ty {
                transform_type(ty, transform);
            }
            transform_expr(iter, transform);
            transform_stmts(body, transform);
            if let Some(body) = else_body {
                transform_stmts(body, transform);
            }
        }
        HirStmt::TupleUnpack { targets, value } => {
            for target in targets {
                transform_type(&mut target.ty, transform);
            }
            transform_expr(value, transform);
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => {
            for (_, ty) in before.iter_mut().chain(after.iter_mut()) {
                transform_type(ty, transform);
            }
            transform_type(&mut star.1, transform);
            transform_expr(value, transform);
        }
        HirStmt::Assert { test, msg } => {
            transform_expr(test, transform);
            if let Some(msg) = msg {
                transform_expr(msg, transform);
            }
        }
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => {
            transform_stmts(body, transform);
            for ty in body_error_types {
                transform_type(ty, transform);
            }
            for handler in handlers {
                if let Some(ty) = &mut handler.error_resolved_type {
                    transform_type(ty, transform);
                }
                transform_stmts(&mut handler.body, transform);
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            transform_stmts(body, transform);
            transform_stmts(finalbody, transform);
        }
        HirStmt::FieldAssign {
            field_ty, value, ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(value, transform);
        }
        HirStmt::NestedFieldAssign {
            field_ty,
            nested_field_ty,
            value,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_type(nested_field_ty, transform);
            transform_expr(value, transform);
        }
        HirStmt::SubscriptAssign {
            index,
            value,
            object_ty,
            ..
        }
        | HirStmt::SubscriptAugAssign {
            index,
            value,
            object_ty,
            ..
        } => {
            transform_type(object_ty, transform);
            transform_expr(index, transform);
            transform_expr(value, transform);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            object_ty,
            ..
        } => {
            transform_type(object_ty, transform);
            transform_expr(outer_index, transform);
            transform_expr(inner_index, transform);
            transform_expr(value, transform);
        }
        HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            field_ty,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(outer_index, transform);
            transform_expr(inner_index, transform);
            transform_expr(value, transform);
        }
        HirStmt::AttributeSubscriptAssign {
            index,
            value,
            field_ty,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(index, transform);
            transform_expr(value, transform);
        }
        HirStmt::Delete { object, index } => {
            transform_expr(object, transform);
            transform_expr(index, transform);
        }
        HirStmt::With { items, body } => {
            for item in items {
                transform_expr(&mut item.context, transform);
                if let HirWithItemKind::Python {
                    entered_type,
                    enter_error_type,
                    exit_error_type,
                    ..
                } = &mut item.kind
                {
                    transform_type(entered_type, transform);
                    transform_type(enter_error_type, transform);
                    transform_type(exit_error_type, transform);
                }
            }
            transform_stmts(body, transform);
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            transform_async_with_kind(kind, transform);
            transform_stmts(body, transform);
        }
        HirStmt::NestedFunction { func, .. } => transform_hir_function_types(func, transform),
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => {
            transform_expr(subject, transform);
            transform_type(subject_ty, transform);
            for arm in arms {
                transform_pattern(&mut arm.pattern, transform);
                if let Some(guard) = &mut arm.guard {
                    transform_expr(guard, transform);
                }
                transform_stmts(&mut arm.body, transform);
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
}

fn transform_async_with_kind<F>(kind: &mut HirAsyncWithKind, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    match kind {
        HirAsyncWithKind::TaskScope => {}
        HirAsyncWithKind::TaskGroup { context } => {
            if let Some(context) = context {
                transform_expr(context, transform);
            }
        }
        HirAsyncWithKind::TaskTimeout { duration } => transform_expr(duration, transform),
        HirAsyncWithKind::UserDefined {
            context,
            enter_value_ty,
            enter_error_ty,
            exit_error_ty,
        } => {
            transform_expr(context, transform);
            transform_type(enter_value_ty, transform);
            transform_type(enter_error_ty, transform);
            transform_type(exit_error_ty, transform);
        }
        HirAsyncWithKind::Python {
            context,
            entered_type,
            enter_error_type,
            exit_error_type,
            active_error_type,
            ..
        } => {
            transform_expr(context, transform);
            transform_type(entered_type, transform);
            transform_type(enter_error_type, transform);
            transform_type(exit_error_type, transform);
            transform_type(active_error_type, transform);
        }
    }
}

fn transform_pattern<F>(pattern: &mut HirPattern, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    match pattern {
        HirPattern::Capture { ty, .. } => transform_type(ty, transform),
        HirPattern::Literal { value } => transform_expr(value, transform),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                transform_pattern(pattern, transform);
            }
        }
        HirPattern::Class {
            class_type, fields, ..
        } => {
            transform_type(class_type, transform);
            for (_, pattern) in fields {
                transform_pattern(pattern, transform);
            }
        }
        HirPattern::Wildcard | HirPattern::None | HirPattern::Value { .. } => {}
    }
}
