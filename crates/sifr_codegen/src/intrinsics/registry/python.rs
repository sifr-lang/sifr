use crate::{render_expr, RustExpr};

fn map_python_error(expr: String) -> RustExpr {
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

pub(crate) fn lower_py_import_module(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let name = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::import_module(({name}).as_str())"
    )))
}

pub(crate) fn lower_py_get_attr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let name = render_expr(&args[2]);
    Some(map_python_error(format!(
        "sifr_runtime::python::get_attr(({handle}, {token}), ({name}).as_str())"
    )))
}

pub(crate) fn lower_py_get_item_str(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let key = render_expr(&args[2]);
    Some(map_python_error(format!(
        "sifr_runtime::python::get_item_str(({handle}, {token}), ({key}).as_str())"
    )))
}

pub(crate) fn lower_py_call(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let positional = render_expr(&args[2]);
    let keyword = render_expr(&args[3]);
    Some(map_python_error(format!(
        r#"{{
            let __sifr_python_args = ({positional})
                .iter()
                .map(|__sifr_python_arg| (__sifr_python_arg._handle, __sifr_python_arg._token))
                .collect::<Vec<(i64, i64)>>();
            let __sifr_python_kwargs = ({keyword})
                .iter()
                .map(|__sifr_python_kwarg| (__sifr_python_kwarg.0.as_str(), (__sifr_python_kwarg.1._handle, __sifr_python_kwarg.1._token)))
                .collect::<Vec<(&str, (i64, i64))>>();
            sifr_runtime::python::call_object(({handle}, {token}), &__sifr_python_args, &__sifr_python_kwargs)
        }}"#
    )))
}

pub(crate) fn lower_py_call_attr(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let name = render_expr(&args[2]);
    let positional = render_expr(&args[3]);
    let keyword = render_expr(&args[4]);
    Some(map_python_error(format!(
        r#"{{
            let __sifr_python_args = ({positional})
                .iter()
                .map(|__sifr_python_arg| (__sifr_python_arg._handle, __sifr_python_arg._token))
                .collect::<Vec<(i64, i64)>>();
            let __sifr_python_kwargs = ({keyword})
                .iter()
                .map(|__sifr_python_kwarg| (__sifr_python_kwarg.0.as_str(), (__sifr_python_kwarg.1._handle, __sifr_python_kwarg.1._token)))
                .collect::<Vec<(&str, (i64, i64))>>();
            sifr_runtime::python::call_attr(({handle}, {token}), ({name}).as_str(), &__sifr_python_args, &__sifr_python_kwargs)
        }}"#
    )))
}

pub(crate) fn lower_py_close(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::close_object(({handle}, {token}))"
    )))
}

pub(crate) fn lower_py_enter_context(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::enter_context(({handle}, {token}))"
    )))
}

pub(crate) fn lower_py_exit_context(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::exit_context(({handle}, {token}))"
    )))
}
