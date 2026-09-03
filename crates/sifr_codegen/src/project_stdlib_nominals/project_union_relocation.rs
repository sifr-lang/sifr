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
mod tests {
    use super::*;

    #[test]
    fn project_union_owner_is_relocated_out_of_the_nominal_module() {
        let union_name = "GeneratedProjectUnion";
        let shared_source = format!(
            r#"
pub enum {union_name} {{ Value(i64) }}
impl ::std::fmt::Display for {union_name} {{
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{
        write!(formatter, "union")
    }}
}}
pub struct Carrier;
impl Carrier {{
    pub fn value(&self) -> {union_name} {{ {union_name}::Value(1) }}
}}
"#
        );
        let unions = HashMap::from([(union_name.to_string(), vec![Type::Int, Type::Str])]);
        let mut registry = ProjectNominalRegistry::default();

        let (relocated_source, relocated_names) =
            relocate_project_unions(&shared_source, &unions, &mut registry);

        assert_eq!(relocated_names, HashSet::from([union_name.to_string()]));
        assert!(registry.shared_rust_names.contains(union_name));
        registry
            .shared_rust_names
            .insert("SharedNominal".to_string());
        assert_eq!(
            shared_nominal_reexport_names(&registry, &relocated_names),
            vec!["SharedNominal".to_string()]
        );
        assert!(!relocated_source.contains(&format!("enum {union_name}")));
        assert!(!relocated_source.contains(&format!("Display for {union_name}")));
        assert!(
            relocated_source.contains(&format!("fn value(&self) -> {union_name}")),
            "{relocated_source}"
        );
    }
}
