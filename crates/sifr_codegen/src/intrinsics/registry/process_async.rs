//! Async native process intrinsic lowerers.

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

fn boxed_async_process_helper_call(name: &str, args: Vec<RustExpr>) -> RustExpr {
    path_call(&["Box", "pin"], vec![path_call(&[name], args)])
}

fn async_process_owned_args(args: &[RustExpr]) -> Vec<RustExpr> {
    vec![
        RustExpr::Clone(Box::new(arg_expr(args, 0))),
        RustExpr::Clone(Box::new(arg_expr(args, 1))),
        RustExpr::Clone(Box::new(arg_expr(args, 2))),
        RustExpr::Clone(Box::new(arg_expr(args, 3))),
        arg_expr(args, 4),
        RustExpr::Clone(Box::new(arg_expr(args, 5))),
    ]
}

fn string_lit(value: &str) -> RustExpr {
    RustExpr::Literal(crate::RustLiteral::Str(value.to_string()))
}

fn empty_string_vec() -> RustExpr {
    RustExpr::Vec(vec![])
}

fn async_shell_process_owned_args(script: RustExpr) -> Vec<RustExpr> {
    vec![
        string_lit("sh"),
        RustExpr::Vec(vec![string_lit("-c"), RustExpr::Clone(Box::new(script))]),
        empty_string_vec(),
        string_lit(""),
        RustExpr::Literal(crate::RustLiteral::Bool(false)),
        string_lit("inherit"),
    ]
}

pub(crate) fn lower_process_async_run(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 6 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_run",
        async_process_owned_args(args),
    ))
}

pub(crate) fn lower_process_async_shell_run(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_run",
        async_shell_process_owned_args(arg_expr(args, 0)),
    ))
}

pub(crate) fn lower_process_async_shell_output(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let mut owned_args = async_shell_process_owned_args(arg_expr(args, 0));
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 1))));
    owned_args.push(arg_expr(args, 2));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_output",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_shell_output_timeout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let mut owned_args = async_shell_process_owned_args(arg_expr(args, 0));
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 1))));
    owned_args.push(arg_expr(args, 2));
    owned_args.push(arg_expr(args, 3));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_output_timeout",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_run_timeout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 7 {
        return None;
    }
    let mut owned_args = async_process_owned_args(args);
    owned_args.push(arg_expr(args, 6));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_run_timeout",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_output(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    let mut owned_args = async_process_owned_args(args);
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 6))));
    owned_args.push(arg_expr(args, 7));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_output",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_output_timeout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 9 {
        return None;
    }
    let mut owned_args = async_process_owned_args(args);
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 6))));
    owned_args.push(arg_expr(args, 7));
    owned_args.push(arg_expr(args, 8));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_output_timeout",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_spawn(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 9 {
        return None;
    }
    let mut owned_args = async_process_owned_args(args);
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 6))));
    owned_args.push(RustExpr::Clone(Box::new(arg_expr(args, 7))));
    owned_args.push(arg_expr(args, 8));
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_spawn",
        owned_args,
    ))
}

pub(crate) fn lower_process_async_wait(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_wait",
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_handle_wait(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_handle_wait",
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_kill(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_kill",
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_terminate(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_terminate",
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_child_stdin(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_async_child_stdin"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_child_stdout(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_async_child_stdout"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_child_stderr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_async_child_stderr"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_pipe_read_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_pipe_read_all",
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_pipe_read(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_pipe_read",
        vec![arg_expr(args, 0), arg_expr(args, 1)],
    ))
}

pub(crate) fn lower_process_async_pipe_reader_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_async_pipe_reader_close"],
        vec![arg_expr(args, 0)],
    ))
}

pub(crate) fn lower_process_async_pipe_write_all(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(boxed_async_process_helper_call(
        "__sifr_process_async_pipe_write_all",
        vec![
            arg_expr(args, 0),
            RustExpr::Clone(Box::new(arg_expr(args, 1))),
        ],
    ))
}

pub(crate) fn lower_process_async_pipe_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(path_call(
        &["__sifr_process_async_pipe_close"],
        vec![arg_expr(args, 0)],
    ))
}
