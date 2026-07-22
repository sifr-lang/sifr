use crate::conversion;
use crate::document_store::DiagnosticsMode;
use crate::errors::LspResult;
use crate::progress::{begin_notification, end_notification, ProgressKind};
use crate::session::Session;
use lsp_server::{Connection, Message, Notification};
use serde_json::{json, Value};
use sifr_analysis::WorkspaceTracePhase;

#[derive(Default)]
pub(crate) struct DiagnosticsController;

impl DiagnosticsController {
    pub(crate) fn publish_all(connection: &Connection, session: &mut Session) -> LspResult<()> {
        let mode = session.store().settings().diagnostics_mode;
        if mode == DiagnosticsMode::Off {
            session.clear_diagnostic_jobs();
            return Ok(());
        }
        let uris = session.document_uris();
        let progress = session.begin_progress(ProgressKind::FullDiagnostics, uris.len());
        if let Some(handle) = &progress {
            publish_progress(
                connection,
                begin_notification(handle, "Checking Sifr workspace"),
            )?;
        }
        for uri in &uris {
            session.schedule_document_diagnostics(uri)?;
        }
        if let Some(handle) = progress {
            let flush_result = Self::flush_ready(connection, session, mode);
            let message = format!("checked {} document(s)", uris.len());
            let end_result = publish_progress(connection, end_notification(&handle, &message));
            session.end_progress(handle, &message);
            flush_result?;
            end_result?;
        } else {
            Self::flush_ready(connection, session, mode)?;
        }
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
                session.trace(
                    WorkspaceTracePhase::StaleRejection,
                    format!(
                        "skipped_diagnostics uri={} captured={:?}",
                        job.uri, job.version
                    ),
                );
                continue;
            }
            let diagnostics = document_diagnostics(session, &job.uri)?;
            if !session.document_version_matches(&job.uri, job.version)? {
                session.trace(
                    WorkspaceTracePhase::StaleRejection,
                    format!(
                        "skipped_diagnostics_publish uri={} captured={:?}",
                        job.uri, job.version
                    ),
                );
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

fn publish_progress(connection: &Connection, params: Value) -> LspResult<()> {
    connection
        .sender
        .send(Message::Notification(Notification {
            method: "$/progress".to_string(),
            params,
        }))
        .map_err(|error| {
            crate::errors::LspError::internal(format!("failed to publish progress: {error}"))
        })
}

pub(crate) fn document_diagnostics(session: &mut Session, uri: &str) -> LspResult<Vec<Value>> {
    let position_encoding = session.position_encoding();
    let source = session.store().document(uri)?.text().to_string();
    // Load-time diagnostics are replaced whenever the document analysis owner is
    // opened or updated. Publication still captures and checks the document
    // version in `DiagnosticsController` before this shortcut can be observed.
    if !session.load_diagnostics(uri).is_empty() {
        return session
            .load_diagnostics(uri)
            .iter()
            .cloned()
            .map(|diagnostic| conversion::diagnostic(diagnostic, &source, position_encoding))
            .collect::<LspResult<Vec<_>>>();
    }
    let mut diagnostics = session.with_document_analysis(uri, |snapshot, host, file, source| {
        let diagnostics = snapshot
            .diagnostics(host, file)
            .map_err(|error| crate::errors::LspError::internal(error.message))?
            .into_value();
        diagnostics
            .into_iter()
            .map(|diagnostic| conversion::diagnostic(diagnostic, source, position_encoding))
            .collect::<LspResult<Vec<_>>>()
    })?;
    match session.python_declaration_snapshot(uri) {
        Ok(python) => diagnostics.extend(
            python
                .diagnostics
                .into_iter()
                .map(|diagnostic| conversion::diagnostic(diagnostic, &source, position_encoding))
                .collect::<LspResult<Vec<_>>>()?,
        ),
        Err(error) => session.trace(
            WorkspaceTracePhase::LspTiming,
            format!(
                "python_declaration_diagnostics_failed uri={uri} error={}",
                error.message()
            ),
        ),
    }
    Ok(diagnostics)
}
