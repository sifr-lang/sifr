use sifr_diagnostics::{
    DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, RelatedKind,
    RenderedDiagnostic, Severity, SourceMap, SourceSpan,
};
use std::collections::BTreeMap;

use crate::FrontendSourceContext;
use ruff_text_size::TextRange;

pub(crate) fn diagnostic_with_source_range(
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let mut builder = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template(message_template);
    for (name, value) in args {
        builder = builder.arg(name, value.clone());
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    if code.declared_severity() == Severity::Error {
        let _ = sink.emit_error(diagnostic);
    } else {
        sink.emit(diagnostic);
    }
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => diagnostic_with_code(
            "internal compiler error: frontend diagnostic renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => diagnostic_with_code(
            format!("internal compiler error: invalid frontend diagnostic span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

pub(crate) fn diagnostic_with_source_range_args_help(
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
    message_template: &'static str,
    args: BTreeMap<String, DiagnosticArg>,
    extra_args: BTreeMap<String, DiagnosticArg>,
    help: Option<String>,
) -> RenderedDiagnostic {
    diagnostic_with_source_ranges_args_help(
        code,
        source_context,
        range,
        &[],
        message_template,
        args,
        extra_args,
        help,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn diagnostic_with_source_ranges_args_help(
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
    related_ranges: &[(TextRange, String)],
    message_template: &'static str,
    mut args: BTreeMap<String, DiagnosticArg>,
    extra_args: BTreeMap<String, DiagnosticArg>,
    help: Option<String>,
) -> RenderedDiagnostic {
    args.extend(extra_args);
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let mut builder = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template(message_template);
    for (name, value) in args {
        builder = builder.arg_owned(&name, value);
    }
    for (range, label) in related_ranges {
        builder = builder.related(
            SourceSpan::new(source_id, *range),
            RelatedKind::Note,
            Some(label.clone()),
        );
    }
    if let Some(help) = help {
        builder = builder.help(help);
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    if code.declared_severity() == Severity::Error {
        let _ = sink.emit_error(diagnostic);
    } else {
        sink.emit(diagnostic);
    }
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => diagnostic_with_code(
            "internal compiler error: frontend diagnostic renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => diagnostic_with_code(
            format!("internal compiler error: invalid frontend diagnostic span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

pub(crate) fn diagnostic_with_code(
    message: impl Into<String>,
    code: DiagnosticCode,
) -> RenderedDiagnostic {
    let message = message.into();
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: message.clone(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([("message".to_string(), DiagnosticArg::String(message))]),
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}
