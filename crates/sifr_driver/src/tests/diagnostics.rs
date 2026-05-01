use crate::{
    apply_diagnostic_recovery_limits, compile_error_label_for_code, compile_errors_to_diagnostics,
    CompileError, CompilerDiagnostic, DiagnosticSpan, Severity,
};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn test_compile_error_to_diagnostic_has_stable_code_and_url() {
    let err = CompileError::with_code(
        "unexpected token",
        DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
    );
    let diag = err.to_diagnostic();
    assert_eq!(diag.code, "SIFR-PARSE-0002");
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.url, "https://sifr.sh/docs/errors/SIFR-PARSE-0002");
    assert_eq!(diag.message, "unexpected token");
}

#[test]
fn test_compile_error_labels_are_derived_from_diagnostic_codes() {
    let cases = [
        (
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
            "internal compiler error",
        ),
        (DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE, "build error"),
        (DiagnosticCode::STDLIB_CACHE_FAILURE, "build error"),
        (DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, "type error"),
        (DiagnosticCode::STDLIB_ARGUMENT_TYPE_MISMATCH, "type error"),
        (DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST, "build error"),
        (DiagnosticCode::WORKSPACE_UNRESOLVED_IMPORT, "build error"),
        (DiagnosticCode::WORKSPACE_IMPORT_CYCLE, "build error"),
        (
            DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
            "parse error",
        ),
        (DiagnosticCode::CODEGEN_BACKEND_FAILURE, "codegen error"),
        (DiagnosticCode::BUILD_MATERIALIZATION_FAILURE, "build error"),
        (DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE, "build error"),
        (DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE, "build error"),
        (DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE, "build error"),
        (DiagnosticCode::BUILD_ARTIFACT_MISSING, "build error"),
        (DiagnosticCode::TYPE_MISMATCH, "type error"),
    ];

    for (code, label) in cases {
        assert_eq!(compile_error_label_for_code(code), label);
        assert_eq!(
            CompileError::with_code("message", code).to_string(),
            format!("{label}: message")
        );
    }
}

#[test]
fn test_compile_errors_to_diagnostics_preserves_order() {
    let errors = vec![
        CompileError::with_code("first", DiagnosticCode::TYPE_MISMATCH),
        CompileError::with_code("second", DiagnosticCode::CODEGEN_BACKEND_FAILURE),
    ];
    let diagnostics = compile_errors_to_diagnostics(&errors);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "first");
    assert_eq!(diagnostics[1].message, "second");
    assert_eq!(diagnostics[0].code, "SIFR-TYPE-0002");
    assert_eq!(diagnostics[1].code, "SIFR-CODEGEN-0002");
}

#[test]
fn test_workspace_resolution_errors_have_stable_codes_and_urls() {
    let cases = [
        (
            "could not resolve import 'helper'; tried entry-relative '/tmp/helper.sifr' and workspace-relative '/tmp/lib/helper.sifr'",
            DiagnosticCode::WORKSPACE_UNRESOLVED_IMPORT,
        ),
        (
            "module 'helper' is ambiguous in workspace '/tmp/ws': matches '/tmp/a/helper.sifr' and '/tmp/b/helper.sifr'; reorder [source].roots or rename one module to disambiguate",
            DiagnosticCode::WORKSPACE_AMBIGUOUS_IMPORT,
        ),
        (
            "module 'helpers.list_node' resolves to file '/tmp/ws/lib/helpers/list_node.sifr' but parent name 'helpers' is also a module file '/tmp/ws/lib/helpers.sifr'; package directories are not supported",
            DiagnosticCode::WORKSPACE_NAMESPACE_COLLISION,
        ),
    ];

    for (message, code) in cases {
        let diagnostic = CompileError::with_code(message, code).to_diagnostic();
        assert_eq!(diagnostic.code, code.code());
        assert_eq!(
            diagnostic.url,
            format!("https://sifr.sh/docs/errors/{}", code.code())
        );
    }
}

#[test]
fn test_workspace_codes_do_not_derive_from_message_prefixes() {
    let diagnostic = CompileError::with_code(
        "could not resolve import 'helper'; this looks like a workspace diagnostic",
        DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
    )
    .to_diagnostic();

    assert_eq!(diagnostic.code, "SIFR-BUILD-0003");
}

#[test]
fn test_apply_diagnostic_recovery_limits_summarizes_similar_diagnostics() {
    let mut diagnostics = Vec::new();
    for idx in 0..8 {
        diagnostics.push(CompilerDiagnostic {
            code: "SIFR-TYPE-0002".to_string(),
            severity: Severity::Error,
            message: "type mismatch: expected 'int', got 'str'".to_string(),
            url: "https://sifr.sh/docs/errors/SIFR-TYPE-0002".to_string(),
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
        .all(|diag| diag.message == "type mismatch: expected 'int', got 'str'"));
    assert_eq!(bounded[5].message, "... +3 more similar diagnostics");
}

#[test]
fn test_apply_diagnostic_recovery_limits_caps_top_level_diagnostics() {
    let diagnostics: Vec<CompilerDiagnostic> = (0..60)
        .map(|idx| CompilerDiagnostic {
            code: format!("SIFR-TYPE-{:04}", idx),
            severity: Severity::Error,
            message: format!("error {idx}"),
            url: "https://sifr.sh/docs/errors/SIFR-TYPE-0002".to_string(),
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
