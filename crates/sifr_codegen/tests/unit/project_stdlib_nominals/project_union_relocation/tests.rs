
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
