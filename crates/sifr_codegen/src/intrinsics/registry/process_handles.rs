//! Native process child/pipe handle intrinsic shims.

use crate::RustExpr;

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn clone_arg(args: &[RustExpr], idx: usize) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(arg_expr(args, idx)),
        method: "clone".to_string(),
        args: vec![],
    }
}

fn path_call(parts: &[&str], args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(
            parts.iter().map(|part| (*part).to_string()).collect(),
        )),
        args,
    }
}

pub(crate) fn lower_process_spawn(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_spawn"],
        vec![
            clone_arg(args, 0),
            clone_arg(args, 1),
            clone_arg(args, 2),
            clone_arg(args, 3),
            arg_expr(args, 4),
            clone_arg(args, 5),
            clone_arg(args, 6),
            clone_arg(args, 7),
        ],
    ))
}

fn lower_one_arg_call(args: &[RustExpr], function_name: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(&[function_name], vec![arg_expr(args, 0)]))
}

pub(crate) fn lower_process_child_stdout(args: &[RustExpr]) -> Option<RustExpr> {
    lower_one_arg_call(args, "__sifr_process_child_stdout")
}

pub(crate) fn lower_process_child_stderr(args: &[RustExpr]) -> Option<RustExpr> {
    lower_one_arg_call(args, "__sifr_process_child_stderr")
}

pub(crate) fn lower_process_child_stdin(args: &[RustExpr]) -> Option<RustExpr> {
    lower_one_arg_call(args, "__sifr_process_child_stdin")
}

pub(crate) fn lower_process_pipe_read_all(args: &[RustExpr]) -> Option<RustExpr> {
    lower_one_arg_call(args, "__sifr_process_pipe_read_all")
}

pub(crate) fn lower_process_pipe_close(args: &[RustExpr]) -> Option<RustExpr> {
    lower_one_arg_call(args, "__sifr_process_pipe_close")
}

pub(crate) fn lower_process_pipe_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_pipe_write_all"],
        vec![arg_expr(args, 0), clone_arg(args, 1)],
    ))
}
