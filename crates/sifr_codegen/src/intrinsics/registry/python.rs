use crate::{render_expr, RustExpr};

pub(crate) fn lower_python_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "py_import_module" => lower_py_import_module(args),
        "py_get_attr" => lower_py_get_attr(args),
        "py_get_item_str" => lower_py_get_item_str(args),
        "py_call" => lower_py_call(args),
        "py_call_attr" => lower_py_call_attr(args),
        "py_close" => lower_py_close(args),
        "py_enter_context" => lower_py_enter_context(args),
        "py_exit_context" => lower_py_exit_context(args),
        "py_run_coroutine_blocking" => lower_py_run_coroutine_blocking(args),
        "py_from_none" => lower_py_from_none(args),
        "py_from_bool" => lower_py_from_bool(args),
        "py_from_int" => lower_py_from_int(args),
        "py_from_float" => lower_py_from_float(args),
        "py_from_str" => lower_py_from_str(args),
        "py_from_bytes" => lower_py_from_bytes(args),
        "py_from_list" => lower_py_from_list(args),
        "py_from_tuple" => lower_py_from_tuple(args),
        "py_from_dict_str" => lower_py_from_dict_str(args),
        "py_from_record" => lower_py_from_record(args),
        "py_to_none" => lower_py_to_none(args),
        "py_to_bool" => lower_py_to_bool(args),
        "py_to_int" => lower_py_to_int(args),
        "py_to_i8" => lower_py_to_i8(args),
        "py_to_i16" => lower_py_to_i16(args),
        "py_to_i32" => lower_py_to_i32(args),
        "py_to_i64" => lower_py_to_i64(args),
        "py_to_u8" => lower_py_to_u8(args),
        "py_to_u16" => lower_py_to_u16(args),
        "py_to_u32" => lower_py_to_u32(args),
        "py_to_u64" => lower_py_to_u64(args),
        "py_to_isize" => lower_py_to_isize(args),
        "py_to_usize" => lower_py_to_usize(args),
        "py_to_float" => lower_py_to_float(args),
        "py_to_str" => lower_py_to_str(args),
        "py_to_bytes" => lower_py_to_bytes(args),
        "py_copy_list_bool" => lower_handle_conversion(args, "copy_list_bool"),
        "py_copy_list_int" => lower_handle_conversion(args, "copy_list_int"),
        "py_copy_list_i32" => lower_handle_conversion(args, "copy_list_i32"),
        "py_copy_list_u8" => lower_handle_conversion(args, "copy_list_u8"),
        "py_copy_list_float" => lower_handle_conversion(args, "copy_list_float"),
        "py_copy_list_str" => lower_handle_conversion(args, "copy_list_str"),
        "py_copy_list_bytes" => lower_handle_conversion(args, "copy_list_bytes"),
        "py_copy_tuple_bool" => lower_handle_conversion(args, "copy_tuple_bool"),
        "py_copy_tuple_int" => lower_handle_conversion(args, "copy_tuple_int"),
        "py_copy_tuple_i32" => lower_handle_conversion(args, "copy_tuple_i32"),
        "py_copy_tuple_u8" => lower_handle_conversion(args, "copy_tuple_u8"),
        "py_copy_tuple_float" => lower_handle_conversion(args, "copy_tuple_float"),
        "py_copy_tuple_str" => lower_handle_conversion(args, "copy_tuple_str"),
        "py_copy_tuple_bytes" => lower_handle_conversion(args, "copy_tuple_bytes"),
        "py_copy_dict_str_bool" => lower_handle_conversion(args, "copy_dict_str_bool"),
        "py_copy_dict_str_int" => lower_handle_conversion(args, "copy_dict_str_int"),
        "py_copy_dict_str_i32" => lower_handle_conversion(args, "copy_dict_str_i32"),
        "py_copy_dict_str_u8" => lower_handle_conversion(args, "copy_dict_str_u8"),
        "py_copy_dict_str_float" => lower_handle_conversion(args, "copy_dict_str_float"),
        "py_copy_dict_str_str" => lower_handle_conversion(args, "copy_dict_str_str"),
        "py_copy_dict_str_bytes" => lower_handle_conversion(args, "copy_dict_str_bytes"),
        "py_copy_record_fields" => lower_py_copy_record_fields(args),
        _ => None,
    }
}

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

fn object_expr(handle: String, token: String) -> String {
    format!("({handle}, {token})")
}

fn lower_handle_conversion(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::{function}({})",
        object_expr(handle, token)
    )))
}

fn object_handles_expr(values: String) -> String {
    format!(
        r#"({values})
                .iter()
                .map(|__sifr_python_value| (__sifr_python_value.0, __sifr_python_value.1))
                .collect::<Vec<(i64, i64)>>()"#
    )
}

fn keyed_object_handles_expr(values: String) -> String {
    format!(
        r#"({values})
                .iter()
                .map(|__sifr_python_value| (__sifr_python_value.0.as_str(), (__sifr_python_value.1.0, __sifr_python_value.1.1)))
                .collect::<Vec<(&str, (i64, i64))>>()"#
    )
}

fn lower_object_list_constructor(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let values = render_expr(&args[0]);
    Some(map_python_error(format!(
        r#"{{
            let __sifr_python_values = {};
            sifr_runtime::python::{function}(&__sifr_python_values)
        }}"#,
        object_handles_expr(values)
    )))
}

fn lower_keyed_object_constructor(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let values = render_expr(&args[0]);
    Some(map_python_error(format!(
        r#"{{
            let __sifr_python_values = {};
            sifr_runtime::python::{function}(&__sifr_python_values)
        }}"#,
        keyed_object_handles_expr(values)
    )))
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
                .map(|__sifr_python_arg| (__sifr_python_arg.0, __sifr_python_arg.1))
                .collect::<Vec<(i64, i64)>>();
            let __sifr_python_kwargs = ({keyword})
                .iter()
                .map(|__sifr_python_kwarg| (__sifr_python_kwarg.0.as_str(), (__sifr_python_kwarg.1.0, __sifr_python_kwarg.1.1)))
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
                .map(|__sifr_python_arg| (__sifr_python_arg.0, __sifr_python_arg.1))
                .collect::<Vec<(i64, i64)>>();
            let __sifr_python_kwargs = ({keyword})
                .iter()
                .map(|__sifr_python_kwarg| (__sifr_python_kwarg.0.as_str(), (__sifr_python_kwarg.1.0, __sifr_python_kwarg.1.1)))
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

pub(crate) fn lower_py_run_coroutine_blocking(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "run_coroutine_blocking")
}

pub(crate) fn lower_py_from_none(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(map_python_error(
        "sifr_runtime::python::from_none()".to_string(),
    ))
}

pub(crate) fn lower_py_from_bool(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let value = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::from_bool({value})"
    )))
}

pub(crate) fn lower_py_from_int(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let value = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::from_int({value})"
    )))
}

pub(crate) fn lower_py_from_float(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let value = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::from_float({value})"
    )))
}

pub(crate) fn lower_py_from_str(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let value = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::from_str(({value}).as_str())"
    )))
}

pub(crate) fn lower_py_from_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let value = render_expr(&args[0]);
    Some(map_python_error(format!(
        "sifr_runtime::python::from_bytes(({value}).as_slice())"
    )))
}

pub(crate) fn lower_py_from_list(args: &[RustExpr]) -> Option<RustExpr> {
    lower_object_list_constructor(args, "from_list")
}

pub(crate) fn lower_py_from_tuple(args: &[RustExpr]) -> Option<RustExpr> {
    lower_object_list_constructor(args, "from_tuple")
}

pub(crate) fn lower_py_from_dict_str(args: &[RustExpr]) -> Option<RustExpr> {
    lower_keyed_object_constructor(args, "from_dict_str")
}

pub(crate) fn lower_py_from_record(args: &[RustExpr]) -> Option<RustExpr> {
    lower_keyed_object_constructor(args, "from_record")
}

pub(crate) fn lower_py_to_none(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_none")
}

pub(crate) fn lower_py_to_bool(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_bool")
}

pub(crate) fn lower_py_to_int(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_int")
}

pub(crate) fn lower_py_to_i8(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_i8")
}

pub(crate) fn lower_py_to_i16(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_i16")
}

pub(crate) fn lower_py_to_i32(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_i32")
}

pub(crate) fn lower_py_to_i64(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_i64")
}

pub(crate) fn lower_py_to_u8(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_u8")
}

pub(crate) fn lower_py_to_u16(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_u16")
}

pub(crate) fn lower_py_to_u32(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_u32")
}

pub(crate) fn lower_py_to_u64(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_u64")
}

pub(crate) fn lower_py_to_isize(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_isize")
}

pub(crate) fn lower_py_to_usize(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_usize")
}

pub(crate) fn lower_py_to_float(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_float")
}

pub(crate) fn lower_py_to_str(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_str")
}

pub(crate) fn lower_py_to_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "to_bytes")
}

pub(crate) fn lower_py_copy_record_fields(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let fields = render_expr(&args[2]);
    Some(map_python_error(format!(
        r#"{{
            let __sifr_python_fields = ({fields})
                .iter()
                .map(|__sifr_python_field| __sifr_python_field.as_str())
                .collect::<Vec<&str>>();
            sifr_runtime::python::copy_record_fields(({handle}, {token}), &__sifr_python_fields)
        }}"#
    )))
}
