//! Environment intrinsic lowerers for registry migration.

use crate::{RustExpr, RustParam, RustType};

pub(super) fn lower_env_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; if __k.is_empty() || __k.contains('=') || __k.as_bytes().contains(&0) {{ None }} else {{ std::env::var(__k).ok() }} }}",
        args[0]
    )))
}

pub(super) fn lower_env_set(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; let __v = {}; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0) {{ std::env::set_var(__k, __v); }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_env_unset(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) {{ std::env::remove_var(__k); }} }}",
        args[0]
    )))
}

pub(super) fn lower_env_keys(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "0".to_string(),
                        }),
                        method: "to_string_lossy".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_env_values(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__kv".to_string())),
                            field: "1".to_string(),
                        }),
                        method: "to_string_lossy".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_env_items(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "env".to_string(),
                    "vars_os".to_string(),
                ])),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__kv".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}={}".to_string(),
                    args: vec![
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__kv".to_string())),
                                field: "0".to_string(),
                            }),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        },
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__kv".to_string())),
                                field: "1".to_string(),
                            }),
                            method: "to_string_lossy".to_string(),
                            args: vec![],
                        },
                    ],
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<String>>".to_string(),
        args: vec![],
    })
}
