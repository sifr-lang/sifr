use super::implementation::AnalysisHost;
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    DocumentVersion, FrontendMode, ProjectRoot, SourcePath, SourceText, WorkspaceSession,
};
use std::path::Path;

impl AnalysisHost {
    pub fn open_project_with_overlays(
        root: &ProjectRoot,
        overlays: Vec<(SourcePath, Option<String>, DocumentVersion, SourceText)>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = WorkspaceSession::project_with_external_defs_and_auxiliary_sources(
            root.clone(),
            sifr_driver::stdlib_external_defs()?,
            sifr_driver::stdlib_tooling_sources()?,
        );
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
        let mut session = WorkspaceSession::single_file_with_external_defs_and_auxiliary_sources(
            path.clone(),
            mode,
            sifr_driver::stdlib_external_defs()?,
            sifr_driver::stdlib_tooling_sources()?,
        );
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
        path: &Path,
    ) -> Result<sifr_frontend::FileId, AnalysisError> {
        self.session
            .context()
            .and_then(|context| {
                context
                    .source_map()
                    .files
                    .into_iter()
                    .find(|file| paths_match(file.canonical_path.as_path(), path))
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

fn paths_match(candidate: &Path, requested: &Path) -> bool {
    if candidate == requested {
        return true;
    }
    // Project source maps may store module paths relative to the project root
    // while LSP document URIs arrive as absolute paths. The lookup is scoped to
    // one analysis host, so a relative suffix match cannot cross project roots.
    if candidate.is_absolute() || !requested.is_absolute() {
        return false;
    }
    requested.ends_with(candidate)
}

#[cfg(test)]
mod tests {
    use super::paths_match;
    use std::path::Path;

    #[test]
    fn path_match_accepts_exact_paths() {
        assert!(paths_match(
            Path::new("/tmp/project/src/main.sifr"),
            Path::new("/tmp/project/src/main.sifr"),
        ));
    }

    #[test]
    fn path_match_accepts_project_relative_source_paths() {
        assert!(paths_match(
            Path::new("src/main.sifr"),
            Path::new("/tmp/project/src/main.sifr"),
        ));
    }

    #[test]
    fn path_match_rejects_unrelated_paths() {
        assert!(!paths_match(
            Path::new("src/main.sifr"),
            Path::new("/tmp/project/src/helper.sifr"),
        ));
    }
}
