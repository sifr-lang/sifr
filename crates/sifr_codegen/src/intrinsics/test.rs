//! Test/assertion intrinsic lowerers for registry migration.

use crate::RustExpr;

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
        args: vec![RustExpr::Ident(args[0].clone())],
    })
}

pub(super) fn lower_assert_false(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MacroCall {
        name: "assert".to_string(),
        args: vec![RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::Ident(format!("({})", args[0]))),
        }],
    })
}

pub(super) fn lower_assert_almost_eq(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "assert!(({} - ({})).abs() < {}, \"assert_almost_eq failed: {{}} != {{}} (tolerance {{}})\", {}, {}, {})",
        args[0], args[1], args[2], args[0], args[1], args[2]
    )))
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
