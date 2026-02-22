//! Datetime intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustParam, RustType};

pub(super) fn lower_datetime_now(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    // chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "chrono".to_string(),
                    "Local".to_string(),
                    "now".to_string(),
                ])),
                args: vec![],
            }),
            method: "format".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Str(
                "%Y-%m-%dT%H:%M:%S".to_string(),
            ))],
        }),
        method: "to_string".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_datetime_now_struct(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "{ use chrono::{Datelike, Timelike}; let __dt = chrono::Local::now(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64] }".to_string(),
    ))
}

pub(super) fn lower_datetime_format(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    // chrono::NaiveDateTime::parse_from_str(&dt_str, &fmt).map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()).map_err(|e| ValueError { message: e.to_string() })
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "chrono".to_string(),
                        "NaiveDateTime".to_string(),
                        "parse_from_str".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident(args[0].clone())),
                        },
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident(args[1].clone())),
                        },
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
            method: "map_err".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "e".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::StructInit {
                    name: "ValueError".to_string(),
                    fields: vec![(
                        "message".to_string(),
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("e".to_string())),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                    )],
                }),
                is_move: false,
            }],
        }),
        method: "ok".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_datetime_from_timestamp(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use chrono::Utc; let __ts = {} as i64; chrono::DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt: chrono::DateTime<Utc>| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).ok_or_else(|| ValueError {{ message: \"invalid timestamp\".to_string() }}) }}",
        args[0]
    )))
}
