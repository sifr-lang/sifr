use crate::queries::FileTextEdits;
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use ruff_text_size::{TextRange, TextSize};
use sifr_frontend::FileId;

pub(super) fn full_range(source: &str) -> Result<TextRange, AnalysisError> {
    let end = u32::try_from(source.len()).map_err(|_| {
        AnalysisError::new(
            AnalysisErrorKind::InvalidFormatRange,
            "source is too large to format through TextRange",
        )
    })?;
    Ok(TextRange::new(TextSize::new(0), TextSize::new(end)))
}

pub(super) fn source_edit_to_text_edit(edit: &sifr_lint::SourceEdit) -> sifr_format::TextEdit {
    sifr_format::TextEdit {
        range: TextRange::new(TextSize::new(edit.byte_start), TextSize::new(edit.byte_end)),
        replacement: edit.replacement.clone(),
    }
}

pub(super) fn fixed_source_edits(file: FileId, source: &str, fixed: &str) -> Vec<FileTextEdits> {
    if source == fixed {
        Vec::new()
    } else {
        vec![FileTextEdits {
            file,
            edits: vec![sifr_format::TextEdit {
                range: full_range(source)
                    .unwrap_or_else(|_| TextRange::new(TextSize::new(0), TextSize::new(u32::MAX))),
                replacement: fixed.to_string(),
            }],
        }]
    }
}

pub(super) fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}
