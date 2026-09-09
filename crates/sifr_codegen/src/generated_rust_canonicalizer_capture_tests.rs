use super::{canonicalize_generated_rust_source, item_dependencies::item_dependency_names};
use std::collections::HashSet;

fn canonical(source: &str) -> String {
    canonicalize_generated_rust_source(source).expect("capture demand must canonicalize")
}

#[test]
fn implicit_format_capture_retains_constant_and_import_after_rewrite() {
    let output = canonical(
        r#"
        mod __sifr_generated_support { pub const ACTIVE: i64 = 1; }
        use crate::__sifr_generated_support::{ACTIVE};
        fn main() { println!("{}", ACTIVE); }
        "#,
    );
    assert!(output.contains("const ACTIVE"), "{output}");
    assert!(
        output.contains("use crate::sifr_generated_generated_support"),
        "{output}"
    );
    assert!(output.contains("{ACTIVE}"), "{output}");
}

#[test]
fn implicit_format_capture_retains_width_precision_and_aliased_imports() {
    let output = canonical(
        r#"
        mod support {
            pub const VALUE: f64 = 1.25;
            pub const WIDTH: usize = 8;
            pub const PRECISION: usize = 2;
            pub const UNUSED: usize = 99;
        }
        use support::VALUE as value;
        use support::WIDTH as width;
        use support::PRECISION as precision;
        use support::UNUSED;
        fn main() { println!("{value:>width$.precision$}"); }
        "#,
    );
    for name in ["VALUE", "WIDTH", "PRECISION"] {
        assert!(output.contains(&format!("const {name}")), "{output}");
    }
    assert!(!output.contains("UNUSED"), "{output}");
    assert!(output.contains("as value"), "{output}");
    assert!(output.contains("as width"), "{output}");
    assert!(output.contains("as precision"), "{output}");
}

#[test]
fn implicit_format_capture_prunes_import_shadowed_only_by_local_binding() {
    let output = canonical(
        r#"
        mod support { pub const VALUE: i64 = 99; }
        use support::VALUE as value;
        fn main() { let value = 7; println!("{value}"); }
        "#,
    );
    assert!(!output.contains("const VALUE"), "{output}");
    assert!(!output.contains("use support"), "{output}");
    assert!(output.contains("{value}"), "{output}");
}

#[test]
fn implicit_format_capture_retains_item_before_and_after_nested_shadowing() {
    let output = canonical(
        r#"
        const value: i64 = 42;
        fn main() {
            println!("{value}");
            { const value: i64 = 7; println!("{value}"); }
            println!("{value}");
        }
        "#,
    );
    assert!(output.contains("const value"), "{output}");
}

#[test]
fn implicit_format_capture_respects_lexical_binding_boundaries() {
    let cases = [
        (r#"fn main() { let value = format!("{value}"); }"#, true),
        (
            r#"fn main() { let value = 1; println!("{value}"); }"#,
            false,
        ),
        (r#"fn main(value: i64) { println!("{value}"); }"#, false),
        (
            r#"fn main() { let f = |value| format!("{value}"); }"#,
            false,
        ),
        (
            r#"fn main() { let f = |value| format!("{value}"); println!("{value}"); }"#,
            true,
        ),
        (
            r#"fn main() { for value in [1] { println!("{value}"); } }"#,
            false,
        ),
        (
            r#"fn main() { for value in [1] { println!("{value}"); } println!("{value}"); }"#,
            true,
        ),
        (
            r#"fn main() { if let Some(value) = Some(1) { println!("{value}"); } }"#,
            false,
        ),
        (
            r#"fn main() { if let Some(value) = Some(1) { println!("{value}"); } else { println!("{value}"); } }"#,
            true,
        ),
        (
            r#"fn main() { while let Some(value) = Some(1) { println!("{value}"); } }"#,
            false,
        ),
        (
            r#"fn main() { match Some(1) { Some(value) => println!("{value}"), None => () } }"#,
            false,
        ),
        (
            r#"fn main() { match Some(1) { Some(value) => println!("{value}"), None => println!("{value}") } }"#,
            true,
        ),
        (
            r#"fn main() { let value = 1; fn nested() { println!("{value}"); } }"#,
            true,
        ),
        (
            r#"fn main() { println!("{value}"); const value: i64 = 1; }"#,
            false,
        ),
        (
            r#"fn main() { let value = 1; println!("{}", crate::value); }"#,
            true,
        ),
        (r#"fn main() { println!("{{value}}"); }"#, false),
        (r#"fn main() { println!("{value}", value = 1); }"#, false),
        (r#"fn main() { println!("{value}", value = value); }"#, true),
    ];
    let definitions = HashSet::from(["value".to_string()]);
    for (source, demanded) in cases {
        let item = syn::parse_str(source).expect("valid lexical regression");
        assert_eq!(
            item_dependency_names(&item, &definitions).contains("value"),
            demanded,
            "{source}",
        );
    }
}

#[test]
fn implicit_format_capture_support_imports_preserve_block_binding_lifetimes() {
    let cases = [
        (r#"fn main() { let value = format!("{value}"); }"#, true),
        (
            r#"fn main() { let first = 1; let value = first; let last = value; println!("{value} {last}"); }"#,
            false,
        ),
        (
            r#"fn main() { let value = 1; { let first = value; let value = first; println!("{value}"); } println!("{value}"); }"#,
            false,
        ),
        (
            r#"fn main() { { let first = 1; let value = first; println!("{value}"); } { println!("{value}"); } }"#,
            true,
        ),
        (
            r#"fn main() { let first = 1; let value = first; fn nested() { println!("{value}"); } }"#,
            true,
        ),
        (
            r#"fn main() { let first = 1; let value = first; let capture = || format!("{value}"); }"#,
            false,
        ),
    ];
    let definitions = HashSet::from(["value".to_string()]);
    for (source, demanded) in cases {
        let imports =
            crate::stdlib_filter::rust_source_unqualified_item_names(source, &definitions)
                .expect("valid support import consumer");
        assert_eq!(imports.contains("value"), demanded, "{source}");
    }
}

#[test]
fn implicit_format_capture_nested_macros_and_glob_imports_remain_live() {
    let output = canonical(
        r#"
        mod support { pub const VALUE: i64 = 1; pub const UNUSED: i64 = 2; }
        use support::*;
        fn main() { assert_eq!(format!("{VALUE}"), "1"); }
        "#,
    );
    assert!(output.contains("const VALUE"), "{output}");
    assert!(!output.contains("const UNUSED"), "{output}");
}

#[test]
fn implicit_format_capture_native_output_preserves_imports_and_scopes() {
    let output = canonical(
        r#"
        mod support { pub const VALUE: f64 = 1.25; pub const WIDTH: usize = 6; }
        use support::VALUE as value;
        use support::WIDTH as width;
        const precision: usize = 2;
        fn main() {
            println!("{value:>width$.precision$}");
            { const value: i64 = 7; println!("{value}"); }
            println!("{value}");
        }
        "#,
    );
    assert_native_output(&output, "  1.25\n7\n1.25\n");
}

#[test]
fn implicit_format_capture_retains_nested_module_import_aliases() {
    let output = canonical(
        r#"
        mod support { pub const VALUE: i64 = 5; pub const WIDTH: usize = 3; }
        mod consumer {
            use crate::support::VALUE as value;
            use crate::support::WIDTH as width;
            pub fn show() { println!("{value:width$}"); }
        }
        fn main() { consumer::show(); }
        "#,
    );
    assert!(output.contains("const VALUE"), "{output}");
    assert!(output.contains("const WIDTH"), "{output}");
    assert_native_output(&output, "  5\n");
}

#[test]
fn implicit_format_capture_value_shadow_preserves_imported_type_namespace() {
    let output = canonical(
        r#"
        mod support { pub struct Value { pub number: i64 } }
        use support::Value;
        fn main() {
            let Value = 7;
            let object = Value { number: 5 };
            let typed: Value = object;
            println!("{Value}:{}", typed.number);
        }
        "#,
    );
    assert!(output.contains("struct Value"), "{output}");
    assert_native_output(&output, "7:5\n");
    let definitions = HashSet::from(["Value".to_string()]);
    for source in [
        "fn main() { let Value = 7; let object = Value { number: 5 }; }",
        "fn main() { let Value = 7; let object: Value; }",
    ] {
        let item = syn::parse_str(source).expect("valid type namespace regression");
        assert!(
            item_dependency_names(&item, &definitions).contains("Value"),
            "{source}"
        );
    }
    let item = syn::parse_quote!(
        fn main<Value>() {
            let object: Value;
        }
    );
    assert!(item_dependency_names(&item, &definitions).is_empty());
}

fn assert_native_output(output: &str, expected: &str) {
    let directory = tempfile::tempdir().expect("owned native regression directory");
    let source = directory.path().join("capture.rs");
    let binary = directory.path().join("capture");
    std::fs::write(&source, &output).expect("write canonical native regression");
    let compilation = std::process::Command::new("rustc")
        .arg("--edition=2024")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("execute rustc");
    assert!(
        compilation.status.success(),
        "{output}\n{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let execution = std::process::Command::new(&binary)
        .output()
        .expect("execute native regression");
    assert!(execution.status.success());
    assert_eq!(String::from_utf8_lossy(&execution.stdout), expected);
}
