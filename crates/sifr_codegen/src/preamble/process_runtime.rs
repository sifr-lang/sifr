//! Runtime support for generated process status helpers.

use crate::{RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, Visibility};

pub(crate) fn build_process_status_items() -> Vec<RustItem> {
    vec![
        RustItem::Attr("#[cfg(unix)]".to_string()),
        RustItem::Fn {
            name: "__sifr_process_exit_signal".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "status".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Named("std::process::ExitStatus".to_string())),
                },
            }],
            ret: Some(RustType::Option(Box::new(RustType::I64))),
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__signal".to_string(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "os".to_string(),
                            "unix".to_string(),
                            "process".to_string(),
                            "ExitStatusExt".to_string(),
                            "signal".to_string(),
                        ])),
                        args: vec![RustExpr::Ident("status".to_string())],
                    },
                },
                RustStmt::Return(Some(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__signal".to_string())),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "__signal".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__signal".to_string())),
                            ty: RustType::I64,
                        }),
                        is_move: false,
                    }],
                })),
            ],
            is_async: false,
        },
        RustItem::Attr("#[cfg(not(unix))]".to_string()),
        RustItem::Fn {
            name: "__sifr_process_exit_signal".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "_status".to_string(),
                ty: RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Named("std::process::ExitStatus".to_string())),
                },
            }],
            ret: Some(RustType::Option(Box::new(RustType::I64))),
            body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))],
            is_async: false,
        },
    ]
}
