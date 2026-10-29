use super::ProjectNominalRegistry;
use crate::stdlib_filter::{rust_source_defined_item_names, strip_rust_items_by_name};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

pub(super) fn relocate_project_unions(
    shared_source: &str,
    unions: &HashMap<String, Vec<Type>>,
    registry: &mut ProjectNominalRegistry,
) -> (String, HashSet<String>) {
    let shared_defined_names = rust_source_defined_item_names(shared_source);
    let relocated = unions
        .keys()
        .filter(|name| shared_defined_names.contains(*name))
        .cloned()
        .collect::<HashSet<_>>();
    registry.shared_rust_names.extend(relocated.iter().cloned());
    let relocated_names = relocated.iter().map(String::as_str).collect();
    (
        strip_rust_items_by_name(shared_source, &relocated_names),
        relocated,
    )
}

pub(super) fn shared_nominal_reexport_names(
    registry: &ProjectNominalRegistry,
    relocated_project_unions: &HashSet<String>,
) -> Vec<String> {
    let mut names = registry
        .shared_rust_names
        .difference(relocated_project_unions)
        .cloned()
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[cfg(test)]
#[path = "../../tests/unit/project_stdlib_nominals/project_union_relocation/tests.rs"]
mod tests;
