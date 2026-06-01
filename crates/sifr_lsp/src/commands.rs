use crate::conversion;
use crate::document_store::DocumentStore;
use crate::errors::{LspError, LspResult};
use serde_json::{json, Value};
use sifr_analysis::AnalysisQueryResult;
use sifr_analysis::{DiagnosticId, TestItemId};

pub(crate) struct CommandRegistry;

impl CommandRegistry {
    pub(crate) fn execute(
        store: &mut DocumentStore,
        command: &str,
        arguments: &[Value],
    ) -> LspResult<Value> {
        match command {
            "sifr.restartServer" => Ok(json!({"status": "restart-requested"})),
            "sifr.showServerLogs" => Ok(json!({"status": "stdio-server", "logs": []})),
            "sifr.explainDiagnostic" => explain_diagnostic(store, arguments),
            "sifr.showGeneratedRust" => generated_rust_preview(store, arguments),
            "sifr.checkWorkspace" => {
                Ok(json!({"command": "sifr", "args": ["check"], "status": "metadata-only"}))
            }
            "sifr.runTests" => run_tests(store, arguments),
            _ => Err(LspError::method_not_found(format!(
                "unknown Sifr command: {command}"
            ))),
        }
    }
}

fn explain_diagnostic(store: &mut DocumentStore, arguments: &[Value]) -> LspResult<Value> {
    let id = arguments
        .first()
        .and_then(Value::as_str)
        .map(DiagnosticId::hard)
        .ok_or_else(|| {
            LspError::invalid_params("sifr.explainDiagnostic requires a diagnostic code argument")
        })?;
    for document in store.documents_mut() {
        let explanation = document.with_host(|snapshot, host, _file, _source| {
            snapshot
                .explain_diagnostic(host, &id)
                .map_err(|error| LspError::internal(error.message))
                .map(AnalysisQueryResult::into_value)
        })?;
        if explanation.diagnostic.is_some() {
            return Ok(json!({
                "diagnostic": explanation.diagnostic,
                "unavailableReason": explanation.unavailable_reason
            }));
        }
    }
    Err(LspError::invalid_params(format!(
        "diagnostic code {} is not present in the current workspace snapshot",
        id.code
    )))
}

fn generated_rust_preview(store: &mut DocumentStore, arguments: &[Value]) -> LspResult<Value> {
    let uri = arguments.first().and_then(Value::as_str).ok_or_else(|| {
        LspError::invalid_params("sifr.showGeneratedRust requires a document URI argument")
    })?;
    let document = store.document_mut(uri)?;
    document.with_host(|snapshot, host, file, _source| {
        snapshot
            .generated_rust_preview(host, file, None)
            .map_err(|error| LspError::internal(error.message))
            .map(|result| conversion::generated_rust_preview(result.into_value()))
    })
}

fn run_tests(store: &mut DocumentStore, arguments: &[Value]) -> LspResult<Value> {
    if let Some(test_id) = arguments.first().and_then(Value::as_str) {
        if let Some(document) = store.documents_mut().next() {
            let command = document.with_host(|snapshot, host, _file, _source| {
                snapshot
                    .test_command(host, TestItemId(test_id.to_string()))
                    .map_err(|error| LspError::internal(error.message))
                    .map(AnalysisQueryResult::into_value)
            })?;
            return Ok(conversion::test_command(command));
        }
    }
    let mut tests = Vec::new();
    for document in store.documents_mut() {
        let mut document_tests = document.with_host(|snapshot, host, _file, _source| {
            snapshot
                .discover_tests(host)
                .map_err(|error| LspError::internal(error.message))
                .map(AnalysisQueryResult::into_value)
        })?;
        tests.append(&mut document_tests);
    }
    Ok(Value::Array(
        tests.into_iter().map(conversion::test_item).collect(),
    ))
}
