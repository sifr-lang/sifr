use super::*;
use crate::{RustLiteral, RustType, Visibility};

#[test]
fn removes_generated_pure_bindings_and_silences_source_bindings() {
    let mut items = vec![RustItem::Fn {
        name: "main".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_dead".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(1)),
            },
            RustStmt::Let {
                mutable: false,
                name: "source_dead".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(1)),
            },
            RustStmt::Let {
                mutable: false,
                name: "live".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(2)),
            },
            RustStmt::Expr(RustExpr::Ident("live".to_string())),
        ],
        is_async: false,
    }];
    assert_eq!(remove_unread_pure_bindings_in_items(&mut items), 2);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert_eq!(body.len(), 3);
    assert!(matches!(
        &body[0],
        RustStmt::Let { name, .. } if name == "_source_dead"
    ));
}

#[test]
fn retains_unread_binary_expressions_with_unknown_operator_effects() {
    let mut items = vec![RustItem::Fn {
        name: "main".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_result".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                    op: "/".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                },
            },
            RustStmt::Expr(RustExpr::Literal(RustLiteral::Unit)),
        ],
        is_async: false,
    }];

    assert_eq!(remove_unread_pure_bindings_in_items(&mut items), 0);
    let RustItem::Fn { body, .. } = &items[0] else {
        unreachable!();
    };
    assert!(matches!(body.first(), Some(RustStmt::Let { name, .. }) if name == "__sifr_result"));
}
