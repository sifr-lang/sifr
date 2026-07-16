use crate::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
    CompileResult, CompileResultFull,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_frontend::SourceOrigin;

fn assert_check_compile_error_parity(source: &str, expected_code: DiagnosticCode) {
    let check_errors = type_check_source(source);
    assert!(check_errors
        .iter()
        .any(|error| error.code == expected_code.code()));

    let CompileResult::Errors {
        errors: compile_errors,
    } = compile(source)
    else {
        panic!("invalid source must not reach code generation");
    };
    assert_eq!(
        check_errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>(),
        compile_errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>()
    );
}

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
fn test_parse_source_surfaces_malformed_integer_token_as_typed_diagnostic() {
    let errors = parse_source("def main():\n    value = 0123\n")
        .expect_err("malformed integer token should fail parsing");
    let diagnostic = errors
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::PARSE_LEXICAL_OR_STRING.code())
        .expect("expected lexical parser diagnostic");

    assert_eq!(
        diagnostic.args.get("parser_category"),
        Some(&DiagnosticArg::String("lexical_other".to_string()))
    );
    assert!(matches!(
        diagnostic.args.get("reason"),
        Some(DiagnosticArg::String(reason)) if reason.contains("Invalid decimal integer literal")
    ));
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
fn test_duplicate_python_error_fields_fail_check_and_compile_consistently() {
    let source = r#"
class PythonError(Error):
    message: str
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.buffer(builtins.bytearray, access=read, layout=any)
def view(size: int) -> Result[python.Buffer[uint8], PythonError]: ...
"#;
    let check_errors = type_check_source(source);
    assert!(check_errors.iter().any(|error| {
        error.code == DiagnosticCode::PYZC_INVALID_DECLARATION.code()
            && error
                .message
                .contains("canonical `PythonError` field contract")
    }));

    let CompileResult::Errors {
        errors: compile_errors,
    } = compile(source)
    else {
        panic!("duplicate PythonError fields must not reach code generation");
    };
    assert_eq!(
        check_errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>(),
        compile_errors
            .iter()
            .map(crate::diagnostics::diagnostic_legacy_display)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_python_declaration_shadow_error_fails_check_and_compile_consistently() {
    let source = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str
    code: int

@python(pkg.compute)
def compute(value: int) -> Result[int, PythonError]: ...
"#;
    assert_check_compile_error_parity(source, DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE);
}

#[test]
fn test_local_object_shadow_fails_check_and_compile_consistently() {
    let source = r#"
class Object:
    pass

class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.buffer(builtins.memoryview, access=read, layout=any)
def view(owner: Object) -> Result[python.Buffer[uint8], PythonError]: ...
"#;
    assert_check_compile_error_parity(source, DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE);
}

#[test]
fn test_local_object_record_uses_record_conversion_instead_of_sealed_handle() {
    let source = r#"
class Object:
    value: int

class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@python.buffer(builtins.memoryview, access=read, layout=any)
def view(owner: Object) -> Result[python.Buffer[uint8], PythonError]: ...
"#;
    assert!(type_check_source(source).is_empty());
    let CompileResult::Success { rust_source } = compile(source) else {
        panic!("same-named local record should compile through record conversion");
    };
    assert!(rust_source.contains("from_record_results"), "{rust_source}");
}

#[test]
fn test_imported_python_object_identity_reaches_check_and_compile() {
    let source = r#"
from sifr.python import Object, PythonError

@python.buffer(builtins.memoryview, access=write, layout=any)
def view(own owner: Object) -> Result[python.Buffer[uint8], PythonError]: ...
"#;
    assert!(type_check_source(source).is_empty());
    assert!(matches!(compile(source), CompileResult::Success { .. }));
}

#[test]
fn test_type_check_source_surfaces_reveal_type_as_structured_note() {
    let diagnostics = type_check_source("def main():\n    reveal_type(1)\n");

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, DiagnosticCode::TYPE_REVEAL_TYPE.code());
    assert_eq!(diagnostic.severity, sifr_diagnostics::Severity::Note);
    assert_eq!(
        diagnostic.message_template,
        "revealed type is {revealed_type}"
    );
    assert_eq!(diagnostic.message, "revealed type is int");
    assert_eq!(
        diagnostic.args.get("revealed_type"),
        Some(&DiagnosticArg::String("int".to_string()))
    );
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("reveal_type note should carry a primary span for source-backed checks");
    assert_eq!(primary_span.file.as_deref(), Some("main"));
    assert_eq!(primary_span.line, Some(2));
    assert!(primary_span.byte_end > primary_span.byte_start);
}

#[test]
fn test_type_check_source_surfaces_arithmetic_warning_as_structured_warning() {
    let diagnostics = type_check_source("def multiply(a: int, b: int) -> int:\n    return a * b\n");

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK.code()
    );
    assert_eq!(diagnostic.severity, sifr_diagnostics::Severity::Warning);
    assert_eq!(
        diagnostic.message_template,
        "integer {operation} may overflow at runtime"
    );
    assert_eq!(
        diagnostic.message,
        "integer multiplication may overflow at runtime"
    );
    assert_eq!(
        diagnostic.args.get("operation"),
        Some(&DiagnosticArg::String("multiplication".to_string()))
    );
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("arithmetic warning should carry a primary span");
    assert_eq!(primary_span.file.as_deref(), Some("main"));
    assert_eq!(primary_span.line, Some(2));
    assert!(primary_span.byte_end > primary_span.byte_start);
}

#[test]
fn test_type_check_source_surfaces_blocking_io_direct_call_error() {
    let diagnostics = type_check_source(
        r"@blocking_io
def read_file() -> int:
    return 1

async def main() -> None:
    value: int = read_file()
    await task.sleep(0.0)
    return None
",
    );

    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL.code())
        .expect("workload annotation should produce an async-context error");
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL.code()
    );
    assert_eq!(diagnostic.severity, sifr_diagnostics::Severity::Error);
    assert_eq!(diagnostic.message_template, "{message}");
    assert_eq!(
        diagnostic.message,
        "blocking_io function 'read_file' called directly from async context; use an async API or task.spawn_blocking"
    );
    assert_eq!(
        diagnostic.args.get("message"),
        Some(&DiagnosticArg::String(
            "blocking_io function 'read_file' called directly from async context; use an async API or task.spawn_blocking".to_string()
        ))
    );
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("workload error should carry a primary span");
    assert_eq!(primary_span.file.as_deref(), Some("main"));
    assert_eq!(primary_span.line, Some(6));
    assert!(primary_span.byte_end > primary_span.byte_start);
}

#[test]
fn test_type_check_source_surfaces_unreachable_statement_as_structured_warning() {
    let diagnostics = type_check_source("def value() -> int:\n    return 1\n    return 2\n");

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::FLOW_UNREACHABLE_STATEMENT.code()
    );
    assert_eq!(diagnostic.severity, sifr_diagnostics::Severity::Warning);
    assert_eq!(diagnostic.message_template, "unreachable statement ignored");
    assert!(diagnostic.args.is_empty());
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("unreachable warning should carry a primary span");
    assert_eq!(primary_span.file.as_deref(), Some("main"));
    assert_eq!(primary_span.line, Some(3));
    assert!(primary_span.byte_end > primary_span.byte_start);
}

#[test]
fn test_type_check_source_surfaces_bigint_transition_warning() {
    let diagnostics = type_check_source("def main():\n    value: bigint = 1\n");

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS.code()
    );
    assert_eq!(diagnostic.severity, sifr_diagnostics::Severity::Warning);
    assert_eq!(
        diagnostic.message_template,
        "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values"
    );
    assert_eq!(
        diagnostic.message,
        "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values"
    );
    assert!(diagnostic.args.is_empty());
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("bigint transition warning should carry a primary span");
    assert_eq!(primary_span.file.as_deref(), Some("main"));
    assert_eq!(primary_span.line, Some(2));
    assert!(primary_span.byte_end > primary_span.byte_start);
}

#[test]
fn test_type_check_source_warns_for_bigint_constructor_call() {
    assert_single_bigint_transition_warning(
        "def main():\n    value = bigint(1)\n",
        2,
        "bigint constructor",
    );
}

fn assert_single_bigint_transition_warning(source: &str, line: u32, context: &str) {
    let diagnostics = type_check_source(source);

    assert_eq!(diagnostics.len(), 1, "{context}");
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS.code(),
        "{context}"
    );
    assert_eq!(
        diagnostic.severity,
        sifr_diagnostics::Severity::Warning,
        "{context}"
    );
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .unwrap_or_else(|| panic!("{context} should carry a primary span"));
    assert_eq!(primary_span.file.as_deref(), Some("main"), "{context}");
    assert_eq!(primary_span.line, Some(line), "{context}");
    assert!(primary_span.byte_end > primary_span.byte_start, "{context}");
}

#[test]
fn test_type_check_source_warns_for_bigint_typevar_constraint() {
    assert_single_bigint_transition_warning(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", bigint)\n",
        3,
        "positional TypeVar constraint",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_typevar_bound_keyword() {
    assert_single_bigint_transition_warning(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", bound=bigint)\n",
        3,
        "TypeVar bound keyword",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_typevar_constraints_tuple_keyword() {
    assert_single_bigint_transition_warning(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", constraints=(bigint, str))\n",
        3,
        "TypeVar constraints tuple keyword",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_typevar_constraints_name_keyword() {
    assert_single_bigint_transition_warning(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", constraints=bigint)\n",
        3,
        "TypeVar constraints name keyword",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_pep695_bound() {
    assert_single_bigint_transition_warning(
        "def identity[T: bigint](value: T) -> T:\n    return value\n",
        1,
        "PEP 695 bound",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_pep695_tuple_constraint() {
    assert_single_bigint_transition_warning(
        "def identity[T: (bigint, str)](value: T) -> T:\n    return value\n",
        1,
        "PEP 695 tuple constraint",
    );
}

#[test]
fn test_type_check_source_warns_once_for_bigint_class_pep695_bound() {
    assert_single_bigint_transition_warning(
        "class Box[T: bigint]:\n    value: T\n",
        1,
        "class PEP 695 bound",
    );
}

#[test]
fn test_type_check_source_warns_for_bigint_isinstance_target() {
    assert_single_bigint_transition_warning(
        "def main():\n    value: int = 1\n    if isinstance(value, bigint):\n        print(\"legacy\")\n",
        3,
        "isinstance target",
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
fn test_compile_metadata_reports_generated_source_map_origins() {
    let source = r#"
from sifr.random import randint

def main():
    try:
        x: int = randint(1, 3)
        print(x)
    except ValueError:
        print(0)
"#;
    match compile_with_metadata(source) {
        CompileResultFull::Success {
            generated_source_map,
            rust_source,
            ..
        } => {
            assert!(
                generated_source_map.iter().any(|file| {
                    file.origin == SourceOrigin::GeneratedSupport
                        && file.path == "src/main.rs#stdlib-preamble"
                        && file.source.contains("// --- stdlib: sifr.random ---")
                }),
                "generated source map: {generated_source_map:#?}"
            );
            assert!(generated_source_map.iter().any(|file| {
                file.origin == SourceOrigin::CompilerSynthetic
                    && file.path == "src/main.rs"
                    && file.source == rust_source
            }));
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
    assert!(errors.iter().any(|e| e
        .message
        .contains("unsupported import form: relative import level 2")));
}

#[test]
fn test_check_reports_unsupported_bare_relative_import() {
    let source = r#"
from . import helper

def main():
    print(helper)
"#;
    let errors = check(source);
    assert!(errors.iter().any(|e| e
        .message
        .contains("unsupported import form: bare relative import")));
}

#[test]
fn test_check_reports_unsupported_import_statement() {
    let source = r#"
import helper

def main():
    print("ok")
"#;
    let errors = check(source);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unsupported import form: import helper")));
}
