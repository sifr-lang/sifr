use crate::canonicalize_generated_rust_source;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn canonical_and_compile(source: &str) -> String {
    let canonical = canonicalize_generated_rust_source(source).expect("canonical Rust");
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "sifr-record-variant-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("unique native evidence directory");
    let mut child = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name=record_variant_demand",
            "--crate-type=lib",
            "--emit=metadata",
            "--out-dir",
        ])
        .arg(&directory)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rustc");
    child
        .stdin
        .take()
        .expect("compiler stdin")
        .write_all(canonical.as_bytes())
        .expect("write canonical Rust");
    let result = child.wait_with_output().expect("compiler result");
    assert!(
        result.status.success(),
        "{}\n{canonical}",
        String::from_utf8_lossy(&result.stderr)
    );
    canonical
}

#[test]
fn record_variant_demand_preserves_nested_macro_constructors() {
    let canonical = canonical_and_compile(
        r#"
        enum Part { Literal { value: String }, Nested { child: Box<Part> }, Dead }
        fn main() {
            let parts = vec![Part::Nested {
                child: Box::new(Part::Literal { value: String::from("live") })
            }];
            std::hint::black_box(parts);
        }
        "#,
    );
    assert!(canonical.contains("Literal {"), "{canonical}");
    assert!(canonical.contains("Nested {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_self_constructors_inside_macros() {
    let canonical = canonical_and_compile(
        r#"
        enum Part { Live { value: i64 }, Dead }
        impl Part {
            fn make() -> Vec<Self> { vec![Self::Live { value: 7 }] }
        }
        fn main() { std::hint::black_box(Part::make()); }
        "#,
    );
    assert!(canonical.contains("Live {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_cross_scope_constructors_and_aliases() {
    let canonical = canonical_and_compile(
        r#"
        mod support {
            #[derive(Clone)]
            pub enum Part {
                Direct { value: i64 }, Aliased { value: i64 }, Repeated { value: i64 }, Dead
            }
        }
        use support::Part as ImportedPart;
        fn main() {
            let direct = support::Part::Direct { value: 1 };
            let aliased = vec![ImportedPart::Aliased { value: 2 }];
            let repeated = vec![ImportedPart::Repeated { value: 3 }; 2];
            std::hint::black_box((direct, aliased, repeated));
        }
        "#,
    );
    assert!(canonical.contains("Direct {"), "{canonical}");
    assert!(canonical.contains("Aliased {"), "{canonical}");
    assert!(canonical.contains("Repeated {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_compiles_nested_template_runtime() {
    let source = "def retain(value: Template) -> bool:\n    return True\n\ndef main():\n    precision: str = \"2\"\n    result: bool = retain(t\"score={3.5:.{precision}f}\")\n    assert result\n";
    let parsed = sifr_python_parser::parse_module(source).expect("template source parses");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("template source lowers");
    let canonical = canonical_and_compile(&crate::generate_rust(&lowered.module));
    assert!(canonical.contains("SifrGeneratedTemplateFormatSpecPart"));
    assert!(canonical.contains("Literal {"), "{canonical}");
    assert!(canonical.contains("Interpolation {"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_macro_initializer_effects() {
    let canonical = canonical_and_compile(
        r#"
        struct Carrier { effect: i64, unused: i64 }
        fn next() -> i64 { std::hint::black_box(7) }
        fn main() {
            let values = vec![Carrier { effect: next(), unused: 0 }];
            std::hint::black_box(values);
        }
        "#,
    );
    assert!(canonical.contains("effect: i64"), "{canonical}");
    let tokens = canonical
        .parse::<proc_macro2::TokenStream>()
        .expect("canonical tokens")
        .to_string();
    assert!(tokens.contains("effect : next ()"), "{canonical}");
    assert!(!canonical.contains("unused"), "{canonical}");
}

#[test]
fn removed_union_patterns_are_rewritten_inside_local_and_external_macros() {
    let canonical = canonical_and_compile(
        r#"
        mod support {
            pub enum __SifrUnionExternal { Live(String), Dead(String) }
        }
        use support::__SifrUnionExternal as Imported;
        enum __SifrUnionLocal { Live(String), Dead(String) }
        fn main() {
            assert_eq!(match Imported::Live(String::from("external")) {
                Imported::Live(value) => value,
                Imported::Dead(value) => value,
            }, "external");
            assert_eq!(match __SifrUnionLocal::Live(String::from("local")) {
                __SifrUnionLocal::Live(value) => value,
                __SifrUnionLocal::Dead(value) => value,
            }, "local");
        }
        "#,
    );
    assert!(!canonical.contains("Dead"), "{canonical}");
    assert_eq!(canonical.matches("assert_eq!").count(), 2, "{canonical}");
}

#[test]
fn const_promotion_proves_owned_inputs_and_locals_are_transferred_not_dropped() {
    let canonical = canonical_and_compile(
        r#"
        pub fn discard(_: String) -> i64 { 1 }
        pub fn discard_result(_: Result<String, String>) -> i64 { 2 }
        pub fn discard_option(_: Option<String>) -> i64 { 3 }
        pub fn discard_generic<T>(_: T) -> i64 { 4 }
        pub fn discard_local() -> i64 { let text = String::new(); 5 }
        pub fn discard_alias(value: String) -> i64 { let alias = value; 6 }
        pub fn discard_shadow(value: String) -> i64 { let value = 10; value }
        pub fn transfer(value: String) -> String { value }
        pub fn transfer_local(value: String) -> String { let alias = value; alias }
        pub fn scalar(_: i64) -> i64 { 7 }
        pub fn scalar_alias(value: i64) -> i64 { let alias = value; alias }
        pub fn choose(value: String, flag: bool) -> String {
            if flag { value } else { value }
        }
        pub fn conditional_discard(value: String, flag: bool) -> String {
            if flag { value } else { String::new() }
        }
        pub fn borrowed(_: &String) -> i64 { 8 }
        pub struct Carrier { pub value: String }
        impl Carrier {
            pub fn new(value: String) -> Self { Self { value } }
            pub fn discard_self(self) -> i64 { 9 }
        }
        "#,
    );
    for name in [
        "discard",
        "discard_result",
        "discard_option",
        "discard_generic",
        "discard_local",
        "discard_alias",
        "discard_shadow",
        "discard_self",
        "conditional_discard",
    ] {
        assert!(
            !canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
        assert!(
            !canonical.contains(&format!("const fn {name}<")),
            "{canonical}"
        );
    }
    for name in [
        "transfer",
        "transfer_local",
        "scalar",
        "scalar_alias",
        "borrowed",
        "new",
        "choose",
    ] {
        assert!(
            canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
    }
}
