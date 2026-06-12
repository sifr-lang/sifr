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

pub(crate) fn lower_url_parse(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_parse", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_build(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 5).then(|| {
        path_call(
            "__sifr_url_build",
            vec![
                cloned_arg(args, 0),
                cloned_arg(args, 1),
                cloned_arg(args, 2),
                cloned_arg(args, 3),
                arg_expr(args, 4),
            ],
        )
    })
}

pub(crate) fn lower_url_percent_encode(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_percent_encode", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_percent_decode(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_percent_decode", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_percent_encode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call("__sifr_url_percent_encode_bytes", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_percent_decode_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call("__sifr_url_percent_decode_bytes", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_normalize_path(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_normalize_path", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_query_parse(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_query_parse", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_url_query_build(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_url_query_build", vec![cloned_arg(args, 0)]))
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

pub(crate) fn lower_url_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "url_parse" => lower_url_parse(args),
        "url_build" => lower_url_build(args),
        "url_percent_encode" => lower_url_percent_encode(args),
        "url_percent_decode" => lower_url_percent_decode(args),
        "url_percent_encode_bytes" => lower_url_percent_encode_bytes(args),
        "url_percent_decode_bytes" => lower_url_percent_decode_bytes(args),
        "url_normalize_path" => lower_url_normalize_path(args),
        "url_query_parse" => lower_url_query_parse(args),
        "url_query_build" => lower_url_query_build(args),
        _ => None,
    }
}

pub(crate) fn lower_http_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "http_validate_header_name" => lower_http_validate_header_name(args),
        "http_validate_header_value" => lower_http_validate_header_value(args),
        "http_header_map_from_pairs" => lower_http_header_map_from_pairs(args),
        "http_parse_cookie_header" => lower_http_parse_cookie_header(args),
        "http_build_cookie_header" => lower_http_build_cookie_header(args),
        _ => None,
    }
}
