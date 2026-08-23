use super::implementation::AnalysisHost;
use crate::editor::{EditorFacts, EditorToken};
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use sifr_frontend::{FileId, parse_source_module};

impl AnalysisHost {
    pub(super) fn editor_facts(&mut self, file: FileId) -> Result<EditorFacts, AnalysisError> {
        let source = self.source_text(file)?;
        let parsed = if let Some(module) = self.file_to_module.get(&file).copied() {
            self.context_mut()?.parse_module(module).into_value().parsed
        } else {
            let module_name = self
                .context()?
                .source_file_for_file(file)
                .and_then(|source_file| source_file.module_name.as_deref());
            parse_source_module(&source, module_name).map_err(|diagnostics| {
                AnalysisError::new(
                    AnalysisErrorKind::FrontendDiagnostic,
                    diagnostics.first().map_or_else(
                        || "failed to parse source file".to_string(),
                        |diagnostic| diagnostic.message.clone(),
                    ),
                )
            })?
        };
        let tokens = parsed
            .tokens()
            .iter()
            .filter_map(|token| {
                let start = usize::try_from(token.range.start().to_u32()).ok()?;
                let end = usize::try_from(token.range.end().to_u32()).ok()?;
                let text = source.get(start..end)?.to_string();
                Some(EditorToken {
                    kind: token.kind.as_str().to_string(),
                    text,
                    range: token.range,
                })
            })
            .collect();
        Ok(EditorFacts { source, tokens })
    }
}
