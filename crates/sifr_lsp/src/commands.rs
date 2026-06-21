use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::session::Session;
use serde_json::{json, Value};
use sifr_analysis::AnalysisQueryResult;
use sifr_analysis::DiagnosticId;

const EXPLAIN_DIAGNOSTIC_COMMAND: &str = "sifr.server.explainDiagnostic";
const SHOW_GENERATED_RUST_COMMAND: &str = "sifr.server.showGeneratedRust";

pub(crate) struct CommandRegistry;

impl CommandRegistry {
    pub(crate) fn execute(
        session: &mut Session,
        command: &str,
        arguments: &[Value],
    ) -> LspResult<Value> {
        match command {
            EXPLAIN_DIAGNOSTIC_COMMAND => explain_diagnostic(session, arguments),
            SHOW_GENERATED_RUST_COMMAND => generated_rust_preview(session, arguments),
            _ => Err(LspError::method_not_found(format!(
                "unknown Sifr command: {command}"
            ))),
        }
    }
}

fn explain_diagnostic(session: &mut Session, arguments: &[Value]) -> LspResult<Value> {
    let id = arguments
        .first()
        .and_then(Value::as_str)
        .map(DiagnosticId::hard)
        .ok_or_else(|| {
            LspError::invalid_params(
                "sifr.server.explainDiagnostic requires a diagnostic code argument",
            )
        })?;
    for uri in session.document_uris() {
        let explanation =
            session.with_document_analysis(&uri, |snapshot, host, _file, _source| {
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

fn generated_rust_preview(session: &mut Session, arguments: &[Value]) -> LspResult<Value> {
    let uri = arguments.first().and_then(Value::as_str).ok_or_else(|| {
        LspError::invalid_params("sifr.server.showGeneratedRust requires a document URI argument")
    })?;
    session.with_document_analysis(uri, |snapshot, host, file, _source| {
        snapshot
            .generated_rust_preview(host, file, None)
            .map_err(|error| LspError::internal(error.message))
            .map(|result| conversion::generated_rust_preview(result.into_value()))
    })
}
