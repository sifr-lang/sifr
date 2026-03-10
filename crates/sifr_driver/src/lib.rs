//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build
//!
//! Stdlib `.sifr` files are embedded in the compiler binary via `include_str!`.
//! They are compiled before user code (two-phase compilation).

mod build;
mod diagnostics;
mod frontend;
mod project;
mod rooted_entrypoint;
mod stdlib;
mod test_runner;

pub use build::{build, build_project, check_project};
pub use diagnostics::{
    apply_diagnostic_recovery_limits, compile_errors_to_diagnostics, CompileError, CompilePhase,
    CompileResult, CompileResultFull, CompilerDiagnostic, DiagnosticChild, DiagnosticSpan,
    DiagnosticSuggestion, RelatedSpan, Severity, SuggestionKind,
};
pub use frontend::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
};
pub use sifr_codegen::LoweringStats;
pub use test_runner::run_tests;

#[cfg(test)]
pub(crate) use build::create_invocation_workspace;
#[cfg(test)]
pub(crate) use diagnostics::run_codegen_with_boundary;
#[cfg(test)]
pub(crate) use frontend::FrontendDiagnosticStyle;
#[cfg(test)]
pub(crate) use project::{
    assemble_project_main_rs, collect_project_hir_modules, compile_frontend_modules,
    compute_module_compile_order, discover_test_root_modules, parse_import_closure_modules,
    DiscoveryDiagnosticStyle,
};
#[cfg(test)]
pub(crate) use stdlib::compile_stdlib;
#[cfg(test)]
pub(crate) use test_runner::{compose_test_runner_lib, generate_test_runner_cargo_toml};

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_python_parser::parse_module;
    use sifr_type_system::Type;
    use std::collections::{BTreeSet, HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};

    #[test]
    fn test_run_codegen_with_boundary_reports_string_panic_as_codegen_error() {
        let err = run_codegen_with_boundary("panic boundary test", || {
            panic!("boom");
        })
        .expect_err("panic should be converted into a codegen error");
        assert!(matches!(err.phase, CompilePhase::Codegen));
        assert!(err.message.contains("panic boundary test: boom"));
    }

    #[test]
    fn test_run_codegen_with_boundary_reports_non_string_payload() {
        let err = run_codegen_with_boundary("panic boundary test", || {
            std::panic::panic_any(42_u8);
        })
        .expect_err("panic should be converted into a codegen error");
        assert!(matches!(err.phase, CompilePhase::Codegen));
        assert!(err
            .message
            .contains("panic boundary test: non-string panic payload"));
    }

    #[test]
    fn test_compile_error_to_diagnostic_has_stable_code_and_url() {
        let err = CompileError {
            message: "unexpected token".to_string(),
            phase: CompilePhase::Parse,
        };
        let diag = err.to_diagnostic();
        assert_eq!(diag.code, "SIFR-PARSE-0001");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.url, "https://sifr.dev/docs/errors/SIFR-PARSE-0001");
        assert_eq!(diag.message, "unexpected token");
    }

    #[test]
    fn test_compile_errors_to_diagnostics_preserves_order() {
        let errors = vec![
            CompileError {
                message: "first".to_string(),
                phase: CompilePhase::TypeCheck,
            },
            CompileError {
                message: "second".to_string(),
                phase: CompilePhase::Codegen,
            },
        ];
        let diagnostics = compile_errors_to_diagnostics(&errors);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message, "first");
        assert_eq!(diagnostics[1].message, "second");
        assert_eq!(diagnostics[0].code, "SIFR-TYPE-0001");
        assert_eq!(diagnostics[1].code, "SIFR-CODEGEN-0001");
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
        assert!(matches!(errors[0].phase, CompilePhase::Parse));
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
            .all(|error| matches!(error.phase, CompilePhase::TypeCheck)));

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
    fn test_apply_diagnostic_recovery_limits_summarizes_similar_diagnostics() {
        let mut diagnostics = Vec::new();
        for idx in 0..8 {
            diagnostics.push(CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Error,
                message: "type mismatch: expected 'int', got 'str'".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: Some(DiagnosticSpan {
                    file: Some("main.sifr".to_string()),
                    line: Some(idx + 1),
                    column: Some(1),
                }),
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            });
        }
        let bounded = apply_diagnostic_recovery_limits(&diagnostics);
        assert_eq!(bounded.len(), 6);
        assert!(bounded
            .iter()
            .take(5)
            .all(|d| d.message == "type mismatch: expected 'int', got 'str'"));
        assert_eq!(bounded[5].message, "... +3 more similar diagnostics");
    }

    #[test]
    fn test_apply_diagnostic_recovery_limits_caps_top_level_diagnostics() {
        let diagnostics: Vec<CompilerDiagnostic> = (0..60)
            .map(|idx| CompilerDiagnostic {
                code: format!("SIFR-TYPE-{:04}", idx),
                severity: Severity::Error,
                message: format!("error {idx}"),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            })
            .collect();
        let bounded = apply_diagnostic_recovery_limits(&diagnostics);
        assert_eq!(bounded.len(), 50);
    }

    fn parse_suite(source: &str) -> Vec<sifr_python_ast::Stmt> {
        let parsed = parse_module(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
        assert!(
            parsed.is_valid(),
            "invalid test source: {:?}",
            parsed.errors()
        );
        parsed.into_suite()
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
    fn test_check_only_reports_frontend_phases() {
        let source = r#"
def main():
    x: int = "hello"
"#;
        let errors = check(source);
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .all(|e| matches!(e.phase, CompilePhase::Parse | CompilePhase::TypeCheck)));
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

    #[test]
    fn test_compile_frontend_modules_uses_explicit_diagnostic_style() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
def main():
    print(missing_name)
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let bare_errors = compile_frontend_modules(
            &parsed_modules,
            stdlib_defs.clone(),
            FrontendDiagnosticStyle::Bare,
        )
        .err()
        .expect("bare diagnostic style should still report type errors");
        let prefixed_errors = compile_frontend_modules(
            &parsed_modules,
            stdlib_defs,
            FrontendDiagnosticStyle::ModulePrefixed,
        )
        .err()
        .expect("module-prefixed diagnostic style should report type errors");

        assert!(bare_errors
            .iter()
            .any(|e| !e.message.starts_with("[main] ")));
        assert!(prefixed_errors
            .iter()
            .all(|e| e.message.starts_with("[main] ")));
    }

    #[test]
    fn test_check_and_project_lowering_share_typecheck_contract() {
        let source = r#"
def main():
    print(unknown_symbol)
"#;
        let check_errors = check(source);
        assert!(!check_errors.is_empty(), "check should report type errors");

        let mut parsed_modules = HashMap::new();
        parsed_modules.insert("main".to_string(), parse_suite(source));
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let project_errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should report same frontend type errors");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.message).collect();
        let normalized_project_messages: Vec<String> = project_errors
            .into_iter()
            .map(|e| {
                e.message
                    .strip_prefix("[main] ")
                    .unwrap_or(&e.message)
                    .to_string()
            })
            .collect();
        assert_eq!(check_messages, normalized_project_messages);
    }

    #[test]
    fn test_create_invocation_workspace_returns_unique_paths() {
        let first = create_invocation_workspace("workspace_unique")
            .expect("first workspace should be created");
        let second = create_invocation_workspace("workspace_unique")
            .expect("second workspace should be created");
        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());

        let _ = std::fs::remove_dir_all(first);
        let _ = std::fs::remove_dir_all(second);
    }

    #[test]
    fn test_collect_project_modules_supports_single_level_relative_import() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from .helper import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
def value() -> int:
    return 42
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("single-level relative imports should resolve in project lowering");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("helper"));
    }

    #[test]
    fn test_collect_project_modules_allows_non_main_stdlib_imports() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from helper import area_like

def main():
    print(area_like(2.0))
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
from sifr.math import pi

def area_like(r: float) -> float:
    return r * pi
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve non-main stdlib imports");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("helper"));
    }

    #[test]
    fn test_collect_project_modules_resolves_non_main_local_dependencies() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a_consumer import fetch

def main():
    print(fetch())
"#,
            ),
        );
        parsed_modules.insert(
            "a_consumer".to_string(),
            parse_suite(
                r#"
from z_provider import value

def fetch() -> int:
    return value()
"#,
            ),
        );
        parsed_modules.insert(
            "z_provider".to_string(),
            parse_suite(
                r#"
def value() -> int:
    return 41
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve non-main local imports");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("a_consumer"));
        assert!(result.hir_modules.contains_key("z_provider"));
    }

    #[test]
    fn test_compute_module_compile_order_is_dependency_safe() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
            ),
        );
        parsed_modules.insert(
            "provider".to_string(),
            parse_suite(
                r#"
def value_provider() -> int:
    return 42
"#,
            ),
        );

        let order = compute_module_compile_order(&parsed_modules)
            .expect("compile order should be computed for acyclic graph");
        assert_eq!(
            order,
            vec![
                "provider".to_string(),
                "consumer".to_string(),
                "main".to_string()
            ]
        );
    }

    #[test]
    fn test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order() {
        let mut parsed_modules_a = HashMap::new();
        parsed_modules_a.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules_a.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
            ),
        );
        parsed_modules_a.insert(
            "provider".to_string(),
            parse_suite(
                r#"
def value_provider() -> int:
    return 42
"#,
            ),
        );

        let mut parsed_modules_b = HashMap::new();
        parsed_modules_b.insert(
            "provider".to_string(),
            parse_suite(
                r#"
def value_provider() -> int:
    return 42
"#,
            ),
        );
        parsed_modules_b.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules_b.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
            ),
        );

        let order_a = compute_module_compile_order(&parsed_modules_a)
            .expect("compile order should be computed for acyclic graph");
        let order_b = compute_module_compile_order(&parsed_modules_b)
            .expect("compile order should be deterministic across map insertion order");
        assert_eq!(order_a, order_b);
        assert_eq!(
            order_a,
            vec![
                "provider".to_string(),
                "consumer".to_string(),
                "main".to_string()
            ]
        );
    }

    #[test]
    fn test_assemble_project_main_rs_is_deterministic_against_hashmap_order() {
        let compile_order = vec![
            "provider".to_string(),
            "consumer".to_string(),
            "main".to_string(),
        ];

        let mut rust_files_a = HashMap::new();
        rust_files_a.insert("main".to_string(), "fn main() {}\n".to_string());
        rust_files_a.insert("consumer".to_string(), "pub fn c() {}\n".to_string());
        rust_files_a.insert("provider".to_string(), "pub fn p() {}\n".to_string());

        let mut rust_files_b = HashMap::new();
        rust_files_b.insert("provider".to_string(), "pub fn p() {}\n".to_string());
        rust_files_b.insert("main".to_string(), "fn main() {}\n".to_string());
        rust_files_b.insert("consumer".to_string(), "pub fn c() {}\n".to_string());

        let main_a = assemble_project_main_rs(&compile_order, &rust_files_a);
        let main_b = assemble_project_main_rs(&compile_order, &rust_files_b);
        assert_eq!(main_a, main_b);
        assert_eq!(main_a, "mod provider;\nmod consumer;\n\nfn main() {}\n");
    }

    #[test]
    fn test_collect_project_modules_reports_unknown_module_in_non_main() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from helper import get

def main():
    print(get())
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
from missing_mod import value

def get() -> int:
    return value()
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should fail when non-main imports missing module");
        assert!(errors
            .iter()
            .any(|e| e.message.contains("unknown module 'missing_mod'")));
    }

    #[test]
    fn test_collect_project_modules_cycle_reports_error() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a import value_a

def main():
    print(value_a())
"#,
            ),
        );
        parsed_modules.insert(
            "a".to_string(),
            parse_suite(
                r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
            ),
        );
        parsed_modules.insert(
            "b".to_string(),
            parse_suite(
                r#"
from a import value_a

def value_b() -> int:
    return value_a()
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should fail when there is a dependency cycle");
        assert!(errors
            .iter()
            .any(|e| e.message.contains("module dependency cycle detected")));
        assert!(errors.iter().any(|e| e.message.contains("a -> b -> a")));
    }

    #[test]
    fn test_compute_module_compile_order_cycle_diagnostics_are_canonical_and_stable() {
        let mut parsed_modules_a = HashMap::new();
        parsed_modules_a.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a import value_a

def main():
    print(value_a())
"#,
            ),
        );
        parsed_modules_a.insert(
            "a".to_string(),
            parse_suite(
                r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
            ),
        );
        parsed_modules_a.insert(
            "b".to_string(),
            parse_suite(
                r#"
from c import value_c

def value_b() -> int:
    return value_c()
"#,
            ),
        );
        parsed_modules_a.insert(
            "c".to_string(),
            parse_suite(
                r#"
from a import value_a

def value_c() -> int:
    return value_a()
"#,
            ),
        );

        let mut parsed_modules_b = HashMap::new();
        parsed_modules_b.insert(
            "c".to_string(),
            parse_suite(
                r#"
from a import value_a

def value_c() -> int:
    return value_a()
"#,
            ),
        );
        parsed_modules_b.insert(
            "b".to_string(),
            parse_suite(
                r#"
from c import value_c

def value_b() -> int:
    return value_c()
"#,
            ),
        );
        parsed_modules_b.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a import value_a

def main():
    print(value_a())
"#,
            ),
        );
        parsed_modules_b.insert(
            "a".to_string(),
            parse_suite(
                r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
            ),
        );

        let error_a = compute_module_compile_order(&parsed_modules_a)
            .err()
            .expect("cycle graph should fail compile ordering");
        let error_b = compute_module_compile_order(&parsed_modules_b)
            .err()
            .expect("cycle graph should fail compile ordering");

        let message_a = &error_a[0].message;
        let message_b = &error_b[0].message;
        assert_eq!(message_a, message_b);
        assert!(message_a.contains("module dependency cycle detected: a -> b -> c -> a"));
        assert!(message_a.contains("import chain: a imports b, b imports c, c imports a"));
    }

    #[test]
    fn test_discover_test_root_modules_is_deterministic() {
        let unique = format!(
            "sifr_test_root_discovery_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("test dir should be created");
        std::fs::write(dir.join("z_test.sifr"), "def test_z():\n    assert True\n")
            .expect("z_test should be written");
        std::fs::write(dir.join("test_a.sifr"), "def test_a():\n    assert True\n")
            .expect("test_a should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def helper() -> int:\n    return 1\n",
        )
        .expect("helper should be written");

        let roots = discover_test_root_modules(&dir);
        let names: Vec<String> = roots.keys().cloned().collect();
        assert_eq!(names, vec!["test_a".to_string(), "z_test".to_string()]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_project_and_test_discovery_share_import_closure_membership() {
        let unique = format!(
            "sifr_discovery_parity_positive_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("test_parity.sifr"),
            "from helper import value\n\ndef test_value():\n    assert value() == 42\n",
        )
        .expect("test_parity should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "from shared import BASE\n\ndef value() -> int:\n    return BASE\n",
        )
        .expect("helper should be written");
        std::fs::write(dir.join("shared.sifr"), "BASE: int = 42\n")
            .expect("shared should be written");
        std::fs::write(dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
            .expect("unrelated sibling should be written");

        let project_roots = BTreeSet::from(["main".to_string()]);
        let test_roots = BTreeSet::from(["test_parity".to_string()]);
        let project_modules = parse_import_closure_modules(
            &dir,
            &project_roots,
            DiscoveryDiagnosticStyle::ModuleName,
        )
        .expect("project closure discovery should succeed");
        let test_modules =
            parse_import_closure_modules(&dir, &test_roots, DiscoveryDiagnosticStyle::ModuleName)
                .expect("test closure discovery should succeed");

        let project_support: BTreeSet<String> = project_modules
            .keys()
            .filter(|name| !project_roots.contains(*name))
            .cloned()
            .collect();
        let test_support: BTreeSet<String> = test_modules
            .keys()
            .filter(|name| !test_roots.contains(*name))
            .cloned()
            .collect();

        assert_eq!(
            project_support,
            BTreeSet::from(["helper".to_string(), "shared".to_string()])
        );
        assert_eq!(project_support, test_support);
        assert!(!project_support.contains("unrelated_bad"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_project_and_test_discovery_parity_reports_reachable_parse_errors() {
        let unique = format!(
            "sifr_discovery_parity_negative_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("test_parity.sifr"),
            "from helper import value\n\ndef test_value():\n    assert value() == 1\n",
        )
        .expect("test_parity should be written");
        std::fs::write(dir.join("helper.sifr"), "def value(:\n").expect("helper should be written");
        std::fs::write(
            dir.join("unrelated_ok.sifr"),
            "def spare() -> int:\n    return 1\n",
        )
        .expect("unrelated should be written");

        let project_roots = BTreeSet::from(["main".to_string()]);
        let test_roots = BTreeSet::from(["test_parity".to_string()]);

        let project_errors = parse_import_closure_modules(
            &dir,
            &project_roots,
            DiscoveryDiagnosticStyle::ModuleName,
        )
        .err()
        .expect("project closure should fail on reachable parse error");
        let test_errors =
            parse_import_closure_modules(&dir, &test_roots, DiscoveryDiagnosticStyle::ModuleName)
                .err()
                .expect("test closure should fail on reachable parse error");

        assert!(project_errors
            .iter()
            .any(|e| e.message.contains("[helper]")));
        assert!(test_errors.iter().any(|e| e.message.contains("[helper]")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_project_modules_exports_local_constants() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import get

def main():
    print(get())
"#,
            ),
        );
        parsed_modules.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from constants_mod import ANSWER

def get() -> int:
    return ANSWER
"#,
            ),
        );
        parsed_modules.insert(
            "constants_mod".to_string(),
            parse_suite(
                r#"
ANSWER: int = 42
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve local constant imports");
        let constants = result
            .external_defs
            .constants
            .get("constants_mod")
            .expect("constants module exports should exist");
        assert_eq!(constants.get("ANSWER"), Some(&Type::Int));
    }

    #[test]
    fn test_run_tests_resolves_local_imports_and_constants() {
        let unique = format!(
            "sifr_test_import_parity_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(
            test_dir.join("helper.sifr"),
            r#"
BASE: int = 9

def plus_one(x: int) -> int:
    return x + 1
"#,
        )
        .expect("helper module should be written");
        std::fs::write(
            test_dir.join("test_imports.sifr"),
            r#"
from helper import BASE, plus_one

def test_import_parity():
    assert plus_one(BASE) == 10
"#,
        )
        .expect("test module should be written");

        let result = run_tests(&test_dir).expect("test runner should compile and execute");
        assert!(result, "sifr test run should succeed");

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_run_tests_parallel_invocations_are_isolated() {
        fn make_test_dir(label: &str, expected: i64) -> PathBuf {
            let unique = format!(
                "sifr_test_parallel_isolation_{label}_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("time should move forward")
                    .as_nanos()
            );
            let test_dir = std::env::temp_dir().join(unique);
            std::fs::create_dir_all(&test_dir).expect("test dir should be created");
            std::fs::write(
                test_dir.join("helper.sifr"),
                format!("def value() -> int:\n    return {expected}\n"),
            )
            .expect("helper should be written");
            std::fs::write(
                test_dir.join("test_parallel.sifr"),
                format!(
                    "from helper import value\n\ndef test_value():\n    assert value() == {expected}\n"
                ),
            )
            .expect("test module should be written");
            test_dir
        }

        let first_dir = make_test_dir("first", 11);
        let second_dir = make_test_dir("second", 22);
        let barrier = Arc::new(Barrier::new(3));

        let first_barrier = Arc::clone(&barrier);
        let first_path = first_dir.clone();
        let first = std::thread::spawn(move || {
            first_barrier.wait();
            run_tests(&first_path)
        });

        let second_barrier = Arc::clone(&barrier);
        let second_path = second_dir.clone();
        let second = std::thread::spawn(move || {
            second_barrier.wait();
            run_tests(&second_path)
        });

        barrier.wait();
        let first_result = first.join().expect("first thread should join");
        let second_result = second.join().expect("second thread should join");
        assert!(
            matches!(first_result, Ok(true)),
            "first parallel run_tests invocation should pass: {first_result:?}"
        );
        assert!(
            matches!(second_result, Ok(true)),
            "second parallel run_tests invocation should pass: {second_result:?}"
        );

        let _ = std::fs::remove_dir_all(&first_dir);
        let _ = std::fs::remove_dir_all(&second_dir);
    }

    #[test]
    fn test_run_tests_ignores_unrelated_non_closure_parse_errors() {
        let unique = format!(
            "sifr_test_import_closure_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(
            test_dir.join("helper.sifr"),
            "def value() -> int:\n    return 42\n",
        )
        .expect("helper should be written");
        std::fs::write(
            test_dir.join("test_import_closure.sifr"),
            "from helper import value\n\ndef test_value():\n    assert value() == 42\n",
        )
        .expect("test module should be written");
        std::fs::write(test_dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
            .expect("unrelated sibling should be written");

        let result =
            run_tests(&test_dir).expect("unrelated sibling parse errors should be ignored");
        assert!(result, "sifr test run should succeed");

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_run_tests_reports_deterministic_parse_error_order() {
        let unique = format!(
            "sifr_test_parse_order_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(test_dir.join("test_z_bad.sifr"), "def z(:\n")
            .expect("test_z_bad should be written");
        std::fs::write(test_dir.join("test_a_bad.sifr"), "def a(:\n")
            .expect("test_a_bad should be written");

        let first_messages: Vec<String> = run_tests(&test_dir)
            .err()
            .expect("parse errors should be reported")
            .into_iter()
            .map(|e| e.message)
            .collect();
        let second_messages: Vec<String> = run_tests(&test_dir)
            .err()
            .expect("parse errors should be deterministic")
            .into_iter()
            .map(|e| e.message)
            .collect();

        assert_eq!(first_messages, second_messages);
        assert!(
            first_messages
                .first()
                .is_some_and(|m| m.contains("test_a_bad.sifr")),
            "first parse error should be from lexicographically first fixture: {first_messages:?}"
        );

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_run_tests_frontend_type_errors_use_single_path_prefix() {
        let unique = format!(
            "sifr_test_type_error_prefix_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(
            test_dir.join("helper.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        std::fs::write(
            test_dir.join("test_bad.sifr"),
            "from helper import value\n\ndef test_bad() -> int:\n    return \"bad\"\n",
        )
        .expect("bad test module should be written");

        let errors = run_tests(&test_dir)
            .err()
            .expect("type errors in test module should fail frontend");
        let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        assert!(messages.iter().all(|m| m.contains("test_bad.sifr")));
        assert!(messages
            .iter()
            .all(|m| !m.contains("] [test_bad] return type mismatch")));

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_check_project_resolves_valid_local_imports() {
        let unique = format!(
            "sifr_check_project_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            r#"
from helper import area

def main():
    print(area(2.0))
"#,
        )
        .expect("main module should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            r#"
from sifr.math import pi

def area(radius: float) -> float:
    return pi * radius * radius
"#,
        )
        .expect("helper module should be written");

        let errors = check_project(&dir.join("main.sifr"));
        assert!(
            errors.is_empty(),
            "check_project should succeed: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_project_ignores_unrelated_non_closure_parse_errors() {
        let unique = format!(
            "sifr_check_project_closure_ignore_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main module should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def value() -> int:\n    return 42\n",
        )
        .expect("helper module should be written");
        std::fs::write(dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
            .expect("unrelated sibling should be written");

        let errors = check_project(&dir.join("main.sifr"));
        assert!(
            errors.is_empty(),
            "unrelated sibling parse errors should not affect check_project: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_project_reports_reachable_parse_errors_in_import_closure() {
        let unique = format!(
            "sifr_check_project_closure_reachable_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main module should be written");
        std::fs::write(dir.join("helper.sifr"), "def value(:\n")
            .expect("helper module should be written");
        std::fs::write(
            dir.join("unrelated_ok.sifr"),
            "def spare() -> int:\n    return 1\n",
        )
        .expect("unrelated module should be written");

        let errors = check_project(&dir.join("main.sifr"));
        assert!(
            errors.iter().any(|e| {
                e.message.contains("[helper]")
                    && (e.message.contains("failed to parse")
                        || e.message.contains("Expected a parameter"))
            }),
            "reachable parse errors must still fail check_project: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_project_error_messages_match_build_project() {
        let unique = format!(
            "sifr_check_project_error_parity_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            r#"
from helper import broken

def main():
    print(broken())
"#,
        )
        .expect("main module should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            r#"
def broken() -> int:
    return "bad"
"#,
        )
        .expect("helper module should be written");

        let check_errors = check_project(&dir.join("main.sifr"));
        let build_errors = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
            .err()
            .expect("build_project should fail with same frontend error");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, build_messages);
        assert!(build_messages
            .iter()
            .any(|m| m.contains("[helper] return type mismatch")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_generate_test_runner_cargo_toml_includes_required_crates() {
        let stdlib_modules = HashSet::new();
        let required_crates = HashSet::from([
            "regex".to_string(),
            "rand".to_string(),
            "rand_distr".to_string(),
        ]);

        let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
        assert!(cargo_toml.contains("name = \"sifr_tests\""));
        assert!(cargo_toml.contains("regex = \"1\""));
        assert!(cargo_toml.contains("rand = \"0.8\""));
        assert!(cargo_toml.contains("rand_distr = \"0.4\""));
    }

    #[test]
    fn test_generate_test_runner_cargo_toml_preserves_stdlib_deps() {
        let stdlib_modules = HashSet::from(["sifr.json".to_string()]);
        let required_crates = HashSet::new();

        let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
        assert!(cargo_toml.contains("serde_json = \"1\""));
        assert!(cargo_toml.contains("serde = { version = \"1\", features = [\"derive\"] }"));
    }

    #[test]
    fn test_compose_test_runner_lib_is_test_scoped() {
        let support_modules = vec!["helper".to_string()];
        let all_rust_code = "#[test]\nfn smoke() {}\n";
        let lib_source = compose_test_runner_lib(&support_modules, all_rust_code);
        assert!(lib_source.starts_with("#![cfg(test)]"));
        assert!(lib_source.contains("mod helper;"));
        assert!(lib_source.contains("#[test]\nfn smoke() {}"));
    }
}
