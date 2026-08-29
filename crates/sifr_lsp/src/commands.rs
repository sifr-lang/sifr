use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::session::Session;
use serde_json::{Value, json};
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
            .generated_rust_preview(host, file)
            .map_err(|error| LspError::internal(error.message))
            .map(|result| conversion::generated_rust_preview(result.into_value()))
    })
}

#[cfg(test)]
mod tests {
    use super::{CommandRegistry, SHOW_GENERATED_RUST_COMMAND};
    use crate::request_queue::CancellationTarget;
    use crate::scheduler::Scheduler;
    use crate::session::Session;
    use lsp_server::{ErrorCode, RequestId};
    use serde_json::json;

    #[test]
    fn generated_rust_preview_honors_cancellation_before_compiler_work() {
        let mut session = Session::new();
        let id = RequestId::from(17);
        let method = "workspace/executeCommand";
        session
            .enqueue_request(&id, method, Scheduler::lane_for_method(method))
            .expect("generated Rust request should enqueue");
        let scheduled = session.start_next_request().expect("request should start");
        session
            .begin_request_execution(scheduled.id())
            .expect("request should begin before cancellation");
        assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);

        let error = CommandRegistry::execute(
            &mut session,
            SHOW_GENERATED_RUST_COMMAND,
            &[json!("file:///cancelled.sifr")],
        )
        .expect_err("cancelled preview must stop before document analysis");

        assert_eq!(error.code(), ErrorCode::RequestCanceled as i32);
        session.finish_request(&id);
    }
}
