//! Syntax requests operate on captured text without constructing a semantic host.
use crate::editor::{EditorFacts, EditorToken};
use crate::{FoldingRange, SelectionRange, TextPosition};
use sifr_diagnostics::RenderedDiagnostic;
pub use sifr_format::{format_range, format_source};

fn facts(source: &str) -> Result<EditorFacts, Vec<RenderedDiagnostic>> {
    let parsed = sifr_frontend::parse_source_module(source, None)?;
    let tokens = parsed
        .tokens()
        .iter()
        .filter_map(|token| {
            let text = source.get(token.range.start().to_usize()..token.range.end().to_usize())?;
            Some(EditorToken {
                kind: token.kind.as_str().into(),
                text: text.into(),
                range: token.range,
            })
        })
        .collect();
    Ok(EditorFacts {
        source: source.into(),
        tokens,
    })
}
pub fn folding_ranges(source: &str) -> Result<Vec<FoldingRange>, Vec<RenderedDiagnostic>> {
    Ok(facts(source)?.folding_ranges())
}
pub fn selection_ranges(
    source: &str,
    positions: &[TextPosition],
) -> Result<Vec<SelectionRange>, Vec<RenderedDiagnostic>> {
    Ok(facts(source)?.selection_ranges(positions))
}
