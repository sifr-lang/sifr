use super::StdlibSyntaxSession;

fn assembly(imports: &str, support: &str, body: &str) -> (String, std::ops::Range<usize>) {
    let mut source = String::new();
    if !imports.is_empty() {
        source.push_str(imports);
        source.push_str("\n\n");
    }
    let start = source.len();
    source.push_str(support);
    let end = source.len();
    if !body.is_empty() {
        source.push_str("\n\n");
        source.push_str(body);
    }
    (source, start..end)
}

fn assert_oracle(session: &StdlibSyntaxSession, imports: &str, support: &str, body: &str) {
    let (source, range) = assembly(imports, support, body);
    let expected = syn::parse_file(&source)
        .map(|_| ())
        .map_err(|e| e.to_string());
    let actual = session.validate(&source, range).map_err(|e| e.to_string());
    assert_eq!(actual, expected, "full parser mismatch for {source:?}");
}

#[test]
fn complete_assembly_matches_full_parser_with_warm_support_and_invalid_consumers() {
    let session = StdlibSyntaxSession::default();
    let support = "#[derive(Clone)]\nstruct Record<T> { value: T }\nimpl<T> Record<T> { fn new(value: T) -> Self { Self { value } } }";
    assert_oracle(&session, "", support, "fn first() {}");
    for imports in [
        "",
        "use std::fmt;",
        "#![allow(dead_code)]",
        "#[cfg(any())]",
        "use std::{",
        "#[",
        "#![",
        "/*",
        "const TEXT: &str = r#\"",
    ] {
        for body in [
            "",
            "fn next() {}",
            "fn broken( {}",
            "#![allow(dead_code)]",
            "#[derive(Clone)]",
            "struct Next<T> { value: T }",
            "}",
            "*/ fn after_comment() {}",
            "\"#; fn after_string() {}",
            ";",
        ] {
            assert_oracle(&session, imports, support, body);
        }
    }
    assert!(
        session.reused.get() > 0,
        "the warm cases must exercise reuse"
    );
}

#[test]
fn raw_support_fragment_boundaries_and_attributes_match_full_parser() {
    let session = StdlibSyntaxSession::default();
    for (imports, support, body) in [
        ("", "#![allow(dead_code)]\nstruct A;", "fn main() {}"),
        ("use std::fmt;", "#![allow(dead_code)]\nstruct A;", ""),
        ("", "#[derive(Clone)]", "struct A;"),
        ("", "fn joined() {", "let value = 1; }"),
        ("", "struct A {", "field: i32 }"),
        ("", "macro_rules! m { () => {} }", "; m!();"),
        ("", "struct A; /*", "*/ fn main() {}"),
        ("", "struct A; // tail", "fn main() {}"),
        ("", "struct A;", "/* unterminated"),
        ("", "const TEXT: &str = r#\"", "\"#;"),
        ("", "#[doc = \"é💡\"] struct Café;", "fn main() {}"),
        ("", "struct A;", "const TEXT: &str = \"🦀\";"),
        ("#!/usr/bin/env rust", "struct A;", "fn main() {}"),
        ("\u{feff}", "struct A;", "fn main() {}"),
        ("#! /*comment*/ [allow(dead_code)]", "struct A;", ""),
        ("", "", ""),
        ("use std::fmt;", "", "fn main() {}"),
    ] {
        for _ in 0..2 {
            assert_oracle(&session, imports, support, body);
        }
    }
}

#[test]
fn failed_assembly_never_publishes_support_and_changed_content_is_revalidated() {
    let session = StdlibSyntaxSession::default();
    let good = "struct Shared { field: i32 }";
    assert_oracle(&session, "", good, "fn invalid(");
    assert!(session.validated_support.borrow().is_empty());
    assert_oracle(&session, "", good, "fn valid() {}");
    assert!(session.validated_support.borrow().contains(good));
    assert_oracle(&session, "", "struct Shared { field: }", "fn valid() {}");
    assert_oracle(&session, "", good, "fn invalid(");
    assert_oracle(&session, "", good, "fn valid_again() {}");
    assert_eq!(session.validated_support.borrow().len(), 1);
    assert_eq!(session.reused.get(), 1);
    let independent = StdlibSyntaxSession::default();
    assert_oracle(&independent, "", good, "fn valid() {}");
    assert_eq!(independent.reused.get(), 0);
}

#[test]
fn support_reuse_preserves_canonical_rendering_bytes_and_order() {
    use crate::lib_modules_and_codegen::generate_rust_with_stdlib_for_module_with_structural_policy;
    let parsed = sifr_python_parser::parse_module(
        "def fail_value() -> Result[int, ValueError]:\n    raise ValueError(\"local\")\n",
    )
    .expect("parse source");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("lower source");
    let metadata = crate::StdlibEmissionCode::default();
    let sources = std::collections::HashMap::new();
    let reference = generate_rust_with_stdlib_for_module_with_structural_policy(
        &lowered.module,
        &metadata.bootstrap_view(&sources),
        Some("sifr.fixture"),
        false,
    );
    let session = StdlibSyntaxSession::default();
    assert!(reference.rust_source.contains("struct ValueError"));
    for name in ["sifr.fixture", "_sifr.private_fixture", "sifr.fixture"] {
        let emitted = session.generate_module(&lowered.module, &metadata, name);
        assert_eq!(emitted.rust_source, reference.rust_source);
        assert_eq!(emitted.module_body_source, reference.module_body_source);
        assert_eq!(emitted.required_features, reference.required_features);
        syn::parse_file(&emitted.rust_source).expect("trusted full syntax oracle");
    }
    assert!(session.reused.get() > 0);
}
