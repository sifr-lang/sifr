use super::canonicalize_generated_rust_source;

#[test]
fn shared_suffix_factoring_does_not_cross_branch_local_drops() {
    let source = r#"
        struct Guard(&'static str);
        impl Drop for Guard {
            fn drop(&mut self) { println!("drop {}", self.0); }
        }
        fn main() {
            let choose_left = true;
            if choose_left {
                let _guard = Guard("left");
                println!("left");
                println!("shared");
            } else {
                let _guard = Guard("right");
                println!("right");
                println!("shared");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("shared suffixes must remain before branch-local values drop");

    assert_eq!(
        canonical.matches("println!(\"shared\")").count(),
        2,
        "{canonical}"
    );
}

#[test]
fn private_field_pruning_retains_effects_and_nested_module_demand() {
    let source = r#"
        struct Guard;
        impl Drop for Guard { fn drop(&mut self) { println!("drop"); } }
        fn make_guard() -> Guard { println!("make guard"); Guard }
        fn make_number() -> i64 { println!("make number"); 4 }
        struct Config { live: i64, nested: i64, dead: i64, effect: i64, guard: Guard }
        impl Config {
            fn new() -> Self {
                Self { live: 1, nested: 2, dead: 3, effect: make_number(), guard: make_guard() }
            }
        }
        mod observer {
            pub fn nested(config: &super::Config) -> i64 { config.nested }
        }
        use observer::nested;
        fn main() {
            let config = Config::new();
            println!("{} {}", config.live, nested(&config));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("field pruning must preserve effects and cross-module uses");

    assert!(!canonical.contains("dead: i64"), "{canonical}");
    assert!(canonical.contains("nested: i64"), "{canonical}");
    assert!(canonical.contains("effect: i64"), "{canonical}");
    assert!(canonical.contains("effect: make_number()"), "{canonical}");
    assert!(canonical.contains("guard: Guard"), "{canonical}");
    assert!(canonical.contains("guard: make_guard()"), "{canonical}");
}

#[test]
fn post_parse_identity_cleanup_does_not_infer_types_from_names() {
    let source = r#"
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Choice { None, Value }
        struct Custom;
        impl Custom {
            fn map(self, _f: impl Fn(i64) -> i64) -> Self { self }
            fn into_iter(self) -> i64 { 7 }
        }
        fn main() {
            let choice = Choice::Value;
            assert!(choice != Choice::None);
            let mapped = Custom.map(|value| value).into_iter();
            let values: Option<Vec<i64>> = Some(vec![1]);
            let deref_len = values.as_deref().map_or(0, |value| value.len());
            println!("{mapped} {deref_len}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("syntax cleanup must retain expressions without resolved type provenance");

    assert!(
        canonical.contains("assert_ne!(choice, Choice::None)"),
        "{canonical}"
    );
    assert!(!canonical.contains(".is_none()"), "{canonical}");
    assert!(!canonical.contains(".is_some()"), "{canonical}");
    assert!(
        canonical.contains(".map(|value| value).into_iter()"),
        "{canonical}"
    );
    assert!(
        canonical.contains("as_deref().map_or(0, |value| value.len())"),
        "{canonical}"
    );
}

#[test]
fn dynamic_format_captures_remain_live() {
    let source = r#"
        fn main() {
            let sifr_generated_value = 1.25_f64;
            let sifr_generated_width = 8_usize;
            let sifr_generated_precision = 2_usize;
            println!("{sifr_generated_value:sifr_generated_width$.sifr_generated_precision$}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("value, width, and precision captures must share lexical liveness");

    assert!(
        canonical
            .contains("{sifr_generated_value:sifr_generated_width$.sifr_generated_precision$}"),
        "{canonical}"
    );
    assert!(
        canonical.contains("let sifr_generated_width"),
        "{canonical}"
    );
    assert!(
        canonical.contains("let sifr_generated_precision"),
        "{canonical}"
    );
}
