use super::*;

fn source() -> &'static str {
    r#"
        struct Shared { value: i64 }
        impl ::sifr_runtime::interop::structural::StructuralType for Shared {
            fn shape_identity() -> u64 { 7 }
        }
        impl ::sifr_runtime::interop::structural::StructuralConstruct for Shared {}
        impl ::sifr_runtime::interop::structural::StructuralProject for Shared {}
        impl other::StructuralProject for Shared {}
        impl Shared { fn value(&self) -> i64 { self.value } }
        fn consume(value: Shared) { std::hint::black_box(value); }
    "#
}

#[test]
fn relocated_structural_contracts_have_one_shared_owner_across_importers() {
    let mut implementations = RelocatedStructuralImplementations::default();
    let names = HashSet::from(["Shared".to_string()]);
    for module in ["main", "support"] {
        let local = relocate_project_stdlib_nominals_owned_by(
            source(),
            module,
            &HashSet::from(["main"]),
            &HashSet::new(),
            &names,
            &mut implementations,
        );
        assert!(!local.contains("impl"), "{local}");
        assert!(local.contains("fn consume"), "{local}");
        assert_eq!(local.contains("use crate::Shared;"), module == "support");
    }
    let shared = implementations.append_to_support("struct Shared { value: i64 }");
    for name in ["StructuralType", "StructuralConstruct", "StructuralProject"] {
        assert_eq!(
            shared
                .matches(&format!("structural::{name} for Shared"))
                .count(),
            1,
            "{shared}"
        );
    }
    assert!(!shared.contains("other::StructuralProject"));
    assert!(!shared.contains("fn value("));
    assert!(!shared.contains("fn consume"));
    assert_eq!(shared.matches("struct Shared").count(), 1);
}

#[test]
fn relocated_structural_contracts_do_not_capture_a_same_name_local_owner() {
    let mut implementations = RelocatedStructuralImplementations::default();
    let names = HashSet::from(["Shared".to_string()]);
    let local = relocate_project_stdlib_nominals_owned_by(
        source(),
        "support",
        &HashSet::from(["main"]),
        &names,
        &names,
        &mut implementations,
    );
    assert!(local.contains("struct Shared"));
    assert!(local.contains("StructuralConstruct for Shared"));
    assert!(!local.contains("use crate::Shared;"));
    assert_eq!(implementations.append_to_support("unchanged"), "unchanged");
}

#[test]
#[should_panic(expected = "inconsistent structural contract for shared nominal")]
fn relocated_structural_contracts_never_select_between_conflicting_bodies() {
    let mut implementations = RelocatedStructuralImplementations::default();
    let names = HashSet::from(["Shared".to_string()]);
    for source in [source().to_string(), source().replace("{ 7 }", "{ 8 }")] {
        relocate_project_stdlib_nominals_owned_by(
            &source,
            "main",
            &HashSet::from(["main"]),
            &HashSet::new(),
            &names,
            &mut implementations,
        );
    }
}
