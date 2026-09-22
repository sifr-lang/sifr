use super::canonicalize_generated_rust_source;

#[test]
fn terminal_loop_branches_move_only_after_the_last_repeat_path() {
    let source = r#"
        fn route(value: String, direct: bool, flags: Vec<bool>) -> Vec<String> {
            let mut output = Vec::new();
            if direct { output.push(value.clone()); }
            else {
                for flag in flags {
                    if flag { continue; }
                    output.push(value.clone());
                    break;
                }
            }
            output
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("terminal loop transfer");
    assert!(!canonical.contains("value.clone()"), "{canonical}");
}

#[test]
fn loop_moves_preserve_labeled_repetition_and_later_borrows() {
    let source = r#"
        fn repeated(value: String, flags: Vec<bool>) -> Vec<String> {
            let mut output = Vec::new();
            'outer: for flag in flags {
                output.push(value.clone());
                for _ in 0..1 { if flag { continue 'outer; } }
                break;
            }
            output
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("labeled repeat liveness");
    assert!(
        canonical.contains("output.push(value.clone())"),
        "{canonical}"
    );
}

#[test]
fn typed_none_fallback_does_not_rewrite_lookalike_methods() {
    let source = r#"
        struct Other;
        impl Other { fn unwrap_or_else(self, callback: fn(bool) -> Option<i64>) -> Option<i64> { callback(true) } }
        fn builtin(value: Result<Option<i64>, String>) -> Option<i64> {
            value.unwrap_or_else(|_error| None)
        }
        fn external(value: Other) -> Option<i64> { value.unwrap_or_else(|_error| None) }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("typed fallback identity");
    assert!(canonical.contains("value.unwrap_or(None)"), "{canonical}");
    assert!(
        canonical.contains("value.unwrap_or_else(|_error| None)"),
        "{canonical}"
    );
}

#[test]
fn unknown_import_only_blocks_clone_proofs_in_its_own_scope() {
    let canonical = canonicalize_generated_rust_source(
        r#"
        mod external;
        use external::OtherClone;
        fn uncertain(value: String) -> String { value.clone() }
        mod closed {
            pub fn owned(value: String) -> String { value.clone() }
        }
    "#,
    )
    .expect("scoped trait visibility");
    assert!(canonical.contains("value.clone()"), "{canonical}");
    assert!(
        canonical.contains("fn owned(value: String) -> String {\n        value\n"),
        "{canonical}"
    );
}

#[test]
fn independent_owned_fields_move_but_sibling_borrows_and_repeats_keep_clones() {
    let canonical = canonicalize_generated_rust_source(
        r#"
        #[derive(Clone)]
        pub struct Record { pub left: String, pub right: String }
        pub fn split() -> (String, String) {
            let record: Record = make();
            (record.left.clone(), record.right.clone())
        }
        pub fn repeated() -> (String, String) {
            let record: Record = make();
            (record.left.clone(), record.left.clone())
        }
        pub fn borrowed() {
            let record: Record = make();
            observe(&record, record.left.clone());
        }
    "#,
    )
    .expect("disjoint inert field moves");
    assert!(
        canonical.contains("(record.left, record.right)"),
        "{canonical}"
    );
    assert!(
        canonical.contains("(record.left.clone(), record.left.clone())"),
        "{canonical}"
    );
    assert!(
        canonical.contains("observe(&record, record.left.clone())"),
        "{canonical}"
    );
}

#[test]
fn collected_set_length_preserves_distinct_element_semantics() {
    let canonical = canonicalize_generated_rust_source(
        r#"
        pub fn unique_length(values: Vec<i64>) -> usize {
            let values = values.into_iter().collect::<std::collections::BTreeSet<_>>();
            values.len()
        }
    "#,
    )
    .expect("set cardinality");
    assert!(canonical.contains("BTreeSet"), "{canonical}");
    assert!(!canonical.contains(".count()"), "{canonical}");
}

#[test]
fn typed_string_cleanup_preserves_capacity_effects_and_shadowed_bindings() {
    let canonical = canonicalize_generated_rust_source(r#"
        struct Other;
        impl Other { fn to_string(&self) -> u64 { 7 } }
        fn changed(value: String) -> u64 {
            observe(value.clone());
            let value = Other;
            value.to_string()
        }
        fn capacity(value: &str) -> String {
            let output = { let mut text = String::with_capacity(side_effect()); text.push_str(value); text };
            output
        }
    "#).expect("lexical string identity and effects");
    assert!(canonical.contains("value.to_string()"), "{canonical}");
    assert!(canonical.contains("side_effect()"), "{canonical}");
}

#[test]
fn typed_string_initializer_reuses_owned_identity_concatenation() {
    let output = canonicalize_generated_rust_source(
        r#"
        pub struct Record { pub path: String }
        impl Record {
            pub fn new(path: String) -> Self {
                let value: String = {
                    let mut buffer: String = String::with_capacity(path.len().saturating_add(0usize));
                    buffer.push_str(path.as_str());
                    buffer.push_str("");
                    buffer
                };
                Self { path: value }
            }
        }
        "#,
    ).expect("typed owned string initializer");
    assert!(!output.contains("with_capacity"), "{output}");
    assert!(!output.contains("push_str"), "{output}");
    assert!(!output.contains("path.clone()"), "{output}");
}

#[test]
fn qualified_collection_lookalike_retains_its_cardinality_operation() {
    let source = r#"
        pub fn count(values: Vec<i32>) -> usize {
            let distinct = values.into_iter().collect::<::third_party::Vec<_>>();
            distinct.len()
        }
    "#;
    let output = canonicalize_generated_rust_source(source).expect("qualified collection");
    assert!(
        output.contains("collect::<::third_party::Vec<_>>()"),
        "{output}"
    );
    assert!(output.contains(".len()"), "{output}");
    assert!(!output.contains(".count()"), "{output}");
}
