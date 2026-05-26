use crate::conversion::uri_to_path;
use crate::errors::{LspError, LspResult};
use sifr_analysis::{AnalysisHost, FileId, FrontendInput, SourcePath, SourceText};
use sifr_diagnostics::RenderedDiagnostic;
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
    analysis: DocumentAnalysis,
}

#[derive(Default)]
struct DocumentAnalysis {
    host: Option<AnalysisHost>,
    file: Option<FileId>,
    load_diagnostics: Vec<RenderedDiagnostic>,
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

    pub(crate) fn change_full(
        &mut self,
        uri: &str,
        version: Option<i32>,
        text: String,
    ) -> LspResult<()> {
        let state = self.document_mut(uri)?;
        state.reject_stale(version)?;
        state.version = version;
        state.text = text;
        state.rebuild();
        Ok(())
    }

    pub(crate) fn change_incremental(
        &mut self,
        uri: &str,
        version: Option<i32>,
        range: &serde_json::Value,
        text: &str,
    ) -> LspResult<()> {
        let state = self.document_mut(uri)?;
        state.reject_stale(version)?;
        let range = crate::conversion::lsp_range(range, &state.text)?;
        let start = usize::try_from(range.start().to_u32())
            .map_err(|_| LspError::invalid_params("incremental edit start is out of range"))?;
        let end = usize::try_from(range.end().to_u32())
            .map_err(|_| LspError::invalid_params("incremental edit end is out of range"))?;
        if start > end || end > state.text.len() {
            return Err(LspError::invalid_params(
                "incremental edit range is invalid",
            ));
        }
        state.text.replace_range(start..end, text);
        state.version = version;
        state.rebuild();
        Ok(())
    }

    pub(crate) fn save(&mut self, uri: &str, text: Option<String>) -> bool {
        let Some(state) = self.documents.get_mut(uri) else {
            return false;
        };
        if let Some(text) = text {
            state.text = text;
            state.rebuild();
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

    pub(crate) fn documents_mut(&mut self) -> impl Iterator<Item = &mut DocumentState> {
        self.documents.values_mut()
    }

    pub(crate) fn uri_map(&self) -> BTreeMap<u32, String> {
        self.documents
            .values()
            .filter_map(|document| {
                document
                    .file()
                    .map(|file| (file.as_u32(), document.uri().to_string()))
            })
            .collect()
    }

    pub(crate) fn source_map(&self) -> BTreeMap<u32, String> {
        self.documents
            .values()
            .filter_map(|document| {
                document
                    .file()
                    .map(|file| (file.as_u32(), document.text().to_string()))
            })
            .collect()
    }
}

impl DocumentState {
    fn new(uri: String, path: PathBuf, version: Option<i32>, text: String) -> Self {
        let mut state = Self {
            uri,
            path,
            version,
            text,
            analysis: DocumentAnalysis::default(),
        };
        state.rebuild();
        state
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

    pub(crate) fn file(&self) -> Option<FileId> {
        self.analysis.file
    }

    pub(crate) fn load_diagnostics(&self) -> &[RenderedDiagnostic] {
        &self.analysis.load_diagnostics
    }

    pub(crate) fn with_host<T>(
        &mut self,
        operation: impl FnOnce(&mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let Some(file) = self.analysis.file else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                self.path.display()
            )));
        };
        let Some(host) = self.analysis.host.as_mut() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                self.path.display()
            )));
        };
        let snapshot = host.snapshot();
        let result = operation(host, file, &self.text)?;
        if snapshot.revision() != host.snapshot().revision() {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(result)
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

    fn rebuild(&mut self) {
        let input = FrontendInput {
            path: SourcePath::new(self.path.clone()),
            source: SourceText::new(self.text.clone()),
            mode: sifr_analysis::FrontendMode::SingleFile,
        };
        match AnalysisHost::open_single_file(input) {
            Ok(host) => {
                let file = host.files().first().copied();
                self.analysis = DocumentAnalysis {
                    host: Some(host),
                    file,
                    load_diagnostics: Vec::new(),
                };
            }
            Err(diagnostics) => {
                self.analysis = DocumentAnalysis {
                    host: None,
                    file: None,
                    load_diagnostics: diagnostics,
                };
            }
        }
    }
}
