//! Immutable traversal of all stored HIR expression and statement types.
//! Each node is visited once; recursive Type structure is the consumer's policy.
use crate::{
    HirAsyncWithKind, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt, HirWithItemKind,
};
use sifr_type_system::Type;

pub enum HirNode<'a> {
    Type(&'a Type),
    Expr(&'a HirExpr),
    Function(&'a HirFunction),
}

pub fn visit_hir_function<V>(function: &HirFunction, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    for param in &function.params {
        visit_type(&param.ty, visit);
        if let Some(default) = &param.default {
            visit_hir_expr(default, visit);
        }
    }
    visit_type(&function.return_type, visit);
    visit_stmts(&function.body, visit);
    visit(HirNode::Function(function));
}

fn visit_type<V>(ty: &Type, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    visit(HirNode::Type(ty));
}

pub fn visit_hir_expr<V>(expr: &HirExpr, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    match expr {
        HirExpr::Name { ty, .. } | HirExpr::EnumVariant { ty, .. } => {
            visit_type(ty, visit);
        }
        HirExpr::BinOp {
            left, right, ty, ..
        } => {
            visit_hir_expr(left, visit);
            visit_hir_expr(right, visit);
            visit_type(ty, visit);
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
        | HirExpr::StructuralRecordProject {
            source: operand,
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
            visit_hir_expr(operand, visit);
            visit_type(ty, visit);
        }
        HirExpr::Compare {
            left,
            comparators,
            ty,
            ..
        } => {
            visit_hir_expr(left, visit);
            visit_exprs(comparators, visit);
            visit_type(ty, visit);
        }
        HirExpr::GenericCall {
            type_args,
            args,
            ty,
            ..
        } => {
            for type_arg in type_args {
                visit_type(type_arg, visit);
            }
            visit_exprs(args, visit);
            visit_type(ty, visit);
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
            visit_exprs(values, visit);
            visit_type(ty, visit);
        }
        HirExpr::SuperCall {
            args,
            parent_type,
            ty,
            ..
        } => {
            visit_exprs(args, visit);
            visit_type(parent_type, visit);
            visit_type(ty, visit);
        }
        HirExpr::PythonCall { args, ty, .. } => {
            visit_exprs(args, visit);
            visit_type(ty, visit);
        }
        HirExpr::MethodCall {
            object, args, ty, ..
        } => {
            visit_hir_expr(object, visit);
            visit_exprs(args, visit);
            visit_type(ty, visit);
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ty,
        } => {
            visit_hir_expr(condition, visit);
            visit_hir_expr(then_expr, visit);
            visit_hir_expr(else_expr, visit);
            visit_type(ty, visit);
        }
        HirExpr::RangeLiteral {
            start,
            end,
            step,
            ty,
        } => {
            visit_hir_expr(start, visit);
            visit_hir_expr(end, visit);
            if let Some(step) = step {
                visit_hir_expr(step, visit);
            }
            visit_type(ty, visit);
        }
        HirExpr::DictLiteral {
            keys, values, ty, ..
        } => {
            visit_exprs(keys, visit);
            visit_exprs(values, visit);
            visit_type(ty, visit);
        }
        HirExpr::Index {
            object, index, ty, ..
        } => {
            visit_hir_expr(object, visit);
            visit_hir_expr(index, visit);
            visit_type(ty, visit);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ty,
        } => {
            visit_hir_expr(element, visit);
            visit_hir_expr(collection, visit);
            visit_type(ty, visit);
        }
        HirExpr::FString { parts, ty } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    visit_hir_expr(expr, visit);
                }
            }
            visit_type(ty, visit);
        }
        HirExpr::TemplateString(template) => {
            template.for_each_value(&mut |value| {
                visit_hir_expr(value, visit);
            });
            for interpolation in &template.interpolations {
                visit_type(&interpolation.value_type, visit);
            }
            visit_type(&template.ty, visit);
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ty,
        } => {
            visit_hir_expr(object, visit);
            for bound in [start, stop, step].into_iter().flatten() {
                visit_hir_expr(bound, visit);
            }
            visit_type(ty, visit);
        }
        HirExpr::Lambda {
            params, body, ty, ..
        } => {
            for param in params {
                visit_type(&param.ty, visit);
                if let Some(default) = &param.default {
                    visit_hir_expr(default, visit);
                }
            }
            visit_hir_expr(body, visit);
            visit_type(ty, visit);
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
            visit_hir_expr(expr, visit);
            visit_generators(generators, visit);
            visit_type(ty, visit);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ty,
        } => {
            visit_hir_expr(key_expr, visit);
            visit_hir_expr(val_expr, visit);
            visit_generators(generators, visit);
            visit_type(ty, visit);
        }
        HirExpr::GeneratorExpr {
            expr,
            iter,
            filter,
            ty,
            ..
        } => {
            visit_hir_expr(expr, visit);
            visit_hir_expr(iter, visit);
            if let Some(filter) = filter {
                visit_hir_expr(filter, visit);
            }
            visit_type(ty, visit);
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral => {}
    }
    visit(HirNode::Expr(expr));
}

fn visit_exprs<V>(expressions: &[HirExpr], visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    for expression in expressions {
        visit_hir_expr(expression, visit);
    }
}

fn visit_generators<V>(generators: &[(String, HirExpr, Option<HirExpr>)], visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    for (_, iter, filter) in generators {
        visit_hir_expr(iter, visit);
        if let Some(filter) = filter {
            visit_hir_expr(filter, visit);
        }
    }
}

fn visit_stmts<V>(statements: &[HirStmt], visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    for statement in statements {
        visit_stmt(statement, visit);
    }
}

#[allow(clippy::too_many_lines)]
fn visit_stmt<V>(statement: &HirStmt, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    match statement {
        HirStmt::Let { ty, value, .. } => {
            visit_type(ty, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => visit_hir_expr(value, visit),
        HirStmt::Return { value } => {
            if let Some(value) = value {
                visit_hir_expr(value, visit);
            }
        }
        HirStmt::Expr { expr } => visit_hir_expr(expr, visit),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            visit_hir_expr(condition, visit);
            visit_stmts(then_body, visit);
            for (condition, body) in elif_clauses {
                visit_hir_expr(condition, visit);
                visit_stmts(body, visit);
            }
            if let Some(body) = else_body {
                visit_stmts(body, visit);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            visit_hir_expr(condition, visit);
            visit_stmts(body, visit);
            if let Some(body) = else_body {
                visit_stmts(body, visit);
            }
        }
        HirStmt::For {
            target_ty,
            iter,
            body,
            else_body,
            ..
        } => {
            visit_type(target_ty, visit);
            visit_hir_expr(iter, visit);
            visit_stmts(body, visit);
            if let Some(body) = else_body {
                visit_stmts(body, visit);
            }
        }
        HirStmt::AsyncFor {
            target_ty,
            iter,
            iter_error_ty,
            close_error_ty,
            active_error_ty,
            body,
            else_body,
            ..
        } => {
            visit_type(target_ty, visit);
            visit_type(iter_error_ty, visit);
            if let Some(ty) = close_error_ty {
                visit_type(ty, visit);
            }
            visit_type(active_error_ty, visit);
            visit_hir_expr(iter, visit);
            visit_stmts(body, visit);
            if let Some(body) = else_body {
                visit_stmts(body, visit);
            }
        }
        HirStmt::TupleUnpack { targets, value } => {
            for target in targets {
                visit_type(&target.ty, visit);
            }
            visit_hir_expr(value, visit);
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
            failure,
        } => {
            for target in before.iter().chain(after.iter()) {
                visit_type(&target.ty, visit);
            }
            visit_type(&star.ty, visit);
            visit_hir_expr(value, visit);
            if let Some(failure) = failure {
                visit_type(failure, visit);
            }
        }
        HirStmt::Assert { test, msg } => {
            visit_hir_expr(test, visit);
            if let Some(msg) = msg {
                visit_hir_expr(msg, visit);
            }
        }
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => {
            visit_stmts(body, visit);
            for ty in body_error_types {
                visit_type(ty, visit);
            }
            for handler in handlers {
                if let Some(ty) = &handler.error_resolved_type {
                    visit_type(ty, visit);
                }
                visit_stmts(&handler.body, visit);
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            visit_stmts(body, visit);
            visit_stmts(finalbody, visit);
        }
        HirStmt::FieldAssign {
            field_ty, value, ..
        } => {
            visit_type(field_ty, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::NestedFieldAssign {
            field_ty,
            nested_field_ty,
            value,
            ..
        } => {
            visit_type(field_ty, visit);
            visit_type(nested_field_ty, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::SubscriptAssign {
            index,
            value,
            object_ty,
            failure,
            ..
        } => {
            visit_type(object_ty, visit);
            if let Some(failure) = failure {
                visit_type(failure, visit);
            }
            visit_hir_expr(index, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::SubscriptAugAssign {
            index,
            value,
            object_ty,
            failure,
            ..
        } => {
            visit_type(object_ty, visit);
            if let Some(error_ty) = failure {
                visit_type(error_ty, visit);
            }
            visit_hir_expr(index, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            object_ty,
            outer_failure,
            inner_failure,
            ..
        } => {
            visit_type(object_ty, visit);
            if let Some(failure) = outer_failure {
                visit_type(failure, visit);
            }
            if let Some(failure) = inner_failure {
                visit_type(failure, visit);
            }
            visit_hir_expr(outer_index, visit);
            visit_hir_expr(inner_index, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            field_ty,
            outer_failure,
            inner_failure,
            ..
        } => {
            visit_type(field_ty, visit);
            if let Some(failure) = outer_failure {
                visit_type(failure, visit);
            }
            if let Some(failure) = inner_failure {
                visit_type(failure, visit);
            }
            visit_hir_expr(outer_index, visit);
            visit_hir_expr(inner_index, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::AttributeSubscriptAssign {
            index,
            value,
            field_ty,
            failure,
            ..
        } => {
            visit_type(field_ty, visit);
            if let Some(failure) = failure {
                visit_type(failure, visit);
            }
            visit_hir_expr(index, visit);
            visit_hir_expr(value, visit);
        }
        HirStmt::Delete {
            object,
            index,
            failure,
        } => {
            visit_hir_expr(object, visit);
            visit_hir_expr(index, visit);
            if let Some(failure) = failure {
                visit_type(failure, visit);
            }
        }
        HirStmt::With { items, body } => {
            for item in items {
                visit_hir_expr(&item.context, visit);
                if let HirWithItemKind::Python {
                    entered_type,
                    enter_error_type,
                    exit_error_type,
                    ..
                } = &item.kind
                {
                    visit_type(entered_type, visit);
                    visit_type(enter_error_type, visit);
                    visit_type(exit_error_type, visit);
                }
            }
            visit_stmts(body, visit);
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            visit_async_with_kind(kind, visit);
            visit_stmts(body, visit);
        }
        HirStmt::NestedFunction { func, .. } => visit_hir_function(func, visit),
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => {
            visit_hir_expr(subject, visit);
            visit_type(subject_ty, visit);
            for arm in arms {
                visit_pattern(&arm.pattern, visit);
                if let Some(guard) = &arm.guard {
                    visit_hir_expr(guard, visit);
                }
                visit_stmts(&arm.body, visit);
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
}

fn visit_async_with_kind<V>(kind: &HirAsyncWithKind, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    match kind {
        HirAsyncWithKind::TaskScope => {}
        HirAsyncWithKind::TaskGroup { context } => {
            if let Some(context) = context {
                visit_hir_expr(context, visit);
            }
        }
        HirAsyncWithKind::TaskTimeout { duration } => visit_hir_expr(duration, visit),
        HirAsyncWithKind::UserDefined {
            context,
            enter_value_ty,
            enter_error_ty,
            exit_error_ty,
            active_error_ty,
            ..
        } => {
            visit_hir_expr(context, visit);
            visit_type(enter_value_ty, visit);
            visit_type(enter_error_ty, visit);
            visit_type(exit_error_ty, visit);
            visit_type(active_error_ty, visit);
        }
        HirAsyncWithKind::Python {
            context,
            entered_type,
            enter_error_type,
            exit_error_type,
            active_error_type,
            ..
        } => {
            visit_hir_expr(context, visit);
            visit_type(entered_type, visit);
            visit_type(enter_error_type, visit);
            visit_type(exit_error_type, visit);
            visit_type(active_error_type, visit);
        }
    }
}

fn visit_pattern<V>(pattern: &HirPattern, visit: &mut V)
where
    V: FnMut(HirNode<'_>),
{
    match pattern {
        HirPattern::Capture { ty, .. } => visit_type(ty, visit),
        HirPattern::Literal { value } => visit_hir_expr(value, visit),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                visit_pattern(pattern, visit);
            }
        }
        HirPattern::Class {
            class_type, fields, ..
        } => {
            visit_type(class_type, visit);
            for (_, pattern) in fields {
                visit_pattern(pattern, visit);
            }
        }
        HirPattern::Wildcard | HirPattern::None | HirPattern::Value { .. } => {}
    }
}
