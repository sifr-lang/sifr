use crate::document_store::DocumentState;
use crate::errors::{LspError, LspResult};
use sifr_analysis::{
    AnalysisHost, AnalysisSnapshot, DocumentVersion, FileId, FrontendMode, SourcePath, SourceText,
};
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct LspAnalysisWorkspace {
    documents: BTreeMap<String, LspDocumentAnalysis>,
}

struct LspDocumentAnalysis {
    host: Option<AnalysisHost>,
    file: Option<FileId>,
    load_diagnostics: Vec<RenderedDiagnostic>,
}

impl LspAnalysisWorkspace {
    pub(crate) const WATCHER_STORM_THRESHOLD: usize = 64;

    pub(crate) fn open_document(&mut self, document: &DocumentState) {
        let analysis = LspDocumentAnalysis::open(document);
        self.documents.insert(document.uri().to_string(), analysis);
    }

    pub(crate) fn update_document(&mut self, document: &DocumentState) {
        let uri = document.uri().to_string();
        if let Some(analysis) = self.documents.get_mut(&uri) {
            analysis.update(document);
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(uri, analysis);
        }
    }

    pub(crate) fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    pub(crate) fn record_watcher_events(&mut self, event_count: usize) {
        for analysis in self.documents.values_mut() {
            if let Some(host) = analysis.host.as_mut() {
                host.record_watcher_events(event_count, Self::WATCHER_STORM_THRESHOLD);
            }
        }
    }

    pub(crate) fn load_diagnostics(&self, uri: &str) -> &[RenderedDiagnostic] {
        self.documents
            .get(uri)
            .map_or(&[][..], |analysis| analysis.load_diagnostics.as_slice())
    }

    pub(crate) fn uri_map(&self) -> BTreeMap<u32, String> {
        self.documents
            .iter()
            .filter_map(|(uri, analysis)| analysis.file.map(|file| (file.as_u32(), uri.clone())))
            .collect()
    }

    pub(crate) fn source_map(
        &self,
        documents: &crate::document_store::DocumentStore,
    ) -> BTreeMap<u32, String> {
        self.documents
            .iter()
            .filter_map(|(uri, analysis)| {
                let file = analysis.file?;
                let source = documents.document(uri).ok()?.text().to_string();
                Some((file.as_u32(), source))
            })
            .collect()
    }

    pub(crate) fn with_document<T>(
        &mut self,
        document: &DocumentState,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let analysis = self.documents.get_mut(document.uri()).ok_or_else(|| {
            LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            ))
        })?;
        analysis.with_host(document, operation)
    }
}

impl LspDocumentAnalysis {
    fn open(document: &DocumentState) -> Self {
        match AnalysisHost::open_single_file_overlay(
            SourcePath::new(document.path().to_path_buf()),
            Some(document.uri().to_string()),
            document_version(document),
            SourceText::new(document.text().to_string()),
            FrontendMode::SingleFile,
        ) {
            Ok(host) => Self::from_host(host, document),
            Err(diagnostics) => Self {
                host: None,
                file: None,
                load_diagnostics: diagnostics,
            },
        }
    }

    fn update(&mut self, document: &DocumentState) {
        let result = if let Some(host) = self.host.as_mut() {
            host.upsert_overlay_document(
                SourcePath::new(document.path().to_path_buf()),
                Some(document.uri().to_string()),
                document_version(document),
                SourceText::new(document.text().to_string()),
            )
        } else {
            return *self = Self::open(document);
        };
        match result {
            Ok(()) => {
                if let Some(host) = self.host.take() {
                    *self = Self::from_host(host, document);
                }
            }
            Err(diagnostics) => {
                self.host = None;
                self.file = None;
                self.load_diagnostics = diagnostics;
            }
        }
    }

    fn from_host(host: AnalysisHost, document: &DocumentState) -> Self {
        let file = host.document_file_for_path(document.path()).ok();
        Self {
            host: Some(host),
            file,
            load_diagnostics: Vec::new(),
        }
    }

    fn with_host<T>(
        &mut self,
        document: &DocumentState,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let Some(file) = self.file else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let Some(host) = self.host.as_mut() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let snapshot = host.snapshot();
        let result = operation(&snapshot, host, file, document.text())?;
        if !host.is_snapshot_current(&snapshot) {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(result)
    }
}

fn document_version(document: &DocumentState) -> DocumentVersion {
    DocumentVersion::new(i64::from(document.version().unwrap_or_default()))
}
