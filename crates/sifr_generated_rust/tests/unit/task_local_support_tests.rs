use super::declarations;
use crate::stdlib_filter::{filter_stdlib_ir_to_needed, rust_source_defined_item_names};
use quote::ToTokens;
use std::collections::HashSet;

fn locals(source: &str) -> Vec<super::TaskLocal> {
    syn::parse_file(source)
        .expect("valid Rust")
        .items
        .into_iter()
        .flat_map(|item| {
            if let syn::Item::Macro(item) = item {
                declarations(&item.mac).map_or_else(Vec::new, |locals| locals.0)
            } else {
                Vec::new()
            }
        })
        .collect()
}

#[test]
fn task_local_support_visibility_preserves_names_types_attributes_and_boundaries() {
    let source = r#"
        ::tokio::task_local! {
            #[doc = "static __SIFR_TASK_CANCELLATION: unchanged text"]
            static __SIFR_TASK_CANCELLATION: Carrier;
            pub static __SIFR_TASK_CONTEXT_LABEL: String
        }
        other::task_local! { static PRIVATE: String; }
        task_local! { static UNQUALIFIED: String; }
        mod nested { tokio::task_local! { static NESTED: String; } }
    "#;
    let consumers =
        ["fn consume() { __SIFR_TASK_CANCELLATION.get(); __SIFR_TASK_CONTEXT_LABEL.get(); }"];
    let visible =
        crate::generated_visibility::crate_visible_generated_support_source(source, &consumers);
    let declared = locals(&visible);
    assert_eq!(declared.len(), 2);
    assert_eq!(declared[0].name, "__SIFR_TASK_CANCELLATION");
    assert_eq!(declared[1].name, "__SIFR_TASK_CONTEXT_LABEL");
    for declaration in &declared {
        assert_eq!(declaration.visibility.to_token_stream().to_string(), "pub");
    }
    assert_eq!(declared[0].ty.to_token_stream().to_string(), "Carrier");
    assert_eq!(declared[1].ty.to_token_stream().to_string(), "String");
    assert_eq!(declared[0].attrs.len(), 1);
    let original = syn::parse_file(source).expect("valid source");
    let rewritten = syn::parse_file(&visible).expect("valid rewrite");
    for (original, rewritten) in original.items[1..].iter().zip(&rewritten.items[1..]) {
        assert_eq!(
            original.to_token_stream().to_string(),
            rewritten.to_token_stream().to_string()
        );
    }
    assert_eq!(
        crate::generated_visibility::crate_visible_generated_support_source(&visible, &consumers),
        visible
    );
    // Public module rewriting must not turn support macro statics into a public API.
    assert!(matches!(
        locals(&crate::generated_visibility::publicize_generated_module_source(source))[0]
            .visibility,
        syn::Visibility::Inherited
    ));
}
fn prune_generated_project_owners(
    prelude: &str,
    support: &str,
    consumers: &[&str],
) -> Result<(String, String), String> {
    crate::prune_generated_project_owners(
        prelude,
        support,
        consumers,
        sifr_type_system::BUILTIN_ERROR_CLASSES,
    )
}

#[test]
fn task_local_support_discovery_includes_all_macro_owned_symbols() {
    let names = rust_source_defined_item_names(
        "tokio::task_local! { static FIRST: String; static SECOND: u64; }\n\
         other::task_local! { static PRIVATE: String; }",
    );
    assert_eq!(names, HashSet::from(["FIRST".into(), "SECOND".into()]));
}

#[test]
fn task_local_support_pruning_tracks_direct_transitive_and_absent_demand() {
    let source = "struct Carrier; struct Unused;\n\
        tokio::task_local! { static ACTIVE: Carrier; static IDLE: Unused; }\n\
        fn accessor() { ACTIVE.try_with(|_| ()); }";
    for root in ["ACTIVE", "accessor"] {
        let filtered = filter_stdlib_ir_to_needed(source, &HashSet::from([root.into()]));
        let names = rust_source_defined_item_names(&filtered);
        assert!(names.contains("ACTIVE"), "{filtered}");
        assert!(names.contains("Carrier"), "{filtered}");
        assert!(!names.contains("IDLE"), "{filtered}");
        assert!(!names.contains("Unused"), "{filtered}");
        let consumer = if root == "accessor" {
            "fn consumer() { accessor(); }"
        } else {
            "fn consumer() { ACTIVE.try_with(|_| ()); }"
        };
        let (_, pruned) =
            prune_generated_project_owners("", source, &[consumer]).expect("support pruning");
        assert!(
            rust_source_defined_item_names(&pruned).contains("ACTIVE"),
            "{pruned}"
        );
        assert!(!pruned.contains("IDLE"), "{pruned}");
    }
    let (_, empty) = prune_generated_project_owners("", source, &["fn unrelated() {}"])
        .expect("unused support pruning");
    assert!(empty.is_empty(), "{empty}");
}

#[test]
fn task_local_support_macro_only_owner_survives_pruning() {
    let (_, support) = prune_generated_project_owners(
        "",
        "tokio::task_local! { static ACTIVE: String; }",
        &["async fn consumer() { ACTIVE.scope(String::new(), async {}).await; }"],
    )
    .expect("macro-only support owner");
    assert_eq!(locals(&support)[0].name, "ACTIVE");
}
