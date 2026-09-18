use super::CapturedSource;
use serde::{Deserialize, Serialize};
use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic, SourceMap, SourceSpan};
use std::collections::BTreeMap;

/// Artifact-local source identity plus byte coordinates. No `SourceId`, line,
/// column, snippet, terminal width/color or LSP position encoding is persisted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSpan {
    pub source_identity: Option<String>,
    pub start: u32,
    pub end: u32,
    pub primary: bool,
    pub label: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDiagnostic {
    // Presentation-independent message/arguments/children/suggestion metadata.
    // All spans (including edits) are removed and stored in the tables below.
    fact: RenderedDiagnostic,
    spans: Vec<CanonicalSpan>,
    edits: Vec<Vec<CanonicalSpan>>,
}
impl CanonicalDiagnostic {
    pub fn is_error(&self) -> bool {
        self.fact.severity == sifr_diagnostics::Severity::Error
    }
    pub fn capture(
        diagnostic: &RenderedDiagnostic,
        sources: &BTreeMap<String, CapturedSource>,
    ) -> Result<Self, String> {
        let capture = |span: &DiagnosticSpan| {
            let source_identity = span
                .file
                .as_ref()
                .map(|file| {
                    let source = sources
                        .get(file)
                        .ok_or_else(|| format!("uncaptured diagnostic source: {file}"))?;
                    validate_range(source, span.byte_start, span.byte_end)?;
                    Ok::<_, String>(source.identity())
                })
                .transpose()?;
            Ok::<_, String>(CanonicalSpan {
                source_identity,
                start: span.byte_start,
                end: span.byte_end,
                primary: span.is_primary,
                label: span.label.clone(),
            })
        };
        let spans = diagnostic
            .spans
            .iter()
            .map(capture)
            .collect::<Result<_, _>>()?;
        let edits = diagnostic
            .suggestions
            .iter()
            .map(|suggestion| {
                suggestion
                    .edits
                    .iter()
                    .map(|edit| capture(&edit.span))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<_, _>>()?;
        let mut fact = diagnostic.clone();
        fact.spans.clear();
        for suggestion in &mut fact.suggestions {
            for edit in &mut suggestion.edits {
                edit.span = empty_span();
            }
        }
        Ok(Self { fact, spans, edits })
    }
    pub fn render(
        &self,
        sources: &BTreeMap<String, (CapturedSource, String)>,
        source_map: &mut SourceMap,
    ) -> Result<RenderedDiagnostic, String> {
        let mut remapped = BTreeMap::new();
        let needed: std::collections::BTreeSet<_> = self
            .spans
            .iter()
            .chain(self.edits.iter().flatten())
            .filter_map(|span| span.source_identity.as_ref())
            .collect();
        for identity in needed {
            let (source, display) = sources.get(identity).ok_or("missing canonical source")?;
            if source.identity() != *identity {
                return Err("diagnostic source identity mismatch".into());
            }
            remapped.insert(
                identity.clone(),
                source_map.register_source(display, source.text.clone()),
            );
        }
        let render = |span: &CanonicalSpan| {
            let Some(identity) = &span.source_identity else {
                let mut result = empty_span();
                result.byte_start = span.start;
                result.byte_end = span.end;
                result.is_primary = span.primary;
                result.label.clone_from(&span.label);
                return Ok(result);
            };
            let (source, _) = sources.get(identity).ok_or("missing canonical source")?;
            validate_range(source, span.start, span.end)?;
            let source_id = *remapped.get(identity).ok_or("missing remapped source")?;
            let range = ruff_text_size::TextRange::new(span.start.into(), span.end.into());
            sifr_diagnostics::render_span(
                source_map,
                &SourceSpan::new(source_id, range),
                span.primary,
                span.label.clone(),
            )
            .map_err(|error| format!("{error:?}"))
        };
        let mut result = self.fact.clone();
        if result.suggestions.len() != self.edits.len() {
            return Err("invalid diagnostic edit table".into());
        }
        result.spans = self.spans.iter().map(render).collect::<Result<_, _>>()?;
        for (suggestion, spans) in result.suggestions.iter_mut().zip(&self.edits) {
            if suggestion.edits.len() != spans.len() {
                return Err("invalid diagnostic edit count".into());
            }
            for (edit, span) in suggestion.edits.iter_mut().zip(spans) {
                edit.span = render(span)?;
            }
        }
        Ok(result)
    }
}
fn validate_range(source: &CapturedSource, start: u32, end: u32) -> Result<(), String> {
    if start > end
        || !source.text.is_char_boundary(start as usize)
        || !source.text.is_char_boundary(end as usize)
    {
        return Err("invalid persisted source range".into());
    }
    Ok(())
}
fn empty_span() -> DiagnosticSpan {
    DiagnosticSpan {
        file: None,
        byte_start: 0,
        byte_end: 0,
        line: None,
        column: None,
        end_line: None,
        end_column: None,
        is_primary: false,
        label: None,
        lines: Vec::new(),
    }
}
