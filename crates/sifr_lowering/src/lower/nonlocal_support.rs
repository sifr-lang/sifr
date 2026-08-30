use crate::hir_nodes::{HirExpr, HirStmt};
use ruff_text_size::Ranged;
use sifr_python_ast::{ExceptHandler, Stmt, StmtNonlocal};
use std::collections::HashSet;

use super::LowerCtx;

pub(in crate::lower) fn collect_declared_nonlocals(stmts: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_declared_nonlocals_into(stmts, &mut names);
    names
}

fn collect_declared_nonlocals_into(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Nonlocal(nonlocal) => {
                for name in &nonlocal.names {
                    out.insert(name.to_string());
                }
            }
            Stmt::If(if_stmt) => {
                collect_declared_nonlocals_into(&if_stmt.body, out);
                for clause in &if_stmt.elif_else_clauses {
                    collect_declared_nonlocals_into(&clause.body, out);
                }
            }
            Stmt::While(while_stmt) => {
                collect_declared_nonlocals_into(&while_stmt.body, out);
                collect_declared_nonlocals_into(&while_stmt.orelse, out);
            }
            Stmt::For(for_stmt) => {
                collect_declared_nonlocals_into(&for_stmt.body, out);
                collect_declared_nonlocals_into(&for_stmt.orelse, out);
            }
            Stmt::With(with_stmt) => {
                collect_declared_nonlocals_into(&with_stmt.body, out);
            }
            Stmt::Try(try_stmt) => {
                collect_declared_nonlocals_into(&try_stmt.body, out);
                collect_declared_nonlocals_into(&try_stmt.orelse, out);
                collect_declared_nonlocals_into(&try_stmt.finalbody, out);
                for handler in &try_stmt.handlers {
                    let ExceptHandler::ExceptHandler(handler) = handler;
                    collect_declared_nonlocals_into(&handler.body, out);
                }
            }
            Stmt::Match(match_stmt) => {
                for case in &match_stmt.cases {
                    collect_declared_nonlocals_into(&case.body, out);
                }
            }
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => {}
            _ => {}
        }
    }
}

pub(in crate::lower) fn lower_nonlocal(nonlocal: &StmtNonlocal, ctx: &mut LowerCtx) {
    if ctx.function_scopes.len() < 2 {
        super::flow_diagnostics::nonlocal_requires_enclosing_binding(ctx, nonlocal.range());
        return;
    }

    for name in &nonlocal.names {
        let name_text = name.to_string();
        if ctx.lookup_current_function_binding(&name_text).is_some() {
            super::flow_diagnostics::nonlocal_conflicts_with_current_binding(
                ctx,
                &name_text,
                name.range(),
            );
            continue;
        }
        if ctx.lookup_outer_function_binding(&name_text).is_none() {
            super::flow_diagnostics::nonlocal_missing_enclosing_binding(
                ctx,
                &name_text,
                name.range(),
            );
        }
    }
}

pub(in crate::lower) fn should_rebind_simple_name(ctx: &LowerCtx, name: &str) -> bool {
    ctx.lookup_current_function_binding(name).is_some()
        || (ctx.is_declared_nonlocal(name) && ctx.lookup_outer_function_binding(name).is_some())
}

pub(in crate::lower) fn hir_body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    stmts
        .iter()
        .any(|stmt| hir_stmt_calls_function(stmt, func_name))
}

fn hir_stmt_calls_function(stmt: &HirStmt, func_name: &str) -> bool {
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value }
        | HirStmt::TupleUnpack { value, .. }
        | HirStmt::StarUnpack { value, .. } => hir_expr_calls_function(value, func_name),
        HirStmt::Return { value } => value
            .as_ref()
            .is_some_and(|value| hir_expr_calls_function(value, func_name)),
        HirStmt::Expr { expr } => hir_expr_calls_function(expr, func_name),
        HirStmt::Assert { test, msg } => {
            hir_expr_calls_function(test, func_name)
                || msg
                    .as_ref()
                    .is_some_and(|msg| hir_expr_calls_function(msg, func_name))
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            hir_expr_calls_function(condition, func_name)
                || hir_body_calls_function(then_body, func_name)
                || elif_clauses.iter().any(|(cond, body)| {
                    hir_expr_calls_function(cond, func_name)
                        || hir_body_calls_function(body, func_name)
                })
                || else_body
                    .as_ref()
                    .is_some_and(|body| hir_body_calls_function(body, func_name))
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            hir_expr_calls_function(condition, func_name)
                || hir_body_calls_function(body, func_name)
                || else_body
                    .as_ref()
                    .is_some_and(|body| hir_body_calls_function(body, func_name))
        }
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        }
        | HirStmt::AsyncFor {
            iter,
            body,
            else_body,
            ..
        } => {
            hir_expr_calls_function(iter, func_name)
                || hir_body_calls_function(body, func_name)
                || else_body
                    .as_ref()
                    .is_some_and(|body| hir_body_calls_function(body, func_name))
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            hir_expr_calls_function(index, func_name) || hir_expr_calls_function(value, func_name)
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        }
        | HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            hir_expr_calls_function(outer_index, func_name)
                || hir_expr_calls_function(inner_index, func_name)
                || hir_expr_calls_function(value, func_name)
        }
        HirStmt::Delete { object, index } => {
            hir_expr_calls_function(object, func_name) || hir_expr_calls_function(index, func_name)
        }
        HirStmt::With { items, body } => {
            items
                .iter()
                .any(|item| hir_expr_calls_function(&item.context, func_name))
                || hir_body_calls_function(body, func_name)
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            let context_calls = match kind {
                crate::hir_nodes::HirAsyncWithKind::TaskTimeout { duration } => {
                    hir_expr_calls_function(duration, func_name)
                }
                crate::hir_nodes::HirAsyncWithKind::UserDefined { context, .. }
                | crate::hir_nodes::HirAsyncWithKind::Python { context, .. } => {
                    hir_expr_calls_function(context, func_name)
                }
                crate::hir_nodes::HirAsyncWithKind::TaskGroup {
                    context: Some(context),
                } => hir_expr_calls_function(context, func_name),
                crate::hir_nodes::HirAsyncWithKind::TaskScope
                | crate::hir_nodes::HirAsyncWithKind::TaskGroup { context: None } => false,
            };
            context_calls || hir_body_calls_function(body, func_name)
        }
        HirStmt::Match { subject, arms, .. } => {
            hir_expr_calls_function(subject, func_name)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(|guard| hir_expr_calls_function(guard, func_name))
                        || hir_body_calls_function(&arm.body, func_name)
                })
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            hir_body_calls_function(body, func_name)
                || handlers
                    .iter()
                    .any(|handler| hir_body_calls_function(&handler.body, func_name))
        }
        HirStmt::TryFinally { body, finalbody } => {
            hir_body_calls_function(body, func_name)
                || hir_body_calls_function(finalbody, func_name)
        }
        HirStmt::Break | HirStmt::Continue | HirStmt::Pass | HirStmt::NestedFunction { .. } => {
            false
        }
    }
}

fn hir_expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    match expr {
        HirExpr::Call { func, args, .. }
        | HirExpr::GenericCall { func, args, .. }
        | HirExpr::PythonCall { func, args, .. } => {
            func == func_name || args.iter().any(|arg| hir_expr_calls_function(arg, func_name))
        }
        HirExpr::IteratorCall { args, .. } => args
            .iter()
            .any(|arg| hir_expr_calls_function(arg, func_name)),
        HirExpr::IntrinsicCall { args, .. } => args
            .iter()
            .any(|arg| hir_expr_calls_function(arg, func_name)),
        HirExpr::MethodCall { object, args, .. } => {
            hir_expr_calls_function(object, func_name)
                || args.iter().any(|arg| hir_expr_calls_function(arg, func_name))
        }
        HirExpr::BinOp { left, right, .. } => {
            hir_expr_calls_function(left, func_name) || hir_expr_calls_function(right, func_name)
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|value| hir_expr_calls_function(value, func_name)),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            hir_expr_calls_function(left, func_name)
                || comparators
                    .iter()
                    .any(|expr| hir_expr_calls_function(expr, func_name))
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. } => hir_expr_calls_function(operand, func_name),
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. } => elements
            .iter()
            .any(|expr| hir_expr_calls_function(expr, func_name)),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            hir_expr_calls_function(start, func_name)
                || hir_expr_calls_function(end, func_name)
                || step
                    .as_ref()
                    .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            keys.iter().any(|expr| hir_expr_calls_function(expr, func_name))
                || values
                    .iter()
                    .any(|expr| hir_expr_calls_function(expr, func_name))
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            hir_expr_calls_function(element, func_name)
                || hir_expr_calls_function(collection, func_name)
        }
        HirExpr::Index { object, index, .. } => {
            hir_expr_calls_function(object, func_name) || hir_expr_calls_function(index, func_name)
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            hir_expr_calls_function(object, func_name)
                || start
                    .as_ref()
                    .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
                || stop
                    .as_ref()
                    .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
                || step
                    .as_ref()
                    .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
        }
        HirExpr::FieldAccess { object, .. } => hir_expr_calls_function(object, func_name),
        HirExpr::ConstructorCall { args, .. } | HirExpr::SuperCall { args, .. } => args
            .iter()
            .any(|expr| hir_expr_calls_function(expr, func_name)),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            hir_expr_calls_function(condition, func_name)
                || hir_expr_calls_function(then_expr, func_name)
                || hir_expr_calls_function(else_expr, func_name)
        }
        HirExpr::Lambda { body, .. } => hir_expr_calls_function(body, func_name),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            hir_expr_calls_function(expr, func_name)
                || generators.iter().any(|(_, iter, filter)| {
                    hir_expr_calls_function(iter, func_name)
                        || filter
                            .as_ref()
                            .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            hir_expr_calls_function(key_expr, func_name)
                || hir_expr_calls_function(val_expr, func_name)
                || generators.iter().any(|(_, iter, filter)| {
                    hir_expr_calls_function(iter, func_name)
                        || filter
                            .as_ref()
                            .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
                })
        }
        HirExpr::GeneratorExpr {
            expr,
            iter,
            filter,
            ..
        } => {
            hir_expr_calls_function(expr, func_name)
                || hir_expr_calls_function(iter, func_name)
                || filter
                    .as_ref()
                    .is_some_and(|expr| hir_expr_calls_function(expr, func_name))
        }
        HirExpr::FString { parts, .. } => parts.iter().any(|part| {
            matches!(part, crate::hir_nodes::HirFStringPart::Expr(expr) if hir_expr_calls_function(expr, func_name))
        }),
        HirExpr::TemplateString(template) => {
            let mut found = false;
            template.for_each_value(&mut |value| {
                found |= hir_expr_calls_function(value, func_name);
            });
            found
        }
        HirExpr::EnumVariant { .. }
        | HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral => false,
    }
}
