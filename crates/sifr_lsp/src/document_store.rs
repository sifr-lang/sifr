use crate::conversion::uri_to_path;
use crate::document_events::{CompactedDocumentChange, DocumentContentChange};
use crate::errors::{LspError, LspResult};
use sifr_source::PositionEncoding;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DiagnosticsMode {
    Off,
    #[default]
    OpenFiles,
    Workspace,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceSettings {
    pub(crate) diagnostics_mode: DiagnosticsMode,
    pub(crate) trace_server: TraceMode,
    pub(crate) format_enable: bool,
    pub(crate) lint_enable: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TraceMode {
    #[default]
    Off,
    Messages,
    Verbose,
}

pub(crate) struct DocumentStore {
    documents: BTreeMap<String, DocumentState>,
    settings: WorkspaceSettings,
}

pub(crate) struct DocumentState {
    uri: String,
    path: PathBuf,
    version: Option<i32>,
    text: String,
}

impl DocumentStore {
    pub(crate) fn new() -> Self {
        Self {
            documents: BTreeMap::new(),
            settings: WorkspaceSettings {
                diagnostics_mode: DiagnosticsMode::OpenFiles,
                trace_server: TraceMode::Off,
                format_enable: true,
                lint_enable: true,
            },
        }
    }

    pub(crate) fn settings(&self) -> &WorkspaceSettings {
        &self.settings
    }

    pub(crate) fn apply_settings(&mut self, settings: WorkspaceSettings) {
        self.settings = settings;
    }

    pub(crate) fn open(
        &mut self,
        uri: String,
        language_id: &str,
        version: Option<i32>,
        text: String,
    ) -> LspResult<()> {
        if language_id != crate::capabilities::LANGUAGE_ID {
            return Err(LspError::invalid_params(format!(
                "unsupported language id {language_id:?}; expected sifr"
            )));
        }
        let path = uri_to_path(&uri)?;
        let state = DocumentState::new(uri.clone(), path, version, text);
        self.documents.insert(uri, state);
        Ok(())
    }

    pub(crate) fn apply_compacted_change(
        &mut self,
        uri: &str,
        version: Option<i32>,
        change: &CompactedDocumentChange,
        position_encoding: PositionEncoding,
    ) -> LspResult<bool> {
        let state = self.document_mut(uri)?;
        state.reject_stale(version)?;
        let previous = state.text.clone();
        for item in &change.changes {
            match item {
                DocumentContentChange::Full { text } => {
                    state.text.clone_from(text);
                }
                DocumentContentChange::Incremental { range, text } => {
                    state.apply_incremental_change(range, text, position_encoding)?;
                }
            }
        }
        state.version = version;
        Ok(state.text != previous)
    }

    pub(crate) fn save(&mut self, uri: &str, text: Option<String>) -> bool {
        let Some(state) = self.documents.get_mut(uri) else {
            return false;
        };
        if let Some(text) = text {
            state.text = text;
        }
        true
    }

    pub(crate) fn close(&mut self, uri: &str) -> bool {
        self.documents.remove(uri).is_some()
    }

    pub(crate) fn document(&self, uri: &str) -> LspResult<&DocumentState> {
        self.documents
            .get(uri)
            .ok_or_else(|| LspError::invalid_params(format!("document is not open: {uri}")))
    }

    pub(crate) fn document_mut(&mut self, uri: &str) -> LspResult<&mut DocumentState> {
        self.documents
            .get_mut(uri)
            .ok_or_else(|| LspError::invalid_params(format!("document is not open: {uri}")))
    }

    pub(crate) fn document_uris(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    pub(crate) fn documents(&self) -> impl Iterator<Item = &DocumentState> {
        self.documents.values()
    }
}

impl DocumentState {
    fn new(uri: String, path: PathBuf, version: Option<i32>, text: String) -> Self {
        Self {
            uri,
            path,
            version,
            text,
        }
    }

    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    pub(crate) fn version(&self) -> Option<i32> {
        self.version
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn reject_stale(&self, version: Option<i32>) -> LspResult<()> {
        if let (Some(next), Some(current)) = (version, self.version) {
            if next <= current {
                return Err(LspError::invalid_params(format!(
                    "stale document version {next}; current version is {current}"
                )));
            }
        }
        Ok(())
    }

    fn apply_incremental_change(
        &mut self,
        range: &serde_json::Value,
        text: &str,
        position_encoding: PositionEncoding,
    ) -> LspResult<()> {
        let range = crate::conversion::lsp_range(range, &self.text, position_encoding)?;
        let start = usize::try_from(range.start().to_u32())
            .map_err(|_| LspError::invalid_params("incremental edit start is out of range"))?;
        let end = usize::try_from(range.end().to_u32())
            .map_err(|_| LspError::invalid_params("incremental edit end is out of range"))?;
        if start > end || end > self.text.len() {
            return Err(LspError::invalid_params(
                "incremental edit range is invalid",
            ));
        }
        self.text.replace_range(start..end, text);
        Ok(())
    }
}
