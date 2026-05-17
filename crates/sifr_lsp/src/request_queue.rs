use lsp_server::RequestId;
use std::collections::BTreeSet;

#[derive(Default)]
pub(crate) struct RequestQueue {
    pending: BTreeSet<String>,
    shutdown_requested: bool,
}

impl RequestQueue {
    pub(crate) fn start(&mut self, id: &RequestId) -> Result<(), &'static str> {
        if self.shutdown_requested {
            return Err("server is shutting down");
        }
        self.pending.insert(request_key(id));
        Ok(())
    }

    pub(crate) fn finish(&mut self, id: &RequestId) {
        self.pending.remove(&request_key(id));
    }

    pub(crate) fn remove_pending(&mut self, id: &RequestId) -> bool {
        self.pending.remove(&request_key(id))
    }

    pub(crate) fn begin_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.pending.clear();
    }
}

fn request_key(id: &RequestId) -> String {
    format!("{id:?}")
}
