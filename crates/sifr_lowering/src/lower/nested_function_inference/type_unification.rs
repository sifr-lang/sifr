use super::{unify_matching_defaultdict_aliases, Type};

pub(super) fn has_conflicting_inference(current: &Type, incoming: &Type) -> bool {
    match (current, incoming) {
        (Type::Unknown, _) | (_, Type::Unknown) => false,
        (Type::List(current_elem), Type::List(incoming_elem)) => {
            has_conflicting_inference(current_elem, incoming_elem)
        }
        (Type::Dict(current_key, current_value), Type::Dict(incoming_key, incoming_value)) => {
            has_conflicting_inference(current_key, incoming_key)
                || has_conflicting_inference(current_value, incoming_value)
        }
        _ => !current.is_assignable_to(incoming) && !incoming.is_assignable_to(current),
    }
}

pub(super) fn unify_types(current: Type, incoming: Type) -> Type {
    let current = collapse_literal(current);
    let incoming = collapse_literal(incoming);

    if current.is_unknown() {
        return incoming;
    }
    if incoming.is_unknown() {
        return current;
    }
    if current == incoming {
        return current;
    }
    if let Some(alias) = unify_matching_defaultdict_aliases(&current, &incoming) {
        return alias;
    }

    match (&current, &incoming) {
        (Type::List(current_elem), Type::List(incoming_elem)) => Type::List(Box::new(unify_types(
            (**current_elem).clone(),
            (**incoming_elem).clone(),
        ))),
        (Type::Set(current_elem), Type::Set(incoming_elem)) => Type::Set(Box::new(unify_types(
            (**current_elem).clone(),
            (**incoming_elem).clone(),
        ))),
        (Type::Dict(current_key, current_value), Type::Dict(incoming_key, incoming_value)) => {
            Type::Dict(
                Box::new(unify_types(
                    (**current_key).clone(),
                    (**incoming_key).clone(),
                )),
                Box::new(unify_types(
                    (**current_value).clone(),
                    (**incoming_value).clone(),
                )),
            )
        }
        (Type::Float, Type::Int) | (Type::Int, Type::Float) => Type::Float,
        _ if incoming.is_assignable_to(&current) => current,
        _ if current.is_assignable_to(&incoming) => incoming,
        _ => current,
    }
}

pub(super) fn type_contains_unknown_or_any(ty: &Type) -> bool {
    match ty {
        Type::Unknown | Type::Any => true,
        Type::List(elem) => type_contains_unknown_or_any(elem),
        Type::Dict(key, value) => {
            type_contains_unknown_or_any(key) || type_contains_unknown_or_any(value)
        }
        Type::Tuple(elements) => elements.iter().any(type_contains_unknown_or_any),
        _ => false,
    }
}

fn collapse_literal(ty: Type) -> Type {
    match ty {
        Type::LiteralInt(_) => Type::Int,
        Type::LiteralStr(_) => Type::Str,
        Type::LiteralBool(_) => Type::Bool,
        Type::List(elem_ty) => Type::List(Box::new(collapse_literal(*elem_ty))),
        Type::Set(elem_ty) => Type::Set(Box::new(collapse_literal(*elem_ty))),
        Type::Dict(key_ty, value_ty) => Type::Dict(
            Box::new(collapse_literal(*key_ty)),
            Box::new(collapse_literal(*value_ty)),
        ),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name,
            type_args,
            body: Box::new(collapse_literal(*body)),
        },
        other => other,
    }
}
