use crate::{
    apply_diagnostic_recovery_limits, diagnostic_label_for_code, render_package_diagnostic,
};
use sifr_diagnostics::DiagnosticArg;
use sifr_diagnostics::DiagnosticCode;
use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic, Severity};
use sifr_package::{CargoPackageId, PackageDiagnostic};
use std::path::Path;

fn test_diagnostic(
    code: &str,
    message: String,
    span: Option<DiagnosticSpan>,
) -> RenderedDiagnostic {
    RenderedDiagnostic {
        code: code.to_string(),
        severity: Severity::Error,
        message,
        message_template: "{message}".to_string(),
        args: std::collections::BTreeMap::new(),
        url: format!("https://docs.sifr.sh/errors/{code}"),
        spans: span.into_iter().collect(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn primary_test_span(file: &str, line: u32, column: u32) -> DiagnosticSpan {
    let byte_start = (line.saturating_sub(1) * 100) + column.saturating_sub(1);
    DiagnosticSpan {
        file: Some(file.to_string()),
        byte_start,
        byte_end: byte_start + 1,
        line: Some(line),
        column: Some(column),
        end_line: Some(line),
        end_column: Some(column),
        is_primary: true,
        label: None,
        lines: Vec::new(),
    }
}

#[test]
fn test_render_package_diagnostic_preserves_manifest_origin_and_help() {
    let diagnostic = PackageDiagnostic::manifest_exports_not_production(
        &CargoPackageId("path+file:///demo#pkg@0.1.0".to_string()),
        Path::new("/demo/sifr.toml"),
    );

    let rendered = render_package_diagnostic(diagnostic);

    assert_eq!(
        rendered.code,
        DiagnosticCode::PACKAGE_MANIFEST_EXPORTS_NOT_PRODUCTION.code()
    );
    assert_eq!(
        rendered.args.get("origin_kind"),
        Some(&DiagnosticArg::String("sifr_manifest".to_string()))
    );
    assert_eq!(
        rendered.args.get("manifest_path"),
        Some(&DiagnosticArg::String("/demo/sifr.toml".to_string()))
    );
    assert_eq!(
        rendered.args.get("manifest_key"),
        Some(&DiagnosticArg::String("exports.modules".to_string()))
    );
    assert!(rendered.help.is_some());
}

#[test]
fn test_apply_diagnostic_recovery_limits_deduplicates_exact_recovery_keys() {
    let duplicate_span = primary_test_span("main.sifr", 3, 5);
    let diagnostics = vec![
        test_diagnostic(
            "SIFR-TYPE-0002",
            "first duplicate".to_string(),
            Some(duplicate_span.clone()),
        ),
        test_diagnostic(
            "SIFR-TYPE-0002",
            "second duplicate".to_string(),
            Some(duplicate_span),
        ),
        test_diagnostic(
            "SIFR-TYPE-0002",
            "same line but distinct range".to_string(),
            Some(primary_test_span("main.sifr", 3, 6)),
        ),
    ];

    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 2);
    assert_eq!(bounded[0].message, "first duplicate");
    assert_eq!(bounded[1].message, "same line but distinct range");
}

#[test]
fn test_apply_diagnostic_recovery_limits_uses_registry_dedupe_args_only() {
    let mut first = test_diagnostic(
        DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY.code(),
        "5 additional diagnostics omitted by recovery cap (top-level diagnostic stream)"
            .to_string(),
        None,
    );
    first.severity = Severity::Note;
    first.message_template =
        "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})"
            .to_string();
    first
        .args
        .insert("omitted_count".to_string(), DiagnosticArg::Unsigned(5));
    first.args.insert(
        "omitted_kind".to_string(),
        DiagnosticArg::String("diagnostics".to_string()),
    );
    first.args.insert(
        "cap_kind".to_string(),
        DiagnosticArg::String("top-level diagnostic stream".to_string()),
    );

    let mut second = first.clone();
    second.severity = Severity::Note;
    second.message =
        "9 additional diagnostics omitted by recovery cap (top-level diagnostic stream)"
            .to_string();
    second
        .args
        .insert("omitted_count".to_string(), DiagnosticArg::Unsigned(9));

    let bounded = apply_diagnostic_recovery_limits(&[first, second]);
    assert_eq!(bounded.len(), 1);
    assert_eq!(
        bounded[0].args.get("omitted_count"),
        Some(&DiagnosticArg::Unsigned(5))
    );
}

#[test]
fn test_compiler_diagnostic_has_stable_code_and_url() {
    let diagnostic = crate::diagnostics::diagnostic_with_code(
        "unexpected token",
        DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
    );
    assert_eq!(diagnostic.code, "SIFR-PARSE-0002");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.url,
        "https://docs.sifr.sh/errors/SIFR-PARSE-0002"
    );
    assert_eq!(diagnostic.message, "unexpected token");
}

#[test]
fn test_diagnostic_labels_are_derived_from_diagnostic_codes() {
    let cases = [
        (
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
            "internal compiler error",
        ),
        (DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY, "note"),
        (DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE, "build error"),
        (DiagnosticCode::STDLIB_CACHE_FAILURE, "build error"),
        (DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, "type error"),
        (DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST, "build error"),
        (DiagnosticCode::WORKSPACE_UNRESOLVED_IMPORT, "build error"),
        (DiagnosticCode::WORKSPACE_IMPORT_CYCLE, "build error"),
        (
            DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
            "parse error",
        ),
        (DiagnosticCode::BUILD_MATERIALIZATION_FAILURE, "build error"),
        (DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE, "build error"),
        (DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE, "build error"),
        (DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE, "build error"),
        (DiagnosticCode::BUILD_ARTIFACT_MISSING, "build error"),
        (DiagnosticCode::FMT_FORMATTING_DRIFT, "format error"),
        (DiagnosticCode::LINT_TRAILING_WHITESPACE, "lint warning"),
        (DiagnosticCode::TYPE_MISMATCH, "type error"),
    ];

    for (code, label) in cases {
        assert_eq!(diagnostic_label_for_code(code), label);
        assert_eq!(
            crate::diagnostics::diagnostic_legacy_display(
                &crate::diagnostics::diagnostic_with_code("message", code)
            ),
            format!("{label}: message")
        );
    }
}

#[test]
fn test_compiler_diagnostics_preserve_order() {
    let diagnostics = [
        crate::diagnostics::diagnostic_with_code("first", DiagnosticCode::TYPE_MISMATCH),
        crate::diagnostics::diagnostic_with_code(
            "second",
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        ),
    ];
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "first");
    assert_eq!(diagnostics[1].message, "second");
    assert_eq!(diagnostics[0].code, "SIFR-TYPE-0002");
    assert_eq!(diagnostics[1].code, "SIFR-BUILD-0002");
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
            "module 'helpers.nodes' resolves to file '/tmp/ws/lib/helpers/nodes.sifr' but parent name 'helpers' is also a module file '/tmp/ws/lib/helpers.sifr'; package directories are not supported",
            DiagnosticCode::WORKSPACE_NAMESPACE_COLLISION,
        ),
    ];

    for (message, code) in cases {
        let diagnostic = crate::diagnostics::diagnostic_with_code(message, code);
        assert_eq!(diagnostic.code, code.code());
        assert_eq!(
            diagnostic.url,
            format!("https://docs.sifr.sh/errors/{}", code.code())
        );
    }
}

#[test]
fn test_workspace_codes_do_not_derive_from_message_prefixes() {
    let diagnostic = crate::diagnostics::diagnostic_with_code(
        "could not resolve import 'helper'; this looks like a workspace diagnostic",
        DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
    );

    assert_eq!(diagnostic.code, "SIFR-BUILD-0003");
}

#[test]
fn test_apply_diagnostic_recovery_limits_summarizes_similar_diagnostics() {
    let mut diagnostics = Vec::new();
    for idx in 0..8 {
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            "type mismatch: expected 'int', got 'str'".to_string(),
            Some(primary_test_span("main.sifr", idx + 1, 1)),
        ));
    }
    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 6);
    assert!(bounded
        .iter()
        .take(5)
        .all(|diag| diag.message == "type mismatch: expected 'int', got 'str'"));
    assert_eq!(bounded[5].code, "SIFR-INTERNAL-0002");
    assert_eq!(bounded[5].severity, Severity::Note);
    assert_eq!(
        bounded[5].message,
        "3 additional diagnostics omitted by recovery cap (similar-diagnostic group)"
    );
    assert!(bounded[5].spans.is_empty());
    assert_eq!(
        bounded[5].args.get("omitted_count"),
        Some(&DiagnosticArg::Unsigned(3))
    );
}

#[test]
fn test_apply_diagnostic_recovery_limits_keeps_distinct_reveal_types_until_top_level_cap() {
    let mut diagnostics = Vec::new();
    for idx in 0..60 {
        let mut diagnostic = test_diagnostic(
            DiagnosticCode::TYPE_REVEAL_TYPE.code(),
            format!("revealed type is T{idx}"),
            Some(primary_test_span("main.sifr", idx + 1, 1)),
        );
        diagnostic.severity = Severity::Note;
        diagnostic.message_template = "revealed type is {revealed_type}".to_string();
        diagnostic.args.insert(
            "revealed_type".to_string(),
            DiagnosticArg::String(format!("T{idx}")),
        );
        diagnostics.push(diagnostic);
    }

    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 50);
    assert_eq!(
        bounded
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::TYPE_REVEAL_TYPE.code())
            .count(),
        49
    );
    assert_eq!(
        bounded.last().map(|diagnostic| diagnostic.message.as_str()),
        Some("11 additional reveal_type results omitted by recovery cap (top-level diagnostic stream)")
    );
}

#[test]
fn test_apply_diagnostic_recovery_limits_reports_reveal_type_kind_in_similar_group_cap() {
    let mut diagnostics = Vec::new();
    for idx in 0..8 {
        let mut diagnostic = test_diagnostic(
            DiagnosticCode::TYPE_REVEAL_TYPE.code(),
            "revealed type is Repeated".to_string(),
            Some(primary_test_span("main.sifr", idx + 1, 1)),
        );
        diagnostic.severity = Severity::Note;
        diagnostic.message_template = "revealed type is {revealed_type}".to_string();
        diagnostic.args.insert(
            "revealed_type".to_string(),
            DiagnosticArg::String("Repeated".to_string()),
        );
        diagnostics.push(diagnostic);
    }

    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 6);
    let summary = bounded.last().expect("similar group cap should summarize");
    assert_eq!(
        summary.message,
        "3 additional reveal_type results omitted by recovery cap (similar-diagnostic group)"
    );
    assert_eq!(
        summary.args.get("omitted_kind"),
        Some(&DiagnosticArg::String("reveal_type results".to_string()))
    );
}

#[test]
fn test_apply_diagnostic_recovery_limits_reports_reveal_type_count_in_mixed_top_level_cap() {
    let mut diagnostics: Vec<RenderedDiagnostic> = (0..55)
        .map(|idx| test_diagnostic(&format!("SIFR-TYPE-{idx:04}"), format!("error {idx}"), None))
        .collect();
    for idx in 0..5 {
        let mut diagnostic = test_diagnostic(
            DiagnosticCode::TYPE_REVEAL_TYPE.code(),
            format!("revealed type is T{idx}"),
            Some(primary_test_span("main.sifr", idx + 1, 1)),
        );
        diagnostic.severity = Severity::Note;
        diagnostic.message_template = "revealed type is {revealed_type}".to_string();
        diagnostic.args.insert(
            "revealed_type".to_string(),
            DiagnosticArg::String(format!("T{idx}")),
        );
        diagnostics.push(diagnostic);
    }

    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 50);
    assert_eq!(
        bounded.last().map(|diagnostic| diagnostic.message.as_str()),
        Some(
            "11 additional diagnostics (including 5 reveal_type results) omitted by recovery cap (top-level diagnostic stream)"
        )
    );
    assert_eq!(
        bounded
            .last()
            .and_then(|diagnostic| diagnostic.args.get("omitted_kind")),
        Some(&DiagnosticArg::String(
            "diagnostics (including 5 reveal_type results)".to_string()
        ))
    );
}

#[test]
fn test_apply_diagnostic_recovery_limits_caps_top_level_diagnostics() {
    let diagnostics: Vec<RenderedDiagnostic> = (0..60)
        .map(|idx| test_diagnostic(&format!("SIFR-TYPE-{idx:04}"), format!("error {idx}"), None))
        .collect();
    let bounded = apply_diagnostic_recovery_limits(&diagnostics);
    assert_eq!(bounded.len(), 50);
    assert_eq!(bounded[49].code, "SIFR-INTERNAL-0002");
    assert_eq!(bounded[49].severity, Severity::Note);
    assert_eq!(
        bounded[49].message,
        "11 additional diagnostics omitted by recovery cap (top-level diagnostic stream)"
    );
}
