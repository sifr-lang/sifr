//! Compiler-owned method surface for the sealed raw Python object.

use super::{FunctionType, LowerCtx, TextRange, Type};

pub(super) fn for_call(
    object_type: &Type,
    method: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<FunctionType> {
    let method_type = method_type(object_type, method, range, ctx);
    if method_type.is_some() {
        super::workload_annotations::reject_async_direct_raw_python_method(ctx, method, range);
    }
    method_type
}

pub(super) fn method_type(
    object_type: &Type,
    method: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<FunctionType> {
    if !object_type.is_python_object_contract() || !is_raw_method(method) {
        return None;
    }
    let Some(python_error) = ctx
        .class_types
        .get("PythonError")
        .filter(|ty| ty.is_python_error_contract())
        .cloned()
    else {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
            "raw Python Object methods require the canonical `PythonError` field contract; import `PythonError` from `sifr.python`".to_string(),
            range,
        );
        return None;
    };
    let object = object_type.clone();
    let args = Type::List(Box::new(object.clone()));
    let kwargs = Type::List(Box::new(Type::Tuple(vec![Type::Str, object.clone()])));
    let params = match method {
        "get_attr" => vec![("name".to_string(), Type::Str)],
        "get_item" => vec![("key".to_string(), Type::Str)],
        "call" => vec![("args".to_string(), args), ("kwargs".to_string(), kwargs)],
        "call_method" => vec![
            ("name".to_string(), Type::Str),
            ("args".to_string(), args),
            ("kwargs".to_string(), kwargs),
        ],
        _ => return None,
    };
    Some(FunctionType::all_borrow(
        params,
        Type::Result(Box::new(object), Box::new(python_error)),
    ))
}

pub(super) fn is_raw_method(method: &str) -> bool {
    matches!(method, "get_attr" | "get_item" | "call" | "call_method")
}
