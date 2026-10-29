use super::*;
use crate::{RustEnumVariant, RustLiteral, Visibility};

#[test]
fn collects_unqualified_import_symbols() {
    let items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "m".to_string(),
            ty: RustType::Named("HashMap<String, i64>".to_string()),
        }],
        ret: Some(RustType::Named("BigInt".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "s".to_string(),
                ty: Some(RustType::Named("HashSet<String>".to_string())),
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "HashSet".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "VecDeque".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            }),
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Mutex".to_string(), "new".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            }),
        ],
        is_async: false,
    }];
    let needs = collect_import_needs_from_items(&items);
    assert!(needs.collections.needs_hashmap);
    assert!(needs.collections.needs_hashset);
    assert!(needs.collections.needs_vecdeque);
    assert!(needs.runtime.needs_mutex);
    assert!(needs.runtime.numeric.needs_bigint);
    assert!(!needs.runtime.numeric.needs_decimal);
    assert!(!needs.runtime.numeric.needs_bigdecimal);
}

#[test]
fn collects_imports_from_enum_tuple_payloads() {
    let items = vec![RustItem::Enum {
        name: "Payload".to_string(),
        visibility: Visibility::Private,
        derives: Vec::new(),
        repr: None,
        variants: vec![RustEnumVariant {
            name: "Mapping".to_string(),
            tuple_fields: vec![RustType::HashMap(
                Box::new(RustType::String_),
                Box::new(RustType::I64),
            )],
            fields: Vec::new(),
            value: None,
        }],
    }];

    let needs = collect_import_needs_from_items(&items);

    assert!(needs.collections.needs_hashmap);
}

#[test]
fn ignores_fully_qualified_symbols() {
    let items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "m".to_string(),
            ty: RustType::Named("std::collections::HashMap<String, i64>".to_string()),
        }],
        ret: Some(RustType::Named("num_bigint::BigInt".to_string())),
        body: vec![RustStmt::Expr(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "sync".to_string(),
                "Mutex".to_string(),
                "new".to_string(),
            ])),
            args: vec![RustExpr::Literal(RustLiteral::Int(1))],
        })],
        is_async: false,
    }];
    let needs = collect_import_needs_from_items(&items);
    assert!(!needs.collections.needs_hashmap);
    assert!(!needs.collections.needs_hashset);
    assert!(!needs.collections.needs_vecdeque);
    assert!(!needs.runtime.needs_mutex);
    assert!(!needs.runtime.numeric.needs_bigint);
    assert!(!needs.runtime.numeric.needs_decimal);
    assert!(!needs.runtime.numeric.needs_bigdecimal);
}

#[test]
fn collects_sifr_runtime_integer_symbols() {
    let items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "value".to_string(),
            ty: RustType::Named("SifrInt".to_string()),
        }],
        ret: Some(RustType::Named("SifrInt".to_string())),
        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                "SifrInt".to_string(),
                "from_i64".to_string(),
            ])),
            args: vec![RustExpr::Literal(RustLiteral::Int(1))],
        }))],
        is_async: false,
    }];

    let needs = collect_import_needs_from_items(&items);

    assert!(needs.runtime.needs_sifr_int);
    assert!(needs.runtime.needs_sifr_runtime);
}

#[test]
fn collects_import_symbols_from_verbatim_statements() {
    let items = vec![RustItem::Fn {
        name: "demo".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![],
        ret: None,
        body: vec![RustStmt::Verbatim(
            "let value: SifrInt = SifrInt::from_i64(1);".to_string(),
        )],
        is_async: false,
    }];

    let needs = collect_import_needs_from_items(&items);

    assert!(needs.runtime.needs_sifr_int);
    assert!(needs.runtime.needs_sifr_runtime);
}

#[test]
fn absolute_runtime_paths_require_the_crate_without_an_unqualified_import() {
    let needs = collect_import_needs_from_source(
        "fn bridge(n: i64) { ::sifr_runtime::interop::SifrIntBridge::from(n); }",
    );

    assert!(!needs.runtime.needs_sifr_int);
    assert!(needs.runtime.needs_sifr_runtime);
}
