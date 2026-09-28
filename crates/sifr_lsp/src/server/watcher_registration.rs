use crate::errors::{LspError, LspResult};
use lsp_server::{Connection, Message, Request, RequestId, Response};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use url::Url;

/// Dynamic registration is authority only after the client acknowledges this request.
pub(super) struct WatcherRegistration {
    supported: bool,
    pending: Option<RequestId>,
    acknowledged: bool,
    next_id: u64,
    registered_id: Option<String>,
    workspace_roots: BTreeSet<PathBuf>,
}

impl WatcherRegistration {
    pub(super) fn new(initialize_params: &Value) -> Self {
        let workspace_roots = match initialize_params
            .get("workspaceFolders")
            .and_then(Value::as_array)
        {
            Some(folders) => folders
                .iter()
                .filter_map(|folder| folder.get("uri").and_then(Value::as_str))
                .filter_map(file_path)
                .collect(),
            None => initialize_params
                .get("rootUri")
                .and_then(Value::as_str)
                .and_then(file_path)
                .into_iter()
                .collect(),
        };
        Self {
            supported: initialize_params
                .pointer("/capabilities/workspace/didChangeWatchedFiles/dynamicRegistration")
                .and_then(Value::as_bool)
                == Some(true),
            pending: None,
            acknowledged: false,
            next_id: 0,
            registered_id: None,
            workspace_roots,
        }
    }

    pub(super) fn covers(&self, path: &Path) -> bool {
        self.acknowledged
            && self
                .workspace_roots
                .iter()
                .any(|root| path.starts_with(root))
    }

    pub(super) fn change_workspace_folders(&mut self, params: &Value) {
        if let Some(removed) = params.pointer("/event/removed").and_then(Value::as_array) {
            for root in removed
                .iter()
                .filter_map(|folder| folder.get("uri").and_then(Value::as_str))
                .filter_map(file_path)
            {
                self.workspace_roots.remove(&root);
            }
        }
        if let Some(added) = params.pointer("/event/added").and_then(Value::as_array) {
            for root in added
                .iter()
                .filter_map(|folder| folder.get("uri").and_then(Value::as_str))
                .filter_map(file_path)
            {
                self.workspace_roots.insert(root);
            }
        }
        self.acknowledged = false;
    }

    pub(super) fn invalidate(&mut self) {
        self.acknowledged = false;
    }

    pub(super) fn started(&self) -> bool {
        self.next_id != 0
    }

    #[cfg(test)]
    pub(super) fn acknowledged(&self) -> bool {
        self.acknowledged
    }

    pub(super) fn register(&mut self, connection: &Connection) -> LspResult<()> {
        if !self.supported {
            return Ok(());
        }
        self.acknowledged = false;
        let old_id = self.registered_id.take().or_else(|| {
            self.pending
                .take()
                .map(|_| format!("sifr/watched-files/{}", self.next_id))
        });
        if let Some(old_id) = old_id {
            self.next_id += 1;
            connection
                .sender
                .send(Message::Request(Request {
                    id: RequestId::from(format!("sifr/unregister-watchers/{}", self.next_id)),
                    method: "client/unregisterCapability".to_string(),
                    params: json!({"unregisterations": [{
                        "id": old_id, "method": "workspace/didChangeWatchedFiles"
                    }]}),
                }))
                .map_err(|error| {
                    LspError::internal(format!("failed to unregister watched files: {error}"))
                })?;
        }
        self.next_id += 1;
        let id = RequestId::from(format!("sifr/watched-files/{}", self.next_id));
        self.pending = Some(id.clone());
        let request = Request {
            id,
            method: "client/registerCapability".to_string(),
            params: json!({
                "registrations": [{
                    "id": format!("sifr/watched-files/{}", self.next_id),
                    "method": "workspace/didChangeWatchedFiles",
                    "registerOptions": { "watchers": [{ "globPattern": "**/*", "kind": 7 }] }
                }]
            }),
        };
        connection
            .sender
            .send(Message::Request(request))
            .map_err(|error| {
                LspError::internal(format!("failed to register watched files: {error}"))
            })
    }

    pub(super) fn respond(&mut self, response: &Response) {
        if self.pending.as_ref() != Some(&response.id) {
            return;
        }
        self.pending = None;
        self.acknowledged = response.response_result.is_ok();
        if self.acknowledged {
            self.registered_id = Some(format!("sifr/watched-files/{}", self.next_id));
        }
    }
}

fn file_path(uri: &str) -> Option<PathBuf> {
    Url::parse(uri).ok()?.to_file_path().ok()
}
