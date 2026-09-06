use super::{
    DiagnosticCode, Expr, ExprAttribute, ExprCall, FunctionType, HirExpr, LowerCtx, Ranged, Type,
    call_argument_ranges_by_param, consume_owned_value, lower_signature_call_args,
};

pub(super) fn method_function_type(ty: &Type, method_name: &str) -> Option<FunctionType> {
    match ty.resolve_alias() {
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods
            .iter()
            .find(|(candidate, _)| candidate == method_name)
            .map(|(_, function_type)| function_type.clone())
            .or_else(|| {
                super::super::callable_fields::callable_field_function_type(ty, method_name)
            }),
        Type::StructuralRecord(_) => {
            super::super::callable_fields::callable_field_function_type(ty, method_name)
        }
        _ => None,
    }
}

pub(super) fn method_param_defaults(
    ctx: &LowerCtx,
    owner_type: &Type,
    method_name: &str,
    source_key: &str,
) -> Option<Vec<(usize, HirExpr)>> {
    let compiler_key = match owner_type.resolve_alias() {
        Type::Class {
            identity: Some(identity),
            ..
        } => Some(format!("{identity}.{method_name}")),
        _ => None,
    };
    compiler_key
        .and_then(|key| ctx.compiler_method_defaults.get(&key).cloned())
        .or_else(|| ctx.function_defaults.get(source_key).cloned())
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
    if method_name == "__init__"
        && ctx.current_parent_class.is_none()
        && ctx
            .current_class
            .as_ref()
            .is_some_and(|class| ctx.error_types.contains(class))
    {
        let parent_type = super::super::classes::root_error_type();
        let signature = FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            parent_type.clone(),
        );
        let Some(args) = lower_signature_call_args(call, "Error", &signature, None, ctx) else {
            return Some(None);
        };
        if !matches!(
            args[0].ty().resolve_alias(),
            Type::Str | Type::LiteralStr(_)
        ) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "Error.__init__(): expected 'str' message, got '{}'",
                    args[0].ty().display_name()
                ),
                call.range(),
            );
            return Some(None);
        }
        return Some(Some(HirExpr::SuperCall {
            parent_class: "Error".to_string(),
            parent_type: parent_type.clone(),
            method: "new".to_string(),
            args,
            ty: parent_type,
        }));
    }
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
    let defining_name = if method_name == "__init__" {
        parent_name.clone()
    } else {
        ctx.class_method_origins
            .get(&format!("{parent_name}.{method_name}"))
            .cloned()
            .unwrap_or_else(|| parent_name.clone())
    };
    let defining_type = if defining_name == parent_name {
        parent_type
    } else {
        ctx.class_types
            .get(&defining_name)
            .cloned()
            .unwrap_or(parent_type)
    };
    let defaults_key = if method_name == "__init__" {
        defining_name.clone()
    } else {
        format!("{defining_name}.{method_name}")
    };
    let method_type = if method_name == "__init__" {
        ctx.functions.get(&defining_name).cloned()
    } else {
        method_function_type(&defining_type, &method_name)
    };
    let Some(function_type) = method_type else {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!("parent class '{parent_name}' has no method '{method_name}'"),
            attr.attr.range(),
        );
        return Some(None);
    };
    let method_defaults = method_param_defaults(ctx, &defining_type, &method_name, &defaults_key);
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
    if method_name == "__init__"
        && ctx
            .current_class
            .as_ref()
            .is_some_and(|class| ctx.error_types.contains(class))
    {
        for (arg, (name, expected, _)) in args.iter().zip(&function_type.params) {
            if !arg.ty().is_assignable_to(expected) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "{parent_name}.__init__(): argument '{name}' expected '{}', got '{}'",
                        expected.display_name(),
                        arg.ty().display_name()
                    ),
                    call.range(),
                );
                return Some(None);
            }
        }
    }
    super::super::sequence_guards::invalidate_mutable_call_sequence_guards(
        ctx,
        &args,
        function_type
            .params
            .iter()
            .map(|(_, _, convention)| *convention),
    );
    let return_type = *function_type.return_type;
    Some(Some(HirExpr::SuperCall {
        parent_class: defining_name,
        parent_type: defining_type,
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
    if let Some(result) = super::attached_api_calls::try_lower_type_call(attr, call, ctx) {
        return Some(result);
    }
    let Expr::Name(name) = attr.value.as_ref() else {
        return None;
    };
    let class_name = name.id.to_string();
    let class_type = ctx.class_types.get(&class_name).cloned()?;
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
    let method_defaults = method_param_defaults(ctx, &class_type, &method_name, &defaults_key);
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
    super::super::sequence_guards::invalidate_mutable_call_sequence_guards(
        ctx,
        &args,
        function_type
            .params
            .iter()
            .map(|(_, _, convention)| *convention),
    );
    Some(Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: format!("{class_name}::{method_name}"),
        args,
        ty: *function_type.return_type,
    }))
}
