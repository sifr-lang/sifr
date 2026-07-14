use super::super::{object_ops, ObjectHandle, PythonError};
use super::CallbackOwnerState;
use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyTuple, PyType};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct CallbackFailureSlot<E> {
    first: Arc<Mutex<Option<(u64, E)>>>,
}

impl<E> CallbackFailureSlot<E> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            first: Arc::new(Mutex::new(None)),
        }
    }

    pub fn record(&self, entry_sequence: u64, error: E) {
        let mut first = self
            .first
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if first
            .as_ref()
            .is_none_or(|(current, _)| entry_sequence < *current)
        {
            *first = Some((entry_sequence, error));
        }
    }

    pub fn take(&self) -> Option<E> {
        self.first
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .map(|(_, error)| error)
    }

    pub fn take_if_owner_first(&self, owner: &CallbackOwnerState) -> Option<E> {
        let sequence = owner.first_failure()?.entry_sequence;
        let mut first = self
            .first
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if first.as_ref().map(|(current, _)| *current) != Some(sequence) {
            return None;
        }
        let (_, error) = first.take()?;
        owner.observe_failure(sequence);
        Some(error)
    }

    #[must_use]
    pub fn first_sequence(&self) -> Option<u64> {
        self.first
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|(sequence, _)| *sequence)
    }
}

impl<E> Clone for CallbackFailureSlot<E> {
    fn clone(&self) -> Self {
        Self {
            first: Arc::clone(&self.first),
        }
    }
}

impl<E> Default for CallbackFailureSlot<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallbackHandlerFailure {
    pub exception_type: String,
    pub message: String,
}

impl CallbackHandlerFailure {
    #[must_use]
    pub fn new(exception_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            exception_type: exception_type.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CallbackExecutionError {
    Handler(CallbackHandlerFailure),
    Infrastructure(PythonError),
}

impl From<PythonError> for CallbackExecutionError {
    fn from(error: PythonError) -> Self {
        Self::Infrastructure(error)
    }
}

pub(super) fn validate_call_shape(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
    expected_arity: usize,
) -> PyResult<()> {
    if kwargs.is_some_and(|values| !values.is_empty()) {
        return Err(PyTypeError::new_err(
            "Sifr callback parameters are positional-only",
        ));
    }
    if args.len() != expected_arity {
        return Err(PyTypeError::new_err(format!(
            "Sifr callback expected {expected_arity} positional arguments, received {}",
            args.len()
        )));
    }
    Ok(())
}

pub(super) fn collect_args(args: &Bound<'_, PyTuple>) -> Result<Vec<ObjectHandle>, PythonError> {
    args.iter()
        .map(|value| object_ops::store_object(value.unbind()))
        .collect()
}

pub(super) fn result_object(
    py: Python<'_>,
    result: ObjectHandle,
) -> Result<Py<PyAny>, PythonError> {
    let cloned = object_ops::clone_handle(py, &result);
    drop(result);
    cloned
}

pub fn attach_callback_failure_evidence<T>(
    mut outcome: Result<T, PythonError>,
    owners: &[&CallbackOwnerState],
) -> Result<T, PythonError> {
    let Err(primary) = &mut outcome else {
        return outcome;
    };
    let mut seen_owners = Vec::new();
    for owner in owners {
        if seen_owners.contains(&owner.owner_id()) {
            continue;
        }
        seen_owners.push(owner.owner_id());
        let Some(evidence) = owner.first_failure() else {
            continue;
        };
        let secondary = format!(
            "secondary Sifr callback failure {} at entry {}: {}",
            evidence.exception_type, evidence.entry_sequence, evidence.message
        );
        if primary.context.is_empty() {
            primary.context = secondary;
        } else {
            primary.context.push_str("; ");
            primary.context.push_str(&secondary);
        }
    }
    outcome
}

pub(super) fn execution_error(
    py: Python<'_>,
    owner: &super::CallbackOwnerState,
    entry_sequence: u64,
    error: CallbackExecutionError,
) -> PyErr {
    match error {
        CallbackExecutionError::Handler(failure) => {
            owner.record_failure(
                entry_sequence,
                failure.exception_type.clone(),
                failure.message.clone(),
            );
            handler_error(py, &failure)
        }
        CallbackExecutionError::Infrastructure(error) => python_error(error),
    }
}

pub(super) fn python_error(error: PythonError) -> PyErr {
    let message = error.to_string();
    let exception_type = error.exception_type;
    let kind = error.kind;
    if exception_type == "TypeError" || kind == "conversion" {
        PyTypeError::new_err(message)
    } else if exception_type.starts_with("SifrCallback") {
        Python::try_attach(|py| named_callback_error(py, &exception_type, message.clone()))
            .unwrap_or_else(|| PyRuntimeError::new_err(message))
    } else {
        PyRuntimeError::new_err(message)
    }
}

fn handler_error(py: Python<'_>, failure: &CallbackHandlerFailure) -> PyErr {
    let error_type = py
        .import("__sifr_callbacks__")
        .and_then(|module| module.getattr("SifrCallbackError"))
        .and_then(|value| value.cast_into::<PyType>().map_err(Into::into));
    error_type.map_or_else(
        |error| PyRuntimeError::new_err(format!("failed to construct SifrCallbackError: {error}")),
        |error_type| {
            PyErr::from_type(
                error_type,
                (failure.exception_type.clone(), failure.message.clone()),
            )
        },
    )
}

fn named_callback_error(py: Python<'_>, name: &str, message: String) -> PyErr {
    let error_type = py
        .import("__sifr_callbacks__")
        .and_then(|module| module.getattr(name))
        .and_then(|value| value.cast_into::<PyType>().map_err(Into::into));
    match error_type {
        Ok(error_type) => PyErr::from_type(error_type, (message,)),
        Err(_) => PyRuntimeError::new_err(message),
    }
}
