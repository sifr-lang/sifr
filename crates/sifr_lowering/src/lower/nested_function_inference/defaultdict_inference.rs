use super::{
    lookup_name_type, unify_name_binding, unify_types, Expr, ExprCall, FunctionEnv, HashMap,
    LocalFunctionState, LowerCtx, Type, DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS,
    DEFAULTDICT_SET_ALIAS,
};

pub(super) fn infer_defaultdict_call_type(call: &ExprCall) -> Option<Type> {
    let Expr::Name(name) = call.func.as_ref() else {
        return None;
    };
    if name.id != "defaultdict" || call.arguments.args.len() != 1 {
        return None;
    }
    let Some(Expr::Name(factory)) = call.arguments.args.first() else {
        return None;
    };
    let (alias, value_ty) = match factory.id.as_str() {
        "int" => (DEFAULTDICT_INT_ALIAS, Type::Int),
        "list" => (DEFAULTDICT_LIST_ALIAS, Type::List(Box::new(Type::Unknown))),
        "set" => (DEFAULTDICT_SET_ALIAS, Type::Set(Box::new(Type::Unknown))),
        _ => return None,
    };
    Some(Type::alias(
        alias,
        Type::Dict(Box::new(Type::Unknown), Box::new(value_ty)),
    ))
}

pub(super) fn refine_defaultdict_method_call(
    object: &Expr,
    method: &str,
    arg_types: &[Type],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    let Expr::Subscript(subscript) = object else {
        return;
    };
    let Expr::Name(name) = subscript.value.as_ref() else {
        return;
    };
    let Type::Alias {
        name: alias_name,
        type_args,
        body,
    } = lookup_name_type(name.id.as_str(), env, states, ctx)
    else {
        return;
    };
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return;
    };
    let refined_value_ty = match (alias_name.as_str(), method, value_ty.as_ref()) {
        (DEFAULTDICT_SET_ALIAS, "add", Type::Set(elem_ty)) if arg_types.len() == 1 => Type::Set(
            Box::new(unify_types(*elem_ty.clone(), arg_types[0].clone())),
        ),
        (DEFAULTDICT_LIST_ALIAS, "append", Type::List(elem_ty)) if arg_types.len() == 1 => {
            Type::List(Box::new(unify_types(
                *elem_ty.clone(),
                arg_types[0].clone(),
            )))
        }
        _ => return,
    };
    unify_name_binding(
        name.id.as_str(),
        Type::Alias {
            name: alias_name,
            type_args,
            body: Box::new(Type::Dict(key_ty.clone(), Box::new(refined_value_ty))),
        },
        env,
        states,
        current_function,
    );
}

pub(super) fn refine_defaultdict_subscript(
    object: &Expr,
    object_ty: &Type,
    index_ty: &Type,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
) -> Option<Type> {
    let Expr::Name(name) = object else {
        return None;
    };
    let Type::Alias {
        name: alias_name,
        type_args,
        body,
    } = object_ty
    else {
        return None;
    };
    if !matches!(
        alias_name.as_str(),
        DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
    ) {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return None;
    };
    unify_name_binding(
        name.id.as_str(),
        Type::Alias {
            name: alias_name.clone(),
            type_args: type_args.clone(),
            body: Box::new(Type::Dict(
                Box::new(unify_types(*key_ty.clone(), index_ty.clone())),
                value_ty.clone(),
            )),
        },
        env,
        states,
        current_function,
    );
    Some(*value_ty.clone())
}

pub(super) fn unify_matching_defaultdict_aliases(current: &Type, incoming: &Type) -> Option<Type> {
    let (
        Type::Alias {
            name: current_name,
            type_args,
            body: current_body,
        },
        Type::Alias {
            name: incoming_name,
            body: incoming_body,
            ..
        },
    ) = (current, incoming)
    else {
        return None;
    };
    if current_name != incoming_name
        || !matches!(
            current_name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        )
    {
        return None;
    }
    Some(Type::Alias {
        name: current_name.clone(),
        type_args: type_args.clone(),
        body: Box::new(unify_types(
            (**current_body).clone(),
            (**incoming_body).clone(),
        )),
    })
}
