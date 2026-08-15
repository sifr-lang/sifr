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
    ty.contains_unknown_or_any()
}

pub(super) fn replace_inference_holes_with_any(ty: Type) -> Type {
    match ty {
        Type::Unknown => Type::Any,
        Type::List(element) => Type::List(Box::new(replace_inference_holes_with_any(*element))),
        Type::Set(element) => Type::Set(Box::new(replace_inference_holes_with_any(*element))),
        Type::Dict(key, value) => Type::Dict(
            Box::new(replace_inference_holes_with_any(*key)),
            Box::new(replace_inference_holes_with_any(*value)),
        ),
        Type::Tuple(elements) => Type::Tuple(
            elements
                .into_iter()
                .map(replace_inference_holes_with_any)
                .collect(),
        ),
        Type::Union(elements) => sifr_type_system::make_union(
            elements
                .into_iter()
                .map(replace_inference_holes_with_any)
                .collect(),
        ),
        Type::Intersection(elements) => Type::Intersection(
            elements
                .into_iter()
                .map(replace_inference_holes_with_any)
                .collect(),
        ),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name,
            type_args,
            body: Box::new(replace_inference_holes_with_any(*body)),
        },
        Type::Iterable(element) => {
            Type::Iterable(Box::new(replace_inference_holes_with_any(*element)))
        }
        Type::Iterator(element) => {
            Type::Iterator(Box::new(replace_inference_holes_with_any(*element)))
        }
        Type::Result(ok, error) => Type::Result(
            Box::new(replace_inference_holes_with_any(*ok)),
            Box::new(replace_inference_holes_with_any(*error)),
        ),
        other => other,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inference_hole_replacement_recanonicalizes_and_deduplicates_unions() {
        let ty = Type::Union(vec![Type::Unknown, Type::Any]);

        assert_eq!(replace_inference_holes_with_any(ty), Type::Any);
    }
}
