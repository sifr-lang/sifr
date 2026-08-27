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
use serde_json::{Value, json};
use std::path::Path;

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
        "sifr/sysroot" => sysroot_status(&params),
        "sifr/debugCacheStats" => {
            let stats = session.python_declaration_cache_stats();
            Ok(json!({
                "pythonDeclarations": {
                    "hits": stats.hits,
                    "misses": stats.misses,
                    "externalFingerprintRuns": stats.external_fingerprint_runs,
                    "snapshotBuilds": stats.snapshot_builds,
                }
            }))
        }
        "sifr/debugTrace" => Ok(Value::String(session.trace_snapshot().render_text())),
        _ => Err(LspError::method_not_found(format!(
            "unsupported Sifr LSP request: {method}"
        ))),
    }
}

fn sysroot_status(params: &Value) -> LspResult<Value> {
    let expected_root = params
        .get("expectedRoot")
        .map(|raw| {
            raw.as_str().ok_or_else(|| {
                LspError::invalid_params("sifr/sysroot expectedRoot must be a string")
            })
        })
        .transpose()?;
    let expected_toolchain_id = params
        .get("expectedToolchainId")
        .map(|raw| {
            raw.as_str().ok_or_else(|| {
                LspError::invalid_params("sifr/sysroot expectedToolchainId must be a string")
            })
        })
        .transpose()?;
    Ok(sysroot_status_from_probe(
        sifr_analysis::tooling_sysroot_probe(),
        expected_root,
        expected_toolchain_id,
    ))
}

pub(crate) fn sysroot_status_from_probe(
    probe: sifr_analysis::ToolingSysrootProbe,
    expected_root: Option<&str>,
    expected_toolchain_id: Option<&str>,
) -> Value {
    if let Some(status) = probe.status {
        return sysroot_status_success(status, expected_root, expected_toolchain_id);
    }
    if let Some(diagnostic) = probe.diagnostic {
        return sysroot_status_failure(diagnostic);
    }
    json!({
        "ok": false,
        "kind": "broken",
        "diagnostics": [{
            "kind": "internal",
            "message": "Sifr sysroot resolver returned no status or diagnostic",
        }],
        "observedPaths": {},
    })
}

fn sysroot_status_success(
    status: sifr_analysis::ToolingSysrootStatus,
    expected_root: Option<&str>,
    expected_toolchain_id: Option<&str>,
) -> Value {
    let observed_root = status.root.to_string_lossy().to_string();
    let observed_toolchain_id = status.toolchain_id.clone();
    let response = json!({
        "ok": true,
        "kind": "resolved",
        "root": observed_root,
        "toolchainId": observed_toolchain_id,
        "diagnostics": [],
        "observedPaths": {
            "sysroot": observed_root,
        },
    });
    let root_matches = expected_root.is_none_or(|root| Path::new(root) == status.root);
    let toolchain_matches =
        expected_toolchain_id.is_none_or(|toolchain_id| toolchain_id == observed_toolchain_id);
    if root_matches && toolchain_matches {
        return response;
    }
    let expected_root = expected_root.unwrap_or("<not supplied>");
    let expected_toolchain_id = expected_toolchain_id.unwrap_or("<not supplied>");
    let diagnostic = format!(
        "Sifr sysroot mismatch: CLI root {expected_root}; LSP root {observed_root}; CLI toolchain {expected_toolchain_id}; LSP toolchain {observed_toolchain_id}"
    );
    json!({
        "ok": false,
        "kind": "mismatch",
        "root": observed_root,
        "toolchainId": observed_toolchain_id,
        "expectedRoot": expected_root,
        "expectedToolchainId": expected_toolchain_id,
        "diagnostics": [{
            "kind": "mismatch",
            "message": diagnostic,
            "expectedRoot": expected_root,
            "observedRoot": observed_root,
            "expectedToolchainId": expected_toolchain_id,
            "observedToolchainId": observed_toolchain_id,
        }],
        "observedPaths": {
            "sysroot": observed_root,
        },
    })
}

fn sysroot_status_failure(diagnostic: sifr_analysis::ToolingSysrootDiagnostic) -> Value {
    json!({
        "ok": false,
        "kind": "broken",
        "diagnostics": [{
            "kind": "resolution",
            "message": diagnostic.message,
            "binaryPath": diagnostic.binary_path,
            "attemptedSysroot": diagnostic.attempted_sysroot,
            "assetPath": diagnostic.asset_path,
        }],
        "observedPaths": {
            "binary": diagnostic.binary_path,
            "attemptedSysroot": diagnostic.attempted_sysroot,
            "asset": diagnostic.asset_path,
        },
    })
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
