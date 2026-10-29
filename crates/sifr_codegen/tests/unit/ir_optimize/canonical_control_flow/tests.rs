use super::*;
use crate::{RustType, Visibility};

#[test]
fn removes_constant_branches_and_unreachable_tails() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: Some(RustType::I64),
        body: vec![
            RustStmt::If {
                cond: RustExpr::Literal(RustLiteral::Bool(true)),
                then_body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                    1,
                ))))],
                else_body: Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                    RustLiteral::Int(2),
                )))]),
            },
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(3)))),
        ],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 3);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(body.as_slice(), [RustStmt::Block(branch)] if branch.len() == 1));
}

#[test]
fn canonicalizes_equality_assertions_and_identifier_format_captures() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![RustStmt::Assert {
            cond: RustExpr::Paren(Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}".to_string(),
                    args: vec![RustExpr::Ident("value".to_string())],
                }),
                op: "==".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Str("1".to_string()))),
            })),
            msg: None,
        }],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 2);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    let [RustStmt::Expr(RustExpr::MacroCall { name, args })] = body.as_slice() else {
        panic!("expected canonical assertion macro, got {body:?}");
    };
    assert_eq!(name, "assert_eq");
    assert!(matches!(
        args.first(),
        Some(RustExpr::MethodCall { receiver, method, args })
            if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "value")
                && method == "to_string"
                && args.is_empty()
    ));
}

#[test]
fn canonicalizes_empty_optional_wildcard_scaffolds_as_discarded_values() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![RustStmt::IfLet {
            pattern: "Some(_)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("values".to_string())),
                method: "pop".to_string(),
                args: Vec::new(),
            },
            then_body: Vec::new(),
            else_body: None,
        }],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 1);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(
        body.as_slice(),
        [RustStmt::Let {
            name,
            value: RustExpr::MethodCall { method, .. },
            ..
        }] if name == "_" && method == "pop"
    ));
}

#[test]
fn canonicalizes_returns_only_along_tail_position_paths() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: Some(RustType::I64),
        body: vec![RustStmt::If {
            cond: RustExpr::Ident("condition".to_string()),
            then_body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                1,
            ))))],
            else_body: Some(vec![RustStmt::Match {
                expr: RustExpr::Ident("choice".to_string()),
                arms: vec![
                    crate::RustMatchArm {
                        pattern: "0".to_string(),
                        bindings: Vec::new(),
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::Literal(
                            RustLiteral::Int(2),
                        )))],
                    },
                    crate::RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: Vec::new(),
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::Literal(
                            RustLiteral::Int(3),
                        )))],
                    },
                ],
            }]),
        }],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 3);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    let [
        RustStmt::If {
            then_body,
            else_body: Some(else_body),
            ..
        },
    ] = body.as_slice()
    else {
        panic!("expected tail if, got {body:?}");
    };
    assert!(matches!(then_body.as_slice(), [RustStmt::TailExpr(_)]));
    let [RustStmt::Match { arms, .. }] = else_body.as_slice() else {
        panic!("expected tail match, got {else_body:?}");
    };
    assert!(
        arms.iter()
            .all(|arm| matches!(arm.body.as_slice(), [RustStmt::TailExpr(_)]))
    );
}

#[test]
fn removes_identity_maps_and_forwarding_closures() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("result".to_string())),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "value".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Ident("value".to_string())),
                    is_move: false,
                }],
            }),
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("result".to_string())),
                method: "map_err".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "error".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "Error".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![RustExpr::Ident("error".to_string())],
                    }),
                    is_move: false,
                }],
            }),
        ],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 2);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(
        body.first(),
        Some(RustStmt::Expr(RustExpr::Ident(name))) if name == "result"
    ));
    assert!(matches!(
        body.get(1),
        Some(RustStmt::Expr(RustExpr::MethodCall { args, .. }))
            if matches!(args.as_slice(), [RustExpr::Path(path)] if path == &["Error", "from"])
    ));
}

#[test]
fn removes_scalar_and_formatting_identities() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![
            RustStmt::Expr(RustExpr::BinOp {
                left: Box::new(RustExpr::Paren(Box::new(RustExpr::Ident(
                    "length".to_string(),
                )))),
                op: "+".to_string(),
                right: Box::new(RustExpr::Verbatim("0usize".to_string())),
            }),
            RustStmt::Expr(RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "{}".to_string(),
                args: vec![RustExpr::Ident("value".to_string())],
            }),
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Literal(RustLiteral::Str(String::new()))),
                method: "to_string".to_string(),
                args: Vec::new(),
            }),
        ],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 3);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(
        body.first(),
        Some(RustStmt::Expr(RustExpr::Paren(inner)))
            if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "length")
    ));
    assert!(matches!(
        body.get(1),
        Some(RustStmt::Expr(RustExpr::MethodCall { method, .. })) if method == "to_string"
    ));
    assert!(matches!(
        body.get(2),
        Some(RustStmt::Expr(RustExpr::FnCall { func, args }))
            if args.is_empty()
                && matches!(func.as_ref(), RustExpr::Path(path) if path == &["String", "new"])
    ));
}

#[test]
fn flattens_nested_format_macros_into_outer_format_arguments() {
    let mut items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![RustStmt::Expr(RustExpr::FormatMacro {
            name: "println".to_string(),
            format_str: "repr = {}, exact = {:?}".to_string(),
            args: vec![
                RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{:?}".to_string(),
                    args: vec![RustExpr::Ident("value".to_string())],
                },
                RustExpr::Ident("exact".to_string()),
            ],
        })],
        is_async: false,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 1);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(
        body.as_slice(),
        [RustStmt::Expr(RustExpr::FormatMacro {
            format_str,
            args,
            ..
        })] if format_str == "repr = {:?}, exact = {:?}"
            && matches!(args.as_slice(), [RustExpr::Ident(value), RustExpr::Ident(exact)]
                if value == "value" && exact == "exact")
    ));
}

#[test]
fn canonicalizes_zero_capacity_strings() {
    let mut items = vec![RustItem::Const {
        name: "VALUE".to_string(),
        ty: RustType::String_,
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "String".to_string(),
                "with_capacity".to_string(),
            ])),
            args: vec![RustExpr::Verbatim("0usize".to_string())],
        },
        visibility: Visibility::Private,
    }];

    assert_eq!(simplify_control_flow_in_items(&mut items), 1);
    let RustItem::Const { value, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(
        value,
        RustExpr::FnCall { func, args }
            if args.is_empty()
                && matches!(func.as_ref(), RustExpr::Path(path) if path == &["String", "new"])
    ));
}
