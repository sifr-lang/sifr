
use super::*;

fn place(root: u32, fields: &[&str]) -> Place {
    Place {
        root: BindingId(root),
        projections: fields
            .iter()
            .map(|field| {
                PlaceProjection::Field(FieldIdentity {
                    declaring_class: "Owner".to_string(),
                    field: (*field).to_string(),
                })
            })
            .collect(),
    }
}

#[test]
fn overlap_uses_binding_identity_and_projection_prefixes() {
    assert!(places_overlap(
        &place(1, &["helper"]),
        &place(1, &["helper", "items"])
    ));
    assert!(places_overlap(&place(1, &[]), &place(1, &["helper"])));
    assert!(!places_overlap(
        &place(1, &["helper"]),
        &place(1, &["other"])
    ));
    assert!(!places_overlap(
        &place(1, &["helper"]),
        &place(2, &["helper"])
    ));
}

#[test]
fn recursive_nominal_field_is_not_a_supported_projection() {
    let node_ref = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Node".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let node = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Node".to_string(),
        fields: vec![(
            "next".to_string(),
            Type::Union(vec![node_ref.clone(), Type::None]),
        )]
        .into(),
        methods: Vec::new().into(),
        parent_class: None,
    };

    assert!(is_recursive_field(&node, "next", &node_ref));
}
