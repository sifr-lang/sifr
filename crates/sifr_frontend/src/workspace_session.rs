use super::{
    DiskSourceProvider, DocumentVersion, FrontendContext, FrontendInput, FrontendMode,
    ModuleGraphView, OverlayDocument, OverlaySourceProvider, ProjectRoot, SourceDependency,
    SourceMapView, SourcePath, SourceText, TrackingSourceProvider,
};
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceSnapshotId(u64);

impl WorkspaceSnapshotId {
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceRevision(u64);

impl WorkspaceRevision {
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceSessionTarget {
    SingleFile(WorkspaceSingleFileTarget),
    Project(ProjectRoot),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSingleFileTarget {
    pub path: SourcePath,
    pub mode: FrontendMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCompilerOptions {
    pub mode: FrontendMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspacePackageConfigIdentity {
    pub workspace_root: Option<SourcePath>,
    pub entrypoint: Option<SourcePath>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceCacheRegistryHandles {
    pub parse_generation: u64,
    pub hir_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceDirtyScopeReport {
    pub scope: WorkspaceDirtyScope,
    pub reasons: Vec<WorkspaceDirtyReason>,
}

impl Default for WorkspaceDirtyScopeReport {
    fn default() -> Self {
        Self {
            scope: WorkspaceDirtyScope::None,
            reasons: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkspaceDirtyScope {
    #[default]
    None,
    Workspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceDirtyReason {
    SessionReload,
    OverlayChanged,
    AnalysisDocumentUpdate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub id: WorkspaceSnapshotId,
    pub revision: WorkspaceRevision,
    pub target: WorkspaceSessionTarget,
    pub overlays: Vec<OverlayDocument>,
    pub source_dependencies: Vec<SourceDependency>,
    pub source_map: Option<SourceMapView>,
    pub module_graph: Option<ModuleGraphView>,
    pub compiler_options: WorkspaceCompilerOptions,
    pub package_config_identity: WorkspacePackageConfigIdentity,
    pub dirty_scope_report: WorkspaceDirtyScopeReport,
    pub cache_registry: WorkspaceCacheRegistryHandles,
}

pub struct WorkspaceSession {
    target: WorkspaceSessionTarget,
    overlays: BTreeMap<PathBuf, OverlayDocument>,
    single_file_source: Option<SourceText>,
    source_dependencies: Vec<SourceDependency>,
    context: Option<FrontendContext>,
    revision: WorkspaceRevision,
    next_snapshot_id: u64,
    compiler_options: WorkspaceCompilerOptions,
    package_config_identity: WorkspacePackageConfigIdentity,
    dirty_scope_report: WorkspaceDirtyScopeReport,
    cache_registry: WorkspaceCacheRegistryHandles,
}

impl WorkspaceSession {
    pub fn open_project(root: ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = Self::project(root);
        session.reload()?;
        Ok(session)
    }

    #[must_use]
    pub fn project(root: ProjectRoot) -> Self {
        let package_config_identity = WorkspacePackageConfigIdentity {
            workspace_root: Some(root.root.clone()),
            entrypoint: Some(root.entrypoint.clone()),
        };
        Self::new(
            WorkspaceSessionTarget::Project(root),
            WorkspaceCompilerOptions {
                mode: FrontendMode::ProjectEntrypoint,
            },
            package_config_identity,
        )
    }

    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = Self::single_file(input.path.clone(), input.mode);
        session.single_file_source = Some(input.source.clone());
        session.context = Some(FrontendContext::load_single_file(input)?);
        Ok(session)
    }

    #[must_use]
    pub fn single_file(path: SourcePath, mode: FrontendMode) -> Self {
        Self::new(
            WorkspaceSessionTarget::SingleFile(WorkspaceSingleFileTarget { path, mode }),
            WorkspaceCompilerOptions { mode },
            WorkspacePackageConfigIdentity::default(),
        )
    }

    fn new(
        target: WorkspaceSessionTarget,
        compiler_options: WorkspaceCompilerOptions,
        package_config_identity: WorkspacePackageConfigIdentity,
    ) -> Self {
        Self {
            target,
            overlays: BTreeMap::new(),
            single_file_source: None,
            source_dependencies: Vec::new(),
            context: None,
            revision: WorkspaceRevision(0),
            next_snapshot_id: 0,
            compiler_options,
            package_config_identity,
            dirty_scope_report: WorkspaceDirtyScopeReport::default(),
            cache_registry: WorkspaceCacheRegistryHandles::default(),
        }
    }

    pub fn reload(&mut self) -> Result<(), Vec<RenderedDiagnostic>> {
        let had_context = self.context.is_some();
        match &self.target {
            WorkspaceSessionTarget::Project(root) => {
                let mut overlay_provider = OverlaySourceProvider::new(DiskSourceProvider::new());
                for overlay in self.overlays.values().cloned() {
                    overlay_provider.insert_overlay(overlay);
                }
                let mut provider = TrackingSourceProvider::new(overlay_provider);
                let context = FrontendContext::load_project_with_provider(root, &mut provider)?;
                let (_, dependencies) = provider.into_parts();
                self.context = Some(context);
                self.source_dependencies = dependencies;
            }
            WorkspaceSessionTarget::SingleFile(target) => {
                let input = self.single_file_input(target);
                self.context = Some(FrontendContext::load_single_file(input)?);
                self.source_dependencies = Vec::new();
            }
        }
        self.revision.0 += 1;
        self.dirty_scope_report = if had_context {
            WorkspaceDirtyScopeReport {
                scope: WorkspaceDirtyScope::Workspace,
                reasons: vec![WorkspaceDirtyReason::SessionReload],
            }
        } else {
            WorkspaceDirtyScopeReport::default()
        };
        Ok(())
    }

    pub fn upsert_overlay(
        &mut self,
        path: SourcePath,
        uri: Option<String>,
        version: DocumentVersion,
        source: SourceText,
        disk_source: Option<&str>,
    ) {
        let overlay = OverlayDocument::new(path, uri, version, source, disk_source);
        self.overlays
            .insert(overlay.path.as_path().to_path_buf(), overlay);
        self.revision.0 += 1;
        self.dirty_scope_report = WorkspaceDirtyScopeReport {
            scope: WorkspaceDirtyScope::Workspace,
            reasons: vec![WorkspaceDirtyReason::OverlayChanged],
        };
    }

    pub fn remove_overlay(&mut self, path: &Path) -> Option<OverlayDocument> {
        let removed = self.overlays.remove(path);
        if removed.is_some() {
            self.revision.0 += 1;
            self.dirty_scope_report = WorkspaceDirtyScopeReport {
                scope: WorkspaceDirtyScope::Workspace,
                reasons: vec![WorkspaceDirtyReason::OverlayChanged],
            };
        }
        removed
    }

    pub fn record_analysis_document_update(&mut self) {
        self.revision.0 += 1;
        self.dirty_scope_report = WorkspaceDirtyScopeReport {
            scope: WorkspaceDirtyScope::Workspace,
            reasons: vec![WorkspaceDirtyReason::AnalysisDocumentUpdate],
        };
    }

    #[must_use]
    pub fn snapshot(&mut self) -> WorkspaceSnapshot {
        let id = WorkspaceSnapshotId(self.next_snapshot_id);
        self.next_snapshot_id += 1;
        WorkspaceSnapshot {
            id,
            revision: self.revision,
            target: self.target.clone(),
            overlays: self.overlays.values().cloned().collect(),
            source_dependencies: self.source_dependencies.clone(),
            source_map: self.context.as_ref().map(FrontendContext::source_map),
            module_graph: self.context.as_ref().map(FrontendContext::module_graph),
            compiler_options: self.compiler_options.clone(),
            package_config_identity: self.package_config_identity.clone(),
            dirty_scope_report: self.dirty_scope_report.clone(),
            cache_registry: self.cache_registry.clone(),
        }
    }

    #[must_use]
    pub fn revision(&self) -> WorkspaceRevision {
        self.revision
    }

    #[must_use]
    pub fn overlays(&self) -> &BTreeMap<PathBuf, OverlayDocument> {
        &self.overlays
    }

    #[must_use]
    pub fn source_dependencies(&self) -> &[SourceDependency] {
        &self.source_dependencies
    }

    #[must_use]
    pub fn context(&self) -> Option<&FrontendContext> {
        self.context.as_ref()
    }

    #[must_use]
    pub fn context_mut(&mut self) -> Option<&mut FrontendContext> {
        self.context.as_mut()
    }

    fn single_file_input(&self, target: &WorkspaceSingleFileTarget) -> FrontendInput {
        let source = self.overlays.get(target.path.as_path()).map_or_else(
            || {
                self.single_file_source
                    .clone()
                    .unwrap_or_else(|| SourceText::new(""))
            },
            |overlay| overlay.source.clone(),
        );
        FrontendInput {
            path: target.path.clone(),
            source,
            mode: target.mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WorkspaceDirtyReason, WorkspaceDirtyScope, WorkspaceSession, WorkspaceSessionTarget,
    };
    use crate::{
        DocumentVersion, FrontendMode, ProjectRoot, SourceDependencyKind, SourcePath, SourceText,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn project_session_snapshot_records_overlay_and_dependencies() {
        let temp = TempProject::new("workspace_session_project");
        temp.write("main.sifr", "from helper import value\nprint(value)\n");
        temp.write("helper.sifr", "value = 1\n");
        let root = ProjectRoot {
            root: SourcePath::new(temp.root.clone()),
            entrypoint: SourcePath::new(temp.root.join("main.sifr")),
        };
        let expected_root = root.root.clone();
        let expected_entrypoint = root.entrypoint.clone();
        let mut session = WorkspaceSession::open_project(root).expect("project opens");
        let opened_revision = session.revision().as_u64();
        assert_eq!(
            session.snapshot().dirty_scope_report.scope,
            WorkspaceDirtyScope::None
        );

        session.upsert_overlay(
            SourcePath::new(temp.root.join("helper.sifr")),
            Some("file:///helper.sifr".to_string()),
            DocumentVersion::new(4),
            SourceText::new("value = 2\n"),
            Some("value = 1\n"),
        );
        assert_eq!(session.revision().as_u64(), opened_revision + 1);
        session.reload().expect("overlay-backed reload succeeds");
        assert_eq!(session.revision().as_u64(), opened_revision + 2);
        let snapshot = session.snapshot();
        assert_eq!(
            snapshot.dirty_scope_report.scope,
            WorkspaceDirtyScope::Workspace
        );
        assert_eq!(
            snapshot.dirty_scope_report.reasons,
            vec![WorkspaceDirtyReason::SessionReload]
        );

        assert!(matches!(
            snapshot.target,
            WorkspaceSessionTarget::Project(_)
        ));
        assert_eq!(snapshot.revision, session.revision());
        assert_eq!(
            snapshot.compiler_options.mode,
            FrontendMode::ProjectEntrypoint
        );
        assert_eq!(
            snapshot.package_config_identity.workspace_root,
            Some(expected_root)
        );
        assert_eq!(
            snapshot.package_config_identity.entrypoint,
            Some(expected_entrypoint)
        );
        assert_eq!(snapshot.cache_registry.parse_generation, 0);
        assert_eq!(snapshot.cache_registry.hir_generation, 0);
        assert_eq!(snapshot.overlays.len(), 1);
        assert!(!snapshot.overlays[0].matches_disk);
        assert_eq!(snapshot.overlays[0].source.as_str(), "value = 2\n");
        assert!(snapshot
            .source_dependencies
            .iter()
            .any(|dependency| matches!(dependency.kind, SourceDependencyKind::FileRead)));
        assert_eq!(
            snapshot
                .source_map
                .as_ref()
                .expect("source map exists")
                .files
                .len(),
            2
        );
        assert_eq!(
            snapshot
                .module_graph
                .as_ref()
                .expect("module graph exists")
                .modules
                .len(),
            2
        );

        let removed = session.remove_overlay(&temp.root.join("helper.sifr"));
        assert!(removed.is_some());
        assert_eq!(session.revision().as_u64(), snapshot.revision.as_u64() + 1);
        let removed_snapshot = session.snapshot();
        assert!(removed_snapshot.overlays.is_empty());
        assert_eq!(
            removed_snapshot.dirty_scope_report.reasons,
            vec![WorkspaceDirtyReason::OverlayChanged]
        );
    }

    #[test]
    fn single_file_session_freezes_overlay_state() {
        let path = SourcePath::new("scratch.sifr");
        let mut session = WorkspaceSession::single_file(path.clone(), FrontendMode::SingleFile);
        session.upsert_overlay(
            path,
            Some("file:///scratch.sifr".to_string()),
            DocumentVersion::new(1),
            SourceText::new("x = 1\n"),
            None,
        );
        session.reload().expect("single file overlay reloads");
        let first = session.snapshot();
        let second = session.snapshot();

        assert_eq!(first.id.as_u64(), 0);
        assert_eq!(second.id.as_u64(), 1);
        assert_eq!(first.overlays.len(), 1);
        assert_eq!(first.compiler_options.mode, FrontendMode::SingleFile);
        assert!(first.source_dependencies.is_empty());
        assert!(first.module_graph.is_some());
    }

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "sifr_frontend_{name}_{}_{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create temp project");
            Self { root }
        }

        fn write(&self, relative: &str, source: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, source).expect("write source");
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(Path::new(&self.root));
        }
    }
}
