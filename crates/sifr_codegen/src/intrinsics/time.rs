//! Time intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_time_now(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "time".to_string(),
                        "SystemTime".to_string(),
                        "now".to_string(),
                    ])),
                    args: vec![],
                }),
                method: "duration_since".to_string(),
                args: vec![RustExpr::Path(vec![
                    "std".to_string(),
                    "time".to_string(),
                    "UNIX_EPOCH".to_string(),
                ])],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        }),
        method: "as_secs_f64".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_sleep(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "thread".to_string(),
            "sleep".to_string(),
        ])),
        args: vec![RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
                "from_secs_f64".to_string(),
            ])),
            args: vec![RustExpr::Ident(args[0].clone())],
        }],
    })
}

pub(super) fn lower_time_format(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "secs".to_string(),
                ty: None,
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident(args[0].clone())),
                    ty: RustType::I64,
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "dt".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "chrono".to_string(),
                            "DateTime".to_string(),
                            "from_timestamp".to_string(),
                        ])),
                        args: vec![
                            RustExpr::Ident("secs".to_string()),
                            RustExpr::Literal(RustLiteral::Int(0)),
                        ],
                    }),
                    method: "unwrap_or_default".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("dt".to_string())),
                method: "format".to_string(),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(format!("({})", args[1]))),
                }],
            }),
            method: "to_string".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_perf_counter(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ fn __monotonic() -> f64 { static __START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); let s = __START.get_or_init(std::time::Instant::now); s.elapsed().as_secs_f64() } __monotonic() }".to_string(),
    ))
}

pub(super) fn lower_monotonic(args: &[String]) -> Option<RustExpr> {
    lower_perf_counter(args)
}

pub(super) fn lower_strptime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ValueError> {{ use chrono::NaiveDateTime; let __s = {}; let __fmt = {}; NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).map_err(|e| ValueError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_gmtime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__ts".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Ident(args[0].clone())),
                ty: RustType::I64,
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "DateTime::<Utc>".to_string(),
                        "from_timestamp".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__ts".to_string()),
                        RustExpr::Literal(RustLiteral::Int(0)),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("dt".to_string())),
                            method: "format".to_string(),
                            args: vec![RustExpr::Literal(RustLiteral::Str(
                                "%Y-%m-%dT%H:%M:%S".to_string(),
                            ))],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_localtime(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__ts".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Ident(args[0].clone())),
                ty: RustType::I64,
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "DateTime::<Utc>".to_string(),
                        "from_timestamp".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("__ts".to_string()),
                        RustExpr::Literal(RustLiteral::Int(0)),
                    ],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "dt".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("dt".to_string())),
                                method: "with_timezone".to_string(),
                                args: vec![RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(RustExpr::Path(vec!["Local".to_string()])),
                                }],
                            }),
                            method: "format".to_string(),
                            args: vec![RustExpr::Literal(RustLiteral::Str(
                                "%Y-%m-%dT%H:%M:%S".to_string(),
                            ))],
                        }),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            }),
            method: "unwrap_or_default".to_string(),
            args: vec![],
        })),
    })
}

pub(super) fn lower_time_strptime_compat(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<i64>, ValueError> {{ let __s = {}; let __fmt = {}; chrono::NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| {{ use chrono::Datelike; use chrono::Timelike; vec![dt.year() as i64, dt.month() as i64, dt.day() as i64, dt.hour() as i64, dt.minute() as i64, dt.second() as i64, dt.weekday().num_days_from_monday() as i64, dt.ordinal() as i64] }}).map_err(|e| ValueError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_time_gmtime_compat(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike, Utc}; let __dt = Utc::now().naive_utc(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }".to_string(),
    ))
}

pub(super) fn lower_time_localtime_compat(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike, Local}; let __dt = Local::now().naive_local(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }".to_string(),
    ))
}
