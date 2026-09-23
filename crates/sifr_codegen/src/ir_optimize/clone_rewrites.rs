use super::{is_copy_type, not_expr, string_key_loop_rewrite::rewrite_string_key_loop_iter};
use crate::{RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType};

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

pub(super) fn optimize_item(item: &mut RustItem) -> usize {
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

pub(super) fn optimize_block(body: &mut Vec<RustStmt>) -> usize {
    let mut removed = 0usize;
    for stmt in body.iter_mut() {
        removed += optimize_stmt(stmt);
    }
    let before = body.len();
    body.retain(|stmt| !is_self_assignment(stmt));
    removed + (before - body.len())
}

pub(super) fn is_self_assignment(stmt: &RustStmt) -> bool {
    matches!(
        stmt,
        RustStmt::Assign {
            target: RustExpr::Ident(target),
            value: RustExpr::Ident(value),
        } if target == value
    )
}

pub(super) fn optimize_stmt(stmt: &mut RustStmt) -> usize {
    match stmt {
        RustStmt::Verbatim(_) | RustStmt::LetDecl { .. } => 0,
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
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            optimize_expr(expr)
        }
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
            if else_body.as_ref().is_some_and(Vec::is_empty)
                && !then_body
                    .last()
                    .is_some_and(|stmt| matches!(stmt, RustStmt::TailExpr(_)))
            {
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
            if else_body.as_ref().is_some_and(Vec::is_empty)
                && !then_body
                    .last()
                    .is_some_and(|stmt| matches!(stmt, RustStmt::TailExpr(_)))
            {
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
        RustStmt::For { var, iter, body } => {
            let mut removed = rewrite_string_key_loop_iter(var, iter, body);
            removed += optimize_expr(iter);
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
            if matches!(cond, RustExpr::Literal(RustLiteral::Bool(true))) {
                *stmt = RustStmt::Loop {
                    body: std::mem::take(body),
                };
                removed += 1;
            }
            removed
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            optimize_block(body)
        }
    }
}

pub(super) fn optimize_expr(expr: &mut RustExpr) -> usize {
    match expr {
        RustExpr::Clone(inner) => {
            let mut removed = optimize_expr(inner);
            if matches!(inner.as_ref(), RustExpr::Clone(_)) || should_remove_clone(inner.as_ref()) {
                let replacement =
                    *std::mem::replace(inner, Box::new(RustExpr::Literal(RustLiteral::Unit)));
                *expr = replacement;
                removed += 1;
            }
            removed
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::Verbatim(_) => 0,
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
            let mut removed = optimize_expr(receiver);
            for arg in args.iter_mut() {
                removed += optimize_expr(arg);
            }
            if let Some(replacement) =
                super::clone_chain_rewrite::take_compounded_method_clone(receiver, method, args)
            {
                *expr = replacement;
                removed += 1;
            } else if method == "skip" && args.len() == 1 && is_zero_usize_expr(&args[0]) {
                let replacement =
                    *std::mem::replace(receiver, Box::new(RustExpr::Literal(RustLiteral::Unit)));
                *expr = replacement;
                removed += 1;
            } else if method == "map_or_else" && args.len() == 2 && is_identity_closure(&args[1]) {
                if is_known_std_fallible_receiver(receiver.as_ref()) {
                    *method = "unwrap_or_else".to_string();
                    args.pop();
                    removed += 1;
                }
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
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => optimize_expr(duration) + optimize_expr(future) + optimize_expr(error),
        RustExpr::FormatMacro { args, .. } => {
            let mut removed = 0usize;
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::BinOp { left, op, right } => {
            let mut removed = optimize_expr(left) + optimize_expr(right);
            if let Some(replacement) = simplified_bool_comparison(left, op, right) {
                *expr = replacement;
                removed += 1;
            }
            removed
        }
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
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            optimize_block(body)
        }
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

pub(super) fn should_remove_clone(inner: &RustExpr) -> bool {
    match inner {
        RustExpr::Literal(_) => true,
        RustExpr::Ref { .. } => true,
        RustExpr::Cast { ty, .. } => is_copy_type(ty),
        RustExpr::Paren(inner) => should_remove_clone(inner),
        RustExpr::FnCall { func, .. } => matches!(
            func.as_ref(),
            RustExpr::Path(path)
                if path.first().is_some_and(|segment| segment == "SifrInt")
        ),
        _ => false,
    }
}

pub(super) fn is_zero_usize_expr(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Literal(RustLiteral::Int(0)) => true,
        RustExpr::Cast { expr, ty } => {
            matches!(ty, RustType::Named(name) if name == "usize") && is_zero_usize_expr(expr)
        }
        RustExpr::Paren(inner) => is_zero_usize_expr(inner),
        _ => false,
    }
}

pub(super) fn is_identity_closure(expr: &RustExpr) -> bool {
    let RustExpr::Closure { params, body, .. } = expr else {
        return false;
    };
    let [RustParam::Named { name, .. }] = params.as_slice() else {
        return false;
    };
    match body.as_ref() {
        RustExpr::Ident(body_name) => body_name == name,
        RustExpr::Paren(inner) => {
            matches!(inner.as_ref(), RustExpr::Ident(body_name) if body_name == name)
        }
        _ => false,
    }
}

pub(super) fn is_known_std_fallible_receiver(expr: &RustExpr) -> bool {
    matches!(
        expr,
        RustExpr::FnCall { func, .. }
            if matches!(
                func.as_ref(),
                RustExpr::Path(parts)
                    if matches!(
                        parts.as_slice(),
                        [type_name, method] if type_name == "Decimal" && method == "checked_div"
                    )
            )
    )
}

pub(super) fn simplified_bool_comparison(
    left: &RustExpr,
    op: &str,
    right: &RustExpr,
) -> Option<RustExpr> {
    if !matches!(op, "==" | "!=") {
        return None;
    }
    if let RustExpr::Literal(RustLiteral::Bool(value)) = right {
        return Some(if (*value && op == "==") || (!*value && op == "!=") {
            left.clone()
        } else {
            not_expr(left.clone())
        });
    }
    if let RustExpr::Literal(RustLiteral::Bool(value)) = left {
        return Some(if (*value && op == "==") || (!*value && op == "!=") {
            right.clone()
        } else {
            not_expr(right.clone())
        });
    }
    None
}
