use crate::{render_expr, RustExpr};

pub(crate) fn lower_python_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "py_local_callback_echo" => lower_py_local_callback_echo(args),
        "py_threadsafe_callback_echo" => lower_py_threadsafe_callback_echo(args),
        "py_close_callback" => lower_py_close_callback(args),
        "local_callback" => lower_py_callback(args, "local_callback", "LocalCallback", "local"),
        "threadsafe_callback" => lower_py_callback(
            args,
            "threadsafe_callback",
            "ThreadsafeCallback",
            "threadsafe",
        ),
        _ => None,
    }
}

fn map_python_error(expr: impl std::fmt::Display) -> RustExpr {
    RustExpr::Ident(format!(
        r#"({expr}).map_err(|__sifr_python_error| PythonError {{
            message: __sifr_python_error.message,
            kind: __sifr_python_error.kind,
            exception_type: __sifr_python_error.exception_type,
            traceback: __sifr_python_error.traceback,
            context: __sifr_python_error.context,
        }})"#
    ))
}

pub(crate) fn lower_py_local_callback_echo(args: &[RustExpr]) -> Option<RustExpr> {
    lower_callback_constructor(args, "local_callback_echo")
}

pub(crate) fn lower_py_threadsafe_callback_echo(args: &[RustExpr]) -> Option<RustExpr> {
    lower_callback_constructor(args, "threadsafe_callback_echo")
}

pub(crate) fn lower_py_close_callback(args: &[RustExpr]) -> Option<RustExpr> {
    lower_callback_conversion(args, "close_callback")
}

fn lower_py_callback(
    args: &[RustExpr],
    function: &str,
    class_name: &str,
    kind: &str,
) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let handler = render_expr(&args[0]);
    Some(map_python_error(format!(
        r#"sifr_runtime::python::{function}(move |__sifr_callback_arg| {{
            let __sifr_callback_object = Object {{
                _handle: __sifr_callback_arg.0,
                _token: __sifr_callback_arg.1,
            }};
            match {handler}(&__sifr_callback_object) {{
                Ok(__sifr_callback_result) => Ok((
                    __sifr_callback_result._handle,
                    __sifr_callback_result._token,
                )),
                Err(__sifr_callback_error) => Err(sifr_runtime::python::PythonError {{
                    message: __sifr_callback_error.message,
                    kind: __sifr_callback_error.kind,
                    exception_type: __sifr_callback_error.exception_type,
                    traceback: __sifr_callback_error.traceback,
                    context: __sifr_callback_error.context,
                }}),
            }}
        }})
        .map(|__sifr_python_callback| {{
            let mut __sifr_callback = {class_name}::new();
            __sifr_callback._handle = __sifr_python_callback.handle;
            __sifr_callback._token = __sifr_python_callback.token;
            __sifr_callback.callable = Object {{
                _handle: __sifr_python_callback.object_handle,
                _token: __sifr_python_callback.object_token,
            }};
            __sifr_callback.kind = "{kind}".to_string();
            __sifr_callback
        }})"#
    )))
}

fn lower_callback_constructor(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(map_python_error(format!(
        r#"sifr_runtime::python::{function}().map(|__sifr_python_callback| {{
            (
                __sifr_python_callback.handle,
                __sifr_python_callback.token,
                __sifr_python_callback.object_handle,
                __sifr_python_callback.object_token,
                __sifr_python_callback.kind,
            )
        }})"#
    )))
}

fn lower_callback_conversion(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::{function}(({handle}, {token}))"
    )))
}
