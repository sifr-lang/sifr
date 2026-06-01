use crate::model::{ChildSeverity, DiagnosticArg, DiagnosticSuggestion, RelatedKind, Severity};
use crate::source_map::{SourceMap, SourceMapError, SourceSpan};
use crate::{DiagnosticSink, SifrDiagnostic};
use ruff_text_size::TextSize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sifr_source::LineMap;
use std::collections::BTreeMap;

mod presentation;

pub use presentation::{
    render_compact_diagnostics, render_compact_envelope, render_human_diagnostics,
    render_human_envelope, render_json_diagnostics, render_json_envelope, render_sink_compact,
    render_sink_human, render_sink_json, PresentationRenderError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticEnvelope {
    pub version: u32,
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderedDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub message_template: String,
    pub args: BTreeMap<String, DiagnosticArg>,
    pub url: String,
    pub spans: Vec<DiagnosticSpan>,
    pub children: Vec<RenderedDiagnosticChild>,
    #[schemars(required)]
    pub help: Option<String>,
    pub suggestions: Vec<RenderedDiagnosticSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderedDiagnosticChild {
    pub severity: ChildSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderedDiagnosticSuggestion {
    pub message: String,
    pub applicability: crate::model::SuggestionApplicability,
    pub edits: Vec<RenderedSuggestionEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderedSuggestionEdit {
    pub span: DiagnosticSpan,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticSpan {
    #[schemars(required)]
    pub file: Option<String>,
    pub byte_start: u32,
    pub byte_end: u32,
    #[schemars(required)]
    pub line: Option<u32>,
    #[schemars(required)]
    pub column: Option<u32>,
    #[schemars(required)]
    pub end_line: Option<u32>,
    #[schemars(required)]
    pub end_column: Option<u32>,
    pub is_primary: bool,
    #[schemars(required)]
    pub label: Option<String>,
    pub lines: Vec<DiagnosticSpanLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticSpanLine {
    pub text: String,
    pub highlight_start: u32,
    pub highlight_end: u32,
}

#[derive(Debug, Clone)]
struct SortableDiagnostic<'a> {
    diagnostic: &'a SifrDiagnostic,
    insertion_order: u64,
}

pub fn render_sink(
    sink: &DiagnosticSink,
    source_map: &SourceMap,
) -> Result<DiagnosticEnvelope, SourceMapError> {
    let mut diagnostics: Vec<_> = sink
        .diagnostics()
        .iter()
        .map(|entry| SortableDiagnostic {
            diagnostic: entry.diagnostic(),
            insertion_order: entry.insertion_order(),
        })
        .collect();
    diagnostics.sort_by_cached_key(|entry| {
        ordering_key(entry.diagnostic, entry.insertion_order, source_map)
    });

    let rendered = diagnostics
        .into_iter()
        .map(|entry| render_diagnostic(entry.diagnostic, source_map))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DiagnosticEnvelope {
        version: 1,
        diagnostics: rendered,
    })
}

fn render_diagnostic(
    diagnostic: &SifrDiagnostic,
    source_map: &SourceMap,
) -> Result<RenderedDiagnostic, SourceMapError> {
    let mut spans = Vec::new();
    if let Some(primary_span) = diagnostic.primary_span() {
        spans.push(render_span(source_map, &primary_span, true, None)?);
        for related in diagnostic.related_spans() {
            let label = related.label.clone().or_else(|| match related.kind {
                RelatedKind::Label => None,
                RelatedKind::Note => Some("note".to_string()),
                RelatedKind::Origin => Some("origin".to_string()),
                RelatedKind::ReplacementTarget => Some("replacement target".to_string()),
            });
            spans.push(render_span(source_map, &related.span, false, label)?);
        }
    }
    let suggestions = render_suggestions(diagnostic.suggestions(), source_map)?;
    Ok(RenderedDiagnostic {
        code: diagnostic.code().code().to_string(),
        severity: diagnostic.severity(),
        message: diagnostic.message().to_string(),
        message_template: diagnostic.message_template().to_string(),
        args: diagnostic.args().clone(),
        url: diagnostic.code().docs_url(),
        spans,
        children: diagnostic
            .children()
            .iter()
            .map(|child| RenderedDiagnosticChild {
                severity: child.severity,
                message: child.message.clone(),
            })
            .collect(),
        help: diagnostic.help().map(str::to_string),
        suggestions,
    })
}

fn render_suggestions(
    suggestions: &[DiagnosticSuggestion],
    source_map: &SourceMap,
) -> Result<Vec<RenderedDiagnosticSuggestion>, SourceMapError> {
    suggestions
        .iter()
        .map(|suggestion| {
            let edits = suggestion
                .edits
                .iter()
                .map(|edit| {
                    Ok(RenderedSuggestionEdit {
                        span: render_span(source_map, &edit.span, false, None)?,
                        replacement: edit.replacement.clone(),
                    })
                })
                .collect::<Result<Vec<_>, SourceMapError>>()?;
            Ok(RenderedDiagnosticSuggestion {
                message: suggestion.message.clone(),
                applicability: suggestion.applicability,
                edits,
            })
        })
        .collect()
}

fn ordering_key(
    diagnostic: &SifrDiagnostic,
    insertion_order: u64,
    source_map: &SourceMap,
) -> DiagnosticOrderingKey {
    let primary = diagnostic.primary_span();
    let (path_rank, path) = primary
        .as_ref()
        .and_then(|span| source_map.display_path(span.source_id).map(str::to_string))
        .map_or((1, String::new()), |path| (0, path));
    let (byte_start, byte_end) = primary.as_ref().map_or((u32::MAX, u32::MAX), |span| {
        (span.range.start().to_u32(), span.range.end().to_u32())
    });
    DiagnosticOrderingKey {
        path_rank,
        path,
        byte_start,
        byte_end,
        severity_rank: severity_rank(diagnostic.severity()),
        kind_rank: u8::from(primary.is_none()),
        code: diagnostic.code().code().to_string(),
        message_template: diagnostic.message_template().to_string(),
        args: canonical_args_bytes(diagnostic.args()),
        insertion_order,
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DiagnosticOrderingKey {
    path_rank: u8,
    path: String,
    byte_start: u32,
    byte_end: u32,
    severity_rank: u8,
    kind_rank: u8,
    code: String,
    message_template: String,
    args: Vec<u8>,
    insertion_order: u64,
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Note => 2,
    }
}

fn canonical_args_bytes(args: &BTreeMap<String, DiagnosticArg>) -> Vec<u8> {
    serde_json::to_vec(args).unwrap_or_default()
}

fn render_span(
    source_map: &SourceMap,
    span: &SourceSpan,
    is_primary: bool,
    label: Option<String>,
) -> Result<DiagnosticSpan, SourceMapError> {
    source_map.validate_span(span)?;
    let source = source_map
        .source(span.source_id)
        .ok_or(SourceMapError::UnknownSource(span.source_id))?;
    let text = source.as_str();
    let line_map = source.line_map();
    let byte_start = span.range.start().to_u32();
    let byte_end = span.range.end().to_u32();
    let (line, column) = line_column(text, line_map, byte_start);
    let (end_line, end_column) = line_column(text, line_map, byte_end);
    Ok(DiagnosticSpan {
        file: source_map.display_path(span.source_id).map(str::to_string),
        byte_start,
        byte_end,
        line: Some(line),
        column: Some(column),
        end_line: Some(end_line),
        end_column: Some(end_column),
        is_primary,
        label,
        lines: span_lines(text, line_map, byte_start, byte_end),
    })
}

fn line_column(text: &str, line_map: &LineMap, byte_pos: u32) -> (u32, u32) {
    let byte_pos = TextSize::new(byte_pos);
    let line_starts = line_map.line_starts();
    let line_index = match line_starts.binary_search(&byte_pos) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    };
    let line_start = usize::try_from(line_starts[line_index].to_u32()).unwrap_or(0);
    let byte_pos_usize = usize::try_from(byte_pos.to_u32()).unwrap_or(text.len());
    let prefix = text.get(line_start..byte_pos_usize).unwrap_or_default();
    (
        u32::try_from(line_index + 1).unwrap_or(u32::MAX),
        u32::try_from(prefix.chars().count() + 1).unwrap_or(u32::MAX),
    )
}

// CRLF sources retain `\r` in serialized line text. Human renderers should
// normalize or strip it before printing snippets to a terminal.
fn span_lines(
    text: &str,
    line_map: &LineMap,
    byte_start: u32,
    byte_end: u32,
) -> Vec<DiagnosticSpanLine> {
    let byte_start = TextSize::new(byte_start);
    let byte_end = TextSize::new(byte_end);
    let line_starts = line_map.line_starts();
    let start_line_index = match line_starts.binary_search(&byte_start) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    };
    let end_line_index = match line_starts.binary_search(&byte_end) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    };
    (start_line_index..=end_line_index)
        .map(|line_index| {
            render_line(
                text,
                line_map,
                line_index,
                byte_start.to_u32(),
                byte_end.to_u32(),
            )
        })
        .collect()
}

fn render_line(
    text: &str,
    line_map: &LineMap,
    line_index: usize,
    byte_start: u32,
    byte_end: u32,
) -> DiagnosticSpanLine {
    let line = u32::try_from(line_index).unwrap_or(u32::MAX);
    let full_range = line_map
        .line_full_byte_range(line)
        .unwrap_or_else(|| ruff_text_size::TextRange::new(line_map.eof(), line_map.eof()));
    let line_start = usize::try_from(full_range.start().to_u32()).unwrap_or(0);
    let line_end = usize::try_from(full_range.end().to_u32()).unwrap_or(text.len());
    let line_text = text[line_start..line_end]
        .trim_end_matches('\n')
        .to_string();
    let highlight_start_byte = usize::try_from(byte_start)
        .unwrap_or(line_start)
        .saturating_sub(line_start)
        .min(line_text.len());
    let highlight_end_byte = usize::try_from(byte_end)
        .unwrap_or(line_start)
        .saturating_sub(line_start)
        .min(line_text.len());
    let highlight_start = char_column(&line_text, highlight_start_byte);
    let highlight_end = char_column(&line_text, highlight_end_byte).max(highlight_start);
    DiagnosticSpanLine {
        text: line_text,
        highlight_start,
        highlight_end,
    }
}

fn char_column(text: &str, byte_offset: usize) -> u32 {
    u32::try_from(text.get(..byte_offset).unwrap_or_default().chars().count() + 1)
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::render_sink;
    use crate::model::{DiagnosticBuilder, DiagnosticSink, Severity};
    use crate::source_map::{SourceMap, SourceSpan};
    use crate::DiagnosticCode;
    use ruff_text_size::{TextRange, TextSize};

    fn source_map_with_text(text: &str) -> (SourceMap, crate::SourceId) {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source("main.sifr", text);
        (source_map, source_id)
    }

    #[test]
    fn renders_multibyte_utf8_columns_as_char_offsets() {
        let (source_map, source_id) = source_map_with_text("a = '🦀'\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(5), TextSize::new(9)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "crab")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        let span = &envelope.diagnostics[0].spans[0];
        assert_eq!(span.byte_start, 5);
        assert_eq!(span.byte_end, 9);
        assert_eq!(span.column, Some(6));
        assert_eq!(span.end_column, Some(7));
    }

    #[test]
    fn renders_multiline_span_lines() {
        let (source_map, source_id) = source_map_with_text("first\nsecond\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(3), TextSize::new(9)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "x")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        assert_eq!(envelope.diagnostics[0].spans[0].lines.len(), 2);
    }

    #[test]
    fn renders_three_line_span_highlights_middle_line_fully() {
        let (source_map, source_id) = source_map_with_text("one\ntwo\nthree\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(2), TextSize::new(11)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "x")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        let lines = &envelope.diagnostics[0].spans[0].lines;
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1].text, "two");
        assert_eq!(lines[1].highlight_start, 1);
        assert_eq!(lines[1].highlight_end, 4);
    }

    #[test]
    fn eof_zero_length_span_has_exclusive_end_position() {
        let (source_map, source_id) = source_map_with_text("x\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(2), TextSize::new(2)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "x")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        let span = &envelope.diagnostics[0].spans[0];
        assert_eq!(span.byte_start, 2);
        assert_eq!(span.byte_end, 2);
        assert_eq!(span.line, Some(2));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.end_line, Some(2));
        assert_eq!(span.end_column, Some(1));
    }

    #[test]
    fn crlf_source_has_stable_line_and_column_positions() {
        let (source_map, source_id) = source_map_with_text("a\r\nb\r\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(3), TextSize::new(4)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "b")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        let span = &envelope.diagnostics[0].spans[0];
        assert_eq!(span.line, Some(2));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "b\r");
    }

    #[test]
    fn rendered_diagnostic_json_round_trips_without_losing_arg_kinds_or_nulls() {
        let (source_map, source_id) = source_map_with_text("x\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(0), TextSize::new(1)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name} {count}")
                .arg("name", "x")
                .arg("count", 5_u64)
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"help\":null"));
        assert!(json.contains("\"label\":null"));
        assert!(json.contains("\"kind\":\"unsigned\""));
        let decoded: super::DiagnosticEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn orders_by_path_then_span_then_args_then_insertion_order() {
        let (mut source_map, left_source) = source_map_with_text("aaaa\n");
        let right_source = source_map.register_source("main.sifr", "bbbb\n");
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(
                DiagnosticCode::TEST_SOURCE_ERROR,
                Severity::Error,
                SourceSpan::new(
                    right_source,
                    TextRange::new(TextSize::new(0), TextSize::new(1)),
                ),
            )
            .message_template("undefined variable: {name}")
            .arg("name", "b")
            .build(),
        );
        sink.emit_error(
            DiagnosticBuilder::source(
                DiagnosticCode::TEST_SOURCE_ERROR,
                Severity::Error,
                SourceSpan::new(
                    left_source,
                    TextRange::new(TextSize::new(0), TextSize::new(1)),
                ),
            )
            .message_template("undefined variable: {name}")
            .arg("name", "a")
            .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        assert_eq!(envelope.diagnostics[0].args["name"], "a".into());
        assert_eq!(envelope.diagnostics[1].args["name"], "b".into());
    }

    #[test]
    fn diagnostics_differing_only_in_args_sort_by_canonical_json_bytes() {
        let (source_map, source_id) = source_map_with_text("x\n");
        let span = SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(0), TextSize::new(1)),
        );
        let mut sink = DiagnosticSink::new();
        sink.emit_error(
            DiagnosticBuilder::source(
                DiagnosticCode::TEST_SOURCE_ERROR,
                Severity::Error,
                span.clone(),
            )
            .message_template("undefined variable: {name}")
            .arg("name", "z")
            .build(),
        );
        sink.emit_error(
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span)
                .message_template("undefined variable: {name}")
                .arg("name", "a")
                .build(),
        );
        let envelope = render_sink(&sink, &source_map).unwrap();
        assert_eq!(envelope.diagnostics[0].args["name"], "a".into());
        assert_eq!(envelope.diagnostics[1].args["name"], "z".into());
    }
}
