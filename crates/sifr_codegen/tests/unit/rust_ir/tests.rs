use super::*;

#[test]
fn constructs_struct_with_derives() {
    let item = RustItem::Struct {
        name: "Point".to_string(),
        visibility: Visibility::Pub,
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        fields: vec![
            ("x".to_string(), RustType::I64),
            ("y".to_string(), RustType::I64),
        ],
    };

    match item {
        RustItem::Struct {
            name,
            derives,
            fields,
            ..
        } => {
            assert_eq!(name, "Point");
            assert_eq!(derives, vec!["Debug".to_string(), "Clone".to_string()]);
            assert_eq!(fields.len(), 2);
        }
        _ => unreachable!("constructed as Struct"),
    }
}

#[test]
fn constructs_impl_block_with_method() {
    let method = RustItem::Fn {
        name: "sum".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::SelfParam { mutable: false }],
        ret: Some(RustType::I64),
        body: vec![RustStmt::Return(Some(RustExpr::BinOp {
            left: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident("self".to_string())),
                field: "x".to_string(),
            }),
            op: "+".to_string(),
            right: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident("self".to_string())),
                field: "y".to_string(),
            }),
        }))],
        is_async: false,
    };
    let item = RustItem::Impl {
        target: "Point".to_string(),
        type_params: vec![],
        trait_: None,
        items: vec![method],
    };

    match item {
        RustItem::Impl { target, items, .. } => {
            assert_eq!(target, "Point");
            assert_eq!(items.len(), 1);
        }
        _ => unreachable!("constructed as Impl"),
    }
}

#[test]
fn constructs_function_with_let_if_return() {
    let func = RustItem::Fn {
        name: "abs_like".to_string(),
        visibility: Visibility::Pub,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "v".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::I64),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "is_negative".to_string(),
                ty: Some(RustType::Bool),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("v".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                },
            },
            RustStmt::If {
                cond: RustExpr::Ident("is_negative".to_string()),
                then_body: vec![RustStmt::Return(Some(RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(RustExpr::Ident("v".to_string())),
                }))],
                else_body: Some(vec![RustStmt::Return(Some(RustExpr::Ident(
                    "v".to_string(),
                )))]),
            },
        ],
        is_async: false,
    };

    match func {
        RustItem::Fn { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], RustStmt::Let { .. }));
            assert!(matches!(body[1], RustStmt::If { .. }));
        }
        _ => unreachable!("constructed as Fn"),
    }
}

#[test]
fn constructs_match_expression() {
    let expr = RustExpr::Match {
        expr: Box::new(RustExpr::Ident("value".to_string())),
        arms: vec![
            RustMatchArm {
                pattern: "0".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                    "zero".to_string(),
                ))))],
            },
            RustMatchArm {
                pattern: "_".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                    "many".to_string(),
                ))))],
            },
        ],
    };

    match expr {
        RustExpr::Match { arms, .. } => assert_eq!(arms.len(), 2),
        _ => unreachable!("constructed as Match"),
    }
}

#[test]
fn constructs_closure_expression() {
    let expr = RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "x".to_string(),
            ty: RustType::I64,
        }],
        body: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("x".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
        }),
        is_move: false,
    };

    match expr {
        RustExpr::Closure { params, .. } => assert_eq!(params.len(), 1),
        _ => unreachable!("constructed as Closure"),
    }
}

#[test]
fn constructs_parenthesized_expression() {
    let expr = RustExpr::Paren(Box::new(RustExpr::BinOp {
        left: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
        op: "+".to_string(),
        right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
    }));

    match expr {
        RustExpr::Paren(inner) => assert!(matches!(*inner, RustExpr::BinOp { .. })),
        _ => unreachable!("constructed as Paren"),
    }
}
