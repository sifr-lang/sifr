use crate::diagnostics::DiagnosticsController;
use crate::document_store::{DiagnosticsMode, TraceMode, WorkspaceSettings};
use crate::errors::{optional_i32, required_string, LspError, LspResult};
use crate::session::Session;
use lsp_server::Connection;
use serde_json::Value;

pub(crate) fn handle(
    session: &mut Session,
    connection: &Connection,
    method: &str,
    params: Value,
) -> LspResult<()> {
    match method {
        "initialized" => initialized(session),
        "$/cancelRequest" => cancel_request(session, &params),
        "workspace/didChangeConfiguration" => workspace_did_change_configuration(session, params),
        "workspace/didChangeWatchedFiles" => {
            workspace_did_change_watched_files(session, connection)
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
            session.trace(format!("ignored unsupported notification {method}"));
            Ok(())
        }
    }
}

fn initialized(session: &mut Session) -> LspResult<()> {
    session.note_initialized();
    Ok(())
}

fn cancel_request(session: &mut Session, params: &Value) -> LspResult<()> {
    if let Some(id) = params.get("id") {
        session.cancel_request(id);
    }
    Ok(())
}

fn workspace_did_change_configuration(session: &mut Session, params: Value) -> LspResult<()> {
    let root = params.get("settings").unwrap_or(&params);
    let diagnostics = root
        .pointer("/sifr/diagnostics/mode")
        .or_else(|| root.pointer("/sifr.diagnostics.mode"))
        .and_then(Value::as_str)
        .map(parse_diagnostics_mode)
        .transpose()?
        .unwrap_or(session.store().settings().diagnostics_mode);
    let trace_server = root
        .pointer("/sifr/lsp/trace/server")
        .or_else(|| root.pointer("/sifr.lsp.trace.server"))
        .and_then(Value::as_str)
        .map(parse_trace_mode)
        .transpose()?
        .unwrap_or(session.store().settings().trace_server);
    let format_enable = root
        .pointer("/sifr/format/enable")
        .or_else(|| root.pointer("/sifr.format.enable"))
        .and_then(Value::as_bool)
        .unwrap_or(session.store().settings().format_enable);
    let lint_enable = root
        .pointer("/sifr/lint/enable")
        .or_else(|| root.pointer("/sifr.lint.enable"))
        .and_then(Value::as_bool)
        .unwrap_or(session.store().settings().lint_enable);
    session.store_mut().apply_settings(WorkspaceSettings {
        diagnostics_mode: diagnostics,
        trace_server,
        format_enable,
        lint_enable,
    });
    Ok(())
}

fn workspace_did_change_watched_files(
    session: &mut Session,
    connection: &Connection,
) -> LspResult<()> {
    DiagnosticsController::publish_all(connection, session.store_mut())
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
    session
        .store_mut()
        .open(uri.clone(), &language_id, version, text)?;
    let mode = session.store().settings().diagnostics_mode;
    let document = session.store_mut().document_mut(&uri)?;
    DiagnosticsController::publish_document(connection, document, mode)
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
    for change in changes {
        if let Some(range) = change.get("range") {
            let text = required_string(change, "/text")?;
            session
                .store_mut()
                .change_incremental(&uri, version, range, &text)?;
        } else {
            let text = required_string(change, "/text")?;
            session.store_mut().change_full(&uri, version, text)?;
        }
    }
    let mode = session.store().settings().diagnostics_mode;
    let document = session.store_mut().document_mut(&uri)?;
    DiagnosticsController::publish_document(connection, document, mode)
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
    if session.store_mut().save(&uri, text) {
        let mode = session.store().settings().diagnostics_mode;
        let document = session.store_mut().document_mut(&uri)?;
        DiagnosticsController::publish_document(connection, document, mode)?;
    }
    Ok(())
}

fn text_document_did_close(
    session: &mut Session,
    connection: &Connection,
    params: Value,
) -> LspResult<()> {
    let uri = required_string(&params, "/textDocument/uri")?;
    if !session.store_mut().close(&uri) {
        session.trace(format!("ignored close for unopened document {uri}"));
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

fn parse_diagnostics_mode(value: &str) -> LspResult<DiagnosticsMode> {
    match value {
        "off" => Ok(DiagnosticsMode::Off),
        "open-files" => Ok(DiagnosticsMode::OpenFiles),
        "workspace" => Ok(DiagnosticsMode::Workspace),
        _ => Err(LspError::invalid_params(format!(
            "unknown sifr.diagnostics.mode value {value:?}"
        ))),
    }
}

fn parse_trace_mode(value: &str) -> LspResult<TraceMode> {
    match value {
        "off" => Ok(TraceMode::Off),
        "messages" => Ok(TraceMode::Messages),
        "verbose" => Ok(TraceMode::Verbose),
        _ => Err(LspError::invalid_params(format!(
            "unknown sifr.lsp.trace.server value {value:?}"
        ))),
    }
}
