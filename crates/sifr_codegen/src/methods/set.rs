//! Set method lowerers for registry lowering.

use crate::{RustExpr, RustStmt};

fn is_already_borrowed_rendered_expr(arg: &RustExpr) -> bool {
    match arg {
        RustExpr::Ref { .. } => true,
        RustExpr::MethodCall { method, .. } => method == "as_str",
        RustExpr::Paren(inner)
        | RustExpr::Try(inner)
        | RustExpr::Await(inner)
        | RustExpr::Clone(inner) => is_already_borrowed_rendered_expr(inner),
        _ => false,
    }
}

fn render_borrowed_arg_expr(arg: &RustExpr) -> RustExpr {
    match arg {
        RustExpr::Ref { expr, .. } if is_already_borrowed_rendered_expr(expr) => {
            expr.as_ref().clone()
        }
        RustExpr::Ref { .. } => arg.clone(),
        _ if is_already_borrowed_rendered_expr(arg) => arg.clone(),
        _ => RustExpr::Ref {
            mutable: false,
            expr: Box::new(arg.clone()),
        },
    }
}

fn lower_set_op_collect(object: &RustExpr, args: &[RustExpr], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object.clone()),
                method: method.to_string(),
                args: vec![render_borrowed_arg_expr(&args[0])],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        method: "collect::<std::collections::HashSet<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_add(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let lowered_value = if matches!(&args[0], RustExpr::Ident(_)) {
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(args[0].clone()))),
            method: "clone".to_string(),
            args: vec![],
        }
    } else {
        args[0].clone()
    };
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "insert".to_string(),
        args: vec![lowered_value],
    })
}

pub(super) fn lower_remove(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "remove".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_discard(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "remove".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_contains(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "contains".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_clear(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "clear".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_copy(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "clone".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_issubset(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "is_subset".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_issuperset(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "is_superset".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_isdisjoint(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(object.clone()),
        method: "is_disjoint".to_string(),
        args: vec![render_borrowed_arg_expr(&args[0])],
    })
}

pub(super) fn lower_pop(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__v".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(object.clone()),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "next".to_string(),
                        args: vec![],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                },
            },
            RustStmt::IfLet {
                pattern: "Some(ref __val)".to_string(),
                expr: RustExpr::Ident("__v".to_string()),
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: "remove".to_string(),
                    args: vec![RustExpr::Ident("__val".to_string())],
                })],
                else_body: None,
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__v".to_string()))),
    })
}

pub(super) fn lower_union(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_set_op_collect(object, args, "union")
}

pub(super) fn lower_intersection(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_set_op_collect(object, args, "intersection")
}

pub(super) fn lower_difference(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_set_op_collect(object, args, "difference")
}

pub(super) fn lower_symmetric_difference(object: &RustExpr, args: &[RustExpr]) -> Option<RustExpr> {
    lower_set_op_collect(object, args, "symmetric_difference")
}
