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
