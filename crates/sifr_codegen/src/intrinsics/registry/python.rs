use crate::{render_expr, RustExpr};

pub(crate) fn lower_python_intrinsic(name: &str, args: &[RustExpr]) -> Option<RustExpr> {
    match name {
        "py_call" => lower_py_call(args),
        "py_call_attr" => lower_py_call_attr(args),
        "py_buffer_u8" => lower_py_buffer_u8(args),
        "py_copy_buffer_u8" => lower_py_copy_buffer_u8(args),
        "py_release_buffer" => lower_py_release_buffer(args),
        "py_arrow_array" => lower_py_arrow_array(args),
        "py_arrow_stream" => lower_py_arrow_stream(args),
        "py_arrow_schema" => lower_py_arrow_schema(args),
        "py_release_arrow" => lower_py_release_arrow(args),
        "py_dlpack_tensor" => lower_py_dlpack_tensor(args),
        "py_release_dlpack" => lower_py_release_dlpack(args),
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
        "py_enter_context" => lower_py_enter_context(args),
        "py_exit_context" => lower_py_exit_context(args),
        "py_exit_context_with_error" => lower_py_exit_context_with_error(args),
        "py_run_coroutine_blocking" => lower_py_run_coroutine_blocking(args),
        "py_from_list" => lower_py_from_list(args),
        "py_from_tuple" => lower_py_from_tuple(args),
        "py_from_dict_str" => lower_py_from_dict_str(args),
        "py_from_record" => lower_py_from_record(args),
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

fn object_expr(handle: &str, token: &str) -> String {
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
        object_expr(&handle, &token)
    )))
}

fn object_handles_expr(values: &str) -> String {
    format!(
        r#"({values})
                .iter()
                .map(|__sifr_python_value| (__sifr_python_value.0, __sifr_python_value.1))
                .collect::<Vec<(i64, i64)>>()"#
    )
}

fn keyed_object_handles_expr(values: &str) -> String {
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
        object_handles_expr(&values)
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
        keyed_object_handles_expr(&values)
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

pub(crate) fn lower_py_buffer_u8(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let require_writable = render_expr(&args[2]);
    Some(map_python_error(format!(
        r#"sifr_runtime::python::buffer_u8(({handle}, {token}), {require_writable}).map(|__sifr_python_buffer| {{
            (
                __sifr_python_buffer.handle,
                __sifr_python_buffer.token,
                __sifr_python_buffer.len_bytes,
                __sifr_python_buffer.item_size,
                __sifr_python_buffer.readonly,
                __sifr_python_buffer.dimensions,
                __sifr_python_buffer.shape,
                __sifr_python_buffer.strides,
                __sifr_python_buffer.suboffsets,
                __sifr_python_buffer.c_contiguous,
                __sifr_python_buffer.f_contiguous,
                __sifr_python_buffer.format,
            )
        }})"#
    )))
}

pub(crate) fn lower_py_copy_buffer_u8(args: &[RustExpr]) -> Option<RustExpr> {
    lower_buffer_conversion(args, "copy_buffer_u8")
}

pub(crate) fn lower_py_release_buffer(args: &[RustExpr]) -> Option<RustExpr> {
    lower_buffer_conversion(args, "release_buffer")
}

fn lower_buffer_conversion(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::{function}(({handle}, {token}))"
    )))
}

pub(crate) fn lower_py_arrow_array(args: &[RustExpr]) -> Option<RustExpr> {
    lower_arrow_export(args, "arrow_array")
}

pub(crate) fn lower_py_arrow_stream(args: &[RustExpr]) -> Option<RustExpr> {
    lower_arrow_export(args, "arrow_stream")
}

pub(crate) fn lower_py_arrow_schema(args: &[RustExpr]) -> Option<RustExpr> {
    lower_arrow_export(args, "arrow_schema")
}

pub(crate) fn lower_py_release_arrow(args: &[RustExpr]) -> Option<RustExpr> {
    lower_arrow_conversion(args, "release_arrow")
}

fn lower_arrow_export(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        r#"sifr_runtime::python::{function}(({handle}, {token})).map(|__sifr_python_arrow| {{
            (
                __sifr_python_arrow.handle,
                __sifr_python_arrow.token,
                __sifr_python_arrow.kind,
                __sifr_python_arrow.capsule_names,
                __sifr_python_arrow.producer_module,
                __sifr_python_arrow.producer_type,
                __sifr_python_arrow.copy_possible,
            )
        }})"#
    )))
}

fn lower_arrow_conversion(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::{function}(({handle}, {token}))"
    )))
}

pub(crate) fn lower_py_dlpack_tensor(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        r#"sifr_runtime::python::dlpack_tensor(({handle}, {token})).map(|__sifr_python_dlpack| {{
            (
                __sifr_python_dlpack.handle,
                __sifr_python_dlpack.token,
                __sifr_python_dlpack.dtype_code,
                __sifr_python_dlpack.dtype_bits,
                __sifr_python_dlpack.dtype_lanes,
                __sifr_python_dlpack.dtype,
                __sifr_python_dlpack.device_type,
                __sifr_python_dlpack.device_id,
                __sifr_python_dlpack.dimensions,
                __sifr_python_dlpack.shape,
                __sifr_python_dlpack.strides,
                __sifr_python_dlpack.byte_offset,
                __sifr_python_dlpack.has_deleter,
                __sifr_python_dlpack.stream_sync_required,
            )
        }})"#
    )))
}

pub(crate) fn lower_py_release_dlpack(args: &[RustExpr]) -> Option<RustExpr> {
    lower_dlpack_conversion(args, "release_dlpack")
}

fn lower_dlpack_conversion(args: &[RustExpr], function: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    Some(map_python_error(format!(
        "sifr_runtime::python::{function}(({handle}, {token}))"
    )))
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

pub(crate) fn lower_py_exit_context_with_error(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 7 {
        return None;
    }
    let handle = render_expr(&args[0]);
    let token = render_expr(&args[1]);
    let kind = render_expr(&args[2]);
    let exception_type = render_expr(&args[3]);
    let message = render_expr(&args[4]);
    let traceback = render_expr(&args[5]);
    let context = render_expr(&args[6]);
    Some(map_python_error(format!(
        r#"sifr_runtime::python::exit_context_with_error(
            ({handle}, {token}),
            ({kind}).as_str(),
            ({exception_type}).as_str(),
            ({message}).as_str(),
            ({traceback}).as_str(),
            ({context}).as_str(),
        )"#
    )))
}

pub(crate) fn lower_py_run_coroutine_blocking(args: &[RustExpr]) -> Option<RustExpr> {
    lower_handle_conversion(args, "run_coroutine_blocking")
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
