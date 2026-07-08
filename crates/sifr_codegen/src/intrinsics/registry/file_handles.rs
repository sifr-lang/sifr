//! Builtin file-open intrinsic lowerer.

use crate::{RustExpr, RustStmt};

fn owned_str(arg: &RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg.clone()),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn path_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__path".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn mode_as_str_expr() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__mode".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

fn open_file_handle_expr() -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "fs".to_string(),
                "open_file".to_string(),
            ])),
            args: vec![path_as_str_expr(), mode_as_str_expr()],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident("__io_err".to_string())],
    }))
}

fn native_file_handle_new_expr(handle_id: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "NativeFileHandle".to_string(),
            "new".to_string(),
        ])),
        args: vec![handle_id],
    }
}

pub(crate) fn lower_builtin_open(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Try(Box::new(RustExpr::FnCall {
        func: Box::new(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__path".to_string(),
                    ty: None,
                    value: owned_str(&args[0]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__mode".to_string(),
                    ty: None,
                    value: owned_str(&args[1]),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__handle_id".to_string(),
                    ty: None,
                    value: open_file_handle_expr(),
                },
                RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(
                        vec!["Ok::<FileHandle, IOError>".to_string()],
                    )),
                    args: vec![RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "FileHandle".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![
                            native_file_handle_new_expr(RustExpr::Ident("__handle_id".to_string())),
                            RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__mode".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                        ],
                    }],
                })),
            ],
            is_move: false,
            is_async: false,
        }),
        args: vec![],
    })))
}
