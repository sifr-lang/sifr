//! URL and HTTP primitive intrinsic lowerers.

use crate::RustExpr;

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn path_call(name: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![name.to_string()])),
        args,
    }
}

fn cloned_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::Clone(Box::new(arg_expr(args, idx)))
}

pub(crate) fn lower_http_validate_header_name(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            "__sifr_http_validate_header_name",
            vec![cloned_arg(args, 0)],
        )
    })
}

pub(crate) fn lower_http_validate_header_value(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            "__sifr_http_validate_header_value",
            vec![cloned_arg(args, 0)],
        )
    })
}

pub(crate) fn lower_http_header_map_from_pairs(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            "__sifr_http_header_map_from_pairs",
            vec![cloned_arg(args, 0)],
        )
    })
}

pub(crate) fn lower_http_parse_cookie_header(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call("__sifr_http_parse_cookie_header", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_http_build_cookie_header(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call("__sifr_http_build_cookie_header", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_http_validate_method(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_method", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_http_validate_status(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_status", vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_http_validate_version(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_version", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_http_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "http_validate_method" => lower_http_validate_method(args),
        "http_validate_status" => lower_http_validate_status(args),
        "http_validate_version" => lower_http_validate_version(args),
        "http_validate_header_name" => lower_http_validate_header_name(args),
        "http_validate_header_value" => lower_http_validate_header_value(args),
        "http_header_map_from_pairs" => lower_http_header_map_from_pairs(args),
        "http_parse_cookie_header" => lower_http_parse_cookie_header(args),
        "http_build_cookie_header" => lower_http_build_cookie_header(args),
        _ => None,
    }
}
