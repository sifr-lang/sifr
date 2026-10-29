
use super::*;

fn class(identity: Option<&str>) -> Type {
    Type::Class {
        identity: identity.map(str::to_string),
        type_args: Vec::new(),
        name: "Selected".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

#[test]
fn canonical_binding_key_requires_the_selected_owner_symbol() {
    let selected = class(Some("fixture.Selected"));
    let mismatched = class(Some("fixture.Other"));

    assert_eq!(
        canonical_owner_binding_key("Selected", &selected),
        Some("fixture.Selected")
    );
    assert_eq!(canonical_owner_binding_key("Selected", &mismatched), None);
}

#[test]
fn module_less_owner_does_not_gain_a_synthetic_binding_key() {
    assert_eq!(canonical_owner_binding_key("Selected", &class(None)), None);
}
