use crate::{RustExpr, RustItem, RustLiteral, RustStmt, RustType};

const MUTATING_METHODS: &[&str] = &[
    "append",
    "aclose",
    "anext",
    "by_index",
    "by_name",
    "clear",
    "entry",
    "extend",
    "flush",
    "get_mut",
    "insert",
    "kill",
    "increment",
    "merge_from",
    "next",
    "pop",
    "push",
    "push_str",
    "read_string",
    "remove",
    "reverse",
    "rotate",
    "seek",
    "set",
    "set_bool",
    "set_level",
    "set_list",
    "setstate",
    "sort",
    "sort_by",
    "take",
    "try_wait",
    "write",
    "write_all",
    "writerow",
    "writerows",
    "writeln",
    "__aenter__",
    "__aexit__",
    "__next__",
    "__sifr_join_all",
    "__sifr_spawn_infallible",
    "__sifr_spawn_result",
];

/// Remove conservatively-trivial `.clone()` expressions from IR items.
///
/// This pass is intentionally narrow: it only removes clones on expressions
/// that are always safe to move without changing semantics.
pub(crate) fn remove_trivial_clones_in_items(items: &mut [RustItem]) -> usize {
    let mut removed = 0usize;
    for item in items {
        removed += optimize_item(item);
    }
    removed
}

pub(crate) fn remove_unneeded_mutability_in_items(items: &mut [RustItem]) -> usize {
    let mut removed = 0usize;
    for item in items {
        removed += remove_unneeded_mutability_in_item(item);
    }
    removed
}

fn remove_unneeded_mutability_in_item(item: &mut RustItem) -> usize {
    match item {
        RustItem::Impl { items, .. } => items
            .iter_mut()
            .map(remove_unneeded_mutability_in_item)
            .sum(),
        RustItem::Trait { methods, .. } => methods
            .iter_mut()
            .map(remove_unneeded_mutability_in_item)
            .sum(),
        RustItem::Fn { body, .. } => remove_unneeded_mutability_in_block(body),
        _ => 0,
    }
}

fn remove_unneeded_mutability_in_block(body: &mut [RustStmt]) -> usize {
    remove_unneeded_mutability_in_block_with_tail_expr(body, None)
}

fn remove_unneeded_mutability_in_block_with_tail_expr(
    body: &mut [RustStmt],
    tail_expr: Option<&RustExpr>,
) -> usize {
    let mut removed = 0usize;
    for stmt in body.iter_mut() {
        removed += remove_nested_unneeded_mutability(stmt);
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

fn remove_nested_unneeded_mutability(stmt: &mut RustStmt) -> usize {
    match stmt {
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
            remove_expr_unneeded_mutability(value)
        }
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            remove_expr_unneeded_mutability(value) + remove_unneeded_mutability_in_block(else_body)
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            remove_expr_unneeded_mutability(target) + remove_expr_unneeded_mutability(value)
        }
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => {
            remove_expr_unneeded_mutability(expr)
        }
        RustStmt::Assert { cond, msg } => {
            remove_expr_unneeded_mutability(cond)
                + msg
                    .as_mut()
                    .map(remove_expr_unneeded_mutability)
                    .unwrap_or(0)
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            remove_expr_unneeded_mutability(cond)
                + remove_unneeded_mutability_in_block(then_body)
                + else_body
                    .as_mut()
                    .map(|body| remove_unneeded_mutability_in_block(body))
                    .unwrap_or(0)
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            remove_expr_unneeded_mutability(expr)
                + remove_unneeded_mutability_in_block(then_body)
                + else_body
                    .as_mut()
                    .map(|body| remove_unneeded_mutability_in_block(body))
                    .unwrap_or(0)
        }
        RustStmt::Match { expr, arms } => {
            let mut removed = remove_expr_unneeded_mutability(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += remove_expr_unneeded_mutability(guard);
                }
                removed += remove_unneeded_mutability_in_block(&mut arm.body);
            }
            removed
        }
        RustStmt::For { iter, body, .. } => {
            remove_expr_unneeded_mutability(iter) + remove_unneeded_mutability_in_block(body)
        }
        RustStmt::With { items, body } => {
            let mut removed = 0usize;
            for item in items {
                removed += remove_expr_unneeded_mutability(&mut item.value);
            }
            removed + remove_unneeded_mutability_in_block(body)
        }
        RustStmt::While { cond, body } => {
            remove_expr_unneeded_mutability(cond) + remove_unneeded_mutability_in_block(body)
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            remove_unneeded_mutability_in_block(body)
        }
    }
}

fn remove_expr_unneeded_mutability(expr: &mut RustExpr) -> usize {
    match expr {
        RustExpr::Block { stmts, expr } => {
            let removed =
                remove_unneeded_mutability_in_block_with_tail_expr(stmts, expr.as_deref());
            removed
                + expr
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr))
                    .unwrap_or(0)
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            remove_expr_unneeded_mutability(cond)
                + remove_expr_unneeded_mutability(then_expr)
                + else_expr
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr))
                    .unwrap_or(0)
        }
        RustExpr::Match { expr, arms } => {
            let mut removed = remove_expr_unneeded_mutability(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += remove_expr_unneeded_mutability(guard);
                }
                removed += remove_unneeded_mutability_in_block(&mut arm.body);
            }
            removed
        }
        RustExpr::ClosureBlock { body, .. } => remove_unneeded_mutability_in_block(body),
        RustExpr::MethodCall { receiver, args, .. } => {
            remove_expr_unneeded_mutability(receiver)
                + args
                    .iter_mut()
                    .map(remove_expr_unneeded_mutability)
                    .sum::<usize>()
        }
        RustExpr::FnCall { func, args } => {
            remove_expr_unneeded_mutability(func)
                + args
                    .iter_mut()
                    .map(remove_expr_unneeded_mutability)
                    .sum::<usize>()
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::FormatMacro { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => args.iter_mut().map(remove_expr_unneeded_mutability).sum(),
        RustExpr::BinOp { left, right, .. } => {
            remove_expr_unneeded_mutability(left) + remove_expr_unneeded_mutability(right)
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => remove_expr_unneeded_mutability(operand),
        RustExpr::Field { expr, .. } => remove_expr_unneeded_mutability(expr),
        RustExpr::Index { expr, index } => {
            remove_expr_unneeded_mutability(expr) + remove_expr_unneeded_mutability(index)
        }
        RustExpr::Slice { expr, start, stop } => {
            remove_expr_unneeded_mutability(expr)
                + start
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr))
                    .unwrap_or(0)
                + stop
                    .as_mut()
                    .map(|expr| remove_expr_unneeded_mutability(expr))
                    .unwrap_or(0)
        }
        RustExpr::Ref { expr, .. } => remove_expr_unneeded_mutability(expr),
        RustExpr::StructInit { fields, .. } => fields
            .iter_mut()
            .map(|(_, value)| remove_expr_unneeded_mutability(value))
            .sum(),
        RustExpr::TimeoutAwait { duration, future } => {
            remove_expr_unneeded_mutability(duration) + remove_expr_unneeded_mutability(future)
        }
        RustExpr::Range { start, end } => {
            remove_expr_unneeded_mutability(start) + remove_expr_unneeded_mutability(end)
        }
        RustExpr::Closure { body, .. } => remove_expr_unneeded_mutability(body),
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => 0,
    }
}

fn option_mut_pattern_name(pattern: &str) -> Option<String> {
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

fn is_callable_binding_value(value: &RustExpr) -> bool {
    matches!(
        value,
        RustExpr::Closure { .. } | RustExpr::ClosureBlock { .. }
    )
}

fn stmts_mutate_name(stmts: &[RustStmt], name: &str) -> bool {
    stmts.iter().any(|stmt| stmt_mutates_name(stmt, name))
}

fn stmt_mutates_name(stmt: &RustStmt, name: &str) -> bool {
    match stmt {
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
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => expr_mutates_name(expr, name),
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
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => false,
    }
}

fn assignment_target_mutates_name(target: &RustExpr, name: &str) -> bool {
    match target {
        RustExpr::Ident(target_name) => target_name == name,
        RustExpr::Field { expr, .. } | RustExpr::Index { expr, .. } => {
            root_expr_is_name(expr, name)
        }
        _ => expr_mutates_name(target, name),
    }
}

fn root_expr_is_name(expr: &RustExpr, name: &str) -> bool {
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

fn expr_mutates_name(expr: &RustExpr, name: &str) -> bool {
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
        } => {
            (root_expr_is_name(receiver, name) && MUTATING_METHODS.contains(&method.as_str()))
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
        RustExpr::ClosureBlock { body, .. } => stmts_mutate_name(body, name),
        RustExpr::StructInit { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_mutates_name(value, name)),
        RustExpr::TimeoutAwait { duration, future } => {
            expr_mutates_name(duration, name) || expr_mutates_name(future, name)
        }
        RustExpr::Range { start, end } => {
            expr_mutates_name(start, name) || expr_mutates_name(end, name)
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => false,
    }
}

fn optimize_item(item: &mut RustItem) -> usize {
    match item {
        RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => 0,
        RustItem::Struct { .. } | RustItem::TupleStruct { .. } => 0,
        RustItem::Enum { variants, .. } => {
            let mut removed = 0usize;
            for variant in variants {
                if let Some(value) = &mut variant.value {
                    removed += optimize_expr(value);
                }
            }
            removed
        }
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            let mut removed = 0usize;
            for method in methods {
                removed += optimize_item(method);
            }
            removed
        }
        RustItem::Fn { body, .. } => optimize_block(body),
        RustItem::TraitMethodSig { .. } => 0,
        RustItem::TypeAlias { .. } => 0,
        RustItem::Const { value, .. } | RustItem::Static { value, .. } => optimize_expr(value),
    }
}

fn optimize_block(body: &mut Vec<RustStmt>) -> usize {
    let mut removed = 0usize;
    for stmt in body.iter_mut() {
        removed += optimize_stmt(stmt);
    }
    let before = body.len();
    body.retain(|stmt| !is_self_assignment(stmt));
    removed + (before - body.len())
}

fn is_self_assignment(stmt: &RustStmt) -> bool {
    matches!(
        stmt,
        RustStmt::Assign {
            target: RustExpr::Ident(target),
            value: RustExpr::Ident(value),
        } if target == value
    )
}

fn optimize_stmt(stmt: &mut RustStmt) -> usize {
    match stmt {
        RustStmt::Let { value, .. } => optimize_expr(value),
        RustStmt::LetPattern { value, .. } => optimize_expr(value),
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            let mut removed = optimize_expr(value);
            removed += optimize_block(else_body);
            removed
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            optimize_expr(target) + optimize_expr(value)
        }
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => optimize_expr(expr),
        RustStmt::Assert { cond, msg } => {
            optimize_expr(cond) + msg.as_mut().map(optimize_expr).unwrap_or(0)
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let mut removed = optimize_expr(cond);
            removed += optimize_block(then_body);
            if let Some(else_body) = else_body {
                removed += optimize_block(else_body);
            }
            if else_body.as_ref().is_some_and(Vec::is_empty) {
                *else_body = None;
                removed += 1;
            }
            removed
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            let mut removed = optimize_expr(expr);
            removed += optimize_block(then_body);
            if let Some(else_body) = else_body {
                removed += optimize_block(else_body);
            }
            if else_body.as_ref().is_some_and(Vec::is_empty) {
                *else_body = None;
                removed += 1;
            }
            removed
        }
        RustStmt::Match { expr, arms } => {
            let mut removed = optimize_expr(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += optimize_expr(guard);
                }
                removed += optimize_block(&mut arm.body);
            }
            removed
        }
        RustStmt::For { iter, body, .. } => {
            let mut removed = optimize_expr(iter);
            removed += optimize_block(body);
            removed
        }
        RustStmt::With { items, body } => {
            let mut removed = 0usize;
            for item in items {
                removed += optimize_expr(&mut item.value);
            }
            removed += optimize_block(body);
            removed
        }
        RustStmt::While { cond, body } => {
            let mut removed = optimize_expr(cond);
            removed += optimize_block(body);
            removed
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            optimize_block(body)
        }
    }
}

fn optimize_expr(expr: &mut RustExpr) -> usize {
    match expr {
        RustExpr::Clone(inner) => {
            let mut removed = optimize_expr(inner);
            if should_remove_clone(inner.as_ref()) {
                let replacement =
                    *std::mem::replace(inner, Box::new(RustExpr::Literal(RustLiteral::Unit)));
                *expr = replacement;
                removed += 1;
            }
            removed
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => 0,
        RustExpr::MethodCall { receiver, args, .. } => {
            let mut removed = optimize_expr(receiver);
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::FnCall { func, args } => {
            let mut removed = optimize_expr(func);
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => {
            let mut removed = 0usize;
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::TimeoutAwait { duration, future } => {
            optimize_expr(duration) + optimize_expr(future)
        }
        RustExpr::FormatMacro { args, .. } => {
            let mut removed = 0usize;
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::BinOp { left, right, .. } => optimize_expr(left) + optimize_expr(right),
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Try(operand)
        | RustExpr::Paren(operand)
        | RustExpr::Await(operand) => optimize_expr(operand),
        RustExpr::Field { expr, .. } => optimize_expr(expr),
        RustExpr::Index { expr, index } => optimize_expr(expr) + optimize_expr(index),
        RustExpr::Slice { expr, start, stop } => {
            optimize_expr(expr)
                + start.as_mut().map(|s| optimize_expr(s)).unwrap_or(0)
                + stop.as_mut().map(|s| optimize_expr(s)).unwrap_or(0)
        }
        RustExpr::Ref { expr, .. } => optimize_expr(expr),
        RustExpr::Cast { expr, .. } => optimize_expr(expr),
        RustExpr::Block { stmts, expr } => {
            let mut removed = optimize_block(stmts);
            if let Some(expr) = expr {
                removed += optimize_expr(expr);
            }
            removed
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            let mut removed = optimize_expr(cond) + optimize_expr(then_expr);
            if let Some(else_expr) = else_expr {
                removed += optimize_expr(else_expr);
            }
            removed
        }
        RustExpr::Match { expr, arms } => {
            let mut removed = optimize_expr(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += optimize_expr(guard);
                }
                removed += optimize_block(&mut arm.body);
            }
            removed
        }
        RustExpr::Closure { body, .. } => optimize_expr(body),
        RustExpr::ClosureBlock { body, .. } => optimize_block(body),
        RustExpr::StructInit { fields, .. } => {
            let mut removed = 0usize;
            for (_, value) in fields {
                removed += optimize_expr(value);
            }
            removed
        }
        RustExpr::Range { start, end } => optimize_expr(start) + optimize_expr(end),
    }
}

fn should_remove_clone(inner: &RustExpr) -> bool {
    match inner {
        RustExpr::Literal(_) => true,
        RustExpr::Ref { .. } => true,
        RustExpr::Cast { ty, .. } => is_copy_type(ty),
        _ => false,
    }
}

fn is_copy_type(ty: &RustType) -> bool {
    match ty {
        RustType::I64 | RustType::F64 | RustType::Bool | RustType::Unit => true,
        RustType::Ref { .. } => true,
        RustType::Tuple(items) => items.iter().all(is_copy_type),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustParam, Visibility};

    #[test]
    fn removes_clone_on_literals_and_copy_casts() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "n".to_string(),
                ty: RustType::I64,
            }],
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "a".to_string(),
                    ty: None,
                    value: RustExpr::Clone(Box::new(RustExpr::Literal(RustLiteral::Int(1)))),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "b".to_string(),
                    ty: None,
                    value: RustExpr::Clone(Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("n".to_string())),
                        ty: RustType::I64,
                    })),
                },
            ],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 2);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let RustStmt::Let { value: first, .. } = &body[0] else {
            panic!("expected let statement");
        };
        assert!(matches!(first, RustExpr::Literal(RustLiteral::Int(1))));

        let RustStmt::Let { value: second, .. } = &body[1] else {
            panic!("expected let statement");
        };
        assert!(matches!(second, RustExpr::Cast { .. }));
    }

    #[test]
    fn keeps_clone_on_non_trivial_identifier() {
        let mut items = vec![RustItem::Const {
            name: "X".to_string(),
            visibility: Visibility::Private,
            ty: RustType::String_,
            value: RustExpr::Clone(Box::new(RustExpr::Ident("value".to_string()))),
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 0);
        let RustItem::Const { value, .. } = &items[0] else {
            panic!("expected const item");
        };
        assert!(matches!(value, RustExpr::Clone(_)));
    }

    #[test]
    fn optimizes_nested_clone_sites() {
        let mut items = vec![RustItem::Fn {
            name: "nested".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("consume".to_string())),
                args: vec![RustExpr::Clone(Box::new(RustExpr::Literal(
                    RustLiteral::Str("x".to_string()),
                )))],
            })],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 1);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let RustStmt::Expr(RustExpr::FnCall { args, .. }) = &body[0] else {
            panic!("expected fn call expression");
        };
        assert!(matches!(
            args.first(),
            Some(RustExpr::Literal(RustLiteral::Str(s))) if s == "x"
        ));
    }

    #[test]
    fn preserves_mutable_callable_bindings() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "apply".to_string(),
                    ty: None,
                    value: RustExpr::ClosureBlock {
                        params: vec![],
                        body: vec![],
                        is_move: false,
                        is_async: false,
                    },
                },
                RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident("apply".to_string())),
                    args: vec![],
                }),
            ],
            is_async: false,
        }];

        let removed = remove_unneeded_mutability_in_items(&mut items);
        assert_eq!(removed, 0);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        assert!(matches!(
            body.first(),
            Some(RustStmt::Let {
                mutable: true,
                name,
                ..
            }) if name == "apply"
        ));
    }
}
