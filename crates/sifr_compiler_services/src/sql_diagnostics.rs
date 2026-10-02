//! Shared structured SQL provider diagnostic rendering.
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::BTreeMap;

/// Types without a closed component descriptor fail identically before dispatch.
pub fn unsupported_hole_diagnostics(
    document: &sifr_frontend::SqlEditorDocumentView,
    source_document: &str,
) -> Vec<RenderedDiagnostic> {
    document.parameter_protocol_types.iter().enumerate().filter_map(|(index, ty)| {
        if ty.is_some() { return None; }
        let mapping = document.template.mappings.iter().find(|mapping| {
            matches!(mapping.kind, sifr_frontend::TemplateSourceMapKind::Interpolation { index: found } if found == index)
        })?;
        let mut diagnostic = crate::diagnostics::diagnostic_with_code(
            format!("SQL interpolation type '{}' is not component-safe", document.parameter_types[index]),
            sifr_diagnostics::DiagnosticCode::COMPONENT_PROTOCOL_ENVELOPE,
        );
        diagnostic.spans.push(render_provider_span(&sifr_compiler_component::SourceSpan {
            document: source_document.into(),
            start: mapping.source_range.start().to_u32(),
            end: mapping.source_range.end().to_u32(),
        }, true));
        Some(diagnostic)
    }).collect()
}

pub fn render_provider_diagnostic(
    diagnostic: &sifr_compiler_component::EmbeddedDiagnostic,
) -> RenderedDiagnostic {
    let severity = match diagnostic.severity {
        sifr_compiler_component::DiagnosticSeverity::Error => sifr_diagnostics::Severity::Error,
        sifr_compiler_component::DiagnosticSeverity::Warning => sifr_diagnostics::Severity::Warning,
        sifr_compiler_component::DiagnosticSeverity::Note => sifr_diagnostics::Severity::Note,
    };
    let message = diagnostic.message.clone();
    let mut spans = Vec::with_capacity(1 + diagnostic.related.len());
    spans.push(render_provider_span(&diagnostic.primary, true));
    spans.extend(
        diagnostic
            .related
            .iter()
            .map(|span| render_provider_span(span, false)),
    );
    RenderedDiagnostic {
        code: diagnostic.code.clone(),
        severity,
        message: message.clone(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([(
            "message".to_string(),
            sifr_diagnostics::DiagnosticArg::String(message),
        )]),
        url: format!("https://docs.sifr-lang.org/errors/{}", diagnostic.code),
        spans,
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn render_provider_span(
    span: &sifr_compiler_component::SourceSpan,
    is_primary: bool,
) -> sifr_diagnostics::DiagnosticSpan {
    sifr_diagnostics::DiagnosticSpan {
        file: Some(span.document.clone()),
        byte_start: span.start,
        byte_end: span.end,
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        is_primary,
        label: (!is_primary).then(|| "related SQL location".to_string()),
        lines: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_severity_identity_and_related_spans_survive_shared_rendering() {
        use sifr_compiler_component::{
            DiagnosticLifecycle, DiagnosticSeverity, EmbeddedDiagnostic, SourceSpan,
        };
        for (severity, expected) in [
            (DiagnosticSeverity::Error, sifr_diagnostics::Severity::Error),
            (
                DiagnosticSeverity::Warning,
                sifr_diagnostics::Severity::Warning,
            ),
            (DiagnosticSeverity::Note, sifr_diagnostics::Severity::Note),
        ] {
            let embedded = EmbeddedDiagnostic {
                code: "SIFR-SQLITE-0009".into(),
                severity,
                lifecycle: DiagnosticLifecycle::Active,
                message: "unsupported hole".into(),
                primary: SourceSpan {
                    document: "src/query.sifr".into(),
                    start: 82,
                    end: 89,
                },
                related: vec![SourceSpan {
                    document: "src/query.sifr".into(),
                    start: 50,
                    end: 55,
                }],
            };
            let rendered = render_provider_diagnostic(&embedded);
            assert_eq!(rendered.code, embedded.code);
            assert_eq!(rendered.severity, expected);
            assert_eq!(
                (rendered.spans[0].byte_start, rendered.spans[0].byte_end),
                (82, 89)
            );
            assert!(rendered.spans[0].is_primary);
            assert!(!rendered.spans[1].is_primary);
            assert_eq!(rendered.spans[1].file.as_deref(), Some("src/query.sifr"));
            if expected != sifr_diagnostics::Severity::Error {
                let warning = sifr_ir::LoweringWarningDiagnostic::External {
                    module: "src/query.sifr".into(),
                    diagnostic: Box::new(rendered.clone()),
                };
                assert_eq!(
                    sifr_frontend::warning_diagnostics(None, &[warning.clone()]),
                    vec![rendered.clone()]
                );
                let source = " ".repeat(100);
                let relocated = sifr_frontend::warning_diagnostics(
                    Some(sifr_frontend::FrontendSourceContext {
                        display_path: "src/actual.sifr",
                        source: &source,
                    }),
                    &[warning],
                );
                assert_eq!(relocated[0].severity, expected);
                assert_eq!(relocated[0].code, embedded.code);
                assert!(
                    relocated[0]
                        .spans
                        .iter()
                        .all(|span| span.file.as_deref() == Some("src/actual.sifr"))
                );
            }
        }
    }
}
