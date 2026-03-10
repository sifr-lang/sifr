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
    mod discovery_and_workspace;
    mod project_build_check;
    mod project_graph;
    mod single_file_frontend;
    mod support;
    mod test_runner;

    use super::*;

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
}
