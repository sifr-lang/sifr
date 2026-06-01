use crate::analysis_workspace::LspAnalysisWorkspace;
use crate::document_store::DocumentStore;
use crate::errors::LspResult;
use crate::request_queue::RequestQueue;
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

    pub(crate) fn change_full(
        &mut self,
        uri: &str,
        version: Option<i32>,
        text: String,
    ) -> LspResult<()> {
        self.store.change_full(uri, version, text)?;
        let document = self.store.document(uri)?;
        self.analysis.update_document(document);
        Ok(())
    }

    pub(crate) fn change_incremental(
        &mut self,
        uri: &str,
        version: Option<i32>,
        range: &Value,
        text: &str,
    ) -> LspResult<()> {
        self.store.change_incremental(uri, version, range, text)?;
        let document = self.store.document(uri)?;
        self.analysis.update_document(document);
        Ok(())
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
        self.analysis.close_document(uri);
        self.store.close(uri)
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

    pub(crate) fn start_request(&mut self, id: &RequestId) -> Result<(), &'static str> {
        self.queue.start(id)
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
}

#[cfg(test)]
mod tests {
    use super::Session;

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
            .change_full(
                &uri,
                Some(2),
                "def main() -> int:\n    return \"changed\"\n".to_string(),
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
}
