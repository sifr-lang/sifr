use crate::document_store::{DiagnosticsMode, TraceMode, WorkspaceSettings};
use crate::errors::{LspError, LspResult};
use serde_json::Value;

pub(crate) fn settings_from_initialize_params(
    params: &Value,
    current: &WorkspaceSettings,
) -> LspResult<WorkspaceSettings> {
    let root = params.get("initializationOptions").unwrap_or(&Value::Null);
    parse_workspace_settings(root, current)
}

pub(crate) fn parse_workspace_settings(
    root: &Value,
    current: &WorkspaceSettings,
) -> LspResult<WorkspaceSettings> {
    let diagnostics_mode = root
        .pointer("/sifr/diagnostics/mode")
        .or_else(|| root.pointer("/sifr.diagnostics.mode"))
        .or_else(|| root.get("diagnosticsMode"))
        .and_then(Value::as_str)
        .map(parse_diagnostics_mode)
        .transpose()?
        .unwrap_or(current.diagnostics_mode);
    let trace_server = root
        .pointer("/sifr/lsp/trace/server")
        .or_else(|| root.pointer("/sifr.lsp.trace.server"))
        .or_else(|| root.get("traceServer"))
        .and_then(Value::as_str)
        .map(parse_trace_mode)
        .transpose()?
        .unwrap_or(current.trace_server);
    let format_enable = root
        .pointer("/sifr/format/enable")
        .or_else(|| root.pointer("/sifr.format.enable"))
        .or_else(|| root.get("formatEnable"))
        .and_then(Value::as_bool)
        .unwrap_or(current.format_enable);
    let lint_enable = root
        .pointer("/sifr/lint/enable")
        .or_else(|| root.pointer("/sifr.lint.enable"))
        .or_else(|| root.get("lintEnable"))
        .and_then(Value::as_bool)
        .unwrap_or(current.lint_enable);
    Ok(WorkspaceSettings {
        diagnostics_mode,
        trace_server,
        format_enable,
        lint_enable,
    })
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
