use crate::RustExpr;
use crate::rust_interop_callback::{
    call_scoped_callback_adapter_expr, call_scoped_callbacks, threadsafe_callback_adapter_expr,
};
use crate::rust_interop_direct::{i64_vec_to_bridge_int_vec_expr, sifr_int_bridge_path};
use crate::rust_interop_direct_collections::{
    argument_composite_conversion_required, sifr_composite_to_bridge_expr,
};
use sifr_ir::{HirFunction, HirParam, RustTargetPath};
use sifr_type_system::Type;

pub(crate) fn direct_rust_arg_expr(
    param: &HirParam,
    target: &RustTargetPath,
    func: &HirFunction,
) -> RustExpr {
    let value = RustExpr::Ident(param.name.clone());
    if is_python_callback_constructor_target(target) && is_python_object_callback_type(&param.ty) {
        python_object_callback_adapter_expr(&param.name)
    } else if call_scoped_callbacks(func) && matches!(param.ty.resolve_alias(), Type::Callable(..))
    {
        call_scoped_callback_adapter_expr(param)
    } else if matches!(param.ty.resolve_alias(), Type::Callable(..)) {
        threadsafe_callback_adapter_expr(param, func)
    } else if param.ty == Type::Int {
        RustExpr::FnCall {
            func: Box::new(sifr_int_bridge_path("from")),
            args: vec![value],
        }
    } else if is_int_list(&param.ty) {
        let bridged = i64_vec_to_bridge_int_vec_expr(value, param.convention.is_borrowed());
        if param.convention.is_borrowed() {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(bridged),
            }
        } else {
            bridged
        }
    } else if is_optional_str(&param.ty) {
        RustExpr::MethodCall {
            receiver: Box::new(value),
            method: "clone".to_string(),
            args: Vec::new(),
        }
    } else if is_optional_int(&param.ty) {
        RustExpr::MethodCall {
            receiver: Box::new(value),
            method: "map".to_string(),
            args: vec![sifr_int_bridge_path("from")],
        }
    } else if argument_composite_conversion_required(&param.ty) {
        sifr_composite_to_bridge_expr(&value, &param.ty, param.convention.is_borrowed())
    } else {
        value
    }
}

fn is_int_list(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::List(inner) if inner.resolve_alias() == &Type::Int)
}

fn optional_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Union(members) = ty.resolve_alias() else {
        return None;
    };
    (members.len() == 2
        && members
            .iter()
            .any(|member| member.resolve_alias() == &Type::None))
    .then(|| {
        members
            .iter()
            .find(|member| member.resolve_alias() != &Type::None)
    })
    .flatten()
}

fn is_optional_str(ty: &Type) -> bool {
    optional_inner_type(ty).is_some_and(|inner| inner.resolve_alias() == &Type::Str)
}

fn is_optional_int(ty: &Type) -> bool {
    optional_inner_type(ty).is_some_and(|inner| inner.resolve_alias() == &Type::Int)
}

fn is_python_callback_constructor_target(target: &RustTargetPath) -> bool {
    matches!(
        target.segments.as_slice(),
        [root, module, function]
            if root == "sifr_stdlib"
                && module == "python"
                && matches!(function.as_str(), "py_local_callback" | "py_threadsafe_callback")
    )
}

fn is_python_object_callback_type(ty: &Type) -> bool {
    let Type::Callable(params, _, ret) = ty.resolve_alias() else {
        return false;
    };
    matches!(params.as_slice(), [param] if param.is_python_object_contract())
        && matches!(
            ret.resolve_alias(),
            Type::Result(ok, err)
                if ok.is_python_object_contract() && err.is_python_error_contract()
        )
}

fn python_object_callback_adapter_expr(handler: &str) -> RustExpr {
    RustExpr::compiler_fragment(format!(
        r#"move |__sifr_callback_arg| {{
            match {handler}(&__sifr_callback_arg) {{
                Ok(__sifr_callback_result) => Ok(__sifr_callback_result),
                Err(__sifr_callback_error) => Err(::sifr_stdlib::python::PythonError::without_replay(
                    __sifr_callback_error.kind,
                    __sifr_callback_error.exception_type,
                    __sifr_callback_error.message,
                    __sifr_callback_error.traceback,
                    __sifr_callback_error.context,
                )),
            }}
        }}"#
    ))
}
