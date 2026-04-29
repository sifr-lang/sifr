use super::{DiagnosticEnvelope, DiagnosticSpan, RenderedDiagnostic};
use crate::model::Severity;
use crate::source_map::{SourceMap, SourceMapError};
use crate::DiagnosticSink;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const MAX_COMPACT_REPRESENTATIVE_LOCATIONS: usize = 5;

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
    let mut output = String::new();
    for diagnostic in &envelope.diagnostics {
        let _ = writeln!(
            output,
            "{}[{}]: {}",
            severity_label(diagnostic.severity),
            diagnostic.code,
            diagnostic.message
        );
        if let Some(primary) = primary_span(diagnostic) {
            let _ = writeln!(output, "  --> {}", location_label(primary));
            for line in &primary.lines {
                let _ = writeln!(output, "   | {}", line.text);
            }
        }
        for child in &diagnostic.children {
            let _ = writeln!(
                output,
                "  {}: {}",
                child_severity_label(child.severity),
                child.message
            );
        }
        if let Some(help) = &diagnostic.help {
            let _ = writeln!(output, "  help: {help}");
        }
        let _ = writeln!(output, "  url: {}", diagnostic.url);
    }
    output
}

#[must_use]
pub fn render_compact_envelope(envelope: &DiagnosticEnvelope) -> String {
    let diagnostics = envelope.diagnostics.as_slice();
    let mut grouped: BTreeMap<CompactKey, Vec<&RenderedDiagnostic>> = BTreeMap::new();
    for diagnostic in diagnostics {
        grouped
            .entry(CompactKey::from_diagnostic(diagnostic))
            .or_default()
            .push(diagnostic);
    }

    let mut output = String::new();
    output.push_str(&compact_severity_summary(diagnostics));
    output.push('\n');

    for (key, group) in grouped {
        let severity = group[0].severity;
        let _ = writeln!(
            output,
            "{} [{}] {} (x{})",
            severity_label(severity),
            key.code,
            group[0].message,
            group.len()
        );

        let mut locations = BTreeSet::new();
        for diagnostic in &group {
            if let Some(span) = primary_span(diagnostic) {
                locations.insert(location_label(span));
            }
        }

        for location in locations.iter().take(MAX_COMPACT_REPRESENTATIVE_LOCATIONS) {
            let _ = writeln!(output, "  at {location}");
        }
        if locations.len() > MAX_COMPACT_REPRESENTATIVE_LOCATIONS {
            let _ = writeln!(
                output,
                "  ... +{} more",
                locations.len() - MAX_COMPACT_REPRESENTATIVE_LOCATIONS
            );
        }

        if let Some(help) = group
            .iter()
            .find_map(|diagnostic| diagnostic.help.as_deref())
        {
            let _ = writeln!(output, "  help: {help}");
        }
        let _ = writeln!(output, "  url: {}", group[0].url);
    }

    output
}

pub fn render_json_envelope(envelope: &DiagnosticEnvelope) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(envelope)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CompactKey {
    severity_rank: u8,
    code: String,
    message_template: String,
    primary_display_file: Option<String>,
}

impl CompactKey {
    fn from_diagnostic(diagnostic: &RenderedDiagnostic) -> Self {
        Self {
            severity_rank: severity_rank(diagnostic.severity),
            code: diagnostic.code.clone(),
            message_template: diagnostic.message_template.clone(),
            primary_display_file: primary_span(diagnostic).and_then(|span| span.file.clone()),
        }
    }
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
    format!("summary: {error_count} error(s), {warning_count} warning(s), {note_count} note(s)")
}

fn primary_span(diagnostic: &RenderedDiagnostic) -> Option<&DiagnosticSpan> {
    diagnostic.spans.iter().find(|span| span.is_primary)
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

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Note => 2,
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
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
    use crate::model::{ChildSeverity, DiagnosticBuilder, DiagnosticSink, Severity};
    use crate::render::render_sink;
    use crate::source_map::{SourceMap, SourceSpan};
    use crate::DiagnosticCode;
    use ruff_text_size::{TextRange, TextSize};

    #[test]
    fn compact_groups_by_template_and_primary_file_not_rendered_message() {
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

        assert!(compact.contains("error [SIFR-NAME-0001] undefined variable `x` (x2)"));
        assert!(compact.contains("error [SIFR-NAME-0001] undefined variable `z` (x1)"));
        assert!(compact.contains("  at a.sifr:1:1"));
        assert!(compact.contains("  at b.sifr:1:1"));
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
        assert!(human.contains("  help: please report this compiler bug"));
        assert!(!human.contains("<unknown>"));
        assert!(compact.starts_with("summary: 1 error(s), 1 warning(s), 1 note(s)\n"));
        assert!(compact
            .contains("error [SIFR-INTERNAL-0001] internal compiler panic during codegen (x1)"));
        assert!(compact.contains("warning [SIFR-TYPE-0901] int multiplication may overflow (x1)"));
        assert!(compact.contains("note [SIFR-TYPE-0902] revealed type is `int` (x1)"));
    }

    #[test]
    fn compact_groups_internal_diagnostics_without_locations() {
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

        assert!(compact.contains("summary: 2 error(s), 0 warning(s), 0 note(s)"));
        assert!(compact
            .contains("error [SIFR-INTERNAL-0001] internal compiler panic during codegen (x2)"));
        assert!(!compact.contains("  at "));
    }
}
