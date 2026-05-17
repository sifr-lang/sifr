use crate::document_store::DocumentStore;
use crate::request_queue::RequestQueue;
use lsp_server::RequestId;
use serde_json::Value;

pub(crate) struct Session {
    store: DocumentStore,
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
        if self.queue.cancel(&request_id) {
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
