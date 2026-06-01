use crate::conversion;
use crate::document_store::{DiagnosticsMode, DocumentState, DocumentStore};
use crate::errors::LspResult;
use lsp_server::{Connection, Message, Notification};
use serde_json::{json, Value};

#[derive(Default)]
pub(crate) struct DiagnosticsController;

impl DiagnosticsController {
    pub(crate) fn publish_document(
        connection: &Connection,
        state: &mut DocumentState,
        mode: DiagnosticsMode,
    ) -> LspResult<()> {
        if mode == DiagnosticsMode::Off {
            return Ok(());
        }
        let diagnostics = document_diagnostics(state)?;
        let params = json!({
            "uri": state.uri(),
            "version": state.version(),
            "diagnostics": diagnostics
        });
        connection
            .sender
            .send(Message::Notification(Notification {
                method: "textDocument/publishDiagnostics".to_string(),
                params,
            }))
            .map_err(|error| {
                crate::errors::LspError::internal(format!("failed to publish diagnostics: {error}"))
            })?;
        Ok(())
    }

    pub(crate) fn publish_all(connection: &Connection, store: &mut DocumentStore) -> LspResult<()> {
        let mode = store.settings().diagnostics_mode;
        if mode == DiagnosticsMode::Off {
            return Ok(());
        }
        for state in store.documents_mut() {
            Self::publish_document(connection, state, mode)?;
        }
        Ok(())
    }
}

pub(crate) fn document_diagnostics(state: &mut DocumentState) -> LspResult<Vec<Value>> {
    if !state.load_diagnostics().is_empty() {
        return Ok(state
            .load_diagnostics()
            .iter()
            .cloned()
            .map(conversion::diagnostic)
            .collect());
    }
    state.with_host(|snapshot, host, file, _source| {
        let diagnostics = snapshot
            .diagnostics(host, file)
            .map_err(|error| crate::errors::LspError::internal(error.message))?
            .into_value();
        Ok(diagnostics
            .into_iter()
            .map(conversion::diagnostic)
            .collect())
    })
}
