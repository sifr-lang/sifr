use super::{
    dirty_scope_detail, source_path_detail, target_kind, DiskSourceProvider, DocumentVersion,
    FrontendContext, FrontendInput, FrontendMode, ModuleGraphView, OverlayDocument,
    OverlaySourceProvider, ProjectRoot, SifrBuildInfoCandidate, SifrBuildInfoVerification,
    SourceDependency, SourceMapView, SourcePath, SourceText, TrackingSourceProvider,
    WorkspaceAuxiliarySource, WorkspaceCacheStatus, WorkspaceDebugSnapshot,
    WorkspaceIndexReadinessStatus, WorkspaceMemoryCounters, WorkspaceResidencySnapshot,
    WorkspaceResidencyState, WorkspaceStatusSnapshot, WorkspaceTracePhase, WorkspaceTraceState,
};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_lowering::ExternalDefs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        Self::none()
    }
}

impl WorkspaceDirtyScopeReport {
    #[must_use]
    pub fn none() -> Self {
        Self {
            scope: WorkspaceDirtyScope::None,
            reasons: Vec::new(),
        }
    }

    #[must_use]
    pub fn new(scope: WorkspaceDirtyScope, reasons: Vec<WorkspaceDirtyReason>) -> Self {
        let mut report = Self {
            scope,
            reasons: Vec::new(),
        };
        for reason in reasons {
            report.add_reason(reason);
        }
        report
    }

    pub fn merge(&mut self, other: Self) {
        self.scope = self.scope.merged_with(other.scope);
        for reason in other.reasons {
            self.add_reason(reason);
        }
    }

    fn add_reason(&mut self, reason: WorkspaceDirtyReason) {
        if !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WorkspaceDirtyScope {
    #[default]
    None,
    OneModule {
        path: SourcePath,
    },
    ReverseDependencies {
        path: SourcePath,
    },
    GraphStructure,
    ConfigProject,
    Workspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceDirtyReason {
    DocumentVersionOnly,
    SourceTextChanged,
    ImportSignatureChanged,
    ExportSignatureChanged,
    ParseOptionsChanged,
    CompilerOptionsChanged,
    ConfigChanged,
    PackageManifestChanged,
    PackageGraphChanged,
    FileCreated,
    FileDeleted,
    FileMoved,
    FailedLookupChanged,
    DirectoryEntriesChanged,
    WatcherStorm,
    Unknown,
}

impl WorkspaceDirtyScope {
    fn merged_with(&self, other: Self) -> Self {
        if other.severity() > self.severity() {
            return other;
        }
        if other.severity() < self.severity() {
            return self.clone();
        }
        match (self, other) {
            (Self::OneModule { path: left }, Self::OneModule { path: right }) if left == &right => {
                self.clone()
            }
            (Self::OneModule { .. }, Self::OneModule { .. }) => Self::GraphStructure,
            (
                Self::ReverseDependencies { path: left },
                Self::ReverseDependencies { path: right },
            ) if left == &right => self.clone(),
            (Self::ReverseDependencies { .. }, Self::ReverseDependencies { .. }) => {
                Self::GraphStructure
            }
            (Self::None, Self::None)
            | (Self::GraphStructure, Self::GraphStructure)
            | (Self::ConfigProject, Self::ConfigProject)
            | (Self::Workspace, Self::Workspace) => self.clone(),
            (_, scope) => scope,
        }
    }

    fn severity(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::OneModule { .. } => 1,
            Self::ReverseDependencies { .. } => 2,
            Self::GraphStructure => 3,
            Self::ConfigProject => 4,
            Self::Workspace => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub id: WorkspaceSnapshotId,
    pub revision: WorkspaceRevision,
    pub target: WorkspaceSessionTarget,
    pub overlays: Arc<Vec<OverlayDocument>>,
    pub source_dependencies: Arc<Vec<SourceDependency>>,
    pub source_map: Option<Arc<SourceMapView>>,
    pub module_graph: Option<Arc<ModuleGraphView>>,
    pub compiler_options: Arc<WorkspaceCompilerOptions>,
    pub package_config_identity: Arc<WorkspacePackageConfigIdentity>,
    pub dirty_scope_report: WorkspaceDirtyScopeReport,
    pub cache_registry: WorkspaceCacheRegistryHandles,
    pub residency: Arc<WorkspaceResidencySnapshot>,
    pub debug: Arc<WorkspaceDebugSnapshot>,
}

pub struct WorkspaceSession {
    target: WorkspaceSessionTarget,
    overlays: BTreeMap<PathBuf, OverlayDocument>,
    single_file_source: Option<SourceText>,
    source_dependencies: Vec<SourceDependency>,
    context: Option<FrontendContext>,
    revision: WorkspaceRevision,
    next_snapshot_id: u64,
    snapshot_overlays: Option<Arc<Vec<OverlayDocument>>>,
    snapshot_source_dependencies: Option<Arc<Vec<SourceDependency>>>,
    snapshot_compiler_options: Arc<WorkspaceCompilerOptions>,
    snapshot_package_config_identity: Arc<WorkspacePackageConfigIdentity>,
    base_external_defs: ExternalDefs,
    auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    dirty_scope_report: WorkspaceDirtyScopeReport,
    cache_registry: WorkspaceCacheRegistryHandles,
    residency: WorkspaceResidencyState,
    trace: WorkspaceTraceState,
}

impl WorkspaceSession {
    pub fn open_project(root: ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = Self::project(root);
        session.reload()?;
        Ok(session)
    }

    pub fn open_project_with_external_defs(
        root: ProjectRoot,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session =
            Self::project_with_external_defs_and_auxiliary_sources(root, external_defs, Vec::new());
        session.reload()?;
        Ok(session)
    }

    pub fn open_project_with_external_defs_and_auxiliary_sources(
        root: ProjectRoot,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = Self::project_with_external_defs_and_auxiliary_sources(
            root,
            external_defs,
            auxiliary_sources,
        );
        session.reload()?;
        Ok(session)
    }

    #[must_use]
    pub fn project(root: ProjectRoot) -> Self {
        Self::project_with_external_defs(root, ExternalDefs::default())
    }

    #[must_use]
    pub fn project_with_external_defs(root: ProjectRoot, external_defs: ExternalDefs) -> Self {
        Self::project_with_external_defs_and_auxiliary_sources(root, external_defs, Vec::new())
    }

    #[must_use]
    pub fn project_with_external_defs_and_auxiliary_sources(
        root: ProjectRoot,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Self {
        let compiler_options = WorkspaceCompilerOptions {
            mode: FrontendMode::ProjectEntrypoint,
        };
        let package_config_identity = WorkspacePackageConfigIdentity {
            workspace_root: Some(root.root.clone()),
            entrypoint: Some(root.entrypoint.clone()),
        };
        let target = WorkspaceSessionTarget::Project(root);
        Self::new(
            target,
            compiler_options,
            package_config_identity,
            external_defs,
            auxiliary_sources,
        )
    }

    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::open_single_file_with_external_defs(input, ExternalDefs::default())
    }

    pub fn open_single_file_with_external_defs(
        input: FrontendInput,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::open_single_file_with_external_defs_and_auxiliary_sources(
            input,
            external_defs,
            Vec::new(),
        )
    }

    pub fn open_single_file_with_external_defs_and_auxiliary_sources(
        input: FrontendInput,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut session = Self::single_file_with_external_defs_and_auxiliary_sources(
            input.path.clone(),
            input.mode,
            external_defs,
            auxiliary_sources,
        );
        session.single_file_source = Some(input.source.clone());
        session.context = Some(
            FrontendContext::load_single_file_with_external_defs_and_auxiliary_sources(
                input,
                session.base_external_defs.clone(),
                session.auxiliary_sources.clone(),
            )?,
        );
        session.refresh_residency();
        session.record_compiler_phase_trace();
        Ok(session)
    }

    #[must_use]
    pub fn single_file(path: SourcePath, mode: FrontendMode) -> Self {
        Self::single_file_with_external_defs(path, mode, ExternalDefs::default())
    }

    #[must_use]
    pub fn single_file_with_external_defs(
        path: SourcePath,
        mode: FrontendMode,
        external_defs: ExternalDefs,
    ) -> Self {
        Self::single_file_with_external_defs_and_auxiliary_sources(
            path,
            mode,
            external_defs,
            Vec::new(),
        )
    }

    #[must_use]
    pub fn single_file_with_external_defs_and_auxiliary_sources(
        path: SourcePath,
        mode: FrontendMode,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Self {
        let compiler_options = WorkspaceCompilerOptions { mode };
        let package_config_identity = WorkspacePackageConfigIdentity::default();
        let target = WorkspaceSessionTarget::SingleFile(WorkspaceSingleFileTarget { path, mode });
        Self::new(
            target,
            compiler_options,
            package_config_identity,
            external_defs,
            auxiliary_sources,
        )
    }

    fn new(
        target: WorkspaceSessionTarget,
        compiler_options: WorkspaceCompilerOptions,
        package_config_identity: WorkspacePackageConfigIdentity,
        base_external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Self {
        let residency = WorkspaceResidencyState::for_target(&target);
        let mut trace = WorkspaceTraceState::default();
        trace.record(
            WorkspaceTracePhase::SourceUpdate,
            format!("initialized target={}", target_kind(&target)),
        );
        Self {
            target,
            overlays: BTreeMap::new(),
            single_file_source: None,
            source_dependencies: Vec::new(),
            context: None,
            revision: WorkspaceRevision(0),
            next_snapshot_id: 0,
            snapshot_compiler_options: Arc::new(compiler_options),
            snapshot_package_config_identity: Arc::new(package_config_identity),
            base_external_defs,
            auxiliary_sources,
            snapshot_overlays: None,
            snapshot_source_dependencies: None,
            dirty_scope_report: WorkspaceDirtyScopeReport::default(),
            cache_registry: WorkspaceCacheRegistryHandles::default(),
            residency,
            trace,
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
                let context =
                    FrontendContext::load_project_with_provider_external_defs_and_auxiliary_sources(
                    root,
                    &mut provider,
                    self.base_external_defs.clone(),
                    self.auxiliary_sources.clone(),
                )?;
                let (_, dependencies) = provider.into_parts();
                self.context = Some(context);
                self.source_dependencies = dependencies;
                self.snapshot_source_dependencies = None;
            }
            WorkspaceSessionTarget::SingleFile(target) => {
                let input = self.single_file_input(target);
                self.context = Some(
                    FrontendContext::load_single_file_with_external_defs_and_auxiliary_sources(
                        input,
                        self.base_external_defs.clone(),
                        self.auxiliary_sources.clone(),
                    )?,
                );
                self.source_dependencies = Vec::new();
                self.snapshot_source_dependencies = None;
            }
        }
        self.revision.0 += 1;
        if self.dirty_scope_report.scope == WorkspaceDirtyScope::None
            && self.dirty_scope_report.reasons.is_empty()
        {
            self.dirty_scope_report = if had_context {
                WorkspaceDirtyScopeReport::new(
                    WorkspaceDirtyScope::Workspace,
                    vec![WorkspaceDirtyReason::Unknown],
                )
            } else {
                WorkspaceDirtyScopeReport::default()
            };
        }
        self.refresh_residency();
        self.record_compiler_phase_trace();
        Ok(())
    }

    pub fn record_dirty_scope(&mut self, report: WorkspaceDirtyScopeReport) {
        if report.scope != WorkspaceDirtyScope::None || !report.reasons.is_empty() {
            self.trace.record(
                WorkspaceTracePhase::Invalidation,
                dirty_scope_detail(&report),
            );
            self.dirty_scope_report.merge(report);
            self.revision.0 += 1;
        }
    }

    pub fn record_watcher_events(&mut self, event_count: usize, storm_threshold: usize) {
        if event_count == 0 {
            return;
        }
        let report = if event_count > storm_threshold {
            WorkspaceDirtyScopeReport::new(
                WorkspaceDirtyScope::Workspace,
                vec![WorkspaceDirtyReason::WatcherStorm],
            )
        } else {
            WorkspaceDirtyScopeReport::new(
                WorkspaceDirtyScope::GraphStructure,
                vec![WorkspaceDirtyReason::DirectoryEntriesChanged],
            )
        };
        self.trace.record(
            WorkspaceTracePhase::SourceUpdate,
            format!("watcher_events count={event_count} threshold={storm_threshold}"),
        );
        self.record_dirty_scope(report);
    }

    pub fn upsert_overlay(
        &mut self,
        path: SourcePath,
        uri: Option<String>,
        version: DocumentVersion,
        source: SourceText,
        disk_source: Option<&str>,
    ) {
        let path_for_report = path.clone();
        let previous = self.overlays.get(path_for_report.as_path());
        let had_context = self.context.is_some();
        let text_changed = previous.is_none_or(|overlay| overlay.source != source);
        let overlay = OverlayDocument::new(path, uri, version, source, disk_source);
        self.overlays
            .insert(overlay.path.as_path().to_path_buf(), overlay);
        self.snapshot_overlays = None;
        self.revision.0 += 1;
        self.trace.record(
            WorkspaceTracePhase::SourceUpdate,
            format!(
                "overlay_upsert path={} version={} text_changed={text_changed}",
                source_path_detail(&path_for_report),
                version.as_i64()
            ),
        );
        if had_context {
            self.dirty_scope_report = if text_changed {
                WorkspaceDirtyScopeReport::new(
                    WorkspaceDirtyScope::OneModule {
                        path: path_for_report,
                    },
                    vec![WorkspaceDirtyReason::SourceTextChanged],
                )
            } else {
                WorkspaceDirtyScopeReport::new(
                    WorkspaceDirtyScope::None,
                    vec![WorkspaceDirtyReason::DocumentVersionOnly],
                )
            };
        }
    }

    pub fn remove_overlay(&mut self, path: &Path) -> Option<OverlayDocument> {
        let removed = self.overlays.remove(path);
        if removed.is_some() {
            self.snapshot_overlays = None;
            self.revision.0 += 1;
            self.residency.release_open_file_project(path);
            self.refresh_residency();
            self.dirty_scope_report = WorkspaceDirtyScopeReport::new(
                WorkspaceDirtyScope::OneModule {
                    path: SourcePath::new(path.to_path_buf()),
                },
                vec![WorkspaceDirtyReason::SourceTextChanged],
            );
            self.trace.record(
                WorkspaceTracePhase::SourceUpdate,
                format!("overlay_remove path={}", path.to_string_lossy()),
            );
        }
        removed
    }

    pub fn mark_config_pending_reload(&mut self, path: SourcePath) {
        self.residency.mark_config_pending_reload(path);
        self.record_dirty_scope(WorkspaceDirtyScopeReport::new(
            WorkspaceDirtyScope::ConfigProject,
            vec![WorkspaceDirtyReason::ConfigChanged],
        ));
    }

    pub fn retain_stdlib_root(&mut self, path: SourcePath) {
        self.residency.retain_stdlib_root(path);
    }

    pub fn retain_generated_artifact(&mut self, path: SourcePath) {
        self.residency.retain_generated_artifact(path);
    }

    pub fn record_update_latency_ms(&mut self, latency_ms: u64) {
        self.trace.record_update_latency_ms(latency_ms);
    }

    pub fn record_stale_rejection(&mut self, detail: impl Into<String>) {
        self.trace
            .record(WorkspaceTracePhase::StaleRejection, detail);
    }

    pub fn verify_build_info(
        &mut self,
        candidate: SifrBuildInfoCandidate,
    ) -> SifrBuildInfoVerification {
        let source_map = self.context.as_ref().map(FrontendContext::source_map);
        self.residency.verify_build_info(
            candidate,
            source_map.as_ref(),
            self.snapshot_package_config_identity.as_ref(),
        )
    }

    pub fn record_analysis_document_update(&mut self) {
        self.revision.0 += 1;
        self.dirty_scope_report = WorkspaceDirtyScopeReport::new(
            WorkspaceDirtyScope::Workspace,
            vec![WorkspaceDirtyReason::SourceTextChanged],
        );
    }

    #[must_use]
    pub fn snapshot(&mut self) -> WorkspaceSnapshot {
        let id = WorkspaceSnapshotId(self.next_snapshot_id);
        self.next_snapshot_id += 1;
        let (source_map, module_graph) = self.context.as_mut().map_or((None, None), |context| {
            (
                Some(context.source_map_arc_for_reuse()),
                Some(context.module_graph_arc_for_reuse()),
            )
        });
        let overlays = Arc::clone(
            self.snapshot_overlays
                .get_or_insert_with(|| Arc::new(self.overlays.values().cloned().collect())),
        );
        let source_dependencies = Arc::clone(
            self.snapshot_source_dependencies
                .get_or_insert_with(|| Arc::new(self.source_dependencies.clone())),
        );
        self.trace.record_with_snapshot(
            WorkspaceTracePhase::Cache,
            id,
            self.context.as_ref().map_or_else(
                || "cache unavailable".to_string(),
                |context| format!("cache={:?}", context.cache_reuse_stats()),
            ),
        );
        let residency = self.residency.snapshot();
        let debug = Arc::new(self.debug_snapshot(
            id,
            source_map.as_deref(),
            module_graph.as_deref(),
            residency.as_ref(),
        ));
        WorkspaceSnapshot {
            id,
            revision: self.revision,
            target: self.target.clone(),
            overlays,
            source_dependencies,
            source_map,
            module_graph,
            compiler_options: Arc::clone(&self.snapshot_compiler_options),
            package_config_identity: Arc::clone(&self.snapshot_package_config_identity),
            dirty_scope_report: self.dirty_scope_report.clone(),
            cache_registry: self.cache_registry.clone(),
            residency,
            debug,
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

    fn refresh_residency(&mut self) {
        let source_map = self.context.as_ref().map(FrontendContext::source_map);
        let module_graph = self.context.as_ref().map(FrontendContext::module_graph);
        // Package/config identity is immutable in the current WorkspaceSession
        // model. When config reload starts updating it, this is the handoff
        // that must receive the live identity rather than the snapshot Arc.
        self.residency.refresh_after_reload(
            &self.target,
            &self.overlays,
            &self.source_dependencies,
            source_map.as_ref(),
            module_graph.as_ref(),
            self.snapshot_package_config_identity.as_ref(),
        );
    }

    fn record_compiler_phase_trace(&mut self) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        let module_count = context.module_graph().modules.len();
        self.trace.record(
            WorkspaceTracePhase::Parse,
            format!("modules={module_count}"),
        );
        self.trace.record(
            WorkspaceTracePhase::Lower,
            format!("modules={module_count}"),
        );
        self.trace.record(
            WorkspaceTracePhase::TypeCheck,
            format!("modules={module_count}"),
        );
        self.trace.record(
            WorkspaceTracePhase::Ownership,
            format!("modules={module_count}"),
        );
        self.trace
            .record(WorkspaceTracePhase::Flow, format!("modules={module_count}"));
        self.trace.record(
            WorkspaceTracePhase::Invalidation,
            dirty_scope_detail(&self.dirty_scope_report),
        );
    }

    fn debug_snapshot(
        &self,
        snapshot_id: WorkspaceSnapshotId,
        source_map: Option<&SourceMapView>,
        module_graph: Option<&ModuleGraphView>,
        residency: &WorkspaceResidencySnapshot,
    ) -> WorkspaceDebugSnapshot {
        let cache = self
            .context
            .as_ref()
            .map_or_else(WorkspaceCacheStatus::default, |context| {
                context.cache_reuse_stats().into()
            });
        let source_file_count = source_map.map_or(0, |map| map.files.len());
        let source_text_bytes = source_map.map_or(0, |map| {
            map.files
                .iter()
                .map(|file| file.source.as_str().len())
                .sum()
        });
        let module_count = module_graph.map_or(0, |graph| graph.modules.len());
        WorkspaceDebugSnapshot {
            status: WorkspaceStatusSnapshot {
                snapshot_id,
                revision: self.revision,
                target_kind: target_kind(&self.target),
                open_file_count: self.overlays.len(),
                project_count: residency.projects.len(),
                source_file_count,
                module_count,
                dependency_count: self.source_dependencies.len(),
                cache,
                index_readiness: vec![WorkspaceIndexReadinessStatus {
                    bucket: "frontend".to_string(),
                    readiness: if module_count == 0 {
                        "unavailable".to_string()
                    } else {
                        "exact".to_string()
                    },
                }],
                last_update_latency_ms: self.trace.last_update_latency_ms(),
                memory: WorkspaceMemoryCounters {
                    source_text_bytes,
                    overlay_text_bytes: self
                        .overlays
                        .values()
                        .map(|overlay| overlay.source.as_str().len())
                        .sum(),
                    retained_watchers: residency.watchers.len(),
                    retained_configs: residency.configs.len(),
                    retained_build_info: residency.build_info.is_some(),
                },
            },
            trace: self.trace.snapshot(),
        }
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
#[path = "workspace_session_tests.rs"]
mod workspace_session_tests;
