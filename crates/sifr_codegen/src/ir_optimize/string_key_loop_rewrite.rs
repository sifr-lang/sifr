use crate::{RustExpr, RustParam, RustStmt};

pub(super) fn rewrite_string_key_loop_iter(
    var: &str,
    iter: &mut RustExpr,
    body: &[RustStmt],
) -> usize {
    let Some(chars_iter) = string_to_string_map_chars_source(iter) else {
        return 0;
    };
    if !block_uses_name_only_as_set_key(body, var) {
        return 0;
    }
    *iter = chars_iter;
    1
}

fn string_to_string_map_chars_source(iter: &RustExpr) -> Option<RustExpr> {
    let RustExpr::MethodCall {
        receiver,
        method,
        args,
    } = iter
    else {
        return None;
    };
    if method != "map" || args.len() != 1 {
        return None;
    }
    let RustExpr::Closure { params, body, .. } = &args[0] else {
        return None;
    };
    let [RustParam::Named { name, .. }] = params.as_slice() else {
        return None;
    };
    let RustExpr::MethodCall {
        receiver: closure_receiver,
        method: closure_method,
        args: closure_args,
    } = body.as_ref()
    else {
        return None;
    };
    if closure_method != "to_string"
        || !closure_args.is_empty()
        || !matches!(closure_receiver.as_ref(), RustExpr::Ident(ident) if ident == name)
    {
        return None;
    }
    let RustExpr::MethodCall {
        method: chars_method,
        args: chars_args,
        ..
    } = receiver.as_ref()
    else {
        return None;
    };
    if chars_method == "chars" && chars_args.is_empty() {
        Some(receiver.as_ref().clone())
    } else {
        None
    }
}

fn block_uses_name_only_as_set_key(body: &[RustStmt], name: &str) -> bool {
    let mut found = false;
    body.iter()
        .all(|stmt| stmt_uses_name_only_as_set_key(stmt, name, &mut found))
        && found
}

fn stmt_uses_name_only_as_set_key(stmt: &RustStmt, name: &str, found: &mut bool) -> bool {
    match stmt {
        RustStmt::Verbatim(_) => false,
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
            expr_uses_name_only_as_set_key(value, name, false, found)
        }
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            expr_uses_name_only_as_set_key(value, name, false, found)
                && block_uses_name_only_as_set_key_with_found(else_body, name, found)
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            expr_uses_name_only_as_set_key(target, name, false, found)
                && expr_uses_name_only_as_set_key(value, name, false, found)
        }
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
        }
        RustStmt::Assert { cond, msg } => {
            expr_uses_name_only_as_set_key(cond, name, false, found)
                && msg
                    .as_ref()
                    .is_none_or(|msg| expr_uses_name_only_as_set_key(msg, name, false, found))
        }
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_uses_name_only_as_set_key(cond, name, false, found)
                && block_uses_name_only_as_set_key_with_found(then_body, name, found)
                && else_body.as_ref().is_none_or(|else_body| {
                    block_uses_name_only_as_set_key_with_found(else_body, name, found)
                })
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
                && block_uses_name_only_as_set_key_with_found(then_body, name, found)
                && else_body.as_ref().is_none_or(|else_body| {
                    block_uses_name_only_as_set_key_with_found(else_body, name, found)
                })
        }
        RustStmt::While { cond, body } => {
            expr_uses_name_only_as_set_key(cond, name, false, found)
                && block_uses_name_only_as_set_key_with_found(body, name, found)
        }
        RustStmt::For { iter, body, .. } => {
            expr_uses_name_only_as_set_key(iter, name, false, found)
                && block_uses_name_only_as_set_key_with_found(body, name, found)
        }
        RustStmt::Block(body) | RustStmt::Loop { body } | RustStmt::LocalFn { body, .. } => {
            block_uses_name_only_as_set_key_with_found(body, name, found)
        }
        RustStmt::With { items, body } => {
            items
                .iter()
                .all(|item| expr_uses_name_only_as_set_key(&item.value, name, false, found))
                && block_uses_name_only_as_set_key_with_found(body, name, found)
        }
        RustStmt::Match { expr, arms } => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
                && arms.iter().all(|arm| {
                    arm.guard.as_ref().is_none_or(|guard| {
                        expr_uses_name_only_as_set_key(guard, name, false, found)
                    }) && block_uses_name_only_as_set_key_with_found(&arm.body, name, found)
                })
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => true,
    }
}

fn block_uses_name_only_as_set_key_with_found(
    body: &[RustStmt],
    name: &str,
    found: &mut bool,
) -> bool {
    body.iter()
        .all(|stmt| stmt_uses_name_only_as_set_key(stmt, name, found))
}

fn expr_uses_name_only_as_set_key(
    expr: &RustExpr,
    name: &str,
    allow_direct: bool,
    found: &mut bool,
) -> bool {
    match expr {
        RustExpr::Ident(ident) if ident == name => {
            *found = true;
            allow_direct
        }
        RustExpr::Ref { expr, .. } => {
            expr_uses_name_only_as_set_key(expr, name, allow_direct, found)
        }
        RustExpr::Paren(inner) | RustExpr::Clone(inner) => {
            expr_uses_name_only_as_set_key(inner, name, allow_direct, found)
        }
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            if method == "clone" && args.is_empty() {
                return expr_uses_name_only_as_set_key(receiver, name, allow_direct, found);
            }
            expr_uses_name_only_as_set_key(receiver, name, false, found)
                && args.iter().all(|arg| {
                    expr_uses_name_only_as_set_key(
                        arg,
                        name,
                        matches!(
                            method.as_str(),
                            "contains" | "insert" | "remove" | "take" | "get"
                        ),
                        found,
                    )
                })
        }
        RustExpr::FnCall { func, args } => {
            expr_uses_name_only_as_set_key(func, name, false, found)
                && args
                    .iter()
                    .all(|arg| expr_uses_name_only_as_set_key(arg, name, false, found))
        }
        RustExpr::BinOp { left, right, .. } => {
            expr_uses_name_only_as_set_key(left, name, false, found)
                && expr_uses_name_only_as_set_key(right, name, false, found)
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Field { expr: operand, .. } => {
            expr_uses_name_only_as_set_key(operand, name, false, found)
        }
        RustExpr::Index { expr, index } => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
                && expr_uses_name_only_as_set_key(index, name, false, found)
        }
        RustExpr::Slice {
            expr, start, stop, ..
        } => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
                && start
                    .as_ref()
                    .is_none_or(|start| expr_uses_name_only_as_set_key(start, name, false, found))
                && stop
                    .as_ref()
                    .is_none_or(|stop| expr_uses_name_only_as_set_key(stop, name, false, found))
        }
        RustExpr::Block { stmts, expr } => {
            block_uses_name_only_as_set_key_with_found(stmts, name, found)
                && expr
                    .as_ref()
                    .is_none_or(|expr| expr_uses_name_only_as_set_key(expr, name, false, found))
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            expr_uses_name_only_as_set_key(cond, name, false, found)
                && expr_uses_name_only_as_set_key(then_expr, name, false, found)
                && else_expr
                    .as_ref()
                    .is_none_or(|expr| expr_uses_name_only_as_set_key(expr, name, false, found))
        }
        RustExpr::Match { expr, arms } => {
            expr_uses_name_only_as_set_key(expr, name, false, found)
                && arms.iter().all(|arm| {
                    arm.guard.as_ref().is_none_or(|guard| {
                        expr_uses_name_only_as_set_key(guard, name, false, found)
                    }) && block_uses_name_only_as_set_key_with_found(&arm.body, name, found)
                })
        }
        RustExpr::Closure { body, .. } => expr_uses_name_only_as_set_key(body, name, false, found),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            block_uses_name_only_as_set_key_with_found(body, name, found)
        }
        RustExpr::Array(items) | RustExpr::Vec(items) | RustExpr::Tuple(items) => items
            .iter()
            .all(|item| expr_uses_name_only_as_set_key(item, name, false, found)),
        RustExpr::Range { start, end } => {
            expr_uses_name_only_as_set_key(start, name, false, found)
                && expr_uses_name_only_as_set_key(end, name, false, found)
        }
        RustExpr::FormatMacro { args, .. } | RustExpr::MacroCall { args, .. } => args
            .iter()
            .all(|arg| expr_uses_name_only_as_set_key(arg, name, false, found)),
        RustExpr::StructInit { fields, .. } => fields
            .iter()
            .all(|(_, value)| expr_uses_name_only_as_set_key(value, name, false, found)),
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            expr_uses_name_only_as_set_key(duration, name, false, found)
                && expr_uses_name_only_as_set_key(future, name, false, found)
                && expr_uses_name_only_as_set_key(error, name, false, found)
        }
        RustExpr::Literal(_) | RustExpr::Path(_) | RustExpr::Ident(_) => true,
    }
}
