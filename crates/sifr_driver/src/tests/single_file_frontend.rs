use crate::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
    CompileResult, CompileResultFull,
};
use sifr_diagnostics::DiagnosticCode;

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
        DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY
    );
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
        .all(|error| error.code == DiagnosticCode::TYPE_MISMATCH));

    let check_errors = type_check_source("def main():\n    x: int = \"bad\"\n");
    assert_eq!(errors.len(), check_errors.len());
    assert_eq!(
        errors.iter().map(ToString::to_string).collect::<Vec<_>>(),
        check_errors
            .iter()
            .map(ToString::to_string)
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
        .all(|error| error.code == DiagnosticCode::TYPE_MISMATCH));
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
