//! Native process pipe intrinsic lowerers.

use crate::RustExpr;

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

pub(crate) fn lower_process_child_stdin(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_child_stdin"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_child_stdout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_child_stdout"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_child_stderr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_child_stderr"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_pipe_read_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_pipe_read_all"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_pipe_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_pipe_write_all"],
        vec![
            arg_expr(args, 0),
            RustExpr::Clone(Box::new(arg_expr(args, 1))),
        ],
    ))
}

pub(crate) fn lower_process_pipe_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_pipe_close"],
        vec![arg_expr(args, 0)],
    ))
}
