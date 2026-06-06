//! Unicode intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_arg(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn runtime_call(func: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "unicode".to_string(),
            func.to_string(),
        ])),
        args,
    }
}

fn unicode_data_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "UnicodeDataError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn map_unicode_data_error(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__message".to_string(),
                ty: RustType::Named("String".to_string()),
            }],
            body: Box::new(unicode_data_error(RustExpr::Ident("__message".to_string()))),
            is_move: false,
        }],
    }
}

fn lower_result_unary(args: &[RustExpr], func: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(map_unicode_data_error(runtime_call(
        func,
        vec![ref_arg(arg_expr(args, 0))],
    )))
}

pub(crate) fn lower_unicode_data_version(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(runtime_call("data_version", vec![]))
}

pub(crate) fn lower_unicode_normalize(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(map_unicode_data_error(runtime_call(
        "normalize",
        vec![ref_arg(arg_expr(args, 0)), ref_arg(arg_expr(args, 1))],
    )))
}

pub(crate) fn lower_unicode_is_normalized(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(map_unicode_data_error(runtime_call(
        "is_normalized",
        vec![ref_arg(arg_expr(args, 0)), ref_arg(arg_expr(args, 1))],
    )))
}

pub(crate) fn lower_unicode_name(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "name")
}

pub(crate) fn lower_unicode_lookup(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "lookup")
}

pub(crate) fn lower_unicode_category(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "category")
}

pub(crate) fn lower_unicode_bidirectional(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "bidirectional")
}

pub(crate) fn lower_unicode_combining(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "combining")
}

pub(crate) fn lower_unicode_east_asian_width(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "east_asian_width")
}

pub(crate) fn lower_unicode_mirrored(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "mirrored")
}

pub(crate) fn lower_unicode_decomposition(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "decomposition")
}

pub(crate) fn lower_unicode_decimal(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "decimal")
}

pub(crate) fn lower_unicode_digit(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "digit")
}

pub(crate) fn lower_unicode_numeric_value(args: &[RustExpr]) -> Option<RustExpr> {
    lower_result_unary(args, "numeric_value")
}

pub(crate) fn lower_unicode_case_fold(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call("case_fold", vec![ref_arg(arg_expr(args, 0))]))
}

pub(crate) fn lower_unicode_graphemes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call("graphemes", vec![ref_arg(arg_expr(args, 0))]))
}

pub(crate) fn lower_unicode_grapheme_indices(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call(
        "grapheme_indices",
        vec![ref_arg(arg_expr(args, 0))],
    ))
}

pub(crate) fn lower_unicode_words(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call("words", vec![ref_arg(arg_expr(args, 0))]))
}

pub(crate) fn lower_unicode_word_boundaries(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(runtime_call(
        "word_boundaries",
        vec![ref_arg(arg_expr(args, 0))],
    ))
}
