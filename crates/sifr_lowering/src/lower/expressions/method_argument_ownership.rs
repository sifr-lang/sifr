use super::{
    call_argument_ranges_by_param, consume_owned_value, lower_signature_call_args, DiagnosticCode,
    Expr, ExprAttribute, ExprCall, FunctionType, HirExpr, LowerCtx, Ranged, Type,
};

pub(super) fn method_function_type(ty: &Type, method_name: &str) -> Option<FunctionType> {
    match ty.resolve_alias() {
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods
            .iter()
            .find(|(candidate, _)| candidate == method_name)
            .map(|(_, function_type)| function_type.clone()),
        _ => None,
    }
}

pub(super) fn consume_owned_method_arguments(
    args: &[HirExpr],
    call: &ExprCall,
    function_type: &FunctionType,
    ctx: &mut LowerCtx,
) {
    let ranges = call_argument_ranges_by_param(call, function_type);
    for (index, arg) in args.iter().enumerate() {
        let Some((_, _, convention)) = function_type.params.get(index) else {
            continue;
        };
        if convention.is_owned() {
            let range = ranges
                .get(index)
                .copied()
                .flatten()
                .unwrap_or_else(|| call.range());
            consume_owned_value(arg, range, ctx);
        }
    }
}

pub(super) fn try_lower_super_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Expr::Call(super_call) = attr.value.as_ref() else {
        return None;
    };
    let Expr::Name(name) = super_call.func.as_ref() else {
        return None;
    };
    if name.id.as_str() != "super" {
        return None;
    }
    let method_name = attr.attr.to_string();
    let (Some(parent_name), Some(parent_type)) = (
        ctx.current_parent_class.clone(),
        ctx.current_parent_type.clone(),
    ) else {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_INVALID_BASE,
            "super() used outside of a class with a parent".to_string(),
            attr.value.range(),
        );
        return Some(None);
    };
    let defaults_key = if method_name == "__init__" {
        parent_name.clone()
    } else {
        format!("{parent_name}.{method_name}")
    };
    let method_type = if method_name == "__init__" {
        ctx.functions.get(&parent_name).cloned()
    } else {
        method_function_type(&parent_type, &method_name)
    };
    let Some(function_type) = method_type else {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!("parent class '{parent_name}' has no method '{method_name}'"),
            attr.attr.range(),
        );
        return Some(None);
    };
    let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
    let Some(args) = lower_signature_call_args(
        call,
        &defaults_key,
        &function_type,
        method_defaults.as_deref(),
        ctx,
    ) else {
        return Some(None);
    };
    consume_owned_method_arguments(&args, call, &function_type, ctx);
    let return_type = *function_type.return_type;
    Some(Some(HirExpr::SuperCall {
        parent_class: parent_name,
        parent_type,
        method: if method_name == "__init__" {
            "new".to_string()
        } else {
            method_name
        },
        args,
        ty: return_type,
    }))
}

pub(super) fn try_lower_class_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Expr::Name(name) = attr.value.as_ref() else {
        return None;
    };
    let class_name = name.id.to_string();
    let Some(class_type) = ctx.class_types.get(&class_name).cloned() else {
        return None;
    };
    let method_name = attr.attr.to_string();
    let qualified_method = format!("{class_name}.{method_name}");
    if matches!(class_type.resolve_alias(), Type::Protocol { .. })
        || ctx.class_instance_methods.contains(&qualified_method)
    {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!("type '{class_name}' has no class/static method '{method_name}'"),
            attr.attr.range(),
        );
        return Some(None);
    }
    let Some(function_type) = method_function_type(&class_type, &method_name) else {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!("type '{class_name}' has no class/static method '{method_name}'"),
            attr.attr.range(),
        );
        return Some(None);
    };
    let defaults_key = qualified_method;
    let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
    let Some(args) = lower_signature_call_args(
        call,
        &defaults_key,
        &function_type,
        method_defaults.as_deref(),
        ctx,
    ) else {
        return Some(None);
    };
    consume_owned_method_arguments(&args, call, &function_type, ctx);
    Some(Some(HirExpr::Call {
        func: format!("{class_name}::{method_name}"),
        args,
        ty: *function_type.return_type,
    }))
}
