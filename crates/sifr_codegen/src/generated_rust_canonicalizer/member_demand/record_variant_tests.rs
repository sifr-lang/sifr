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
            pub enum Part { Direct { value: i64 }, Aliased { value: i64 }, Dead }
        }
        use support::Part as ImportedPart;
        fn main() {
            let direct = support::Part::Direct { value: 1 };
            let aliased = vec![ImportedPart::Aliased { value: 2 }];
            std::hint::black_box((direct, aliased));
        }
        "#,
    );
    assert!(canonical.contains("Direct {"), "{canonical}");
    assert!(canonical.contains("Aliased {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_compiles_nested_template_runtime() {
    let source = "def retain(value: Template) -> int:\n    return 1\n\ndef main():\n    precision: int = 2\n    result: int = retain(t\"score={3.5:.{precision}f}\")\n    assert result == 1\n";
    let parsed = sifr_python_parser::parse_module(source).expect("template source parses");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("template source lowers");
    let canonical = canonical_and_compile(&crate::generate_rust(&lowered.module));
    assert!(canonical.contains("SifrGeneratedTemplateFormatSpecPart"));
    assert!(canonical.contains("Literal {"), "{canonical}");
    assert!(canonical.contains("Interpolation {"), "{canonical}");
}
