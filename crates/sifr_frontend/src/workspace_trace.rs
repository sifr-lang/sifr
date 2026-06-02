use crate::{
    FrontendReuseStats, SourcePath, WorkspaceDirtyScopeReport, WorkspaceRevision,
    WorkspaceSessionTarget, WorkspaceSnapshotId,
};
use std::fmt::Write as _;

const MAX_TRACE_EVENTS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkspaceTracePhase {
    SourceUpdate,
    Parse,
    Lower,
    TypeCheck,
    Ownership,
    Flow,
    Cache,
    Invalidation,
    Scheduler,
    Cancellation,
    StaleRejection,
    LspTiming,
}

impl WorkspaceTracePhase {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceUpdate => "source_update",
            Self::Parse => "parse",
            Self::Lower => "lower",
            Self::TypeCheck => "type_check",
            Self::Ownership => "ownership",
            Self::Flow => "flow",
            Self::Cache => "cache",
            Self::Invalidation => "invalidation",
            Self::Scheduler => "scheduler",
            Self::Cancellation => "cancellation",
            Self::StaleRejection => "stale_rejection",
            Self::LspTiming => "lsp_timing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTraceEvent {
    pub sequence: u64,
    pub phase: WorkspaceTracePhase,
    pub snapshot_id: Option<WorkspaceSnapshotId>,
    pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceTraceLog {
    pub events: Vec<WorkspaceTraceEvent>,
}

impl WorkspaceTraceLog {
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        for event in &self.events {
            let snapshot = event
                .snapshot_id
                .map_or_else(|| "-".to_string(), |id| id.as_u64().to_string());
            let _ = writeln!(
                output,
                "{:04} phase={} snapshot={} {}",
                event.sequence,
                event.phase.as_str(),
                snapshot,
                event.detail
            );
        }
        output
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceStatusSnapshot {
    pub snapshot_id: WorkspaceSnapshotId,
    pub revision: WorkspaceRevision,
    pub target_kind: &'static str,
    pub open_file_count: usize,
    pub project_count: usize,
    pub source_file_count: usize,
    pub module_count: usize,
    pub dependency_count: usize,
    pub cache: WorkspaceCacheStatus,
    pub index_readiness: Vec<WorkspaceIndexReadinessStatus>,
    pub last_update_latency_ms: Option<u64>,
    pub memory: WorkspaceMemoryCounters,
}

impl WorkspaceStatusSnapshot {
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        let _ = writeln!(
            output,
            "snapshot={} revision={} target={} open_files={} projects={} sources={} modules={} dependencies={}",
            self.snapshot_id.as_u64(),
            self.revision.as_u64(),
            self.target_kind,
            self.open_file_count,
            self.project_count,
            self.source_file_count,
            self.module_count,
            self.dependency_count
        );
        let _ = writeln!(
            output,
            "cache parse={} source_map={} hir={} diagnostics={} indexes={}",
            self.cache.parse_entries,
            self.cache.source_map_entries,
            self.cache.hir_entries,
            self.cache.diagnostics_entries,
            self.cache.index_entries
        );
        let _ = writeln!(
            output,
            "memory source_text_bytes={} overlay_text_bytes={} retained_watchers={} retained_configs={} retained_build_info={}",
            self.memory.source_text_bytes,
            self.memory.overlay_text_bytes,
            self.memory.retained_watchers,
            self.memory.retained_configs,
            self.memory.retained_build_info
        );
        if let Some(latency) = self.last_update_latency_ms {
            let _ = writeln!(output, "last_update_latency_ms={latency}");
        }
        for readiness in &self.index_readiness {
            let _ = writeln!(
                output,
                "index bucket={} readiness={}",
                readiness.bucket, readiness.readiness
            );
        }
        output
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceCacheStatus {
    pub parse_entries: usize,
    pub source_map_entries: usize,
    pub hir_entries: usize,
    pub diagnostics_entries: usize,
    pub index_entries: usize,
}

impl From<FrontendReuseStats> for WorkspaceCacheStatus {
    fn from(stats: FrontendReuseStats) -> Self {
        Self {
            parse_entries: stats.parse_entries,
            source_map_entries: stats.source_map_entries,
            hir_entries: stats.hir_entries,
            diagnostics_entries: stats.diagnostics_entries,
            index_entries: stats.index_entries,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceIndexReadinessStatus {
    pub bucket: String,
    pub readiness: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceMemoryCounters {
    pub source_text_bytes: usize,
    pub overlay_text_bytes: usize,
    pub retained_watchers: usize,
    pub retained_configs: usize,
    pub retained_build_info: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceDebugSnapshot {
    pub status: WorkspaceStatusSnapshot,
    pub trace: WorkspaceTraceLog,
}

impl WorkspaceDebugSnapshot {
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        output.push_str("[status]\n");
        output.push_str(&self.status.render_text());
        output.push_str("[trace]\n");
        output.push_str(&self.trace.render_text());
        output
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WorkspaceTraceState {
    next_sequence: u64,
    events: Vec<WorkspaceTraceEvent>,
    last_update_latency_ms: Option<u64>,
}

impl WorkspaceTraceState {
    pub(crate) fn record(&mut self, phase: WorkspaceTracePhase, detail: impl Into<String>) {
        self.prune_before_push();
        let event = WorkspaceTraceEvent {
            sequence: self.next_sequence,
            phase,
            snapshot_id: None,
            detail: detail.into(),
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.events.push(event);
    }

    pub(crate) fn record_with_snapshot(
        &mut self,
        phase: WorkspaceTracePhase,
        snapshot_id: WorkspaceSnapshotId,
        detail: impl Into<String>,
    ) {
        self.prune_before_push();
        let event = WorkspaceTraceEvent {
            sequence: self.next_sequence,
            phase,
            snapshot_id: Some(snapshot_id),
            detail: detail.into(),
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.events.push(event);
    }

    pub(crate) fn record_update_latency_ms(&mut self, latency_ms: u64) {
        self.last_update_latency_ms = Some(latency_ms);
        self.record(
            WorkspaceTracePhase::LspTiming,
            format!("last_update_latency_ms={latency_ms}"),
        );
    }

    pub(crate) fn snapshot(&self) -> WorkspaceTraceLog {
        WorkspaceTraceLog {
            events: self.events.clone(),
        }
    }

    pub(crate) fn last_update_latency_ms(&self) -> Option<u64> {
        self.last_update_latency_ms
    }

    fn prune_before_push(&mut self) {
        if self.events.len() >= MAX_TRACE_EVENTS {
            self.events.remove(0);
        }
    }
}

#[must_use]
pub(crate) fn target_kind(target: &WorkspaceSessionTarget) -> &'static str {
    match target {
        WorkspaceSessionTarget::SingleFile(_) => "single_file",
        WorkspaceSessionTarget::Project(_) => "project",
    }
}

#[must_use]
pub(crate) fn source_path_detail(path: &SourcePath) -> String {
    path.as_path().to_string_lossy().into_owned()
}

#[must_use]
pub(crate) fn dirty_scope_detail(report: &WorkspaceDirtyScopeReport) -> String {
    format!("scope={:?} reasons={:?}", report.scope, report.reasons)
}
