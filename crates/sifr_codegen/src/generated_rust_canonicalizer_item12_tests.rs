use super::canonicalize_generated_rust_source;

#[test]
fn item12_cleanup_keeps_effectful_initializer_lookalikes() {
    let source = r#"
        fn new() -> i64 { println!("effect"); 1 }
        fn main() {
            let mut value = new();
            value = 2;
            println!("{value}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("effectful initializers must survive assignment cleanup");

    assert!(canonical.contains("let mut value = new()"), "{canonical}");
    assert!(canonical.contains("value = 2"), "{canonical}");
}

#[test]
fn item12_macro_struct_construction_conservatively_keeps_candidate_fields() {
    let source = r#"
        macro_rules! consume { ($value:expr) => { drop($value) }; }
        fn make_number() -> i64 { println!("effect"); 4 }
        struct Config { live: i64, effect: i64 }
        impl Config {
            fn run() {
                consume!(Self { live: 1, effect: make_number() });
            }
        }
        fn main() {
            Config::run();
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("macro-contained struct construction must keep field layout aligned");

    assert!(canonical.contains("live: i64"), "{canonical}");
    assert!(canonical.contains("effect: i64"), "{canonical}");
    assert!(canonical.contains("effect: make_number()"), "{canonical}");
}

#[test]
fn item12_closed_binary_removes_stale_exact_support_function_imports() {
    let source = r#"
        pub(crate) mod sifr_generated_generated_support {
            pub(super) fn live() {}
        }
        mod consumer {
            use crate::sifr_generated_generated_support::{live, message};
            pub fn run() { live(); }
        }
        fn main() { consumer::run(); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("stale support function import should canonicalize");

    assert!(canonical.contains("live"));
    assert!(!canonical.contains("message"));
}

#[test]
fn item12_residual_clippy_shapes_are_rewritten_inside_macros_and_control_flow() {
    let source = r#"
        use std::collections::HashMap;
        struct SifrInt;
        impl SifrInt { fn normalize_index_or_len(&self, _: usize) -> usize { 0 } }
        fn identity<T: Clone>(value: &T) -> T { value.clone() }
        fn sort_inplace(values: &mut Vec<i64>) { values.sort(); }
        fn recursive() {
            fn step(value: SifrInt) {
                value.normalize_index_or_len(0);
                if false { step(SifrInt); }
            }
            step(SifrInt);
        }
        fn main() {
            let bytes: Vec<u8> = vec![1];
            assert_eq!(bytes.get(0).map(|value| *value), Some(1));
            let values: HashMap<String, i64> = HashMap::new();
            assert!(!values.contains_key(&"missing".to_string()));
            assert_eq!(identity(&"text".to_string()).to_string(), "text");
            let mut lines: Vec<String> = Vec::new();
            lines = vec!["line".to_string()];
            let mut selected: Option<String> = None;
            if true { selected = Some("value".to_string()); }
            let index: SifrInt = SifrInt;
            let checked = { let sifr_generated_checked_read_index = index.clone(); sifr_generated_checked_read_index.normalize_index_or_len(lines.len()) };
            let normalized = "a\nb\r".replace('\n', " ").replace('\r', " ");
            let mut sorted = vec![2, 1];
            sort_inplace(&mut sorted);
            recursive();
            println!("{lines:?}{selected:?}{checked}{normalized}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("residual strict-Clippy shapes must canonicalize");

    assert!(canonical.contains(".get(0).copied()"), "{canonical}");
    assert!(
        canonical.contains("contains_key(\"missing\")"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("identity(&\"text\".to_string()).to_string()"),
        "{canonical}"
    );
    assert!(!canonical.contains("let mut lines"), "{canonical}");
    assert!(!canonical.contains("let mut selected"), "{canonical}");
    assert!(
        canonical.contains("let sifr_generated_checked_read_index = &index"),
        "{canonical}"
    );
    assert!(canonical.contains("replace(["), "{canonical}");
    assert!(canonical.contains("'\\n'"), "{canonical}");
    assert!(canonical.contains("'\\r'"), "{canonical}");
    assert!(
        canonical.contains("fn sort_inplace(values: &mut [i64])"),
        "{canonical}"
    );
    assert!(
        canonical.contains("fn step(value: &SifrInt)"),
        "{canonical}"
    );
}

#[test]
fn item12_replacement_collapse_keeps_sequential_semantics_when_replacement_matches() {
    let source = r#"
        fn main() {
            println!("{}", "ab".replace('a', "b").replace('b', "b"));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("replacement-sensitive chains must remain sequential");

    assert!(canonical.matches(".replace").count() >= 2, "{canonical}");
}
