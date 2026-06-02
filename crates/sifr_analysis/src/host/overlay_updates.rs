use super::implementation::AnalysisHost;
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    DocumentVersion, FrontendMode, ProjectRoot, SourcePath, SourceText, WorkspaceSession,
};

impl AnalysisHost {
    pub fn open_project_with_overlays(
        root: &ProjectRoot,
        overlays: Vec<(SourcePath, Option<String>, DocumentVersion, SourceText)>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = WorkspaceSession::project(root.clone());
        for (path, uri, version, source) in overlays {
            session.upsert_overlay(path, uri, version, source, None);
        }
        session.reload()?;
        Self::new(session)
    }

    pub fn open_single_file_overlay(
        path: SourcePath,
        uri: Option<String>,
        version: DocumentVersion,
        source: SourceText,
        mode: FrontendMode,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = WorkspaceSession::single_file(path.clone(), mode);
        session.upsert_overlay(path, uri, version, source, None);
        session.reload()?;
        Self::new(session)
    }

    pub fn upsert_overlay_document(
        &mut self,
        path: SourcePath,
        uri: Option<String>,
        version: DocumentVersion,
        source: SourceText,
    ) -> Result<(), Vec<RenderedDiagnostic>> {
        self.session
            .upsert_overlay(path, uri, version, source, None);
        self.session.reload()?;
        self.refresh_file_map();
        self.refresh_current_revision();
        self.symbol_index = None;
        self.last_invalidation = None;
        Ok(())
    }

    pub fn record_watcher_events(&mut self, event_count: usize, storm_threshold: usize) {
        self.session
            .record_watcher_events(event_count, storm_threshold);
    }

    pub fn document_file_for_path(
        &self,
        path: &std::path::Path,
    ) -> Result<sifr_frontend::FileId, AnalysisError> {
        self.session
            .context()
            .and_then(|context| {
                context
                    .source_map()
                    .files
                    .into_iter()
                    .find(|file| file.canonical_path.as_path() == path)
                    .map(|file| file.id)
            })
            .ok_or_else(|| {
                AnalysisError::new(
                    AnalysisErrorKind::UnknownFile,
                    format!("analysis file is unavailable for {}", path.display()),
                )
            })
    }
}
