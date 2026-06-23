mod code_action;
mod completion;
mod diagnostics;
mod folding_range;
mod formatting;
mod hover;
mod inlay_hint;
mod navigation;
mod selection_range;
mod semantic_tokens;
mod symbols;
mod type_hierarchy;

use crate::commands::CommandRegistry;
use crate::errors::{LspError, LspResult};
use crate::session::Session;
use serde_json::{json, Value};

pub(crate) fn handle(session: &mut Session, method: &str, params: Value) -> LspResult<Value> {
    match method {
        "workspace/symbol" => symbols::workspace_symbol(session, params),
        "workspace/executeCommand" => execute_command(session, params),
        "textDocument/diagnostic" => diagnostics::text_document_diagnostic(session, params),
        "workspace/diagnostic" => diagnostics::workspace_diagnostic(session),
        "textDocument/completion" => completion::completion(session, params),
        "completionItem/resolve" => completion::resolve(params),
        "textDocument/hover" => hover::hover(session, params),
        "textDocument/signatureHelp" => hover::signature_help(session, params),
        "textDocument/definition" => navigation::definition(session, params),
        "textDocument/declaration" => navigation::declaration(session, params),
        "textDocument/typeDefinition" => navigation::type_definition(session, params),
        "textDocument/references" => navigation::references(session, params),
        "textDocument/prepareRename" => navigation::prepare_rename(session, params),
        "textDocument/rename" => navigation::rename(session, params),
        "textDocument/documentSymbol" => symbols::document_symbol(session, params),
        "textDocument/semanticTokens/full" => semantic_tokens::full(session, params),
        "textDocument/semanticTokens/range" => semantic_tokens::range(session, params),
        "textDocument/inlayHint" => inlay_hint::inlay_hint(session, params),
        "textDocument/documentHighlight" => navigation::document_highlight(session, params),
        "textDocument/foldingRange" => folding_range::folding_range(session, params),
        "textDocument/selectionRange" => selection_range::selection_range(session, params),
        "textDocument/prepareTypeHierarchy" => type_hierarchy::prepare(session, params),
        "typeHierarchy/supertypes" => type_hierarchy::supertypes(session, params),
        "typeHierarchy/subtypes" => type_hierarchy::subtypes(session, params),
        "textDocument/codeAction" => code_action::code_action(session, params),
        "codeAction/resolve" => code_action::resolve(session, params),
        "textDocument/formatting" => formatting::formatting(session, params),
        "textDocument/rangeFormatting" => formatting::range_formatting(session, params),
        "sifr/sysroot" => sysroot_status(),
        "sifr/debugTrace" => Ok(Value::String(session.trace_snapshot().render_text())),
        _ => Err(LspError::method_not_found(format!(
            "unsupported Sifr LSP request: {method}"
        ))),
    }
}

fn sysroot_status() -> LspResult<Value> {
    match sifr_analysis::tooling_sysroot_status() {
        Ok(status) => Ok(json!({
            "ok": true,
            "root": status.root,
            "toolchainId": status.toolchain_id,
        })),
        Err(diagnostics) => Ok(json!({
            "ok": false,
            "diagnostics": diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.message)
                .collect::<Vec<_>>(),
        })),
    }
}

fn execute_command(session: &mut Session, params: Value) -> LspResult<Value> {
    let command = params
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("workspace/executeCommand requires command"))?;
    let arguments = params
        .get("arguments")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    CommandRegistry::execute(session, command, arguments)
}

fn text_document_uri(params: &Value) -> LspResult<String> {
    crate::errors::required_string(params, "/textDocument/uri")
}

fn document_position(
    session: &Session,
    uri: &str,
    params: &Value,
) -> LspResult<sifr_analysis::TextPosition> {
    let source = session.store().document(uri)?.text();
    params
        .get("position")
        .ok_or_else(|| LspError::invalid_params("request requires position"))
        .and_then(|position| {
            crate::conversion::lsp_position_to_utf8(position, source, session.position_encoding())
        })
}

fn code_action_context_diagnostics(params: &Value) -> Vec<sifr_analysis::DiagnosticId> {
    params
        .pointer("/context/diagnostics")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(crate::conversion::diagnostic_id)
                .collect()
        })
        .unwrap_or_default()
}
