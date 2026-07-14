mod current;
mod errors;
mod execution;
mod foreign;
mod ownership;
mod registry;
mod state;

pub use current::{current_callback, current_callback_with_owner, CurrentCallback};
pub use execution::{
    attach_callback_failure_evidence, CallbackExecutionError, CallbackFailureSlot,
    CallbackHandlerFailure,
};
pub use foreign::{
    foreign_callback, foreign_callback_scoped_with_owner, foreign_callback_with_owner,
    ForeignCallback, ForeignCallbackConcurrency,
};
pub use ownership::{
    context_exit_normal_with_callbacks, context_exit_python_error_with_callbacks,
    context_exit_sifr_cause_with_callbacks, semantic_close_with_callbacks, CallbackOwnerSlot,
    RetainedCallbackCleanup, RetainedCallbackGroup,
};
pub use state::{
    CallbackFailureEvidence, CallbackInvocationGuard, CallbackInvocationLease, CallbackOwnerState,
    CallbackOwnerStatus, CallbackOwnerUnregisterGuard,
};

pub(super) fn shutdown_registered_callback_owners() -> Result<(), super::PythonRuntimeError> {
    registry::shutdown_registered_callback_owners()
}

pub(super) fn register_callback_errors(
    py: pyo3::Python<'_>,
) -> Result<(), super::PythonRuntimeError> {
    errors::register_callback_errors(py)
}

#[cfg(test)]
mod tests;
