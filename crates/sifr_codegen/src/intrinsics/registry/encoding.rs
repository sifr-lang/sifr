//! Encoding intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

fn string_lit(value: &str) -> RustExpr {
    RustExpr::Literal(crate::RustLiteral::StaticStr(value.to_string()))
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

fn ref_arg(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn string_view(expr: RustExpr) -> RustExpr {
    match expr {
        RustExpr::Literal(
            crate::RustLiteral::Str(value) | crate::RustLiteral::StaticStr(value),
        ) => RustExpr::Literal(crate::RustLiteral::StaticStr(value)),
        RustExpr::Clone(value) => string_view(*value),
        value @ RustExpr::Ref { .. } => value,
        value => ref_arg(value),
    }
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
            vec![
                string_view(text),
                string_view(encoding),
                string_view(errors),
            ],
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
            vec![ref_arg(data), string_view(encoding), string_view(errors)],
        ),
        "ParseError",
    ))
}
