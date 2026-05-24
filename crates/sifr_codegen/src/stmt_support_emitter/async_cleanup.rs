use super::{RustExpr, RustStmt, Type};

pub(super) fn inject_async_with_return_cleanup(
    stmts: &[RustStmt],
    receiver: &RustExpr,
) -> Vec<RustStmt> {
    stmts
        .iter()
        .flat_map(|stmt| inject_async_with_return_cleanup_stmt(stmt, receiver))
        .collect()
}

pub(super) fn inject_async_with_return_cleanup_stmt(
    stmt: &RustStmt,
    receiver: &RustExpr,
) -> Vec<RustStmt> {
    match stmt {
        RustStmt::Return(Some(value)) => vec![RustStmt::Return(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_async_with_return".to_string(),
                    ty: None,
                    value: value.clone(),
                },
                RustStmt::Expr(async_with_exit_call(receiver.clone(), "Return")),
            ],
            expr: Some(Box::new(RustExpr::Ident(
                "__sifr_async_with_return".to_string(),
            ))),
        }))],
        RustStmt::Return(None) => vec![
            RustStmt::Expr(async_with_exit_call(receiver.clone(), "Return")),
            RustStmt::Return(None),
        ],
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => vec![RustStmt::If {
            cond: cond.clone(),
            then_body: inject_async_with_return_cleanup(then_body, receiver),
            else_body: else_body
                .as_ref()
                .map(|body| inject_async_with_return_cleanup(body, receiver)),
        }],
        RustStmt::IfLet {
            pattern,
            expr,
            then_body,
            else_body,
        } => vec![RustStmt::IfLet {
            pattern: pattern.clone(),
            expr: expr.clone(),
            then_body: inject_async_with_return_cleanup(then_body, receiver),
            else_body: else_body
                .as_ref()
                .map(|body| inject_async_with_return_cleanup(body, receiver)),
        }],
        RustStmt::Match { expr, arms } => vec![RustStmt::Match {
            expr: expr.clone(),
            arms: arms
                .iter()
                .map(|arm| crate::RustMatchArm {
                    pattern: arm.pattern.clone(),
                    bindings: arm.bindings.clone(),
                    guard: arm.guard.clone(),
                    body: inject_async_with_return_cleanup(&arm.body, receiver),
                })
                .collect(),
        }],
        RustStmt::With { items, body } => vec![RustStmt::With {
            items: items.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::Block(body) => {
            vec![RustStmt::Block(inject_async_with_return_cleanup(
                body, receiver,
            ))]
        }
        RustStmt::For { var, iter, body } => vec![RustStmt::For {
            var: var.clone(),
            iter: iter.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::While { cond, body } => vec![RustStmt::While {
            cond: cond.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::Loop { body } => vec![RustStmt::Loop {
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        _ => vec![stmt.clone()],
    }
}

pub(super) fn async_with_exit_call(receiver: RustExpr, cause_variant: &str) -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::Await(Box::new(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "__aexit__".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident(format!("AsyncExitCause::{cause_variant}"))),
        }],
    }))))
}

pub(super) fn inject_async_for_early_exit_cleanup(
    stmts: &[RustStmt],
    receiver: &RustExpr,
    close_error_ty: &Type,
) -> Vec<RustStmt> {
    inject_async_for_early_exit_cleanup_with_breaks(stmts, receiver, close_error_ty, true)
}

pub(super) fn inject_async_for_early_exit_cleanup_with_breaks(
    stmts: &[RustStmt],
    receiver: &RustExpr,
    close_error_ty: &Type,
    include_breaks: bool,
) -> Vec<RustStmt> {
    stmts
        .iter()
        .flat_map(|stmt| {
            inject_async_for_early_exit_cleanup_stmt(stmt, receiver, close_error_ty, include_breaks)
        })
        .collect()
}

pub(super) fn inject_async_for_early_exit_cleanup_stmt(
    stmt: &RustStmt,
    receiver: &RustExpr,
    close_error_ty: &Type,
    include_breaks: bool,
) -> Vec<RustStmt> {
    match stmt {
        RustStmt::Break if include_breaks => vec![
            RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            RustStmt::Break,
        ],
        RustStmt::Return(Some(value)) => vec![RustStmt::Return(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_async_for_return".to_string(),
                    ty: None,
                    value: value.clone(),
                },
                RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            ],
            expr: Some(Box::new(RustExpr::Ident(
                "__sifr_async_for_return".to_string(),
            ))),
        }))],
        RustStmt::Return(None) => vec![
            RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            RustStmt::Return(None),
        ],
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => vec![RustStmt::If {
            cond: cond.clone(),
            then_body: inject_async_for_early_exit_cleanup_with_breaks(
                then_body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
            else_body: else_body.as_ref().map(|body| {
                inject_async_for_early_exit_cleanup_with_breaks(
                    body,
                    receiver,
                    close_error_ty,
                    include_breaks,
                )
            }),
        }],
        RustStmt::IfLet {
            pattern,
            expr,
            then_body,
            else_body,
        } => vec![RustStmt::IfLet {
            pattern: pattern.clone(),
            expr: expr.clone(),
            then_body: inject_async_for_early_exit_cleanup_with_breaks(
                then_body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
            else_body: else_body.as_ref().map(|body| {
                inject_async_for_early_exit_cleanup_with_breaks(
                    body,
                    receiver,
                    close_error_ty,
                    include_breaks,
                )
            }),
        }],
        RustStmt::Match { expr, arms } => vec![RustStmt::Match {
            expr: expr.clone(),
            arms: arms
                .iter()
                .map(|arm| crate::RustMatchArm {
                    pattern: arm.pattern.clone(),
                    bindings: arm.bindings.clone(),
                    guard: arm.guard.clone(),
                    body: inject_async_for_early_exit_cleanup_with_breaks(
                        &arm.body,
                        receiver,
                        close_error_ty,
                        include_breaks,
                    ),
                })
                .collect(),
        }],
        RustStmt::With { items, body } => vec![RustStmt::With {
            items: items.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
        }],
        RustStmt::Block(body) => vec![RustStmt::Block(
            inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
        )],
        RustStmt::For { var, iter, body } => vec![RustStmt::For {
            var: var.clone(),
            iter: iter.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        RustStmt::While { cond, body } => vec![RustStmt::While {
            cond: cond.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        RustStmt::Loop { body } => vec![RustStmt::Loop {
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        _ => vec![stmt.clone()],
    }
}

pub(super) fn async_for_close_call(receiver: RustExpr, close_error_ty: &Type) -> RustExpr {
    let close_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "aclose".to_string(),
        args: vec![],
    }));
    if matches!(close_error_ty.resolve_alias(), Type::Never) {
        close_call
    } else {
        RustExpr::Try(Box::new(close_call))
    }
}
