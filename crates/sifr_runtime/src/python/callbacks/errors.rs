use super::super::{PythonError, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyModule, PyType};
use std::sync::OnceLock;

static CALLBACK_ERRORS_REGISTERED: OnceLock<()> = OnceLock::new();

pub(super) fn register_callback_errors(py: Python<'_>) -> Result<(), PythonRuntimeError> {
    if CALLBACK_ERRORS_REGISTERED.get().is_some() {
        return Ok(());
    }
    let module = PyModule::from_code(
        py,
        c"class SifrCallbackError(RuntimeError):\n    def __init__(self, handler_type, message):\n        super().__init__(message)\n        self.handler_type = handler_type\n        self.message = message\n\nclass SifrCallbackClosedError(RuntimeError):\n    pass\n\nclass SifrCallbackReentrancyError(RuntimeError):\n    pass\n\nclass SifrCallbackCloseReentrancyError(RuntimeError):\n    pass\n",
        c"__sifr_callbacks__.py",
        c"__sifr_callbacks__",
    )
    .map_err(callback_registration_error)?;
    for name in [
        "SifrCallbackError",
        "SifrCallbackClosedError",
        "SifrCallbackReentrancyError",
        "SifrCallbackCloseReentrancyError",
    ] {
        let _validated_type = callback_type(&module, name)?;
    }
    let modules = py
        .import("sys")
        .and_then(|sys| sys.getattr("modules"))
        .map_err(callback_registration_error)?
        .cast_into::<PyDict>()
        .map_err(|error| callback_registration_error(error.into()))?;
    modules
        .set_item("__sifr_callbacks__", &module)
        .map_err(callback_registration_error)?;
    let _already_registered = CALLBACK_ERRORS_REGISTERED.set(()).is_err();
    Ok(())
}

fn callback_type(
    module: &Bound<'_, PyModule>,
    name: &str,
) -> Result<Py<PyType>, PythonRuntimeError> {
    module
        .getattr(name)
        .map_err(callback_registration_error)?
        .cast_into::<PyType>()
        .map(Bound::unbind)
        .map_err(|error| callback_registration_error(error.into()))
}

fn callback_registration_error(error: PyErr) -> PythonRuntimeError {
    PythonRuntimeError::PythonOperationFailed(format!(
        "failed to register callback exceptions: {error}"
    ))
}

#[cfg(test)]
pub(super) fn registered_exception_names(py: Python<'_>) -> Option<Vec<String>> {
    CALLBACK_ERRORS_REGISTERED.get()?;
    let module = py.import("__sifr_callbacks__").ok()?;
    [
        "SifrCallbackError",
        "SifrCallbackClosedError",
        "SifrCallbackReentrancyError",
        "SifrCallbackCloseReentrancyError",
    ]
    .into_iter()
    .map(|name| {
        module
            .getattr(name)
            .ok()?
            .cast_into::<PyType>()
            .ok()?
            .name()
            .ok()
            .map(|name| name.to_string())
    })
    .collect()
}

pub(super) fn closed(owner_id: u64) -> PythonError {
    callback_error(
        "SifrCallbackClosedError",
        format!("Python callback owner {owner_id} is closing or closed"),
        "callback admission",
    )
}

pub(super) fn reentrant(owner_id: u64, callback_id: u64) -> PythonError {
    callback_error(
        "SifrCallbackReentrancyError",
        format!("serial callback {callback_id} for owner {owner_id} is reentrant"),
        "callback admission",
    )
}

pub(super) fn close_from_invocation(owner_id: u64) -> PythonError {
    callback_error(
        "SifrCallbackCloseReentrancyError",
        format!("callback owner {owner_id} cannot close from an accepted invocation"),
        "callback owner close",
    )
}

pub(super) fn unavailable(context: &str) -> PythonError {
    PythonError::runtime(PythonRuntimeError::PythonOperationFailed(format!(
        "Python callback {context} is unavailable"
    )))
}

fn callback_error(exception_type: &str, message: String, context: &str) -> PythonError {
    PythonError {
        kind: "callback".to_string(),
        exception_type: exception_type.to_string(),
        message,
        traceback: String::new(),
        context: context.to_string(),
        replay: None,
    }
}
