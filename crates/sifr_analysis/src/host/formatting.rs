use super::implementation::{AnalysisHost, QueryResult, frontend_diagnostics};
use super::text_edits::full_range;
use crate::queries::FormatOptions;
use crate::snapshot::AnalysisQueryKind;
use ruff_text_size::TextRange;
use sifr_frontend::FileId;

impl AnalysisHost {
    pub fn format_document(
        &mut self,
        file: FileId,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context()?.path_for_file(file);
        let result = sifr_format::format_source(&source, path, options)
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        let edits = if result.formatted == source {
            Vec::new()
        } else {
            vec![sifr_format::TextEdit {
                range: full_range(&source)?,
                replacement: result.formatted,
            }]
        };
        Ok(self.result(AnalysisQueryKind::FormatDocument, edits))
    }

    pub fn format_range(
        &mut self,
        file: FileId,
        range: TextRange,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context()?.path_for_file(file);
        let edits = sifr_format::format_range(&source, range, path, options)
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        Ok(self.result(AnalysisQueryKind::FormatRange, edits))
    }
}
