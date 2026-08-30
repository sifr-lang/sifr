use super::implementation::{AnalysisHost, revision_from_workspace_snapshot};
use crate::sql_editor_runtime::{SqlEditorRuntime, sql_editor_initialization_diagnostic};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{FrontendInput, ProjectRoot, WorkspaceSession};
use std::collections::BTreeMap;

impl AnalysisHost {
    pub fn open_project(root: &ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let profiles =
            sifr_driver::load_sql_editor_profiles(root.root.as_path(), root.entrypoint.as_path())
                .unwrap_or_else(sifr_driver::PreparedSqlProfiles::from_initialization_failure);
        let session = WorkspaceSession::open_project_with_external_defs_and_auxiliary_sources(
            root.clone(),
            sifr_driver::stdlib_external_defs()?,
            sifr_driver::stdlib_tooling_sources()?,
        )?;
        Self::new_with_sql_profiles(session, profiles)
    }

    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        let session = WorkspaceSession::open_single_file_with_external_defs_and_auxiliary_sources(
            input,
            sifr_driver::stdlib_external_defs()?,
            sifr_driver::stdlib_tooling_sources()?,
        )?;
        Self::new(session)
    }

    pub(super) fn new(session: WorkspaceSession) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::new_with_sql_profiles(session, sifr_driver::PreparedSqlProfiles::default())
    }

    pub(super) fn new_with_sql_profiles(
        mut session: WorkspaceSession,
        profiles: sifr_driver::PreparedSqlProfiles,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let snapshot = session.snapshot();
        let Some(current_revision) = revision_from_workspace_snapshot(&snapshot) else {
            return Err(Vec::new());
        };
        let mut host = Self {
            session,
            file_to_module: BTreeMap::new(),
            symbol_index: None,
            last_invalidation: None,
            current_revision,
            sql_editor_runtime: SqlEditorRuntime::new(profiles)
                .map_err(|error| vec![sql_editor_initialization_diagnostic(&error)])?,
        };
        host.refresh_file_map();
        Ok(host)
    }

    pub fn set_sql_cancellation_flag(
        &mut self,
        cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) {
        self.sql_editor_runtime.set_cancellation(cancellation);
    }
}
