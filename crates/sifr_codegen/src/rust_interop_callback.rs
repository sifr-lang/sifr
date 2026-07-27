use crate::rust_interop_direct_collections::{
    bridge_composite_to_sifr_expr, composite_conversion_required, sifr_composite_to_bridge_expr,
};
use crate::{render_expr, RustExpr};
use sifr_ir::{HirFunction, HirParam, RustInteropDecoratorKind};
use sifr_type_system::{OwnershipKind, ParamConvention, Type};

pub(crate) fn call_scoped_callback_adapter_expr(param: &HirParam) -> RustExpr {
    let Type::Callable(params, conventions, result) = param.ty.resolve_alias() else {
        unreachable!("call-scoped callback adapter requires a Callable parameter");
    };
    assert_eq!(
        params.len(),
        conventions.len(),
        "Callable parameter types and conventions must stay aligned"
    );
    let argument_names = (0..params.len())
        .map(|index| format!("__sifr_callback_arg_{index}"))
        .collect::<Vec<_>>();
    let tuple_pattern = match argument_names.as_slice() {
        [] => "()".to_string(),
        [only] => format!("({only},)"),
        many => format!("({})", many.join(", ")),
    };
    let handler_args = params
        .iter()
        .zip(conventions.iter())
        .zip(argument_names.iter())
        .map(|((ty, convention), name)| callback_handler_arg(name, ty, *convention))
        .collect::<Vec<_>>()
        .join(", ");
    let handler = render_expr(&RustExpr::Ident(param.name.clone()));
    let invocation = format!("{handler}({handler_args})");
    let body = callback_return_expr(&invocation, result);

    RustExpr::Verbatim(format!(
        "::sifr_runtime::interop::CallScopedCallbackBridge::new(&move |{tuple_pattern}| {{ {body} }})"
    ))
}

pub(crate) fn call_scoped_callbacks(func: &HirFunction) -> bool {
    !func
        .rust_interop
        .iter()
        .any(|declaration| declaration.kind == RustInteropDecoratorKind::Callback)
}

fn callback_handler_arg(name: &str, ty: &Type, convention: ParamConvention) -> String {
    let converted = if ty.resolve_alias() == &Type::Int {
        format!("{name}.to_i64_saturating()")
    } else if composite_conversion_required(ty) {
        render_expr(&bridge_composite_to_sifr_expr(
            &RustExpr::Ident(name.to_string()),
            ty,
        ))
    } else {
        name.to_string()
    };
    if convention.is_shared_borrow() && ty.ownership() == OwnershipKind::Move {
        format!("&{converted}")
    } else {
        converted
    }
}

fn callback_return_expr(invocation: &str, result: &Type) -> String {
    match result.resolve_alias() {
        Type::Result(ok, _) => {
            let converted = callback_success_expr("__sifr_callback_ok", ok);
            format!(
                "match {invocation} {{ Ok(__sifr_callback_ok) => Ok({converted}), \
                 Err(__sifr_callback_error) => Err(__sifr_callback_error.to_string()) }}"
            )
        }
        Type::None => invocation.to_string(),
        _ => callback_success_expr(invocation, result),
    }
}

fn callback_success_expr(value: &str, ty: &Type) -> String {
    if ty.resolve_alias() == &Type::None {
        return "()".to_string();
    }
    if ty.resolve_alias() == &Type::Int || composite_conversion_required(ty) {
        return render_expr(&sifr_composite_to_bridge_expr(
            &RustExpr::Verbatim(value.to_string()),
            ty,
            false,
        ));
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_callback_adapter_maps_values_and_display_errors() {
        let error = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "EventError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let param = HirParam {
            name: "handler".to_string(),
            ty: Type::Callable(
                vec![Type::Str],
                vec![ParamConvention::borrow()],
                Box::new(Type::Result(Box::new(Type::None), Box::new(error))),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        };

        let rendered = render_expr(&call_scoped_callback_adapter_expr(&param));

        assert!(rendered.contains("CallScopedCallbackBridge::new"));
        assert!(rendered.contains("handler(&__sifr_callback_arg_0)"));
        assert!(rendered.contains("Ok(__sifr_callback_ok) => Ok(())"));
        assert!(rendered.contains("__sifr_callback_error.to_string()"));
    }
}
