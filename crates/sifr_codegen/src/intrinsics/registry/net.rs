//! Async TCP network intrinsic lowerers.

use crate::{RustExpr, RustStmt, RustType};

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

fn boxed_async_net_helper_call(name: &str, args: Vec<RustExpr>, output: &str) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_net_future".to_string(),
            ty: Some(send_awaitable_type(output)),
            value: path_call(&["Box", "pin"], vec![path_call(&[name], args)]),
        }],
        expr: Some(Box::new(RustExpr::Ident("__sifr_net_future".to_string()))),
    }
}

fn cloned_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::Clone(Box::new(arg_expr(args, idx)))
}

pub(crate) fn lower_net_connect_tcp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_connect_tcp",
        vec![
            cloned_arg(args, 0),
            arg_expr(args, 1),
            arg_expr(args, 2),
            cloned_arg(args, 3),
            arg_expr(args, 4),
        ],
        "Result<TcpStream, NetError>",
    ))
}

pub(crate) fn lower_net_listen_tcp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_listen_tcp",
        vec![
            cloned_arg(args, 0),
            arg_expr(args, 1),
            arg_expr(args, 2),
            arg_expr(args, 3),
        ],
        "Result<TcpListener, NetError>",
    ))
}

pub(crate) fn lower_net_lookup_host(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_lookup_host",
        vec![cloned_arg(args, 0), arg_expr(args, 1), arg_expr(args, 2)],
        "Result<Vec<SocketAddr>, NetError>",
    ))
}

pub(crate) fn lower_net_listener_accept(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_listener_accept",
        vec![arg_expr(args, 0)],
        "Result<(TcpStream, SocketAddr), NetError>",
    ))
}

pub(crate) fn lower_net_listener_local_addr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_listener_local_addr"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_listener_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_listener_close"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_tcp_stream_read_chunk(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_stream_read_chunk",
        vec![arg_expr(args, 0), arg_expr(args, 1)],
        "Result<Option<Vec<u8>>, NetError>",
    ))
}

pub(crate) fn lower_net_tcp_stream_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_stream_write",
        vec![arg_expr(args, 0), cloned_arg(args, 1)],
        "Result<i64, NetError>",
    ))
}

pub(crate) fn lower_net_tcp_stream_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_stream_write_all",
        vec![arg_expr(args, 0), cloned_arg(args, 1)],
        "Result<(), NetError>",
    ))
}

pub(crate) fn lower_net_tcp_stream_shutdown_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_stream_shutdown_write",
        vec![arg_expr(args, 0)],
        "Result<(), NetError>",
    ))
}

pub(crate) fn lower_net_tcp_stream_split(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_tcp_stream_split"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_tcp_stream_local_addr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_tcp_stream_local_addr"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_tcp_stream_peer_addr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_tcp_stream_peer_addr"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_tcp_stream_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_stream_close",
        vec![arg_expr(args, 0)],
        "Result<(), NetError>",
    ))
}

pub(crate) fn lower_net_tcp_read_half_read_chunk(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_read_half_read_chunk",
        vec![arg_expr(args, 0), arg_expr(args, 1)],
        "Result<Option<Vec<u8>>, NetError>",
    ))
}

pub(crate) fn lower_net_tcp_read_half_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_tcp_read_half_close"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_net_tcp_write_half_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_write_half_write",
        vec![arg_expr(args, 0), cloned_arg(args, 1)],
        "Result<i64, NetError>",
    ))
}

pub(crate) fn lower_net_tcp_write_half_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_write_half_write_all",
        vec![arg_expr(args, 0), cloned_arg(args, 1)],
        "Result<(), NetError>",
    ))
}

pub(crate) fn lower_net_tcp_write_half_shutdown_write(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_net_helper_call(
        "__sifr_net_tcp_write_half_shutdown_write",
        vec![arg_expr(args, 0)],
        "Result<(), NetError>",
    ))
}

pub(crate) fn lower_net_tcp_write_half_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_net_tcp_write_half_close"],
        vec![arg_expr(args, 0)],
    ))
}
