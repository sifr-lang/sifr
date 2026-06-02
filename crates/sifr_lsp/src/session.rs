use crate::analysis_workspace::LspAnalysisWorkspace;
use crate::document_events::{compact_content_changes, CompactedDocumentChange};
use crate::document_store::DocumentStore;
use crate::errors::LspResult;
use crate::request_queue::{RequestQueue, ScheduledRequest};
use crate::scheduler::WorkLane;
use lsp_server::RequestId;
use serde_json::Value;
use sifr_analysis::{AnalysisHost, AnalysisSnapshot, FileId};
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::BTreeMap;

pub(crate) struct Session {
    store: DocumentStore,
    analysis: LspAnalysisWorkspace,
    queue: RequestQueue,
    initialized: bool,
    shutdown_requested: bool,
    exit_requested: bool,
    traces: Vec<String>,
    diagnostic_jobs: BTreeMap<String, ScheduledDiagnosticJob>,
    next_diagnostic_sequence: u64,
}

pub(crate) struct DocumentChangeSummary {
    pub(crate) raw_change_count: usize,
    pub(crate) compacted_change_count: usize,
    pub(crate) text_changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScheduledDiagnosticJob {
    pub(crate) uri: String,
    pub(crate) version: Option<i32>,
    sequence: u64,
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            store: DocumentStore::new(),
            analysis: LspAnalysisWorkspace::default(),
            queue: RequestQueue::default(),
            initialized: false,
            shutdown_requested: false,
            exit_requested: false,
            traces: Vec::new(),
            diagnostic_jobs: BTreeMap::new(),
            next_diagnostic_sequence: 0,
        }
    }

    pub(crate) fn store(&self) -> &DocumentStore {
        &self.store
    }

    pub(crate) fn store_mut(&mut self) -> &mut DocumentStore {
        &mut self.store
    }

    pub(crate) fn open_document(
        &mut self,
        uri: String,
        language_id: &str,
        version: Option<i32>,
        text: String,
    ) -> LspResult<()> {
        self.store.open(uri.clone(), language_id, version, text)?;
        let document = self.store.document(&uri)?;
        self.analysis.open_document(document);
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
        let text_changed = self
            .store
            .apply_compacted_change(uri, version, &compacted)?;
        let document = self.store.document(uri)?;
        self.analysis.update_document(document);
        Ok(DocumentChangeSummary {
            raw_change_count: compacted.raw_change_count,
            compacted_change_count: compacted.changes.len(),
            text_changed,
        })
    }

    pub(crate) fn save_document(&mut self, uri: &str, text: Option<String>) -> LspResult<bool> {
        if !self.store.save(uri, text) {
            return Ok(false);
        }
        let document = self.store.document(uri)?;
        self.analysis.update_document(document);
        Ok(true)
    }

    pub(crate) fn close_document(&mut self, uri: &str) -> bool {
        self.diagnostic_jobs.remove(uri);
        self.analysis.close_document(uri);
        self.store.close(uri)
    }

    pub(crate) fn record_watcher_events(&mut self, event_count: usize) {
        self.analysis.record_watcher_events(event_count);
        self.trace(format!(
            "recorded {event_count} compacted workspace watcher event(s)"
        ));
    }

    pub(crate) fn document_uris(&self) -> Vec<String> {
        self.store.document_uris()
    }

    pub(crate) fn load_diagnostics(&self, uri: &str) -> &[RenderedDiagnostic] {
        self.analysis.load_diagnostics(uri)
    }

    pub(crate) fn uri_map(&self) -> BTreeMap<u32, String> {
        self.analysis.uri_map()
    }

    pub(crate) fn source_map(&self) -> BTreeMap<u32, String> {
        self.analysis.source_map(&self.store)
    }

    pub(crate) fn with_document_analysis<T>(
        &mut self,
        uri: &str,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let before_version = self.store.document(uri)?.version();
        let result = {
            let document = self.store.document(uri)?;
            self.analysis.with_document(document, operation)?
        };
        let after_version = self.store.document(uri)?.version();
        if after_version != before_version {
            return Err(crate::errors::LspError::request_cancelled(
                "query result was superseded by a newer document version",
            ));
        }
        Ok(result)
    }

    pub(crate) fn enqueue_request(
        &mut self,
        id: &RequestId,
        method: &str,
        lane: WorkLane,
    ) -> Result<(), &'static str> {
        self.queue.enqueue(id, method, lane)?;
        self.trace(format!(
            "queued request {id:?} method={method} lane={lane:?}"
        ));
        Ok(())
    }

    pub(crate) fn start_next_request(&mut self) -> Option<ScheduledRequest> {
        let scheduled = self.queue.start_next()?;
        self.trace(format!(
            "dispatching request {:?} method={} lane={:?}",
            scheduled.id(),
            scheduled.method(),
            scheduled.lane()
        ));
        Some(scheduled)
    }

    pub(crate) fn finish_request(&mut self, id: &RequestId) {
        self.queue.finish(id);
    }

    pub(crate) fn cancel_request(&mut self, id: &Value) {
        let request_id = if let Some(raw) = id.as_i64() {
            let Ok(raw) = i32::try_from(raw) else {
                return;
            };
            RequestId::from(raw)
        } else if let Some(raw) = id.as_str() {
            RequestId::from(raw.to_string())
        } else {
            return;
        };
        if self.queue.remove_pending(&request_id) {
            self.trace(format!("cancelled request {request_id:?}"));
        }
    }

    pub(crate) fn begin_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.queue.begin_shutdown();
        self.diagnostic_jobs.clear();
    }

    pub(crate) fn clear_diagnostic_jobs(&mut self) {
        self.diagnostic_jobs.clear();
    }

    pub(crate) fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    pub(crate) fn note_initialized(&mut self) {
        if self.initialized {
            self.trace("ignored duplicate initialized notification".to_string());
        }
        self.initialized = true;
    }

    pub(crate) fn note_exit_notification(&mut self) {
        self.exit_requested = true;
    }

    pub(crate) fn trace(&mut self, message: String) {
        self.traces.push(message);
    }

    pub(crate) fn schedule_document_diagnostics(
        &mut self,
        uri: &str,
    ) -> LspResult<ScheduledDiagnosticJob> {
        let version = self.store.document(uri)?.version();
        let sequence = if let Some(existing) = self.diagnostic_jobs.get(uri) {
            existing.sequence
        } else {
            let sequence = self.next_diagnostic_sequence;
            self.next_diagnostic_sequence = self.next_diagnostic_sequence.saturating_add(1);
            sequence
        };
        let job = ScheduledDiagnosticJob {
            uri: uri.to_string(),
            version,
            sequence,
        };
        self.diagnostic_jobs.insert(uri.to_string(), job.clone());
        self.trace(format!(
            "scheduled diagnostics for {uri} at version {:?}",
            job.version
        ));
        Ok(job)
    }

    pub(crate) fn take_next_diagnostic_job(&mut self) -> Option<ScheduledDiagnosticJob> {
        let uri = self
            .diagnostic_jobs
            .iter()
            .min_by_key(|(_, job)| job.sequence)
            .map(|(uri, _)| uri.clone())?;
        self.diagnostic_jobs.remove(&uri)
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
    use serde_json::json;

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
