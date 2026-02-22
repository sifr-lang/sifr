//! Test/assertion intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustStmt};

fn arg_expr(args: &[String], idx: usize) -> RustExpr {
    RustExpr::Ident(args[idx].clone())
}

pub(super) fn lower_assert_eq(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert_eq".to_string(),
        args: vec![
            RustExpr::Ident(args[0].clone()),
            RustExpr::Ident(args[1].clone()),
        ],
    })
}

pub(super) fn lower_assert_ne(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert_ne".to_string(),
        args: vec![
            RustExpr::Ident(args[0].clone()),
            RustExpr::Ident(args[1].clone()),
        ],
    })
}

pub(super) fn lower_assert_true(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert".to_string(),
        args: vec![arg_expr(args, 0)],
    })
}

pub(super) fn lower_assert_false(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__cond".to_string(),
            ty: None,
            value: arg_expr(args, 0),
        }],
        expr: Some(Box::new(RustExpr::MacroCall {
            name: "assert".to_string(),
            args: vec![RustExpr::UnaryOp {
                op: "!".to_string(),
                operand: Box::new(RustExpr::Ident("__cond".to_string())),
            }],
        })),
    })
}

pub(super) fn lower_assert_almost_eq(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__lhs".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__rhs".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__tol".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
        ],
        expr: Some(Box::new(RustExpr::MacroCall {
            name: "assert".to_string(),
            args: vec![
                RustExpr::BinOp {
                    left: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__lhs".to_string())),
                            op: "-".to_string(),
                            right: Box::new(RustExpr::Ident("__rhs".to_string())),
                        }),
                        method: "abs".to_string(),
                        args: vec![],
                    }),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Ident("__tol".to_string())),
                },
                RustExpr::Literal(RustLiteral::Str(
                    "assert_almost_eq failed: {} != {} (tolerance {})".to_string(),
                )),
                RustExpr::Ident("__lhs".to_string()),
                RustExpr::Ident("__rhs".to_string()),
                RustExpr::Ident("__tol".to_string()),
            ],
        })),
    })
}

pub(super) fn lower_assert_gt(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert".to_string(),
        args: vec![
            RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(args[0].clone())),
                op: ">".to_string(),
                right: Box::new(RustExpr::Ident(args[1].clone())),
            },
            RustExpr::Literal(crate::RustLiteral::Str(
                "assert_gt failed: {} is not > {}".to_string(),
            )),
            RustExpr::Ident(args[0].clone()),
            RustExpr::Ident(args[1].clone()),
        ],
    })
}

pub(super) fn lower_assert_lt(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert".to_string(),
        args: vec![
            RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(args[0].clone())),
                op: "<".to_string(),
                right: Box::new(RustExpr::Ident(args[1].clone())),
            },
            RustExpr::Literal(crate::RustLiteral::Str(
                "assert_lt failed: {} is not < {}".to_string(),
            )),
            RustExpr::Ident(args[0].clone()),
            RustExpr::Ident(args[1].clone()),
        ],
    })
}
