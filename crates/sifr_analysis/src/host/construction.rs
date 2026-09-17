use super::implementation::{AnalysisHost, revision_from_workspace_snapshot};
use crate::sql_editor_runtime::{SqlEditorRuntime, sql_editor_initialization_diagnostic};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{FrontendInput, ProjectRoot, WorkspaceSession};
use std::collections::BTreeMap;

impl AnalysisHost {
    pub fn open_project(
        compiler: &sifr_driver::CompilerContext,
        root: &ProjectRoot,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let profiles =
            sifr_driver::load_sql_editor_profiles(root.root.as_path(), root.entrypoint.as_path())
                .unwrap_or_else(sifr_driver::PreparedSqlProfiles::from_initialization_failure);
        let session = WorkspaceSession::open_project_with_external_defs_and_auxiliary_sources(
            root.clone(),
            sifr_driver::stdlib_external_defs(compiler)?,
            Vec::new(),
        )?;
        Self::new_with_sql_profiles(compiler, session, profiles)
    }

    pub fn open_single_file(
        compiler: &sifr_driver::CompilerContext,
        input: FrontendInput,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let session = WorkspaceSession::open_single_file_with_external_defs_and_auxiliary_sources(
            input,
            sifr_driver::stdlib_external_defs(compiler)?,
            Vec::new(),
        )?;
        Self::new(compiler, session)
    }

    pub(super) fn new(
        compiler: &sifr_driver::CompilerContext,
        session: WorkspaceSession,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::new_with_sql_profiles(
            compiler,
            session,
            sifr_driver::PreparedSqlProfiles::default(),
        )
    }

    pub(super) fn new_with_sql_profiles(
        compiler: &sifr_driver::CompilerContext,
        mut session: WorkspaceSession,
        profiles: sifr_driver::PreparedSqlProfiles,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        session = session.with_compiler_identity(compiler.identity().clone());
        let snapshot = session.snapshot();
        let Some(current_revision) = revision_from_workspace_snapshot(&snapshot) else {
            return Err(Vec::new());
        };
        let mut host = Self {
            stdlib_navigation: compiler.stdlib_navigation()?,
            compiler: compiler.clone(),
            session,
            file_to_module: BTreeMap::new(),
            symbol_index: None,
            lint_cache: BTreeMap::new(),
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
