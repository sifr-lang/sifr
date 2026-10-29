use super::{rust_source_referenced_item_names, rust_source_required_trait_names};
use std::collections::HashSet;

#[test]
fn local_name_collisions_retain_support_conservatively() {
    let candidates = HashSet::from(["support_value".to_string()]);
    let referenced = rust_source_referenced_item_names(
        "fn run() { let support_value = 1; consume(support_value); }",
        &candidates,
    );

    assert_eq!(referenced, candidates);
}

#[test]
fn method_calls_require_matching_support_traits() {
    let required = rust_source_required_trait_names(
        "fn run(value: Wrapper) { value.render(); }",
        "trait RenderSupport { fn render(&self); }",
    )
    .expect("flat support traits should be accepted");

    assert_eq!(required, HashSet::from(["RenderSupport".to_string()]));
}

#[test]
fn nested_support_traits_violate_the_flat_owner_invariant() {
    let error = rust_source_required_trait_names(
        "fn run(value: Wrapper) { value.render(); }",
        "mod nested { trait RenderSupport { fn render(&self); } }",
    )
    .expect_err("nested support traits must fail closed");

    assert!(error.contains("support traits must be top-level items"));
    assert!(error.contains("RenderSupport"));
}
