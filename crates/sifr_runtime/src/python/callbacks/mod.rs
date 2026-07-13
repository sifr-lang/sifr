mod errors;
mod registry;
mod state;

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
