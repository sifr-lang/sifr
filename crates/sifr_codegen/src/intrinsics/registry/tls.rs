//! Async TLS intrinsic lowerers.

use crate::{RustExpr, RustStmt, RustType};
use sifr_stdlib_manifest::StdlibFeature;

pub(crate) const TLS_REQUIRED_FEATURES: &[StdlibFeature] = &[
    StdlibFeature::SifrRuntime,
    StdlibFeature::TokioRustls,
    StdlibFeature::Rustls,
    StdlibFeature::RustlsPemfile,
    StdlibFeature::RustlsPlatformVerifier,
    StdlibFeature::Tracing,
];

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn path_call(parts: &[&str], args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(
            parts.iter().map(|part| (*part).to_string()).collect(),
        )),
        args,
    }
}

fn send_awaitable_type(output: &str) -> RustType {
    RustType::Named(format!(
        "std::pin::Pin<Box<dyn std::future::Future<Output = {output}> + Send>>"
    ))
}

fn boxed_async_tls_helper_call(name: &str, args: Vec<RustExpr>, output: &str) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_tls_future".to_string(),
            ty: Some(send_awaitable_type(output)),
            value: path_call(&["Box", "pin"], vec![path_call(&[name], args)]),
        }],
        expr: Some(Box::new(RustExpr::Ident("__sifr_tls_future".to_string()))),
    }
}

fn cloned_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::Clone(Box::new(arg_expr(args, idx)))
}

pub(crate) fn lower_tls_client_config_platform(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            &["__sifr_tls_client_config_platform"],
            vec![cloned_arg(args, 0)],
        )
    })
}

pub(crate) fn lower_tls_client_config_with_roots(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        path_call(
            &["__sifr_tls_client_config_with_roots"],
            vec![cloned_arg(args, 0), cloned_arg(args, 1)],
        )
    })
}

pub(crate) fn lower_tls_client_config_with_roots_and_client_auth(
    args: &[RustExpr],
) -> Option<RustExpr> {
    (args.len() == 4).then(|| {
        path_call(
            &["__sifr_tls_client_config_with_roots_and_client_auth"],
            vec![
                cloned_arg(args, 0),
                cloned_arg(args, 1),
                cloned_arg(args, 2),
                cloned_arg(args, 3),
            ],
        )
    })
}

pub(crate) fn lower_tls_server_config(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 3).then(|| {
        path_call(
            &["__sifr_tls_server_config"],
            vec![
                cloned_arg(args, 0),
                cloned_arg(args, 1),
                cloned_arg(args, 2),
            ],
        )
    })
}

pub(crate) fn lower_tls_server_config_require_client_auth(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 4).then(|| {
        path_call(
            &["__sifr_tls_server_config_require_client_auth"],
            vec![
                cloned_arg(args, 0),
                cloned_arg(args, 1),
                cloned_arg(args, 2),
                cloned_arg(args, 3),
            ],
        )
    })
}

pub(crate) fn lower_tls_client_config_close(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call(&["__sifr_tls_client_config_close"], vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_tls_server_config_close(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1)
        .then(|| path_call(&["__sifr_tls_server_config_close"], vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_tls_connect(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 5).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_connect",
            vec![
                arg_expr(args, 0),
                arg_expr(args, 1),
                cloned_arg(args, 2),
                arg_expr(args, 3),
                arg_expr(args, 4),
            ],
            "Result<TlsStream, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_accept(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 4).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_accept",
            vec![
                arg_expr(args, 0),
                arg_expr(args, 1),
                arg_expr(args, 2),
                arg_expr(args, 3),
            ],
            "Result<TlsStream, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_read_chunk(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_read_chunk",
            vec![arg_expr(args, 0), arg_expr(args, 1)],
            "Result<Option<Vec<u8>>, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_write(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_write",
            vec![arg_expr(args, 0), cloned_arg(args, 1)],
            "Result<i64, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_write_all",
            vec![arg_expr(args, 0), cloned_arg(args, 1)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_flush(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_flush",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_close_notify(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_close_notify",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_close(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_stream_close",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_stream_split(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call(&["__sifr_tls_stream_split"], vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_tls_stream_alpn_protocol(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            &["__sifr_tls_stream_alpn_protocol"],
            vec![arg_expr(args, 0)],
        )
    })
}

pub(crate) fn lower_tls_stream_protocol_version(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        path_call(
            &["__sifr_tls_stream_protocol_version"],
            vec![arg_expr(args, 0)],
        )
    })
}

pub(crate) fn lower_tls_read_half_read_chunk(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_read_half_read_chunk",
            vec![arg_expr(args, 0), arg_expr(args, 1)],
            "Result<Option<Vec<u8>>, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_read_half_close(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| path_call(&["__sifr_tls_read_half_close"], vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_tls_write_half_write(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_write_half_write",
            vec![arg_expr(args, 0), cloned_arg(args, 1)],
            "Result<i64, TlsError>",
        )
    })
}

pub(crate) fn lower_tls_write_half_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 2).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_write_half_write_all",
            vec![arg_expr(args, 0), cloned_arg(args, 1)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_write_half_flush(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_write_half_flush",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_write_half_close_notify(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_write_half_close_notify",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_write_half_close(args: &[RustExpr]) -> Option<RustExpr> {
    (args.len() == 1).then(|| {
        boxed_async_tls_helper_call(
            "__sifr_tls_write_half_close",
            vec![arg_expr(args, 0)],
            "Result<(), TlsError>",
        )
    })
}

pub(crate) fn lower_tls_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "tls_client_config_platform" => lower_tls_client_config_platform(args),
        "tls_client_config_with_roots" => lower_tls_client_config_with_roots(args),
        "tls_client_config_with_roots_and_client_auth" => {
            lower_tls_client_config_with_roots_and_client_auth(args)
        }
        "tls_server_config" => lower_tls_server_config(args),
        "tls_server_config_require_client_auth" => {
            lower_tls_server_config_require_client_auth(args)
        }
        "tls_client_config_close" => lower_tls_client_config_close(args),
        "tls_server_config_close" => lower_tls_server_config_close(args),
        "tls_connect" => lower_tls_connect(args),
        "tls_accept" => lower_tls_accept(args),
        "tls_stream_read_chunk" => lower_tls_stream_read_chunk(args),
        "tls_stream_write" => lower_tls_stream_write(args),
        "tls_stream_write_all" => lower_tls_stream_write_all(args),
        "tls_stream_flush" => lower_tls_stream_flush(args),
        "tls_stream_close_notify" => lower_tls_stream_close_notify(args),
        "tls_stream_close" => lower_tls_stream_close(args),
        "tls_stream_split" => lower_tls_stream_split(args),
        "tls_stream_alpn_protocol" => lower_tls_stream_alpn_protocol(args),
        "tls_stream_protocol_version" => lower_tls_stream_protocol_version(args),
        "tls_read_half_read_chunk" => lower_tls_read_half_read_chunk(args),
        "tls_read_half_close" => lower_tls_read_half_close(args),
        "tls_write_half_write" => lower_tls_write_half_write(args),
        "tls_write_half_write_all" => lower_tls_write_half_write_all(args),
        "tls_write_half_flush" => lower_tls_write_half_flush(args),
        "tls_write_half_close_notify" => lower_tls_write_half_close_notify(args),
        "tls_write_half_close" => lower_tls_write_half_close(args),
        _ => None,
    }
}
