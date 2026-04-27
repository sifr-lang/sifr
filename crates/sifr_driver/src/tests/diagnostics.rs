use crate::{
    apply_diagnostic_recovery_limits, compile_errors_to_diagnostics, CompileError, CompilePhase,
    CompilerDiagnostic, DiagnosticSpan, Severity,
};

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
fn test_workspace_resolution_errors_have_stable_codes_and_urls() {
    let cases = [
        (
            "could not resolve import 'helper'; tried entry-relative '/tmp/helper.sifr' and workspace-relative '/tmp/lib/helper.sifr'",
            "SIFR-WORKSPACE-0101",
        ),
        (
            "module 'helper' is ambiguous in workspace '/tmp/ws': matches '/tmp/a/helper.sifr' and '/tmp/b/helper.sifr'; reorder [source].roots or rename one module to disambiguate",
            "SIFR-WORKSPACE-0102",
        ),
        (
            "module 'helpers.list_node' resolves to file '/tmp/ws/lib/helpers/list_node.sifr' but parent name 'helpers' is also a module file '/tmp/ws/lib/helpers.sifr'; package directories are not supported",
            "SIFR-WORKSPACE-0103",
        ),
    ];

    for (message, code) in cases {
        let diagnostic = CompileError {
            message: message.to_string(),
            phase: CompilePhase::Build,
        }
        .to_diagnostic();
        assert_eq!(diagnostic.code, code);
        assert_eq!(
            diagnostic.url,
            format!("https://sifr.dev/docs/errors/{code}")
        );
    }
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
