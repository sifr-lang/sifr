use sifr_ir::HirClass;
use sifr_type_system::class_rust_name;

pub(super) fn target(class: &HirClass) -> String {
    let rust_name = class_rust_name(class.identity.as_deref(), &class.name);
    if class.type_params.is_empty() {
        rust_name
    } else {
        format!("{rust_name}<{}>", class.type_params.join(", "))
    }
}

/// Structural contracts require contracts for their field types as well.
/// Resolve those dependencies by canonical identity, never by a basename.
pub(crate) fn imported_classes<'a>(
    module: &sifr_ir::HirModule,
    stdlib: &'a crate::StdlibEmissionCode,
) -> Vec<&'a HirClass> {
    use std::collections::{BTreeMap, BTreeSet};
    let available = stdlib
        .module_class_templates
        .values()
        .flat_map(|templates| templates.values())
        .filter_map(|class| class.identity.as_deref().map(|identity| (identity, class)))
        .collect::<BTreeMap<_, _>>();
    let mut pending = BTreeSet::new();
    for import in &module.imports {
        if let Some(templates) = stdlib.module_class_templates.get(&import.module) {
            for name in &import.names {
                if let Some(identity) = templates
                    .get(name)
                    .and_then(|class| class.identity.as_ref())
                {
                    pending.insert(identity.clone());
                }
            }
        }
    }
    let mut selected = BTreeMap::new();
    while let Some(identity) = pending.pop_first() {
        if selected.contains_key(&identity) {
            continue;
        }
        let Some(class) = available.get(identity.as_str()).copied() else {
            continue;
        };
        selected.insert(identity, class);
        for (_, field) in &class.fields {
            collect_dependencies(field, &mut pending);
        }
        if let Some(parent) = &class.parent_type {
            collect_dependencies(parent, &mut pending);
        }
    }
    selected.into_values().collect()
}

fn collect_dependencies(
    ty: &sifr_type_system::Type,
    names: &mut std::collections::BTreeSet<String>,
) {
    use sifr_type_system::Type;
    match ty.resolve_alias() {
        Type::Class {
            identity,
            type_args,
            ..
        } => {
            if let Some(identity) = identity {
                names.insert(identity.clone());
            }
            for argument in type_args {
                collect_dependencies(argument, names);
            }
        }
        Type::Enum {
            identity: Some(identity),
            ..
        } => {
            names.insert(identity.clone());
        }
        Type::List(value) | Type::Set(value) => collect_dependencies(value, names),
        Type::Dict(key, value) => {
            collect_dependencies(key, names);
            collect_dependencies(value, names);
        }
        Type::Tuple(values) | Type::Union(values) => {
            for value in values {
                collect_dependencies(value, names);
            }
        }
        _ => {}
    }
}
