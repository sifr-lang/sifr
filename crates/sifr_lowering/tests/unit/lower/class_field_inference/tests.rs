
use super::*;

fn imported_class(local_name: &str, identity: &str) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args: Vec::new(),
        name: local_name.to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

#[test]
fn inferred_field_union_uses_declaration_identity_not_local_spelling() {
    let alpha = imported_class("Zeta", "pkg.Alpha");
    let beta = imported_class("Beta", "pkg.Beta");

    assert_eq!(
        merge_inferred_types(beta.clone(), alpha.clone()),
        sifr_type_system::make_union(vec![alpha, beta])
    );
}
