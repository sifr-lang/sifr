use crate::conversion;
use crate::document_store::DiagnosticsMode;
use crate::errors::LspResult;
use crate::session::Session;
use lsp_server::{Connection, Message, Notification};
use serde_json::{json, Value};

#[derive(Default)]
pub(crate) struct DiagnosticsController;

impl DiagnosticsController {
    pub(crate) fn publish_document(
        connection: &Connection,
        session: &mut Session,
        uri: &str,
        mode: DiagnosticsMode,
    ) -> LspResult<()> {
        if mode == DiagnosticsMode::Off {
            return Ok(());
        }
        let diagnostics = document_diagnostics(session, uri)?;
        let document = session.store().document(uri)?;
        let params = json!({
            "uri": document.uri(),
            "version": document.version(),
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

    pub(crate) fn publish_all(connection: &Connection, session: &mut Session) -> LspResult<()> {
        let mode = session.store().settings().diagnostics_mode;
        if mode == DiagnosticsMode::Off {
            return Ok(());
        }
        for uri in session.document_uris() {
            Self::publish_document(connection, session, &uri, mode)?;
        }
        Ok(())
    }
}

pub(crate) fn document_diagnostics(session: &mut Session, uri: &str) -> LspResult<Vec<Value>> {
    if !session.load_diagnostics(uri).is_empty() {
        return Ok(session
            .load_diagnostics(uri)
            .iter()
            .cloned()
            .map(conversion::diagnostic)
            .collect());
    }
    session.with_document_analysis(uri, |snapshot, host, file, _source| {
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
