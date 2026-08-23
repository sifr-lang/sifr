mod asyncio;
mod asyncio_entry;
#[cfg(test)]
mod asyncio_tests;
mod current;
#[cfg(test)]
mod current_tests;
mod errors;
pub(crate) mod execution;
mod foreign;
mod ownership;
#[cfg(test)]
mod ownership_tests;
mod registry;
mod state;

pub use current::{CurrentCallback, current_callback, current_callback_with_owner};
pub(super) use errors::install_python_callback_origin;
pub use execution::{
    CallbackExecutionError, CallbackFailureSlot, CallbackHandlerFailure,
    attach_callback_failure_evidence, reconcile_callback_outcome,
};
pub use foreign::{
    ForeignCallback, ForeignCallbackConcurrency, foreign_callback,
    foreign_callback_scoped_with_owner, foreign_callback_with_owner,
};
pub use ownership::{
    CallbackOwnerSlot, RetainedCallbackCleanup, RetainedCallbackGroup,
    abandon_callback_owner_after_error, abandon_callback_owner_after_error_async,
    context_enter_with_callbacks, context_exit_normal_with_callbacks,
    context_exit_python_error_with_callbacks, context_exit_sifr_cause_with_callbacks,
    finalize_retained_callbacks, finish_retained_callback_finalization,
    retained_callback_finalization_scope, rollback_retained_callbacks_on_error,
    semantic_close_with_callbacks,
};
pub use state::{
    CallbackFailureEvidence, CallbackInvocationGuard, CallbackInvocationLease,
    CallbackInvocationPollGuard, CallbackOwnerState, CallbackOwnerStatus,
    CallbackOwnerUnregisterGuard, current_callback_origin,
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
pub use asyncio::{
    AsyncioCallback, AsyncioCallbackConcurrency, asyncio_callback_scoped_with_owner,
    asyncio_callback_with_owner,
};
