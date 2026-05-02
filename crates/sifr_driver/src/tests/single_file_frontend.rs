use crate::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
    CompileResult, CompileResultFull,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};

#[test]
fn test_parse_source_returns_suite_for_valid_program() {
    let suite = parse_source("def main():\n    x: int = 1\n")
        .expect("parse_source should return a suite for valid source");
    assert!(!suite.is_empty());
}

#[test]
fn test_parse_source_returns_parse_error_for_invalid_program() {
    let errors = parse_source("def main(:\n").expect_err("invalid source should fail parsing");
    assert!(!errors.is_empty());
    assert_eq!(
        errors[0].code,
        DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY.code()
    );
}

#[test]
fn test_parse_source_classifies_parser_error_categories() {
    let cases = [
        (
            "def main(:\n    pass\n",
            DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
            "expected",
            "parser_category",
        ),
        (
            "def main():\n    x = \"unterminated\n",
            DiagnosticCode::PARSE_LEXICAL_OR_STRING,
            "reason",
            "parser_category",
        ),
        (
            "    x = 1\n",
            DiagnosticCode::PARSE_LAYOUT,
            "reason",
            "parser_category",
        ),
        (
            "def main():\n    1 = 2\n",
            DiagnosticCode::PARSE_INVALID_TARGET,
            "target_kind",
            "parser_category",
        ),
        (
            "def main():\n    f(a=1, 2)\n",
            DiagnosticCode::PARSE_INVALID_CALL_ARGUMENTS,
            "reason",
            "parser_category",
        ),
        (
            "def main():\n    global\n",
            DiagnosticCode::PARSE_MALFORMED_DECLARATION_LIST,
            "declaration_kind",
            "parser_category",
        ),
        (
            "def main():\n    match 1:\n        case *x:\n            pass\n",
            DiagnosticCode::PARSE_INVALID_PATTERN,
            "reason",
            "parser_category",
        ),
        (
            "lazy import value\n",
            DiagnosticCode::PARSE_UNSUPPORTED_SYNTAX,
            "syntax_kind",
            "parser_category",
        ),
    ];

    for (source, expected_code, message_arg, category_arg) in cases {
        let errors = parse_source(source).expect_err("invalid source should fail parsing");
        let diagnostic = errors
            .iter()
            .find(|diagnostic| diagnostic.code == expected_code.code())
            .unwrap_or_else(|| {
                panic!(
                    "expected parser code {} in {errors:?}",
                    expected_code.code()
                )
            });
        assert_eq!(diagnostic.code, expected_code.code(), "{source}");
        assert_eq!(diagnostic.severity, expected_code.declared_severity());
        assert!(diagnostic.args.contains_key(message_arg), "{source}");
        assert!(matches!(
            diagnostic.args.get(category_arg),
            Some(DiagnosticArg::String(category)) if !category.is_empty()
        ));
        assert_ne!(diagnostic.message_template, "{message}");
    }
}

#[test]
fn test_parse_source_normalizes_parser_recovery_messages() {
    let expected_prefixed = parse_source("def main(:\n")
        .expect_err("invalid source should fail parsing")
        .into_iter()
        .find(|diagnostic| {
            diagnostic.code == DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY.code()
        })
        .expect("expected parser recovery diagnostic");
    assert_eq!(
        expected_prefixed.message,
        "syntax error: expected a parameter or the end of the parameter list"
    );
    assert!(!expected_prefixed.message.contains("expected Expected"));

    let recovery_prefixed = parse_source("def f(mut mut items: list[int]):\n    return items\n")
        .expect_err("invalid source should fail parsing")
        .into_iter()
        .find(|diagnostic| {
            diagnostic.code == DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY.code()
                && diagnostic.message.contains("recovery:")
        })
        .expect("non-expected recovery payload should be normalized");
    assert!(recovery_prefixed
        .message
        .starts_with("syntax error: expected recovery: "));
}

#[test]
fn test_lower_source_and_type_check_source_surface_type_errors() {
    let errors = match lower_source("def main():\n    x: int = \"bad\"\n") {
        Ok(_) => panic!("type mismatch should fail lowering/type-check"),
        Err(errors) => errors,
    };
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .all(|error| error.code == DiagnosticCode::TYPE_MISMATCH.code()));

    let check_errors = type_check_source("def main():\n    x: int = \"bad\"\n");
    assert_eq!(errors.len(), check_errors.len());
    assert_eq!(
        errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>(),
        check_errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_compile_hello_world() {
    let source = r#"
def main():
    print("Hello, World!")
"#;
    match compile(source) {
        CompileResult::Success { rust_source } => {
            assert!(rust_source.contains("fn main()"));
            assert!(rust_source.contains("println!"));
            assert!(rust_source.contains("Hello, World!"));
        }
        CompileResult::Errors { errors } => {
            panic!("compilation failed: {:?}", errors);
        }
    }
}

#[test]
fn test_compile_factorial() {
    let source = r#"
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    x: int = factorial(5)
    print(x)
"#;
    match compile(source) {
        CompileResult::Success { rust_source } => {
            assert!(rust_source.contains("fn factorial(n: i64) -> i64"));
            assert!(rust_source.contains("fn main()"));
        }
        CompileResult::Errors { errors } => {
            panic!("compilation failed: {:?}", errors);
        }
    }
}

#[test]
fn test_compile_indexing_path_does_not_emit_unwrap_in_main_body() {
    let source = r#"
def main():
    items: list[int] = [10, 20, 30]
    value: int | None = items[1]
    if value is not None:
        print(value)
"#;
    match compile_with_metadata(source) {
        CompileResultFull::Success { rust_source, .. } => {
            let main_start = rust_source
                .find("fn main()")
                .expect("generated Rust must contain fn main()");
            let main_body = &rust_source[main_start..];
            assert!(
                main_body.contains(".get("),
                "main body should use safe get()-based indexing"
            );
            assert!(
                !main_body.contains(".unwrap("),
                "main body must not rely on data-dependent unwrap for indexing"
            );
            assert!(
                !main_body.contains(".expect("),
                "main body must not rely on data-dependent expect for indexing"
            );
        }
        CompileResultFull::Errors { errors } => {
            panic!("compilation failed: {:?}", errors);
        }
    }
}

#[test]
fn test_type_mismatch_error() {
    let source = r#"
def main():
    x: int = "hello"
"#;
    let errors = check(source);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
}

#[test]
fn test_check_reports_structured_frontend_diagnostics() {
    let source = r#"
def main():
    x: int = "hello"
"#;
    let errors = check(source);
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .all(|error| error.code == DiagnosticCode::TYPE_MISMATCH.code()));
}

#[test]
fn test_check_reports_primary_span_for_ranged_hir_diagnostic() {
    let source = "def main():\n    if 1:\n        pass\n";
    let errors = check(source);
    let diagnostic = errors
        .iter()
        .find(|error| error.code == DiagnosticCode::FLOW_INVALID_CONDITION_TYPE.code())
        .expect("expected invalid condition diagnostic");
    let primary = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("expected primary span");

    assert_eq!(primary.file.as_deref(), Some("main"));
    assert_eq!(primary.line, Some(2));
    assert_eq!(primary.column, Some(8));
    assert_eq!(primary.end_line, Some(2));
    assert_eq!(primary.end_column, Some(9));
}

#[test]
fn test_check_valid_program() {
    let source = r#"
def main():
    x: int = 42
    print(x)
"#;
    let errors = check(source);
    assert!(errors.is_empty());
}

#[test]
fn test_check_reports_unsupported_multi_level_relative_import() {
    let source = r#"
from ..helper import value

def main():
    print(value())
"#;
    let errors = check(source);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported relative import level 2")));
}

#[test]
fn test_check_reports_unsupported_bare_relative_import() {
    let source = r#"
from . import helper

def main():
    print(helper)
"#;
    let errors = check(source);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported bare relative import")));
}

#[test]
fn test_check_reports_unsupported_import_statement() {
    let source = r#"
import helper

def main():
    print("ok")
"#;
    let errors = check(source);
    assert!(errors.iter().any(|e| e
        .message
        .contains("unsupported import statement 'import helper'")));
}
