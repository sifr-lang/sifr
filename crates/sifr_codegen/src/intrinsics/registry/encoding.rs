//! Encoding intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn string_lit(value: &str) -> RustExpr {
    RustExpr::Literal(crate::RustLiteral::Str(value.to_string()))
}

fn runtime_call(func: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "encoding".to_string(),
            func.to_string(),
        ])),
        args,
    }
}

fn error_expr(error_name: &str, message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: error_name.to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn map_string_error(expr: RustExpr, error_name: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__message".to_string(),
                ty: RustType::Named("String".to_string()),
            }],
            body: Box::new(error_expr(
                error_name,
                RustExpr::Ident("__message".to_string()),
            )),
            is_move: false,
        }],
    }
}

fn map_decode_outcome(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__parts".to_string(),
                ty: RustType::Tuple(vec![
                    RustType::Named("String".to_string()),
                    RustType::Vec(Box::new(RustType::Named("String".to_string()))),
                ]),
            }],
            body: Box::new(RustExpr::StructInit {
                name: "DecodeOutcome".to_string(),
                fields: vec![
                    (
                        "text".to_string(),
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__parts".to_string())),
                            field: "0".to_string(),
                        },
                    ),
                    (
                        "recoveries".to_string(),
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__parts".to_string())),
                            field: "1".to_string(),
                        },
                    ),
                ],
            }),
            is_move: false,
        }],
    }
}

fn map_encode_outcome(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__parts".to_string(),
                ty: RustType::Tuple(vec![
                    RustType::Vec(Box::new(RustType::Named("u8".to_string()))),
                    RustType::Vec(Box::new(RustType::Named("String".to_string()))),
                ]),
            }],
            body: Box::new(RustExpr::StructInit {
                name: "EncodeOutcome".to_string(),
                fields: vec![
                    (
                        "data".to_string(),
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__parts".to_string())),
                            field: "0".to_string(),
                        },
                    ),
                    (
                        "recoveries".to_string(),
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__parts".to_string())),
                            field: "1".to_string(),
                        },
                    ),
                ],
            }),
            is_move: false,
        }],
    }
}

fn ref_arg(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

pub(crate) fn lower_encoding_is_supported(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call(
        "is_supported_encoding",
        vec![ref_arg(arg_expr(args, 0))],
    ))
}

pub(crate) fn lower_encoding_canonical_label(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(map_string_error(
        runtime_call("canonical_label", vec![ref_arg(arg_expr(args, 0))]),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_decode_text(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "decode_text",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_decode_recoveries(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "decode_recoveries",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_decode_outcome(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        map_decode_outcome(runtime_call(
            "decode_with_recoveries",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        )),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_decode_incremental_outcome(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    Some(map_string_error(
        map_decode_outcome(runtime_call(
            "incremental_decode_with_recoveries",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
                ref_arg(arg_expr(args, 3)),
                arg_expr(args, 4),
            ],
        )),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_decode_incremental_pending(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "incremental_decode_pending",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
                arg_expr(args, 3),
            ],
        ),
        "DecodeError",
    ))
}

pub(crate) fn lower_encoding_encode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "encode_bytes",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "EncodeError",
    ))
}

pub(crate) fn lower_encoding_encode_outcome(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        map_encode_outcome(runtime_call(
            "encode_with_recoveries",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        )),
        "EncodeError",
    ))
}

pub(crate) fn lower_encoding_encode_recoveries(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "encode_recoveries",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "EncodeError",
    ))
}

pub(crate) fn lower_str_encode_result(args: &[RustExpr]) -> Option<RustExpr> {
    let (text, encoding, errors) = match args {
        [text] => (text.clone(), string_lit("utf-8"), string_lit("strict")),
        [text, encoding] => (text.clone(), encoding.clone(), string_lit("strict")),
        [text, encoding, errors] => (text.clone(), encoding.clone(), errors.clone()),
        _ => return None,
    };
    Some(map_string_error(
        runtime_call(
            "encode_bytes",
            vec![ref_arg(text), ref_arg(encoding), ref_arg(errors)],
        ),
        "ParseError",
    ))
}

pub(crate) fn lower_bytes_decode_result(args: &[RustExpr]) -> Option<RustExpr> {
    let (data, encoding, errors) = match args {
        [data] => (data.clone(), string_lit("utf-8"), string_lit("strict")),
        [data, encoding] => (data.clone(), encoding.clone(), string_lit("strict")),
        [data, encoding, errors] => (data.clone(), encoding.clone(), errors.clone()),
        _ => return None,
    };
    Some(map_string_error(
        runtime_call(
            "decode_text",
            vec![ref_arg(data), ref_arg(encoding), ref_arg(errors)],
        ),
        "ParseError",
    ))
}
