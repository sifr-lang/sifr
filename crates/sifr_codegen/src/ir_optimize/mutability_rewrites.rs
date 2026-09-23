use crate::{RustExpr, RustItem, RustStmt};
use std::collections::HashSet;

use super::compiler_generated_mutating_methods::COMPILER_GENERATED_MUTATING_METHODS;

pub(crate) fn remove_unneeded_mutability_in_items(
    items: &mut [RustItem],
    protected_names: &HashSet<String>,
) -> usize {
    let mut removed = 0usize;
    for item in items {
        removed += remove_unneeded_mutability_in_item(item, protected_names);
    }
    removed
}

pub(super) fn remove_unneeded_mutability_in_item(
    item: &mut RustItem,
    protected_names: &HashSet<String>,
) -> usize {
    match item {
        RustItem::Impl { items, .. } => items
            .iter_mut()
            .map(|item| remove_unneeded_mutability_in_item(item, protected_names))
            .sum(),
        RustItem::Trait { methods, .. } => methods
            .iter_mut()
            .map(|item| remove_unneeded_mutability_in_item(item, protected_names))
            .sum(),
        RustItem::Fn { body, .. } => remove_unneeded_mutability_in_block(body, protected_names),
        _ => 0,
    }
}

pub(super) fn remove_unneeded_mutability_in_block(
    body: &mut [RustStmt],
    protected_names: &HashSet<String>,
) -> usize {
    remove_unneeded_mutability_in_block_with_tail_expr(body, None, protected_names)
}

pub(super) fn remove_unneeded_mutability_in_block_with_tail_expr(
    body: &mut [RustStmt],
    tail_expr: Option<&RustExpr>,
    protected_names: &HashSet<String>,
) -> usize {
    let mut removed = 0usize;
    for stmt in body.iter_mut() {
        removed += remove_nested_unneeded_mutability(stmt, protected_names);
    }

    for idx in 0..body.len() {
        let (head, tail) = body.split_at_mut(idx + 1);
        let stmt = &mut head[idx];
        match stmt {
            RustStmt::Let {
                mutable: true,
                name,
                value,
                ..
            } if !is_callable_binding_value(value)
                && !protected_names.contains(name)
                && !stmts_mutate_name(tail, name)
                && !tail_expr.is_some_and(|expr| expr_mutates_name(expr, name)) =>
            {
                removed += 1;
                if let RustStmt::Let { mutable, .. } = stmt {
                    *mutable = false;
                }
            }
            RustStmt::IfLet {
                pattern, then_body, ..
            } => {
                if let Some(name) = option_mut_pattern_name(pattern) {
                    if !stmts_mutate_name(then_body, &name) {
                        *pattern = format!("Some({name})");
                        removed += 1;
                    }
                }
            }
            _ => {}
        }
    }
    removed
}

pub(super) fn remove_nested_unneeded_mutability(
    stmt: &mut RustStmt,
    protected_names: &HashSet<String>,
) -> usize {
    match stmt {
        RustStmt::Verbatim(_) | RustStmt::LetDecl { .. } => 0,
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
            remove_expr_unneeded_mutability(value, protected_names)
        }
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            remove_expr_unneeded_mutability(value, protected_names)
                + remove_unneeded_mutability_in_block(else_body, protected_names)
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            remove_expr_unneeded_mutability(target, protected_names)
                + remove_expr_unneeded_mutability(value, protected_names)
        }
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            remove_expr_unneeded_mutability(expr, protected_names)
        }
        RustStmt::Assert { cond, msg } => {
            remove_expr_unneeded_mutability(cond, protected_names)
                + msg
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .unwrap_or(0)
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            remove_expr_unneeded_mutability(cond, protected_names)
                + remove_unneeded_mutability_in_block(then_body, protected_names)
                + else_body
                    .as_mut()
                    .map(|body| remove_unneeded_mutability_in_block(body, protected_names))
                    .unwrap_or(0)
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            remove_expr_unneeded_mutability(expr, protected_names)
                + remove_unneeded_mutability_in_block(then_body, protected_names)
                + else_body
                    .as_mut()
                    .map(|body| remove_unneeded_mutability_in_block(body, protected_names))
                    .unwrap_or(0)
        }
        RustStmt::Match { expr, arms } => {
            let mut removed = remove_expr_unneeded_mutability(expr, protected_names);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += remove_expr_unneeded_mutability(guard, protected_names);
                }
                removed += remove_unneeded_mutability_in_block(&mut arm.body, protected_names);
            }
            removed
        }
        RustStmt::For { iter, body, .. } => {
            remove_expr_unneeded_mutability(iter, protected_names)
                + remove_unneeded_mutability_in_block(body, protected_names)
        }
        RustStmt::With { items, body } => {
            let mut removed = 0usize;
            for item in items {
                removed += remove_expr_unneeded_mutability(&mut item.value, protected_names);
            }
            removed + remove_unneeded_mutability_in_block(body, protected_names)
        }
        RustStmt::While { cond, body } => {
            remove_expr_unneeded_mutability(cond, protected_names)
                + remove_unneeded_mutability_in_block(body, protected_names)
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            remove_unneeded_mutability_in_block(body, protected_names)
        }
    }
}

pub(super) fn remove_expr_unneeded_mutability(
    expr: &mut RustExpr,
    protected_names: &HashSet<String>,
) -> usize {
    match expr {
        RustExpr::Block { stmts, expr } => {
            let removed = remove_unneeded_mutability_in_block_with_tail_expr(
                stmts,
                expr.as_deref(),
                protected_names,
            );
            removed
                + expr
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .unwrap_or(0)
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            remove_expr_unneeded_mutability(cond, protected_names)
                + remove_expr_unneeded_mutability(then_expr, protected_names)
                + else_expr
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .unwrap_or(0)
        }
        RustExpr::Match { expr, arms } => {
            let mut removed = remove_expr_unneeded_mutability(expr, protected_names);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += remove_expr_unneeded_mutability(guard, protected_names);
                }
                removed += remove_unneeded_mutability_in_block(&mut arm.body, protected_names);
            }
            removed
        }
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            remove_unneeded_mutability_in_block(body, protected_names)
        }
        RustExpr::MethodCall { receiver, args, .. }
        | RustExpr::SourceMethodCall { receiver, args, .. } => {
            remove_expr_unneeded_mutability(receiver, protected_names)
                + args
                    .iter_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .sum::<usize>()
        }
        RustExpr::FnCall { func, args } => {
            remove_expr_unneeded_mutability(func, protected_names)
                + args
                    .iter_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .sum::<usize>()
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::FormatMacro { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => args
            .iter_mut()
            .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
            .sum(),
        RustExpr::BinOp { left, right, .. } => {
            remove_expr_unneeded_mutability(left, protected_names)
                + remove_expr_unneeded_mutability(right, protected_names)
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => remove_expr_unneeded_mutability(operand, protected_names),
        RustExpr::Field { expr, .. } => remove_expr_unneeded_mutability(expr, protected_names),
        RustExpr::Index { expr, index } => {
            remove_expr_unneeded_mutability(expr, protected_names)
                + remove_expr_unneeded_mutability(index, protected_names)
        }
        RustExpr::Slice { expr, start, stop } => {
            remove_expr_unneeded_mutability(expr, protected_names)
                + start
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .unwrap_or(0)
                + stop
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr, protected_names))
                    .unwrap_or(0)
        }
        RustExpr::Ref { expr, .. } => remove_expr_unneeded_mutability(expr, protected_names),
        RustExpr::StructInit { fields, .. } => fields
            .iter_mut()
            .map(|(_, value)| remove_expr_unneeded_mutability(value, protected_names))
            .sum(),
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            remove_expr_unneeded_mutability(duration, protected_names)
                + remove_expr_unneeded_mutability(future, protected_names)
                + remove_expr_unneeded_mutability(error, protected_names)
        }
        RustExpr::Range { start, end } => {
            remove_expr_unneeded_mutability(start, protected_names)
                + remove_expr_unneeded_mutability(end, protected_names)
        }
        RustExpr::Closure { body, .. } => remove_expr_unneeded_mutability(body, protected_names),
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::Verbatim(_) => 0,
    }
}

pub(super) fn option_mut_pattern_name(pattern: &str) -> Option<String> {
    let name = pattern.strip_prefix("Some(mut ")?.strip_suffix(')')?;
    if name
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        Some(name.to_string())
    } else {
        None
    }
}

pub(super) fn is_callable_binding_value(value: &RustExpr) -> bool {
    matches!(
        value,
        RustExpr::Closure { .. } | RustExpr::ClosureBlock { .. }
    )
}

pub(super) fn stmts_mutate_name(stmts: &[RustStmt], name: &str) -> bool {
    stmts.iter().any(|stmt| stmt_mutates_name(stmt, name))
}

pub(super) fn stmt_mutates_name(stmt: &RustStmt, name: &str) -> bool {
    match stmt {
        // Verbatim is an explicit IR boundary. If it names the local, retain
        // mutability conservatively because structured mutation analysis
        // cannot inspect the operation.
        RustStmt::Verbatim(source) => source
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .any(|token| token == name),
        RustStmt::Assign { target, value } => {
            assignment_target_mutates_name(target, name) || expr_mutates_name(value, name)
        }
        RustStmt::AugAssign { target, value, .. } => {
            assignment_target_mutates_name(target, name) || expr_mutates_name(value, name)
        }
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
            expr_mutates_name(value, name)
        }
        RustStmt::LetElse {
            value, else_body, ..
        } => expr_mutates_name(value, name) || stmts_mutate_name(else_body, name),
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            expr_mutates_name(expr, name)
        }
        RustStmt::Assert { cond, msg } => {
            expr_mutates_name(cond, name)
                || msg.as_ref().is_some_and(|msg| expr_mutates_name(msg, name))
        }
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_mutates_name(cond, name)
                || stmts_mutate_name(then_body, name)
                || else_body
                    .as_ref()
                    .is_some_and(|body| stmts_mutate_name(body, name))
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            expr_mutates_name(expr, name)
                || stmts_mutate_name(then_body, name)
                || else_body
                    .as_ref()
                    .is_some_and(|body| stmts_mutate_name(body, name))
        }
        RustStmt::Match { expr, arms } => {
            expr_mutates_name(expr, name)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(|guard| expr_mutates_name(guard, name))
                        || stmts_mutate_name(&arm.body, name)
                })
        }
        RustStmt::For { iter, body, .. } => {
            expr_mutates_name(iter, name) || stmts_mutate_name(body, name)
        }
        RustStmt::With { items, body } => {
            items
                .iter()
                .any(|item| expr_mutates_name(&item.value, name))
                || stmts_mutate_name(body, name)
        }
        RustStmt::While { cond, body } => {
            expr_mutates_name(cond, name) || stmts_mutate_name(body, name)
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            stmts_mutate_name(body, name)
        }
        RustStmt::LetDecl { .. }
        | RustStmt::Return(None)
        | RustStmt::Break
        | RustStmt::Continue => false,
    }
}

pub(super) fn assignment_target_mutates_name(target: &RustExpr, name: &str) -> bool {
    match target {
        RustExpr::Ident(target_name) => target_name == name,
        RustExpr::Field { expr, .. } | RustExpr::Index { expr, .. } => {
            root_expr_is_name(expr, name)
        }
        _ => expr_mutates_name(target, name),
    }
}

pub(super) fn root_expr_is_name(expr: &RustExpr, name: &str) -> bool {
    match expr {
        RustExpr::Ident(target_name) => target_name == name,
        RustExpr::Path(parts) => parts.len() == 1 && parts[0] == name,
        RustExpr::Paren(expr) => root_expr_is_name(expr, name),
        RustExpr::Field { expr, .. } | RustExpr::Index { expr, .. } => {
            root_expr_is_name(expr, name)
        }
        _ => false,
    }
}

pub(super) fn expr_mutates_name(expr: &RustExpr, name: &str) -> bool {
    match expr {
        RustExpr::Ref {
            mutable: true,
            expr,
        } => root_expr_is_name(expr, name),
        RustExpr::Ref {
            mutable: false,
            expr,
        } => expr_mutates_name(expr, name),
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        }
        | RustExpr::SourceMethodCall {
            receiver,
            method,
            args,
        } => {
            (root_expr_is_name(receiver, name)
                && COMPILER_GENERATED_MUTATING_METHODS.contains(&method.as_str()))
                || expr_mutates_name(receiver, name)
                || args.iter().any(|arg| expr_mutates_name(arg, name))
        }
        RustExpr::FnCall { func, args } => {
            root_expr_is_name(func, name)
                || expr_mutates_name(func, name)
                || args.iter().any(|arg| expr_mutates_name(arg, name))
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::FormatMacro { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => args.iter().any(|arg| expr_mutates_name(arg, name)),
        RustExpr::BinOp { left, right, .. } => {
            expr_mutates_name(left, name) || expr_mutates_name(right, name)
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => expr_mutates_name(operand, name),
        RustExpr::Field { expr, .. } => expr_mutates_name(expr, name),
        RustExpr::Index { expr, index } => {
            expr_mutates_name(expr, name) || expr_mutates_name(index, name)
        }
        RustExpr::Slice { expr, start, stop } => {
            expr_mutates_name(expr, name)
                || start
                    .as_ref()
                    .is_some_and(|expr| expr_mutates_name(expr, name))
                || stop
                    .as_ref()
                    .is_some_and(|expr| expr_mutates_name(expr, name))
        }
        RustExpr::Block { stmts, expr } => {
            stmts_mutate_name(stmts, name)
                || expr
                    .as_ref()
                    .is_some_and(|expr| expr_mutates_name(expr, name))
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            expr_mutates_name(cond, name)
                || expr_mutates_name(then_expr, name)
                || else_expr
                    .as_ref()
                    .is_some_and(|expr| expr_mutates_name(expr, name))
        }
        RustExpr::Match { expr, arms } => {
            expr_mutates_name(expr, name)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(|guard| expr_mutates_name(guard, name))
                        || stmts_mutate_name(&arm.body, name)
                })
        }
        RustExpr::Closure { body, .. } => expr_mutates_name(body, name),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            stmts_mutate_name(body, name)
        }
        RustExpr::StructInit { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_mutates_name(value, name)),
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            expr_mutates_name(duration, name)
                || expr_mutates_name(future, name)
                || expr_mutates_name(error, name)
        }
        RustExpr::Range { start, end } => {
            expr_mutates_name(start, name) || expr_mutates_name(end, name)
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::Verbatim(_) => {
            false
        }
    }
}
