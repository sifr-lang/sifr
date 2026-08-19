use sifr_type_system::{
    substitution_preserves_union_structure_with_class_scopes, Type, UnionStructureClassScope,
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
        return class_types.iter().find_map(|(candidate_name, candidate)| {
            matches!(candidate.resolve_alias(), Type::Class { identity: Some(candidate), .. } if candidate == identity)
                .then_some((candidate_name.as_str(), candidate))
        });
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
mod tests {
    use super::preserves_union_structure;
    use sifr_type_system::Type;
    use std::collections::HashMap;

    fn parent(type_arg: Type, field: Type) -> Type {
        Type::Class {
            identity: Some("models.Parent".to_string()),
            type_args: vec![type_arg],
            name: "Parent".to_string(),
            fields: vec![("value".to_string(), field)],
            methods: Vec::new(),
            parent_class: None,
        }
    }

    #[test]
    fn generic_parent_rejects_only_union_topology_changes() {
        let template = parent(
            Type::TypeVar("T".to_string()),
            Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
        );
        let safe = parent(Type::Str, Type::Union(vec![Type::None, Type::Str]));
        let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
        let unsafe_parent = parent(unsafe_arg, Type::Union(vec![Type::None, Type::Str]));
        let classes = HashMap::from([("Parent".to_string(), template)]);
        let type_params = HashMap::from([("Parent".to_string(), vec!["T".to_string()])]);

        assert!(preserves_union_structure(
            &classes,
            &type_params,
            "Parent",
            &safe
        ));
        assert!(!preserves_union_structure(
            &classes,
            &type_params,
            "Parent",
            &unsafe_parent
        ));
    }

    #[test]
    fn declared_parameters_are_the_parent_binding_authority() {
        let template = parent(
            Type::Str,
            Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
        );
        let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
        let concrete = parent(unsafe_arg, Type::Unknown);
        let classes = HashMap::from([("Parent".to_string(), template)]);
        let type_params = HashMap::from([("Parent".to_string(), vec!["T".to_string()])]);

        assert!(!preserves_union_structure(
            &classes,
            &type_params,
            "Parent",
            &concrete
        ));
    }

    #[test]
    fn canonical_identity_selects_the_parent_and_finds_transitive_hazards() {
        let inner = Type::Class {
            identity: Some("models.Inner".to_string()),
            type_args: vec![Type::TypeVar("U".to_string())],
            name: "Inner".to_string(),
            fields: vec![(
                "value".to_string(),
                Type::Union(vec![Type::None, Type::TypeVar("U".to_string())]),
            )],
            methods: Vec::new(),
            parent_class: None,
        };
        let imported_parent = parent(
            Type::TypeVar("T".to_string()),
            Type::Class {
                identity: Some("models.Inner".to_string()),
                type_args: vec![Type::TypeVar("T".to_string())],
                name: "Inner".to_string(),
                fields: Vec::new(),
                methods: Vec::new(),
                parent_class: None,
            },
        );
        let mut wrong_local = parent(
            Type::TypeVar("V".to_string()),
            Type::TypeVar("V".to_string()),
        );
        let Type::Class { identity, .. } = &mut wrong_local else {
            unreachable!("the parent test helper always returns a class")
        };
        *identity = Some("main.Parent".to_string());
        let unsafe_arg = sifr_type_system::make_union(vec![Type::None, Type::Str]);
        let concrete = parent(unsafe_arg, Type::Unknown);
        let classes = HashMap::from([
            ("Parent".to_string(), wrong_local),
            ("ImportedParent".to_string(), imported_parent),
            ("Inner".to_string(), inner),
        ]);
        let type_params = HashMap::from([
            ("Parent".to_string(), vec!["V".to_string()]),
            ("ImportedParent".to_string(), vec!["T".to_string()]),
            ("Inner".to_string(), vec!["U".to_string()]),
        ]);

        assert!(!preserves_union_structure(
            &classes,
            &type_params,
            "Parent",
            &concrete
        ));
    }
}
