//! URL and HTTP primitive intrinsic lowerers.

use crate::{RustExpr, RustStmt, RustType};

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

fn send_awaitable_type(output: &str) -> RustType {
    RustType::Named(format!(
        "std::pin::Pin<Box<dyn std::future::Future<Output = {output}> + Send>>"
    ))
}

fn boxed_async_http_helper_call(name: &str, args: Vec<RustExpr>, output: &str) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_http_future".to_string(),
            ty: Some(send_awaitable_type(output)),
            value: path_call("Box::pin", vec![path_call(name, args)]),
        }],
        expr: Some(Box::new(RustExpr::Ident("__sifr_http_future".to_string()))),
    }
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

pub(crate) fn lower_http_validate_method(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_method", vec![cloned_arg(args, 0)]))
}

pub(crate) fn lower_http_validate_status(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_status", vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_http_validate_version(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call("__sifr_http_validate_version", vec![cloned_arg(args, 0)]))
}

fn lower_http_client_roundtrip(helper: &str, args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 7).then(|| {
        boxed_async_http_helper_call(
            helper,
            vec![
                arg_expr(args, 0),
                cloned_arg(args, 1),
                cloned_arg(args, 2),
                cloned_arg(args, 3),
                cloned_arg(args, 4),
                arg_expr(args, 5),
                arg_expr(args, 6),
            ],
            "Result<(i64, String, Vec<(String, String)>, Vec<u8>), HttpError>",
        )
    })
}

fn lower_http_server_respond(helper: &str, args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 6).then(|| {
        boxed_async_http_helper_call(
            helper,
            vec![
                arg_expr(args, 0),
                arg_expr(args, 1),
                cloned_arg(args, 2),
                cloned_arg(args, 3),
                arg_expr(args, 4),
                arg_expr(args, 5),
            ],
            "Result<(String, String, String, Vec<(String, String)>, Vec<u8>), HttpError>",
        )
    })
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
        "http_validate_method" => lower_http_validate_method(args),
        "http_validate_status" => lower_http_validate_status(args),
        "http_validate_version" => lower_http_validate_version(args),
        "http_validate_header_name" => lower_http_validate_header_name(args),
        "http_validate_header_value" => lower_http_validate_header_value(args),
        "http_header_map_from_pairs" => lower_http_header_map_from_pairs(args),
        "http_parse_cookie_header" => lower_http_parse_cookie_header(args),
        "http_build_cookie_header" => lower_http_build_cookie_header(args),
        "http1_client_roundtrip_tcp" => {
            lower_http_client_roundtrip("__sifr_http1_client_roundtrip_tcp", args)
        }
        "http2_client_roundtrip_tcp" => {
            lower_http_client_roundtrip("__sifr_http2_client_roundtrip_tcp", args)
        }
        "http1_client_roundtrip_tls" => {
            lower_http_client_roundtrip("__sifr_http1_client_roundtrip_tls", args)
        }
        "http2_client_roundtrip_tls" => {
            lower_http_client_roundtrip("__sifr_http2_client_roundtrip_tls", args)
        }
        "http1_server_respond_tcp" => {
            lower_http_server_respond("__sifr_http1_server_respond_tcp", args)
        }
        "http2_server_respond_tcp" => {
            lower_http_server_respond("__sifr_http2_server_respond_tcp", args)
        }
        "http1_server_respond_tls" => {
            lower_http_server_respond("__sifr_http1_server_respond_tls", args)
        }
        "http2_server_respond_tls" => {
            lower_http_server_respond("__sifr_http2_server_respond_tls", args)
        }
        _ => None,
    }
}
