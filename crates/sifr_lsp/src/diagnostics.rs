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
            session.clear_diagnostic_jobs();
            return Ok(());
        }
        session.schedule_document_diagnostics(uri)?;
        Self::flush_ready(connection, session, mode)
    }

    pub(crate) fn publish_all(connection: &Connection, session: &mut Session) -> LspResult<()> {
        let mode = session.store().settings().diagnostics_mode;
        if mode == DiagnosticsMode::Off {
            session.clear_diagnostic_jobs();
            return Ok(());
        }
        for uri in session.document_uris() {
            session.schedule_document_diagnostics(&uri)?;
        }
        Self::flush_ready(connection, session, mode)?;
        Ok(())
    }

    fn flush_ready(
        connection: &Connection,
        session: &mut Session,
        mode: DiagnosticsMode,
    ) -> LspResult<()> {
        if mode == DiagnosticsMode::Off {
            session.clear_diagnostic_jobs();
            return Ok(());
        }
        while let Some(job) = session.take_next_diagnostic_job() {
            if !session.document_version_matches(&job.uri, job.version)? {
                session.trace(format!(
                    "skipped stale diagnostics for {} captured at version {:?}",
                    job.uri, job.version
                ));
                continue;
            }
            let diagnostics = document_diagnostics(session, &job.uri)?;
            if !session.document_version_matches(&job.uri, job.version)? {
                session.trace(format!(
                    "skipped stale diagnostics publish for {} captured at version {:?}",
                    job.uri, job.version
                ));
                continue;
            }
            let params = json!({
                "uri": job.uri,
                "version": job.version,
                "diagnostics": diagnostics
            });
            connection
                .sender
                .send(Message::Notification(Notification {
                    method: "textDocument/publishDiagnostics".to_string(),
                    params,
                }))
                .map_err(|error| {
                    crate::errors::LspError::internal(format!(
                        "failed to publish diagnostics: {error}"
                    ))
                })?;
        }
        Ok(())
    }
}

pub(crate) fn document_diagnostics(session: &mut Session, uri: &str) -> LspResult<Vec<Value>> {
    // Load-time diagnostics are replaced whenever the document analysis owner is
    // opened or updated. Publication still captures and checks the document
    // version in `DiagnosticsController` before this shortcut can be observed.
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
