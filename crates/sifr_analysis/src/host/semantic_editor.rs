use super::implementation::{unknown_file, AnalysisHost};
use crate::queries::{HoverInfo, SignatureHelp};
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use ruff_text_size::{TextRange, TextSize};
use sifr_frontend::{FileId, ModuleAnalysisView};
use sifr_syntax::{SourceText as SyntaxSourceText, TextPosition};

impl AnalysisHost {
    pub(super) fn semantic_hover(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<HoverInfo>, AnalysisError> {
        let source = self.source_text_for_semantic_query(file)?;
        let Some(offset) = SyntaxSourceText::new(source).byte_offset(position) else {
            return Ok(None);
        };
        let analysis = self.semantic_analysis_for_file(file)?;
        Ok(analysis
            .editor_semantics
            .entries
            .iter()
            .filter(|entry| contains_offset(entry.range, offset))
            .min_by_key(|entry| entry.range.len())
            .map(|entry| HoverInfo {
                contents: entry.detail.clone(),
            }))
    }

    pub(super) fn semantic_signature_help(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<SignatureHelp>, AnalysisError> {
        let source = self.source_text_for_semantic_query(file)?;
        let Some(offset) = SyntaxSourceText::new(source).byte_offset(position) else {
            return Ok(None);
        };
        let analysis = self.semantic_analysis_for_file(file)?;
        Ok(analysis
            .editor_semantics
            .calls
            .iter()
            .filter(|call| {
                contains_offset(call.call_range, offset) && call.callee_range.end() <= offset
            })
            .min_by_key(|call| call.call_range.len())
            .map(|call| SignatureHelp {
                label: call.signature.label.clone(),
                parameters: call.signature.parameters.clone(),
                active_parameter: Some(active_parameter(offset, &call.argument_ranges)),
            }))
    }

    fn semantic_analysis_for_file(
        &mut self,
        file: FileId,
    ) -> Result<ModuleAnalysisView, AnalysisError> {
        let module = self
            .file_to_module
            .get(&file)
            .copied()
            .ok_or_else(|| unknown_file(file))?;
        Ok(self
            .session
            .context_mut()
            .ok_or_else(|| {
                AnalysisError::new(
                    AnalysisErrorKind::FrontendDiagnostic,
                    "analysis workspace session has not loaded frontend state",
                )
            })?
            .analysis_for_module(module)
            .into_value())
    }

    fn source_text_for_semantic_query(&self, file: FileId) -> Result<String, AnalysisError> {
        self.session
            .context()
            .and_then(|context| context.source_text_for_file(file))
            .map(str::to_owned)
            .ok_or_else(|| unknown_file(file))
    }
}

fn contains_offset(range: TextRange, offset: TextSize) -> bool {
    range.start() <= offset && offset <= range.end()
}

fn active_parameter(offset: TextSize, ranges: &[TextRange]) -> u32 {
    if ranges.is_empty() {
        return 0;
    }
    for (index, range) in ranges.iter().enumerate() {
        if offset <= range.end() {
            return u32::try_from(index).unwrap_or(u32::MAX);
        }
    }
    u32::try_from(ranges.len().saturating_sub(1)).unwrap_or(u32::MAX)
}
