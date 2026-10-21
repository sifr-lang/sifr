use sifr_diagnostics::DiagnosticCode;
pub(crate) use sifr_diagnostics::RenderedDiagnostic;
pub(crate) use sifr_semantic_services::diagnostics::diagnostic_with_code;
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return (*msg).to_string();
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    "non-string panic payload".to_string()
}
pub(crate) fn run_codegen_with_boundary<T>(
    context: impl Into<String>,
    f: impl FnOnce() -> T,
) -> Result<T, Box<RenderedDiagnostic>> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(Box::new(diagnostic_with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ))),
    }
}
