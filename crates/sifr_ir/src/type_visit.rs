use crate::{
    HirAsyncWithKind, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt,
    HirTupleTargetBinding, HirWithItemKind,
};
use sifr_type_system::Type;

/// Transform every stored type in a function, including nested expression and
/// statement metadata. This keeps identity rewrites from stopping at public
/// signatures while stale local views remain in code-generation HIR.
pub fn transform_hir_function_types<F>(function: &mut HirFunction, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    transform_hir_function(function, transform, &mut |_| {});
}

/// Visit every expression in a function mutably, including nested expressions
/// and statement bodies.
pub fn visit_hir_function_exprs_mut<F>(function: &mut HirFunction, visit: &mut F)
where
    F: FnMut(&mut HirExpr),
{
    transform_hir_function(function, &mut |_| {}, visit);
}

/// Visit every expression in a statement body mutably.
pub fn visit_hir_stmts_exprs_mut<F>(statements: &mut [HirStmt], visit: &mut F)
where
    F: FnMut(&mut HirExpr),
{
    transform_stmts(statements, &mut |_| {}, visit);
}

/// Visit every storage-root name carried directly by statements.
///
/// Expression-root names are intentionally excluded; combine this with
/// [`visit_hir_stmts_exprs_mut`] when a rewrite must cover both representations.
pub fn visit_hir_stmts_storage_roots_mut<F>(statements: &mut [HirStmt], visit: &mut F)
where
    F: FnMut(&mut String),
{
    visit_stmt_storage_roots(statements, visit);
}

fn visit_stmt_storage_roots<F>(statements: &mut [HirStmt], visit: &mut F)
where
    F: FnMut(&mut String),
{
    for statement in statements {
        match statement {
            HirStmt::FieldAssign { object, .. }
            | HirStmt::NestedFieldAssign { object, .. }
            | HirStmt::SubscriptAssign { object, .. }
            | HirStmt::NestedSubscriptAssign { object, .. }
            | HirStmt::AttributeNestedSubscriptAssign { object, .. }
            | HirStmt::SubscriptAugAssign { object, .. }
            | HirStmt::AttributeAugAssign { object, .. }
            | HirStmt::AttributeSubscriptAssign { object, .. } => visit(object),
            HirStmt::TupleUnpack { targets, .. } => {
                for target in targets {
                    if let HirTupleTargetBinding::Field { object, .. } = &mut target.binding {
                        visit(object);
                    }
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                visit_stmt_storage_roots(then_body, visit);
                for (_, body) in elif_clauses {
                    visit_stmt_storage_roots(body, visit);
                }
                if let Some(body) = else_body {
                    visit_stmt_storage_roots(body, visit);
                }
            }
            HirStmt::While {
                body, else_body, ..
            }
            | HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => {
                visit_stmt_storage_roots(body, visit);
                if let Some(body) = else_body {
                    visit_stmt_storage_roots(body, visit);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                visit_stmt_storage_roots(body, visit);
                for handler in handlers {
                    visit_stmt_storage_roots(&mut handler.body, visit);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                visit_stmt_storage_roots(body, visit);
                visit_stmt_storage_roots(finalbody, visit);
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                visit_stmt_storage_roots(body, visit);
            }
            HirStmt::NestedFunction { func, .. } => {
                visit_stmt_storage_roots(&mut func.body, visit);
            }
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    visit_stmt_storage_roots(&mut arm.body, visit);
                }
            }
            HirStmt::Let { .. }
            | HirStmt::Assign { .. }
            | HirStmt::AugAssign { .. }
            | HirStmt::Return { .. }
            | HirStmt::Expr { .. }
            | HirStmt::StarUnpack { .. }
            | HirStmt::Pass
            | HirStmt::Assert { .. }
            | HirStmt::Raise { .. }
            | HirStmt::Delete { .. }
            | HirStmt::Yield { .. }
            | HirStmt::Break
            | HirStmt::Continue => {}
        }
    }
}

fn transform_hir_function<F, G>(function: &mut HirFunction, transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    for param in &mut function.params {
        transform_type(&mut param.ty, transform);
        if let Some(default) = &mut param.default {
            transform_expr(default, transform, visit);
        }
    }
    transform_type(&mut function.return_type, transform);
    transform_stmts(&mut function.body, transform, visit);
}

fn transform_type<F>(ty: &mut Type, transform: &mut F)
where
    F: FnMut(&mut Type),
{
    transform(ty);
}

fn transform_expr<F, G>(expr: &mut HirExpr, transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    match expr {
        HirExpr::Name { ty, .. } | HirExpr::EnumVariant { ty, .. } => {
            transform_type(ty, transform);
        }
        HirExpr::BinOp {
            left, right, ty, ..
        } => {
            transform_expr(left, transform, visit);
            transform_expr(right, transform, visit);
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
            transform_expr(operand, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::Compare {
            left,
            comparators,
            ty,
            ..
        } => {
            transform_expr(left, transform, visit);
            transform_exprs(comparators, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::GenericCall {
            type_args,
            args,
            ty,
            ..
        } => {
            for type_arg in type_args {
                transform_type(type_arg, transform);
            }
            transform_exprs(args, transform, visit);
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
            transform_exprs(values, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::SuperCall {
            args,
            parent_type,
            ty,
            ..
        } => {
            transform_exprs(args, transform, visit);
            transform_type(parent_type, transform);
            transform_type(ty, transform);
        }
        HirExpr::PythonCall { args, ty, .. } => {
            transform_exprs(args, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::MethodCall {
            object, args, ty, ..
        } => {
            transform_expr(object, transform, visit);
            transform_exprs(args, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ty,
        } => {
            transform_expr(condition, transform, visit);
            transform_expr(then_expr, transform, visit);
            transform_expr(else_expr, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::RangeLiteral {
            start,
            end,
            step,
            ty,
        } => {
            transform_expr(start, transform, visit);
            transform_expr(end, transform, visit);
            if let Some(step) = step {
                transform_expr(step, transform, visit);
            }
            transform_type(ty, transform);
        }
        HirExpr::DictLiteral {
            keys, values, ty, ..
        } => {
            transform_exprs(keys, transform, visit);
            transform_exprs(values, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::Index {
            object, index, ty, ..
        } => {
            transform_expr(object, transform, visit);
            transform_expr(index, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ty,
        } => {
            transform_expr(element, transform, visit);
            transform_expr(collection, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::FString { parts, ty } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    transform_expr(expr, transform, visit);
                }
            }
            transform_type(ty, transform);
        }
        HirExpr::TemplateString(template) => {
            template.for_each_value_mut(&mut |value| {
                transform_expr(value, transform, visit);
            });
            for interpolation in &mut template.interpolations {
                transform_type(&mut interpolation.value_type, transform);
            }
            transform_type(&mut template.ty, transform);
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ty,
        } => {
            transform_expr(object, transform, visit);
            for bound in [start, stop, step].into_iter().flatten() {
                transform_expr(bound, transform, visit);
            }
            transform_type(ty, transform);
        }
        HirExpr::Lambda {
            params, body, ty, ..
        } => {
            for param in params {
                transform_type(&mut param.ty, transform);
                if let Some(default) = &mut param.default {
                    transform_expr(default, transform, visit);
                }
            }
            transform_expr(body, transform, visit);
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
            transform_expr(expr, transform, visit);
            transform_generators(generators, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ty,
        } => {
            transform_expr(key_expr, transform, visit);
            transform_expr(val_expr, transform, visit);
            transform_generators(generators, transform, visit);
            transform_type(ty, transform);
        }
        HirExpr::GeneratorExpr {
            expr,
            iter,
            filter,
            ty,
            ..
        } => {
            transform_expr(expr, transform, visit);
            transform_expr(iter, transform, visit);
            if let Some(filter) = filter {
                transform_expr(filter, transform, visit);
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
    visit(expr);
}

fn transform_exprs<F, G>(expressions: &mut [HirExpr], transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    for expression in expressions {
        transform_expr(expression, transform, visit);
    }
}

fn transform_generators<F, G>(
    generators: &mut [(String, HirExpr, Option<HirExpr>)],
    transform: &mut F,
    visit: &mut G,
) where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    for (_, iter, filter) in generators {
        transform_expr(iter, transform, visit);
        if let Some(filter) = filter {
            transform_expr(filter, transform, visit);
        }
    }
}

fn transform_stmts<F, G>(statements: &mut [HirStmt], transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    for statement in statements {
        transform_stmt(statement, transform, visit);
    }
}

#[allow(clippy::too_many_lines)]
fn transform_stmt<F, G>(statement: &mut HirStmt, transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    match statement {
        HirStmt::Let { ty, value, .. } => {
            transform_type(ty, transform);
            transform_expr(value, transform, visit);
        }
        HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => transform_expr(value, transform, visit),
        HirStmt::Return { value } => {
            if let Some(value) = value {
                transform_expr(value, transform, visit);
            }
        }
        HirStmt::Expr { expr } => transform_expr(expr, transform, visit),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            transform_expr(condition, transform, visit);
            transform_stmts(then_body, transform, visit);
            for (condition, body) in elif_clauses {
                transform_expr(condition, transform, visit);
                transform_stmts(body, transform, visit);
            }
            if let Some(body) = else_body {
                transform_stmts(body, transform, visit);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            transform_expr(condition, transform, visit);
            transform_stmts(body, transform, visit);
            if let Some(body) = else_body {
                transform_stmts(body, transform, visit);
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
            transform_expr(iter, transform, visit);
            transform_stmts(body, transform, visit);
            if let Some(body) = else_body {
                transform_stmts(body, transform, visit);
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
            transform_expr(iter, transform, visit);
            transform_stmts(body, transform, visit);
            if let Some(body) = else_body {
                transform_stmts(body, transform, visit);
            }
        }
        HirStmt::TupleUnpack { targets, value } => {
            for target in targets {
                transform_type(&mut target.ty, transform);
            }
            transform_expr(value, transform, visit);
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
            transform_expr(value, transform, visit);
        }
        HirStmt::Assert { test, msg } => {
            transform_expr(test, transform, visit);
            if let Some(msg) = msg {
                transform_expr(msg, transform, visit);
            }
        }
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => {
            transform_stmts(body, transform, visit);
            for ty in body_error_types {
                transform_type(ty, transform);
            }
            for handler in handlers {
                if let Some(ty) = &mut handler.error_resolved_type {
                    transform_type(ty, transform);
                }
                transform_stmts(&mut handler.body, transform, visit);
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            transform_stmts(body, transform, visit);
            transform_stmts(finalbody, transform, visit);
        }
        HirStmt::FieldAssign {
            field_ty, value, ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(value, transform, visit);
        }
        HirStmt::NestedFieldAssign {
            field_ty,
            nested_field_ty,
            value,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_type(nested_field_ty, transform);
            transform_expr(value, transform, visit);
        }
        HirStmt::SubscriptAssign {
            index,
            value,
            object_ty,
            ..
        } => {
            transform_type(object_ty, transform);
            transform_expr(index, transform, visit);
            transform_expr(value, transform, visit);
        }
        HirStmt::SubscriptAugAssign {
            index,
            value,
            object_ty,
            missing_key_error,
            ..
        } => {
            transform_type(object_ty, transform);
            if let Some(error_ty) = missing_key_error {
                transform_type(error_ty, transform);
            }
            transform_expr(index, transform, visit);
            transform_expr(value, transform, visit);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            object_ty,
            ..
        } => {
            transform_type(object_ty, transform);
            transform_expr(outer_index, transform, visit);
            transform_expr(inner_index, transform, visit);
            transform_expr(value, transform, visit);
        }
        HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            field_ty,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(outer_index, transform, visit);
            transform_expr(inner_index, transform, visit);
            transform_expr(value, transform, visit);
        }
        HirStmt::AttributeSubscriptAssign {
            index,
            value,
            field_ty,
            ..
        } => {
            transform_type(field_ty, transform);
            transform_expr(index, transform, visit);
            transform_expr(value, transform, visit);
        }
        HirStmt::Delete { object, index } => {
            transform_expr(object, transform, visit);
            transform_expr(index, transform, visit);
        }
        HirStmt::With { items, body } => {
            for item in items {
                transform_expr(&mut item.context, transform, visit);
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
            transform_stmts(body, transform, visit);
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            transform_async_with_kind(kind, transform, visit);
            transform_stmts(body, transform, visit);
        }
        HirStmt::NestedFunction { func, .. } => transform_hir_function(func, transform, visit),
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => {
            transform_expr(subject, transform, visit);
            transform_type(subject_ty, transform);
            for arm in arms {
                transform_pattern(&mut arm.pattern, transform, visit);
                if let Some(guard) = &mut arm.guard {
                    transform_expr(guard, transform, visit);
                }
                transform_stmts(&mut arm.body, transform, visit);
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
}

fn transform_async_with_kind<F, G>(kind: &mut HirAsyncWithKind, transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    match kind {
        HirAsyncWithKind::TaskScope => {}
        HirAsyncWithKind::TaskGroup { context } => {
            if let Some(context) = context {
                transform_expr(context, transform, visit);
            }
        }
        HirAsyncWithKind::TaskTimeout { duration } => transform_expr(duration, transform, visit),
        HirAsyncWithKind::UserDefined {
            context,
            enter_value_ty,
            enter_error_ty,
            exit_error_ty,
        } => {
            transform_expr(context, transform, visit);
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
            transform_expr(context, transform, visit);
            transform_type(entered_type, transform);
            transform_type(enter_error_type, transform);
            transform_type(exit_error_type, transform);
            transform_type(active_error_type, transform);
        }
    }
}

fn transform_pattern<F, G>(pattern: &mut HirPattern, transform: &mut F, visit: &mut G)
where
    F: FnMut(&mut Type),
    G: FnMut(&mut HirExpr),
{
    match pattern {
        HirPattern::Capture { ty, .. } => transform_type(ty, transform),
        HirPattern::Literal { value } => transform_expr(value, transform, visit),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                transform_pattern(pattern, transform, visit);
            }
        }
        HirPattern::Class {
            class_type, fields, ..
        } => {
            transform_type(class_type, transform);
            for (_, pattern) in fields {
                transform_pattern(pattern, transform, visit);
            }
        }
        HirPattern::Wildcard | HirPattern::None | HirPattern::Value { .. } => {}
    }
}
