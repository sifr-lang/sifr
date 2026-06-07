//! Runtime support for generated process child handles.

use crate::{RustExpr, RustItem, RustLiteral, RustStmt, RustType, Visibility};

pub(crate) fn build_process_child_items() -> Vec<RustItem> {
    vec![
        RustItem::Static {
            name: "__SIFR_PROCESS_CHILDREN".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named(
                "std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, std::process::Child>>>"
                    .to_string(),
            ),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "LazyLock".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "sync".to_string(),
                            "Mutex".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "collections".to_string(),
                                "HashMap".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }],
                    }),
                    is_move: false,
                }],
            },
        },
        RustItem::Static {
            name: "__SIFR_NEXT_PROCESS_CHILD_ID".to_string(),
            visibility: Visibility::Private,
            ty: RustType::Named("std::sync::atomic::AtomicI64".to_string()),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "atomic".to_string(),
                    "AtomicI64".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            },
        },
        RustItem::Fn {
            name: "__sifr_next_process_child_id".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "__SIFR_NEXT_PROCESS_CHILD_ID".to_string(),
                )),
                method: "fetch_add".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(1)),
                    RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "atomic".to_string(),
                        "Ordering".to_string(),
                        "SeqCst".to_string(),
                    ]),
                ],
            }))],
            is_async: false,
        },
    ]
}
