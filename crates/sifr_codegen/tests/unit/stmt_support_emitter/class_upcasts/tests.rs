use super::render_ancestor_rust_type;
use crate::{RustEmitter, RustExpr};
use sifr_type_system::Type;

fn class(identity: &str, name: &str, parent_class: Option<&str>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: parent_class.map(str::to_string),
    }
}

#[test]
fn basename_only_ancestor_match_does_not_emit_an_upcast() {
    let source = class("pkg.Child", "Child", Some("pkg.Root"));
    let unrelated_root = class("other.Root", "Root", None);
    let value = RustExpr::Ident("value".to_string());

    assert_eq!(
        RustEmitter::new().consuming_value_conversion_for_ir(
            &unrelated_root,
            &source,
            value.clone(),
        ),
        value
    );
}

#[test]
fn ancestor_paths_are_local_stdlib_or_crate_rooted_by_identity() {
    assert_eq!(
        render_ancestor_rust_type(Some("models"), "models.Mid"),
        "Mid"
    );
    assert_eq!(
        render_ancestor_rust_type(Some("adapter"), "models.Mid"),
        "crate::models::Mid"
    );
    assert_eq!(
        render_ancestor_rust_type(Some("adapter"), "sifr.resource.NullContext"),
        sifr_type_system::stdlib_class_rust_name("sifr.resource", "NullContext")
    );
}
