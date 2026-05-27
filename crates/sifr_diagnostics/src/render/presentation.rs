use super::{DiagnosticEnvelope, DiagnosticSpan, DiagnosticSpanLine, RenderedDiagnostic};
use crate::model::Severity;
use crate::source_map::{SourceMap, SourceMapError};
use crate::DiagnosticSink;
use std::fmt::Write as _;

#[derive(Debug)]
pub enum PresentationRenderError {
    SourceMap(SourceMapError),
    Json(serde_json::Error),
}

impl From<SourceMapError> for PresentationRenderError {
    fn from(error: SourceMapError) -> Self {
        Self::SourceMap(error)
    }
}

impl From<serde_json::Error> for PresentationRenderError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn render_sink_human(
    sink: &DiagnosticSink,
    source_map: &SourceMap,
) -> Result<String, SourceMapError> {
    let envelope = super::render_sink(sink, source_map)?;
    Ok(render_human_envelope(&envelope))
}

pub fn render_sink_compact(
    sink: &DiagnosticSink,
    source_map: &SourceMap,
) -> Result<String, SourceMapError> {
    let envelope = super::render_sink(sink, source_map)?;
    Ok(render_compact_envelope(&envelope))
}

pub fn render_sink_json(
    sink: &DiagnosticSink,
    source_map: &SourceMap,
) -> Result<String, PresentationRenderError> {
    let envelope = super::render_sink(sink, source_map)?;
    Ok(render_json_envelope(&envelope)?)
}

#[must_use]
pub fn render_human_envelope(envelope: &DiagnosticEnvelope) -> String {
    render_human_diagnostics(&envelope.diagnostics)
}

#[must_use]
pub fn render_human_diagnostics(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        let _ = writeln!(
            output,
            "{}[{}]: {}",
            severity_label(diagnostic.severity),
            diagnostic.code,
            display_message(diagnostic)
        );
        if let Some(primary) = primary_span(diagnostic) {
            render_span_block(&mut output, primary, SpanRole::Primary);
        } else {
            let _ = writeln!(output, "  = location: <unavailable>");
        }
        for related in diagnostic.spans.iter().filter(|span| !span.is_primary) {
            render_span_block(&mut output, related, SpanRole::Related);
        }
        for child in &diagnostic.children {
            let _ = writeln!(
                output,
                "  = {}: {}",
                child_severity_label(child.severity),
                child.message
            );
        }
        if let Some(help) = &diagnostic.help {
            let _ = writeln!(output, "  = help: {help}");
        }
        for suggestion in &diagnostic.suggestions {
            let _ = writeln!(output, "  = suggestion: {}", suggestion.message);
            for edit in &suggestion.edits {
                let _ = writeln!(
                    output,
                    "    replace {} with {:?}",
                    location_label(&edit.span),
                    edit.replacement
                );
            }
        }
        if !diagnostic.url.is_empty() {
            let _ = writeln!(output, "  = docs: {}", diagnostic.url);
        }
    }
    output
}

#[must_use]
pub fn render_compact_envelope(envelope: &DiagnosticEnvelope) -> String {
    render_compact_diagnostics(&envelope.diagnostics)
}

#[must_use]
pub fn render_compact_diagnostics(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut output = String::new();
    output.push_str(&compact_severity_summary(diagnostics));
    output.push('\n');

    for diagnostic in diagnostics {
        let _ = writeln!(
            output,
            "{} {} {} {}",
            severity_abbreviation(diagnostic.severity),
            diagnostic.code,
            primary_span(diagnostic).map_or_else(|| "<unknown>".to_string(), location_label),
            display_message(diagnostic)
        );
    }

    output
}

pub fn render_json_envelope(envelope: &DiagnosticEnvelope) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(envelope)
}

pub fn render_json_diagnostics(
    diagnostics: &[RenderedDiagnostic],
) -> Result<String, serde_json::Error> {
    let mut output = serde_json::to_string_pretty(diagnostics)?;
    output.push('\n');
    Ok(output)
}

fn compact_severity_summary(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut note_count = 0usize;
    for diagnostic in diagnostics {
        match diagnostic.severity {
            Severity::Error => error_count += 1,
            Severity::Warning => warning_count += 1,
            Severity::Note => note_count += 1,
        }
    }
    format!(
        "{error_count} {}, {warning_count} {}, {note_count} {}",
        plural(error_count, "error", "errors"),
        plural(warning_count, "warning", "warnings"),
        plural(note_count, "note", "notes")
    )
}

fn primary_span(diagnostic: &RenderedDiagnostic) -> Option<&DiagnosticSpan> {
    diagnostic.spans.iter().find(|span| span.is_primary)
}

fn render_span_block(output: &mut String, span: &DiagnosticSpan, role: SpanRole) {
    let _ = writeln!(
        output,
        "{} {}",
        role.location_prefix(),
        location_label(span)
    );
    let gutter_width = span
        .line
        .map(|line| {
            let line_count = u32::try_from(span.lines.len().saturating_sub(1)).unwrap_or(0);
            line.saturating_add(line_count).to_string().len()
        })
        .unwrap_or(1);
    let _ = writeln!(output, "   {:>width$} |", "", width = gutter_width);
    for (index, line) in span.lines.iter().enumerate() {
        let line_number = span
            .line
            .and_then(|line| line.checked_add(u32::try_from(index).unwrap_or(u32::MAX)))
            .map_or_else(String::new, |line| line.to_string());
        let text = terminal_line_text(line);
        let _ = writeln!(
            output,
            "   {line_number:>width$} | {text}",
            width = gutter_width
        );
        let marker = highlight_marker(line);
        let label = span
            .label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_default();
        if label.is_empty() {
            let _ = writeln!(output, "   {:>width$} | {marker}", "", width = gutter_width);
        } else {
            let _ = writeln!(
                output,
                "   {:>width$} | {marker} {label}",
                "",
                width = gutter_width
            );
        }
    }
    if role == SpanRole::Related {
        let _ = writeln!(
            output,
            "   {:>width$} = related span",
            "",
            width = gutter_width
        );
    }
}

fn terminal_line_text(line: &DiagnosticSpanLine) -> String {
    line.text.trim_end_matches('\r').to_string()
}

fn highlight_marker(line: &DiagnosticSpanLine) -> String {
    let start = usize::try_from(line.highlight_start.saturating_sub(1)).unwrap_or(0);
    let end = usize::try_from(line.highlight_end.saturating_sub(1)).unwrap_or(start);
    let marker_width = end.saturating_sub(start).max(1);
    format!("{}{}", " ".repeat(start), "^".repeat(marker_width))
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum SpanRole {
    Primary,
    Related,
}

impl SpanRole {
    const fn location_prefix(self) -> &'static str {
        match self {
            Self::Primary => "  -->",
            Self::Related => "  :::",
        }
    }
}

fn location_label(span: &DiagnosticSpan) -> String {
    match (&span.file, span.line, span.column) {
        (Some(file), Some(line), Some(column)) => format!("{file}:{line}:{column}"),
        (Some(file), Some(line), None) => format!("{file}:{line}"),
        (Some(file), None, _) => file.clone(),
        (None, Some(line), Some(column)) => format!("<unknown>:{line}:{column}"),
        (None, Some(line), None) => format!("<unknown>:{line}"),
        (None, None, Some(column)) => format!("<unknown>:0:{column}"),
        (None, None, None) => "<unknown>".to_string(),
    }
}

fn display_message(diagnostic: &RenderedDiagnostic) -> &str {
    if primary_span(diagnostic).is_some() {
        strip_leading_module_context(&diagnostic.message)
    } else {
        &diagnostic.message
    }
}

fn strip_leading_module_context(message: &str) -> &str {
    let Some(rest) = message.strip_prefix('[') else {
        return message;
    };
    let Some((module, suffix)) = rest.split_once("] ") else {
        return message;
    };
    if module
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-' | '/'))
    {
        suffix
    } else {
        message
    }
}

const fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn severity_abbreviation(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "E",
        Severity::Warning => "W",
        Severity::Note => "N",
    }
}

fn child_severity_label(severity: crate::model::ChildSeverity) -> &'static str {
    match severity {
        crate::model::ChildSeverity::Note => "note",
        crate::model::ChildSeverity::Help => "help",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        render_compact_envelope, render_human_envelope, render_json_envelope, render_sink_compact,
        render_sink_human, render_sink_json,
    };
    use crate::model::{
        ChildSeverity, DiagnosticBuilder, DiagnosticSink, DiagnosticSuggestion, RelatedKind,
        Severity, SuggestionApplicability, SuggestionEdit,
    };
    use crate::render::render_sink;
    use crate::source_map::{SourceMap, SourceSpan};
    use crate::DiagnosticCode;
    use ruff_text_size::{TextRange, TextSize};

    #[test]
    fn compact_renders_one_line_per_retained_diagnostic_without_grouping() {
        let mut source_map = SourceMap::new();
        let first = source_map.register_source("a.sifr", "alpha\n");
        let second = source_map.register_source("b.sifr", "beta\n");
        let mut sink = DiagnosticSink::new();

        for (source_id, name) in [(first, "x"), (first, "y"), (second, "z")] {
            let diagnostic = DiagnosticBuilder::source(
                DiagnosticCode::NAME_UNDEFINED_VARIABLE,
                Severity::Error,
                SourceSpan::new(
                    source_id,
                    TextRange::new(TextSize::new(0), TextSize::new(1)),
                ),
            )
            .message_template("undefined variable `{name}`")
            .arg("name", name)
            .build();
            let _ = sink.emit_error(diagnostic);
        }

        let envelope = render_sink(&sink, &source_map).expect("rendering succeeds");
        let compact = render_compact_envelope(&envelope);
        let compact_from_sink =
            render_sink_compact(&sink, &source_map).expect("rendering succeeds");

        assert!(compact.starts_with("3 errors, 0 warnings, 0 notes\n"));
        assert!(compact.contains("E SIFR-NAME-0001 a.sifr:1:1 undefined variable `x`"));
        assert!(compact.contains("E SIFR-NAME-0001 a.sifr:1:1 undefined variable `y`"));
        assert!(compact.contains("E SIFR-NAME-0001 b.sifr:1:1 undefined variable `z`"));
        assert!(!compact.contains(" (x"));
        assert!(!compact.contains("  at "));
        assert_eq!(compact_from_sink, compact);
    }

    #[test]
    fn presentation_helpers_share_the_canonical_rendered_stream() {
        let mut source_map = SourceMap::new();
        let source = source_map.register_source("main.sifr", "value\n");
        let mut sink = DiagnosticSink::new();
        let diagnostic = DiagnosticBuilder::source(
            DiagnosticCode::TYPE_MISMATCH,
            Severity::Error,
            SourceSpan::new(source, TextRange::new(TextSize::new(0), TextSize::new(5))),
        )
        .message_template("expected `{expected}`, got `{actual}`")
        .arg("expected", "int")
        .arg("actual", "str")
        .child(ChildSeverity::Help, "convert the value before assignment")
        .build();
        let _ = sink.emit_error(diagnostic);

        let envelope = render_sink(&sink, &source_map).expect("rendering succeeds");
        let human = render_human_envelope(&envelope);
        let json = render_json_envelope(&envelope).expect("json rendering succeeds");
        let human_from_sink = render_sink_human(&sink, &source_map).expect("rendering succeeds");
        let json_from_sink = render_sink_json(&sink, &source_map).expect("json rendering succeeds");

        assert!(human.contains("error[SIFR-TYPE-0002]: expected `int`, got `str`"));
        assert!(human.contains("  --> main.sifr:1:1"));
        assert!(human.contains("   | ^^^^^"));
        assert!(json.contains("\"message_template\": \"expected `{expected}`, got `{actual}`\""));
        assert!(json.contains("\"code\": \"SIFR-TYPE-0002\""));
        assert_eq!(human_from_sink, human);
        assert_eq!(json_from_sink, json);
    }

    #[test]
    fn presentation_helpers_render_internal_and_mixed_severity_diagnostics() {
        let mut source_map = SourceMap::new();
        let source = source_map.register_source("main.sifr", "value\n");
        let primary = SourceSpan::new(source, TextRange::new(TextSize::new(0), TextSize::new(5)));
        let mut sink = DiagnosticSink::new();

        let internal =
            DiagnosticBuilder::internal(DiagnosticCode::INTERNAL_COMPILER_PANIC, Severity::Error)
                .message_template("internal compiler panic during {phase}")
                .arg("phase", "codegen")
                .help("please report this compiler bug")
                .build();
        let _ = sink.emit_error(internal);

        let warning = DiagnosticBuilder::source(
            DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK,
            Severity::Warning,
            primary.clone(),
        )
        .message_template("int multiplication may overflow")
        .build();
        sink.emit(warning);

        let note =
            DiagnosticBuilder::source(DiagnosticCode::TYPE_REVEAL_TYPE, Severity::Note, primary)
                .message_template("revealed type is `{revealed_type}`")
                .arg("revealed_type", "int")
                .build();
        sink.emit(note);

        let human = render_sink_human(&sink, &source_map).expect("rendering succeeds");
        let compact = render_sink_compact(&sink, &source_map).expect("rendering succeeds");

        assert!(human.contains("error[SIFR-INTERNAL-0001]: internal compiler panic during codegen"));
        assert!(human.contains("  = location: <unavailable>"));
        assert!(human.contains("  = help: please report this compiler bug"));
        assert!(compact.starts_with("1 error, 1 warning, 1 note\n"));
        assert!(compact
            .contains("E SIFR-INTERNAL-0001 <unknown> internal compiler panic during codegen"));
        assert!(compact.contains("W SIFR-TYPE-0901 main.sifr:1:1 int multiplication may overflow"));
        assert!(compact.contains("N SIFR-TYPE-0902 main.sifr:1:1 revealed type is `int`"));
    }

    #[test]
    fn human_renders_related_spans_suggestions_and_strips_crlf() {
        let mut source_map = SourceMap::new();
        let source =
            source_map.register_source("main.sifr", "    value: int = 1\r\n    other: int = 2\n");
        let primary = SourceSpan::new(source, TextRange::new(TextSize::new(4), TextSize::new(9)));
        let related = SourceSpan::new(source, TextRange::new(TextSize::new(24), TextSize::new(29)));
        let mut sink = DiagnosticSink::new();
        let diagnostic = DiagnosticBuilder::source(
            DiagnosticCode::TYPE_MISMATCH,
            Severity::Error,
            primary.clone(),
        )
        .message_template("carriage return normalized")
        .related(related, RelatedKind::Note, Some("related span".to_string()))
        .child(ChildSeverity::Note, "child note rendered")
        .help("child help rendered")
        .suggestion(DiagnosticSuggestion {
            message: "replace value with safer expression".to_string(),
            applicability: SuggestionApplicability::MachineApplicable,
            edits: vec![SuggestionEdit {
                span: primary,
                replacement: "\"safe\"".to_string(),
            }],
        })
        .build();
        let _ = sink.emit_error(diagnostic);

        let envelope = render_sink(&sink, &source_map).expect("rendering succeeds");
        let human = render_human_envelope(&envelope);
        let json = render_json_envelope(&envelope).expect("json rendering succeeds");

        assert!(human.contains("  ::: main.sifr:2:5"));
        assert!(human.contains("related span"));
        assert!(human.contains("  = note: child note rendered"));
        assert!(human.contains("  = help: child help rendered"));
        assert!(human.contains("  = suggestion: replace value with safer expression"));
        assert!(!human.contains('\r'));
        assert!(json.contains("\\r"));
    }

    #[test]
    fn compact_repeats_internal_diagnostics_without_grouping() {
        let source_map = SourceMap::new();
        let mut sink = DiagnosticSink::new();

        for phase in ["codegen", "codegen"] {
            let diagnostic = DiagnosticBuilder::internal(
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
                Severity::Error,
            )
            .message_template("internal compiler panic during {phase}")
            .arg("phase", phase)
            .build();
            let _ = sink.emit_error(diagnostic);
        }

        let compact = render_sink_compact(&sink, &source_map).expect("rendering succeeds");

        assert!(compact.starts_with("2 errors, 0 warnings, 0 notes\n"));
        assert_eq!(
            compact
                .matches("E SIFR-INTERNAL-0001 <unknown> internal compiler panic during codegen")
                .count(),
            2
        );
        assert!(!compact.contains("  at "));
    }
}
