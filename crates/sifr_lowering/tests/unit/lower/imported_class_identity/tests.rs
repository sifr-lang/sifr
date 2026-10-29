use super::*;

fn class(name: &str, identity: Option<&str>) -> Type {
    Type::Class {
        identity: identity.map(str::to_string),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

fn member_identities(ty: Type) -> Vec<String> {
    let Type::Union(members) = ty else {
        panic!("expected union");
    };
    members.iter().map(Type::union_variant_name).collect()
}

#[test]
fn imported_alias_renaming_keeps_union_identity_order() {
    let original = make_union(vec![
        class("Alpha", Some("pkg.Alpha")),
        class("Beta", Some("pkg.Beta")),
    ]);
    let aliases = HashMap::from([("Alpha".to_string(), "Zeta".to_string())]);
    let renamed = rename_class_identities(&original, &aliases);

    assert_eq!(member_identities(original), member_identities(renamed));
}

#[test]
fn canonical_identity_insertion_reorders_union_members() {
    let mut actual = Type::Union(vec![class("Zeta", None), class("Beta", None)]);
    let identities = HashMap::from([
        ("Zeta".to_string(), "pkg.Alpha".to_string()),
        ("Beta".to_string(), "pkg.Beta".to_string()),
    ]);
    set_canonical_identities(&mut actual, &identities);
    let expected = make_union(vec![
        class("Zeta", Some("pkg.Alpha")),
        class("Beta", Some("pkg.Beta")),
    ]);

    assert_eq!(actual, expected);
}
