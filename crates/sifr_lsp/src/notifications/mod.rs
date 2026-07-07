use crate::diagnostics::DiagnosticsController;
use crate::errors::{optional_i32, required_string, LspError, LspResult};
use crate::session::Session;
use lsp_server::{Connection, Message, Notification, RequestId};
use serde_json::{json, Value};
use sifr_analysis::WorkspaceTracePhase;
use std::fmt::Write as _;

pub(crate) fn handle(
    session: &mut Session,
    connection: &Connection,
    method: &str,
    params: Value,
) -> LspResult<()> {
    match method {
        "initialized" => initialized(session, connection),
        "workspace/didChangeConfiguration" => workspace_did_change_configuration(session, params),
        "workspace/didChangeWatchedFiles" => {
            workspace_did_change_watched_files(session, params, connection)
        }
        "textDocument/didOpen" => text_document_did_open(session, connection, params),
        "textDocument/didChange" => text_document_did_change(session, connection, params),
        "textDocument/didSave" => text_document_did_save(session, connection, params),
        "textDocument/didClose" => text_document_did_close(session, connection, params),
        "exit" => {
            session.note_exit_notification();
            Ok(())
        }
        _ => {
            session.trace(
                WorkspaceTracePhase::LspTiming,
                format!("ignored unsupported notification {method}"),
            );
            Ok(())
        }
    }
}

fn initialized(session: &mut Session, connection: &Connection) -> LspResult<()> {
    session.note_initialized();
    publish_tooling_sysroot_diagnostic(connection, sifr_analysis::tooling_sysroot_probe())?;
    Ok(())
}

fn publish_tooling_sysroot_diagnostic(
    connection: &Connection,
    probe: sifr_analysis::ToolingSysrootProbe,
) -> LspResult<()> {
    let Some(diagnostic) = probe.diagnostic else {
        return Ok(());
    };
    connection
        .sender
        .send(Message::Notification(tooling_sysroot_notification(
            &diagnostic,
        )))
        .map_err(|error| {
            LspError::internal(format!("failed to publish sysroot diagnostic: {error}"))
        })
}

pub(crate) fn tooling_sysroot_notification(
    diagnostic: &sifr_analysis::ToolingSysrootDiagnostic,
) -> Notification {
    let mut message = format!(
        "{}\nbinary: {}\nattempted sysroot: {}",
        diagnostic.message,
        diagnostic.binary_path.display(),
        diagnostic.attempted_sysroot.display()
    );
    if let Some(asset_path) = &diagnostic.asset_path {
        let _ = write!(message, "\ninvalid asset: {}", asset_path.display());
    }
    Notification {
        method: "window/showMessage".to_string(),
        params: json!({
            "type": 1,
            "message": message,
        }),
    }
}

pub(crate) fn cancel_request_id(params: &Value) -> Option<RequestId> {
    let id = params.get("id")?;
    if let Some(raw) = id.as_i64() {
        let raw = i32::try_from(raw).ok()?;
        Some(RequestId::from(raw))
    } else {
        id.as_str().map(|raw| RequestId::from(raw.to_string()))
    }
}

fn workspace_did_change_configuration(session: &mut Session, params: Value) -> LspResult<()> {
    let root = params.get("settings").unwrap_or(&params);
    let previous_mode = session.store().settings().diagnostics_mode;
    let settings = crate::settings::parse_workspace_settings(root, session.store().settings())?;
    let next_mode = settings.diagnostics_mode;
    session.store_mut().apply_settings(settings);
    if previous_mode != next_mode && next_mode == crate::document_store::DiagnosticsMode::Off {
        session.clear_diagnostic_jobs();
    }
    Ok(())
}

fn workspace_did_change_watched_files(
    session: &mut Session,
    params: Value,
    connection: &Connection,
) -> LspResult<()> {
    let changes = params
        .get("changes")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    session.record_watcher_events(changes);
    DiagnosticsController::publish_all(connection, session)
}

fn text_document_did_open(
    session: &mut Session,
    connection: &Connection,
    params: Value,
) -> LspResult<()> {
    let uri = required_string(&params, "/textDocument/uri")?;
    let language_id = required_string(&params, "/textDocument/languageId")?;
    let version = optional_i32(&params, "/textDocument/version")?;
    let text = required_string(&params, "/textDocument/text")?;
    session.open_document(uri.clone(), &language_id, version, text)?;
    let mode = session.store().settings().diagnostics_mode;
    DiagnosticsController::publish_document(connection, session, &uri, mode)
}

fn text_document_did_change(
    session: &mut Session,
    connection: &Connection,
    params: Value,
) -> LspResult<()> {
    let uri = required_string(&params, "/textDocument/uri")?;
    let version = optional_i32(&params, "/textDocument/version")?;
    let changes = params
        .get("contentChanges")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            LspError::invalid_params("textDocument/didChange requires contentChanges")
        })?;
    let summary = session.change_compacted(&uri, version, changes)?;
    session.trace(
        WorkspaceTracePhase::SourceUpdate,
        format!(
            "compacted_did_change uri={uri} raw={} compacted={} text_changed={}",
            summary.raw_change_count, summary.compacted_change_count, summary.text_changed
        ),
    );
    let mode = session.store().settings().diagnostics_mode;
    DiagnosticsController::publish_document(connection, session, &uri, mode)
}

fn text_document_did_save(
    session: &mut Session,
    connection: &Connection,
    params: Value,
) -> LspResult<()> {
    let uri = required_string(&params, "/textDocument/uri")?;
    let text = params
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if session.save_document(&uri, text)? {
        let mode = session.store().settings().diagnostics_mode;
        DiagnosticsController::publish_document(connection, session, &uri, mode)?;
    }
    Ok(())
}

fn text_document_did_close(
    session: &mut Session,
    connection: &Connection,
    params: Value,
) -> LspResult<()> {
    let uri = required_string(&params, "/textDocument/uri")?;
    if !session.close_document(&uri) {
        session.trace(
            WorkspaceTracePhase::SourceUpdate,
            format!("ignored close for unopened document {uri}"),
        );
        return Ok(());
    }
    connection
        .sender
        .send(lsp_server::Message::Notification(
            lsp_server::Notification {
                method: "textDocument/publishDiagnostics".to_string(),
                params: serde_json::json!({
                    "uri": uri,
                    "diagnostics": []
                }),
            },
        ))
        .map_err(|error| LspError::internal(format!("failed to clear diagnostics: {error}")))?;
    Ok(())
}
