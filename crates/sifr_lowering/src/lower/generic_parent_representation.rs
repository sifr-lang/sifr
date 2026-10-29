use sifr_type_system::{
    Type, UnionStructureClassScope, substitution_preserves_union_structure_with_class_scopes,
};
use std::collections::HashMap;

pub(super) fn preserves_union_structure(
    class_types: &HashMap<String, Type>,
    class_type_params: &HashMap<String, Vec<String>>,
    parent_name: &str,
    concrete: &Type,
) -> bool {
    let Some((template_name, template)) = class_template(class_types, concrete, parent_name) else {
        return true;
    };
    let (
        Type::Class {
            fields, methods, ..
        },
        Type::Class {
            type_args: concrete_args,
            ..
        },
    ) = (template.resolve_alias(), concrete.resolve_alias())
    else {
        return true;
    };
    let declared_params = class_type_params
        .get(template_name)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if declared_params.len() != concrete_args.len() {
        return false;
    }
    let bindings = declared_params
        .iter()
        .zip(concrete_args)
        .map(|(param, concrete)| (param.clone(), concrete.clone()))
        .collect::<HashMap<_, _>>();
    let class_scope = |identity: Option<&str>, name: &str| {
        class_candidate(class_types, identity, name).and_then(|(candidate_name, candidate)| {
            class_union_scope(
                candidate,
                class_type_params
                    .get(candidate_name)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            )
        })
    };
    fields.iter().all(|(_, ty)| {
        substitution_preserves_union_structure_with_class_scopes(ty, &bindings, &class_scope)
    }) && methods.iter().all(|(_, method)| {
        method.params.iter().all(|(_, ty, _)| {
            substitution_preserves_union_structure_with_class_scopes(ty, &bindings, &class_scope)
        }) && substitution_preserves_union_structure_with_class_scopes(
            &method.return_type,
            &bindings,
            &class_scope,
        )
    })
}

fn class_template<'a>(
    class_types: &'a HashMap<String, Type>,
    concrete: &Type,
    fallback_name: &str,
) -> Option<(&'a str, &'a Type)> {
    let Type::Class { identity, .. } = concrete.resolve_alias() else {
        return None;
    };
    class_candidate(class_types, identity.as_deref(), fallback_name)
}

fn class_candidate<'a>(
    class_types: &'a HashMap<String, Type>,
    identity: Option<&str>,
    name: &str,
) -> Option<(&'a str, &'a Type)> {
    if let Some(identity) = identity {
        if let Some((candidate_name, candidate)) =
            class_types.get_key_value(name).filter(|(_, candidate)| {
                matches!(candidate.resolve_alias(), Type::Class { identity: Some(candidate), .. } if candidate == identity)
            })
        {
            return Some((candidate_name.as_str(), candidate));
        }
        return class_types
            .iter()
            .filter_map(|(candidate_name, candidate)| {
            matches!(candidate.resolve_alias(), Type::Class { identity: Some(candidate), .. } if candidate == identity)
                .then_some((candidate_name.as_str(), candidate))
            })
            .min_by_key(|(candidate_name, _)| *candidate_name);
    }
    class_types
        .get_key_value(name)
        .map(|(candidate_name, candidate)| (candidate_name.as_str(), candidate))
}

fn class_union_scope(ty: &Type, declared_params: &[String]) -> Option<UnionStructureClassScope> {
    let Type::Class {
        fields, methods, ..
    } = ty.resolve_alias()
    else {
        return None;
    };
    let mut member_types = fields.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
    for (_, method) in methods {
        member_types.extend(method.params.iter().map(|(_, ty, _)| ty.clone()));
        member_types.push(method.return_type.as_ref().clone());
    }
    Some(UnionStructureClassScope {
        type_params: declared_params.to_vec(),
        member_types,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/lower/generic_parent_representation/tests.rs"]
mod tests;
