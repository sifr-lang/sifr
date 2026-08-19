use sifr_type_system::{substitution_preserves_union_structure, Type};
use std::collections::HashMap;

pub(super) fn preserves_union_structure(template: &Type, concrete: &Type) -> bool {
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
    fields
        .iter()
        .all(|(_, ty)| substitution_preserves_union_structure(ty, &bindings))
        && methods.iter().all(|(_, method)| {
            method
                .params
                .iter()
                .all(|(_, ty, _)| substitution_preserves_union_structure(ty, &bindings))
                && substitution_preserves_union_structure(&method.return_type, &bindings)
        })
}

#[cfg(test)]
mod tests {
    use super::preserves_union_structure;
    use sifr_type_system::Type;

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

        assert!(preserves_union_structure(&template, &safe));
        assert!(!preserves_union_structure(&template, &unsafe_parent));
    }
}
