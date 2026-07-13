use super::async_value::{PythonAsyncRequest, PythonAsyncType, PythonAsyncValue};
use super::context_ops::{PythonExitDecision, SifrExitCause};
use super::{ObjectHandle, PythonError};
use crate::cancellation::CancellationCarrier;
use pyo3::prelude::*;

/// Closed, compiler-owned cause input for an async Python context exit.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum PythonAsyncExitCause {
    Normal,
    Python(PythonError),
    Sifr(SifrExitCause),
}

/// Borrow the manager for `__aenter__`; generated code retains its sole owner
/// until the mandatory exit request reaches terminal state.
#[doc(hidden)]
pub async fn submit_async_context_enter(
    manager: &ObjectHandle,
    output: PythonAsyncType,
    carrier: Option<&CancellationCarrier>,
) -> Result<PythonAsyncValue, PythonError> {
    let request = PythonAsyncRequest::borrowed_method(
        manager,
        "__aenter__".to_string(),
        Vec::new(),
        Vec::new(),
        output,
    )?;
    super::async_declaration::submit_async_declaration(request, carrier).await
}

/// Consume the manager through the semantic-close channel and return the
/// truthiness-normalized decision from `__aexit__`.
#[doc(hidden)]
pub async fn submit_async_context_exit(
    manager: ObjectHandle,
    cause: PythonAsyncExitCause,
    carrier: Option<&CancellationCarrier>,
) -> Result<PythonExitDecision, PythonError> {
    let request = PythonAsyncRequest::semantic_context_exit_method(manager, cause)?;
    super::async_declaration::submit_async_context_request(request, carrier).await
}

pub(super) fn materialize_exit_cause(
    py: Python<'_>,
    cause: &PythonAsyncExitCause,
    context: &str,
) -> Result<Vec<Py<PyAny>>, PythonError> {
    match cause {
        PythonAsyncExitCause::Normal => Ok(vec![py.None(), py.None(), py.None()]),
        PythonAsyncExitCause::Python(error) => {
            let (error_type, error_value, traceback) = error.replay(py)?;
            Ok(vec![error_type, error_value, traceback])
        }
        PythonAsyncExitCause::Sifr(cause) => {
            let error_type = super::context_ops::boundary_error_type(py)?;
            let error_value = error_type
                .call1((cause.kind.label(), &cause.sifr_type, &cause.message))
                .map_err(|error| {
                    PythonError::from_pyerr(
                        py,
                        error,
                        "context",
                        format!("{context} SifrBoundaryError construction"),
                    )
                })?;
            Ok(vec![
                error_type.into_any().unbind(),
                error_value.into_any().unbind(),
                py.None(),
            ])
        }
    }
}
