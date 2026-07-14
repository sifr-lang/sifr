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
        c"import contextvars\n\n_callback_origin = contextvars.ContextVar('sifr_callback_origin', default=None)\n\ndef callback_origin():\n    return _callback_origin.get()\n\ndef set_callback_origin(owner_id, callback_id):\n    return _callback_origin.set((owner_id, callback_id))\n\ndef reset_callback_origin(token):\n    _callback_origin.reset(token)\n\nclass SifrCallbackError(RuntimeError):\n    def __init__(self, handler_type, message):\n        super().__init__(message)\n        self.handler_type = handler_type\n        self.message = message\n\nclass SifrCallbackClosedError(RuntimeError):\n    pass\n\nclass SifrCallbackReentrancyError(RuntimeError):\n    pass\n\nclass SifrCallbackCloseReentrancyError(RuntimeError):\n    pass\n\nclass SifrCallbackThreadError(RuntimeError):\n    pass\n",
        c"__sifr_callbacks__.py",
        c"__sifr_callbacks__",
    )
    .map_err(callback_registration_error)?;
    for name in [
        "SifrCallbackError",
        "SifrCallbackClosedError",
        "SifrCallbackReentrancyError",
        "SifrCallbackCloseReentrancyError",
        "SifrCallbackThreadError",
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

pub(super) fn python_callback_origin(py: Python<'_>) -> Result<Option<(u64, u64)>, PythonError> {
    let value = py
        .import("__sifr_callbacks__")
        .and_then(|module| module.call_method0("callback_origin"))
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "callback origin"))?;
    if value.is_none() {
        Ok(None)
    } else {
        value
            .extract::<(u64, u64)>()
            .map(Some)
            .map_err(|error| PythonError::from_pyerr(py, error, "callback", "callback origin"))
    }
}

pub(crate) struct PythonCallbackOriginGuard<'py> {
    py: Python<'py>,
    token: Py<PyAny>,
}

pub(crate) fn install_python_callback_origin(
    py: Python<'_>,
    origin: Option<(u64, u64)>,
) -> PyResult<Option<PythonCallbackOriginGuard<'_>>> {
    let Some(origin) = origin else {
        return Ok(None);
    };
    let token = py
        .import("__sifr_callbacks__")?
        .call_method1("set_callback_origin", origin)
        .map(Bound::unbind)?;
    Ok(Some(PythonCallbackOriginGuard { py, token }))
}

impl Drop for PythonCallbackOriginGuard<'_> {
    fn drop(&mut self) {
        let _ignored = self.py.import("__sifr_callbacks__").and_then(|module| {
            module.call_method1("reset_callback_origin", (self.token.bind(self.py),))
        });
    }
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
    let message = format!("failed to register callback exceptions: {error}");
    drop(error);
    PythonRuntimeError::PythonOperationFailed(message)
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
        "SifrCallbackThreadError",
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

pub(super) fn wrong_thread(owner_id: u64, callback_id: u64) -> PythonError {
    callback_error(
        "SifrCallbackThreadError",
        format!(
            "current callback {callback_id} for owner {owner_id} was invoked on a foreign thread"
        ),
        "callback admission",
    )
}

pub(super) fn wrong_asyncio_loop(owner_id: u64, callback_id: u64) -> PythonError {
    callback_error(
        "SifrCallbackThreadError",
        format!(
            "asyncio callback {callback_id} for owner {owner_id} was invoked outside the application-owned loop"
        ),
        "callback admission",
    )
}

pub(super) fn recorded_handler_failure(
    owner_id: u64,
    evidence: &super::CallbackFailureEvidence,
) -> PythonError {
    callback_error(
        "SifrCallbackError",
        format!(
            "retained callback owner {owner_id} recorded {} at entry {}: {}",
            evidence.exception_type, evidence.entry_sequence, evidence.message
        ),
        "retained callback failure",
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
