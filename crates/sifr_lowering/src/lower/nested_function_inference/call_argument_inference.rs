use super::{
    HashMap, LocalFunctionState, ParamKind, Type, generic_call_inference::InferredCall,
    has_conflicting_inference, unify_types,
};

pub(super) fn unify_inferred_call_arguments(
    function_name: &str,
    state: &LocalFunctionState<'_>,
    inferred: InferredCall,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let positional_params = state
        .params
        .iter()
        .filter(|param| param.kind == ParamKind::Positional)
        .collect::<Vec<_>>();
    let vararg = state
        .params
        .iter()
        .find(|param| param.kind == ParamKind::Vararg);
    for (index, arg_ty) in inferred.positional_types.into_iter().enumerate() {
        if let Some(param) = positional_params.get(index) {
            unify_function_param(function_name, param.name.as_str(), arg_ty, states);
        } else if let Some(param) = vararg {
            unify_variadic_param_value(function_name, param.name.as_str(), arg_ty, states);
        }
    }

    let kwarg = state
        .params
        .iter()
        .find(|param| param.kind == ParamKind::Kwarg);
    for (name, keyword_ty) in inferred.keyword_types {
        let Some(name) = name else {
            if let Some(param) = kwarg {
                unify_function_param(function_name, param.name.as_str(), keyword_ty, states);
            }
            continue;
        };
        if let Some(param) = state.params.iter().find(|param| {
            param.name == name
                && matches!(param.kind, ParamKind::Positional | ParamKind::KeywordOnly)
        }) {
            unify_function_param(function_name, param.name.as_str(), keyword_ty, states);
        } else if let Some(param) = kwarg {
            unify_variadic_param_value(function_name, param.name.as_str(), keyword_ty, states);
        }
    }
}

fn unify_function_param(
    function_name: &str,
    param_name: &str,
    incoming: Type,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let Some(state) = states.get_mut(function_name) else {
        return;
    };
    let Some(param) = state
        .params
        .iter_mut()
        .find(|param| param.name == param_name)
    else {
        return;
    };
    if !param.explicit && has_conflicting_inference(&param.ty, &incoming) {
        param.ty = Type::Unknown;
        state.inference_failed = true;
    } else {
        param.ty = unify_types(param.ty.clone(), incoming);
    }
}

fn unify_variadic_param_value(
    function_name: &str,
    param_name: &str,
    incoming: Type,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let Some(state) = states.get_mut(function_name) else {
        return;
    };
    let Some(param) = state
        .params
        .iter_mut()
        .find(|param| param.name == param_name)
    else {
        return;
    };
    let existing = match param.ty.clone() {
        Type::List(element) => *element,
        Type::Dict(_, value) => *value,
        _ => return,
    };
    if !param.explicit && has_conflicting_inference(&existing, &incoming) {
        param.ty = variadic_container_type(param.kind, Type::Unknown);
        state.inference_failed = true;
        return;
    }
    param.ty = variadic_container_type(param.kind, unify_types(existing, incoming));
}

fn variadic_container_type(kind: ParamKind, element: Type) -> Type {
    match kind {
        ParamKind::Vararg => Type::List(Box::new(element)),
        ParamKind::Kwarg => Type::Dict(Box::new(Type::Str), Box::new(element)),
        ParamKind::Positional | ParamKind::KeywordOnly => element,
    }
}
