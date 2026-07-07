use crate::diagnostics::RenderedDiagnostic;
use ruff_text_size::TextRange;
use sifr_diagnostics::render::render_sink;
use sifr_diagnostics::{
    ChildSeverity, DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, Severity,
    SourceMap, SourceSpan,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn source_diagnostic(
    code: DiagnosticCode,
    display_path: &str,
    source: &str,
    range: TextRange,
    message_template: &'static str,
    args: Vec<(&'static str, String)>,
    notes: Vec<String>,
    help: Option<String>,
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(display_path, source);
    let span = match SourceSpan::new_validated(&source_map, source_id, range) {
        Ok(span) => span,
        Err(error) => {
            return crate::diagnostics::diagnostic_with_code(
                format!("internal compiler error: invalid Rust interop diagnostic span: {error:?}"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            );
        }
    };
    let mut builder =
        DiagnosticBuilder::source(code, Severity::Error, span).message_template(message_template);
    for (name, value) in args {
        builder = builder.arg(name, DiagnosticArg::String(value));
    }
    for note in notes {
        builder = builder.child(ChildSeverity::Note, note);
    }
    if let Some(help) = help {
        builder = builder.help(help);
    }
    let mut sink = DiagnosticSink::new();
    sink.emit_error(builder.build());
    match render_sink(&sink, &source_map) {
        Ok(mut envelope) => envelope.diagnostics.remove(0),
        Err(error) => crate::diagnostics::diagnostic_with_code(
            format!("internal compiler error: failed to render Rust interop diagnostic: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

pub(super) fn render_template(template: &str, args: &[(&'static str, String)]) -> String {
    args.iter()
        .fold(template.to_string(), |message, (name, value)| {
            message.replace(&format!("{{{name}}}"), value)
        })
}
