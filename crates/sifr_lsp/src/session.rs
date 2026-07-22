use crate::analysis_workspace::{LspAnalysisWorkspace, LspFileMaps, LspWorkspaceSymbol};
use crate::cancellation::CancellationToken;
use crate::diagnostic_jobs::{DiagnosticJobs, ScheduledDiagnosticJob};
use crate::document_events::{compact_content_changes, CompactedDocumentChange};
use crate::document_store::DocumentStore;
use crate::errors::{LspError, LspResult};
use crate::progress::{ProgressHandle, ProgressKind, ProgressState};
use crate::python_declarations::PythonDeclarationCache;
use crate::request_queue::{CancellationTarget, RequestQueue, ScheduledRequest};
use crate::scheduler::WorkLane;
use lsp_server::RequestId;
use serde_json::Value;
use sifr_analysis::{
    AnalysisHost, AnalysisSnapshot, FileId, WorkspaceTraceEvent, WorkspaceTraceLog,
    WorkspaceTracePhase,
};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_source::PositionEncoding;

const MAX_LSP_TRACE_EVENTS: usize = 256;

pub(crate) struct Session {
    store: DocumentStore,
    analysis: LspAnalysisWorkspace,
    queue: RequestQueue,
    progress: ProgressState,
    active_request: Option<CancellationToken>,
    initialized: bool,
    position_encoding: PositionEncoding,
    shutdown_requested: bool,
    exit_requested: bool,
    traces: Vec<WorkspaceTraceEvent>,
    next_trace_sequence: u64,
    diagnostic_jobs: DiagnosticJobs,
    pub(crate) python_declarations: PythonDeclarationCache,
}

pub(crate) struct DocumentChangeSummary {
    pub(crate) raw_change_count: usize,
    pub(crate) compacted_change_count: usize,
    pub(crate) text_changed: bool,
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            store: DocumentStore::new(),
            analysis: LspAnalysisWorkspace::default(),
            queue: RequestQueue::default(),
            progress: ProgressState::default(),
            active_request: None,
            initialized: false,
            position_encoding: PositionEncoding::Utf16,
            shutdown_requested: false,
            exit_requested: false,
            traces: Vec::new(),
            next_trace_sequence: 0,
            diagnostic_jobs: DiagnosticJobs::default(),
            python_declarations: PythonDeclarationCache::default(),
        }
    }

    pub(crate) fn store(&self) -> &DocumentStore {
        &self.store
    }

    pub(crate) fn store_mut(&mut self) -> &mut DocumentStore {
        &mut self.store
    }

    pub(crate) fn position_encoding(&self) -> PositionEncoding {
        self.position_encoding
    }

    pub(crate) fn set_position_encoding(&mut self, encoding: PositionEncoding) {
        self.position_encoding = encoding;
    }

    pub(crate) fn open_document(
        &mut self,
        uri: String,
        language_id: &str,
        version: Option<i32>,
        text: String,
    ) -> LspResult<()> {
        self.python_declarations.invalidate_source();
        self.store.open(uri.clone(), language_id, version, text)?;
        let document = self.store.document(&uri)?;
        if self.analysis.open_document(document) {
            self.analysis.refresh_projects(&self.store);
        }
        Ok(())
    }

    pub(crate) fn change_compacted(
        &mut self,
        uri: &str,
        version: Option<i32>,
        changes: &[Value],
    ) -> LspResult<DocumentChangeSummary> {
        let compacted = compact_content_changes(changes)?;
        self.apply_compacted_change(uri, version, compacted)
    }

    fn apply_compacted_change(
        &mut self,
        uri: &str,
        version: Option<i32>,
        compacted: CompactedDocumentChange,
    ) -> LspResult<DocumentChangeSummary> {
        self.python_declarations.invalidate_source();
        let text_changed =
            self.store
                .apply_compacted_change(uri, version, &compacted, self.position_encoding)?;
        let document = self.store.document(uri)?;
        if self.analysis.update_document(document) {
            self.analysis.refresh_projects(&self.store);
        }
        Ok(DocumentChangeSummary {
            raw_change_count: compacted.raw_change_count,
            compacted_change_count: compacted.changes.len(),
            text_changed,
        })
    }

    pub(crate) fn save_document(&mut self, uri: &str, text: Option<String>) -> LspResult<bool> {
        self.python_declarations.invalidate_source();
        if !self.store.save(uri, text) {
            return Ok(false);
        }
        let document = self.store.document(uri)?;
        if self.analysis.update_document(document) {
            self.analysis.refresh_projects(&self.store);
        }
        Ok(true)
    }

    pub(crate) fn close_document(&mut self, uri: &str) -> bool {
        self.python_declarations.invalidate_source();
        self.diagnostic_jobs.remove(uri);
        self.analysis.close_document(uri);
        let closed = self.store.close(uri);
        self.analysis.refresh_projects(&self.store);
        closed
    }

    pub(crate) fn record_watcher_events(&mut self, event_count: usize) {
        self.python_declarations.invalidate_external();
        self.analysis.record_watcher_events(event_count);
        self.trace(
            WorkspaceTracePhase::SourceUpdate,
            format!("compacted_workspace_watcher_events count={event_count}"),
        );
    }

    pub(crate) fn document_uris(&self) -> Vec<String> {
        self.store.document_uris()
    }

    pub(crate) fn can_analyze_document(&self, uri: &str) -> bool {
        self.store
            .document(uri)
            .is_ok_and(|document| self.analysis.can_analyze_document(document))
    }

    pub(crate) fn load_diagnostics(&self, uri: &str) -> &[RenderedDiagnostic] {
        self.analysis.load_diagnostics(uri)
    }

    pub(crate) fn file_maps_for_uri(&self, uri: &str) -> LspResult<LspFileMaps> {
        let document = self.store.document(uri)?;
        self.analysis.file_maps_for_document(document, &self.store)
    }

    pub(crate) fn workspace_symbols(
        &mut self,
        query: &sifr_analysis::SymbolQuery,
    ) -> LspResult<Vec<LspWorkspaceSymbol>> {
        self.check_active_request_cancelled()?;
        let symbols = self.analysis.workspace_symbols(query)?;
        self.check_active_request_cancelled()?;
        self.trace(
            WorkspaceTracePhase::LspTiming,
            format!("workspace_symbols count={}", symbols.len()),
        );
        Ok(symbols)
    }

    pub(crate) fn with_document_analysis<T>(
        &mut self,
        uri: &str,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        self.check_active_request_cancelled()?;
        let before_version = self.store.document(uri)?.version();
        let result = {
            let document = self.store.document(uri)?;
            self.analysis.with_document(document, operation)?
        };
        self.check_active_request_cancelled()?;
        let after_version = self.store.document(uri)?.version();
        if after_version != before_version {
            self.trace(
                WorkspaceTracePhase::StaleRejection,
                format!(
                    "document_version uri={uri} captured={before_version:?} current={after_version:?}"
                ),
            );
            return Err(crate::errors::LspError::request_cancelled(
                "query result was superseded by a newer document version",
            ));
        }
        self.trace(
            WorkspaceTracePhase::LspTiming,
            format!("query uri={uri} version={after_version:?}"),
        );
        Ok(result)
    }

    pub(crate) fn set_work_done_progress_enabled(&mut self, enabled: bool) {
        self.progress.set_enabled(enabled);
    }

    pub(crate) fn begin_progress(
        &mut self,
        kind: ProgressKind,
        work_units: usize,
    ) -> Option<ProgressHandle> {
        let handle = self.progress.begin(kind, work_units)?;
        self.trace(
            WorkspaceTracePhase::LspTiming,
            format!(
                "progress_start token={} kind={kind:?} units={work_units}",
                handle.token()
            ),
        );
        Some(handle)
    }

    pub(crate) fn end_progress(&mut self, handle: ProgressHandle, message: &str) {
        self.trace(
            WorkspaceTracePhase::LspTiming,
            format!("progress_end token={} message={message}", handle.token()),
        );
        self.progress.end(handle, message);
    }

    pub(crate) fn enqueue_request(
        &mut self,
        id: &RequestId,
        method: &str,
        lane: WorkLane,
    ) -> Result<(), &'static str> {
        self.queue.enqueue(id, method, lane)?;
        self.trace(
            WorkspaceTracePhase::Scheduler,
            format!("queued request={id:?} method={method} lane={lane:?}"),
        );
        Ok(())
    }

    pub(crate) fn start_next_request(&mut self) -> Option<ScheduledRequest> {
        let scheduled = self.queue.start_next()?;
        self.trace(
            WorkspaceTracePhase::Scheduler,
            format!(
                "dispatch request={:?} method={} lane={:?}",
                scheduled.id(),
                scheduled.method(),
                scheduled.lane()
            ),
        );
        Some(scheduled)
    }

    pub(crate) fn begin_request_execution(&mut self, id: &RequestId) -> LspResult<()> {
        self.active_request = Some(CancellationToken::new(id));
        self.check_request_cancelled(id)
    }

    pub(crate) fn finish_request(&mut self, id: &RequestId) {
        if self
            .active_request
            .as_ref()
            .is_some_and(|token| token.request_id() == id)
        {
            self.active_request = None;
        }
        self.queue.finish(id);
    }

    pub(crate) fn cancel_request(&mut self, id: &RequestId) -> CancellationTarget {
        let target = self.queue.mark_cancelled(id);
        if target != CancellationTarget::None {
            self.trace(
                WorkspaceTracePhase::Cancellation,
                format!("cancelled request={id:?} target={target:?}"),
            );
        }
        target
    }

    pub(crate) fn check_request_cancelled(&self, id: &RequestId) -> LspResult<()> {
        if self.queue.is_cancelled(id) {
            Err(LspError::request_cancelled(format!(
                "request {id:?} was cancelled"
            )))
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_active_request_cancelled(&self) -> LspResult<()> {
        if let Some(token) = &self.active_request {
            self.check_request_cancelled(token.request_id())?;
        }
        Ok(())
    }

    pub(crate) fn begin_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.queue.begin_shutdown();
        self.diagnostic_jobs.clear();
        self.active_request = None;
    }

    pub(crate) fn clear_diagnostic_jobs(&mut self) {
        self.diagnostic_jobs.clear();
    }

    pub(crate) fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    pub(crate) fn note_initialized(&mut self) {
        if self.initialized {
            self.trace(
                WorkspaceTracePhase::LspTiming,
                "ignored duplicate initialized notification",
            );
        }
        self.initialized = true;
    }

    pub(crate) fn note_exit_notification(&mut self) {
        self.exit_requested = true;
    }

    pub(crate) fn trace(&mut self, phase: WorkspaceTracePhase, detail: impl Into<String>) {
        if self.traces.len() >= MAX_LSP_TRACE_EVENTS {
            self.traces.remove(0);
        }
        self.traces.push(WorkspaceTraceEvent {
            sequence: self.next_trace_sequence,
            phase,
            snapshot_id: None,
            detail: detail.into(),
        });
        self.next_trace_sequence = self.next_trace_sequence.saturating_add(1);
    }

    pub(crate) fn trace_snapshot(&self) -> WorkspaceTraceLog {
        WorkspaceTraceLog {
            events: self.traces.clone(),
        }
    }

    pub(crate) fn schedule_document_diagnostics(
        &mut self,
        uri: &str,
    ) -> LspResult<ScheduledDiagnosticJob> {
        let version = self.store.document(uri)?.version();
        let job = self.diagnostic_jobs.schedule(uri, version);
        self.trace(
            WorkspaceTracePhase::Scheduler,
            format!("scheduled_diagnostics uri={uri} version={:?}", job.version),
        );
        Ok(job)
    }

    pub(crate) fn take_next_diagnostic_job(&mut self) -> Option<ScheduledDiagnosticJob> {
        self.diagnostic_jobs.take_next()
    }

    pub(crate) fn document_version_matches(
        &self,
        uri: &str,
        version: Option<i32>,
    ) -> LspResult<bool> {
        Ok(self.store.document(uri)?.version() == version)
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::progress::ProgressKind;
    use crate::request_queue::CancellationTarget;
    use lsp_server::RequestId;
    use serde_json::json;
    use sifr_analysis::WorkspaceTracePhase;

    #[path = "project_ownership_tests.rs"]
    mod project_ownership_tests;
    #[path = "python_declaration_tests.rs"]
    mod python_declaration_tests;
    #[path = "sysroot_request_tests.rs"]
    mod sysroot_request_tests;

    #[test]
    fn open_document_analysis_uses_unsaved_overlay_text() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return \"disk\"\n")
            .expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(7),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open overlay document");

        let diagnostics =
            crate::diagnostics::document_diagnostics(&mut session, &uri).expect("diagnostics");
        assert!(
            diagnostics.is_empty(),
            "diagnostics should be computed from the unsaved overlay, not disk text"
        );
    }

    #[test]
    fn changed_document_keeps_analysis_in_session_workspace() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        session
            .change_compacted(
                &uri,
                Some(2),
                &[json!({"text": "def main() -> int:\n    return \"changed\"\n"})],
            )
            .expect("change document");

        let diagnostics =
            crate::diagnostics::document_diagnostics(&mut session, &uri).expect("diagnostics");
        assert!(
            !diagnostics.is_empty(),
            "diagnostics should reflect the changed overlay text"
        );
        assert_eq!(
            session.store().document(&uri).expect("document").version(),
            Some(2)
        );
    }

    #[test]
    fn active_request_cancellation_fails_scheduler_boundary_checks() {
        let mut session = Session::new();
        let id = RequestId::from(7);
        session
            .enqueue_request(
                &id,
                "textDocument/references",
                crate::scheduler::WorkLane::Workspace,
            )
            .expect("request should enqueue");
        let scheduled = session.start_next_request().expect("request should start");

        session
            .begin_request_execution(scheduled.id())
            .expect("request should not start cancelled");
        assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);
        assert!(session.check_active_request_cancelled().is_err());
        session.finish_request(&id);
        assert!(session.check_request_cancelled(&id).is_ok());
        let trace = session.trace_snapshot();
        assert!(trace
            .events
            .iter()
            .any(|event| event.phase == WorkspaceTracePhase::Scheduler
                && event.detail.contains("dispatch")));
        assert!(trace
            .events
            .iter()
            .any(|event| event.phase == WorkspaceTracePhase::Cancellation
                && event.detail.contains("InFlight")));
    }

    #[test]
    fn debug_trace_request_exposes_lsp_trace_events() {
        let mut session = Session::new();
        let id = RequestId::from(9);
        session
            .enqueue_request(
                &id,
                "textDocument/completion",
                crate::scheduler::WorkLane::LatencySensitive,
            )
            .expect("request should enqueue");
        let _ = session.start_next_request().expect("request should start");
        assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);

        let trace =
            crate::requests::handle(&mut session, "sifr/debugTrace", serde_json::Value::Null)
                .expect("debug trace request should answer");
        let trace = trace.as_str().expect("trace should be a string");
        assert!(trace.contains("phase=scheduler"));
        assert!(trace.contains("phase=cancellation"));
    }

    #[test]
    fn progress_gate_records_only_delayed_work() {
        let mut session = Session::new();
        session.set_work_done_progress_enabled(true);

        assert!(session
            .begin_progress(ProgressKind::FullDiagnostics, 1)
            .is_none());
        let handle = session
            .begin_progress(ProgressKind::FullDiagnostics, 2)
            .expect("multi-document diagnostics should report progress");
        session.end_progress(handle, "checked 2 document(s)");
    }

    #[test]
    fn document_change_batch_compacts_before_analysis_update() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        let summary = session
            .change_compacted(
                &uri,
                Some(2),
                &[
                    json!({"text": "def main() -> int:\n    return 2\n"}),
                    json!({"text": "def main() -> int:\n    return 3\n"}),
                ],
            )
            .expect("compact change batch");

        assert_eq!(summary.raw_change_count, 2);
        assert_eq!(summary.compacted_change_count, 1);
        assert!(summary.text_changed);
        assert_eq!(
            session.store().document(&uri).expect("document").text(),
            "def main() -> int:\n    return 3\n"
        );
        assert_eq!(
            session.store().document(&uri).expect("document").version(),
            Some(2)
        );
    }

    #[test]
    fn diagnostic_scheduling_debounces_to_latest_document_version() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        session
            .schedule_document_diagnostics(&uri)
            .expect("schedule diagnostics");
        session
            .change_compacted(
                &uri,
                Some(2),
                &[json!({"text": "def main() -> int:\n    return 2\n"})],
            )
            .expect("change document");
        session
            .schedule_document_diagnostics(&uri)
            .expect("reschedule diagnostics");

        let job = session
            .take_next_diagnostic_job()
            .expect("latest diagnostics job should remain");
        assert_eq!(job.version, Some(2));
        assert!(session.take_next_diagnostic_job().is_none());
    }

    #[test]
    fn diagnostic_job_version_guard_rejects_stale_capture() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        let job = session
            .schedule_document_diagnostics(&uri)
            .expect("schedule diagnostics");
        session
            .change_compacted(
                &uri,
                Some(2),
                &[json!({"text": "def main() -> int:\n    return 2\n"})],
            )
            .expect("change document");

        assert!(!session
            .document_version_matches(&job.uri, job.version)
            .expect("document should still exist"));
        session.trace(
            WorkspaceTracePhase::StaleRejection,
            format!("diagnostic_job_version captured={:?}", job.version),
        );
        assert!(session
            .trace_snapshot()
            .render_text()
            .contains("phase=stale_rejection"));
    }

    #[test]
    fn close_document_discards_pending_diagnostic_job() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("write disk source");
        let uri = url::Url::from_file_path(&path)
            .expect("file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        session
            .schedule_document_diagnostics(&uri)
            .expect("schedule diagnostics");

        assert!(session.close_document(&uri));
        assert!(session.take_next_diagnostic_job().is_none());
    }

    #[test]
    fn diagnostic_reschedule_preserves_original_queue_order() {
        let temp = tempfile::tempdir().expect("temp dir");
        let first_path = temp.path().join("first.sifr");
        let second_path = temp.path().join("second.sifr");
        std::fs::write(&first_path, "def main() -> int:\n    return 1\n")
            .expect("write first source");
        std::fs::write(&second_path, "def main() -> int:\n    return 2\n")
            .expect("write second source");
        let first_uri = url::Url::from_file_path(&first_path)
            .expect("first file uri")
            .to_string();
        let second_uri = url::Url::from_file_path(&second_path)
            .expect("second file uri")
            .to_string();

        let mut session = Session::new();
        session
            .open_document(
                first_uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open first document");
        session
            .open_document(
                second_uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 2\n".to_string(),
            )
            .expect("open second document");

        session
            .schedule_document_diagnostics(&first_uri)
            .expect("schedule first diagnostics");
        session
            .schedule_document_diagnostics(&second_uri)
            .expect("schedule second diagnostics");
        session
            .change_compacted(
                &first_uri,
                Some(2),
                &[json!({"text": "def main() -> int:\n    return 3\n"})],
            )
            .expect("change first document");
        session
            .schedule_document_diagnostics(&first_uri)
            .expect("reschedule first diagnostics");

        let first_job = session
            .take_next_diagnostic_job()
            .expect("first diagnostics job should remain first");
        let second_job = session
            .take_next_diagnostic_job()
            .expect("second diagnostics job should remain second");
        assert_eq!(first_job.uri, first_uri);
        assert_eq!(first_job.version, Some(2));
        assert_eq!(second_job.uri, second_uri);
    }
}
