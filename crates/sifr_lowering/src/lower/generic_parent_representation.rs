use sifr_type_system::{
    substitution_preserves_union_structure_with_class_scopes, Type, UnionStructureClassScope,
};
use std::collections::HashMap;

pub(super) fn preserves_union_structure(
    class_types: &HashMap<String, Type>,
    parent_name: &str,
    concrete: &Type,
) -> bool {
    let Some(template) = class_template(class_types, concrete, parent_name) else {
        return true;
    };
    let (
        Type::Class {
            type_args: template_args,
            fields,
            methods,
            ..
        },
        Type::Class {
            type_args: concrete_args,
            ..
        },
    ) = (template.resolve_alias(), concrete.resolve_alias())
    else {
        return true;
    };
    let bindings = template_args
        .iter()
        .zip(concrete_args)
        .filter_map(|(template, concrete)| match template.resolve_alias() {
            Type::TypeVar(name) => Some((name.clone(), concrete.clone())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let class_scope = |identity: Option<&str>, name: &str| {
        class_candidate(class_types, identity, name).and_then(class_union_scope)
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
) -> Option<&'a Type> {
    let Type::Class { identity, .. } = concrete.resolve_alias() else {
        return None;
    };
    class_candidate(class_types, identity.as_deref(), fallback_name)
}

fn class_candidate<'a>(
    class_types: &'a HashMap<String, Type>,
    identity: Option<&str>,
    name: &str,
) -> Option<&'a Type> {
    if let Some(identity) = identity {
        return class_types.values().find(|candidate| {
            matches!(candidate.resolve_alias(), Type::Class { identity: Some(candidate), .. } if candidate == identity)
        });
    }
    class_types.get(name)
}

fn class_union_scope(ty: &Type) -> Option<UnionStructureClassScope> {
    let Type::Class {
        type_args,
        fields,
        methods,
        ..
    } = ty.resolve_alias()
    else {
        return None;
    };
    let type_params = type_args
        .iter()
        .filter_map(|argument| match argument.resolve_alias() {
            Type::TypeVar(name) => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut member_types = fields.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
    for (_, method) in methods {
        member_types.extend(method.params.iter().map(|(_, ty, _)| ty.clone()));
        member_types.push(method.return_type.as_ref().clone());
    }
    Some(UnionStructureClassScope {
        type_params,
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

        assert!(preserves_union_structure(&classes, "Parent", &safe));
        assert!(!preserves_union_structure(
            &classes,
            "Parent",
            &unsafe_parent
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

        assert!(!preserves_union_structure(&classes, "Parent", &concrete));
    }
}
