use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyModule, PyType};
use std::sync::{LazyLock, Mutex, OnceLock};

static BOUNDARY_ERROR_TYPE: OnceLock<Py<PyType>> = OnceLock::new();
static CLEANUP_EVIDENCE: LazyLock<Mutex<Vec<ContextCleanupEvidence>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonExitDecision {
    Propagate,
    Suppress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SifrExitCauseKind {
    OrdinaryError,
    Timeout,
    Cancellation,
    RuntimeFault,
}

impl SifrExitCauseKind {
    const fn label(self) -> &'static str {
        match self {
            Self::OrdinaryError => "ordinary-error",
            Self::Timeout => "timeout",
            Self::Cancellation => "cancellation",
            Self::RuntimeFault => "runtime-fault",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrExitCause {
    pub kind: SifrExitCauseKind,
    pub sifr_type: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextCleanupEvidence {
    pub primary_cause: String,
    pub exception_type: String,
    pub message: String,
    pub context: String,
}

pub(super) fn register_boundary_error(py: Python<'_>) -> Result<(), PythonRuntimeError> {
    if BOUNDARY_ERROR_TYPE.get().is_some() {
        return Ok(());
    }
    let module = PyModule::from_code(
        py,
        c"class SifrBoundaryError(RuntimeError):\n    def __init__(self, cause_kind, sifr_type, message):\n        super().__init__(message)\n        self.cause_kind = cause_kind\n        self.sifr_type = sifr_type\n        self.message = message\n",
        c"__sifr_context__.py",
        c"__sifr_context__",
    )
    .map_err(boundary_registration_error)?;
    let error_type = module
        .getattr("SifrBoundaryError")
        .map_err(boundary_registration_error)?
        .cast_into::<PyType>()
        .map_err(|error| boundary_registration_error(error.into()))?;
    let modules = py
        .import("sys")
        .and_then(|sys| sys.getattr("modules"))
        .map_err(boundary_registration_error)?
        .cast_into::<PyDict>()
        .map_err(|error| boundary_registration_error(error.into()))?;
    modules
        .set_item("__sifr_context__", &module)
        .map_err(boundary_registration_error)?;
    let _already_registered = BOUNDARY_ERROR_TYPE.set(error_type.unbind()).is_err();
    Ok(())
}

pub fn context_exit_normal(object: ObjectHandle) -> Result<PythonExitDecision, PythonError> {
    finish_context_exit(&object, call_exit_normal(&object))
}

pub fn context_exit_python_error(
    object: ObjectHandle,
    error: &PythonError,
) -> Result<PythonExitDecision, PythonError> {
    finish_context_exit(&object, call_exit_python_error(&object, error))
}

pub fn context_exit_sifr_cause(
    object: ObjectHandle,
    cause: &SifrExitCause,
) -> Result<PythonExitDecision, PythonError> {
    finish_context_exit(&object, call_exit_sifr_cause(&object, cause))
}

pub fn attach_secondary_python_error(primary: &mut PythonError, secondary: &PythonError) {
    let evidence = format!(
        "secondary Python context cleanup failure {}: {}",
        secondary.exception_type, secondary.message
    );
    if primary.context.is_empty() {
        primary.context = evidence;
    } else {
        primary.context.push_str("; ");
        primary.context.push_str(&evidence);
    }
}

pub fn record_context_cleanup_evidence(primary_cause: &str, secondary: &PythonError) {
    cleanup_evidence().push(ContextCleanupEvidence {
        primary_cause: primary_cause.to_string(),
        exception_type: secondary.exception_type.clone(),
        message: secondary.message.clone(),
        context: secondary.context.clone(),
    });
}

pub fn record_context_ignored_suppression(primary_cause: &str) {
    cleanup_evidence().push(ContextCleanupEvidence {
        primary_cause: primary_cause.to_string(),
        exception_type: "SifrBoundaryError".to_string(),
        message: "Python context suppression was ignored for a non-Python Sifr cause".to_string(),
        context: "context exit decision".to_string(),
    });
}

pub fn take_context_cleanup_evidence() -> Vec<ContextCleanupEvidence> {
    std::mem::take(&mut *cleanup_evidence())
}

fn call_exit_normal(object: &ObjectHandle) -> Result<PythonExitDecision, PythonError> {
    super::attach(|py| {
        let receiver = clone_handle(py, object)?;
        let result = receiver
            .bind(py)
            .call_method1("__exit__", (py.None(), py.None(), py.None()))
            .map_err(|error| PythonError::from_pyerr(py, error, "context", "__exit__ normal"))?;
        exit_decision(&result)
            .map_err(|error| PythonError::from_pyerr(py, error, "context", "__exit__ decision"))
    })
    .map_err(PythonError::runtime)?
}

fn call_exit_python_error(
    object: &ObjectHandle,
    error: &PythonError,
) -> Result<PythonExitDecision, PythonError> {
    super::attach(|py| {
        let receiver = clone_handle(py, object)?;
        let (error_type, error_value, traceback) = error.replay(py)?;
        let result = receiver
            .bind(py)
            .call_method1(
                "__exit__",
                (
                    error_type.bind(py),
                    error_value.bind(py),
                    traceback.bind(py),
                ),
            )
            .map_err(|exit_error| {
                PythonError::from_pyerr(py, exit_error, "context", "__exit__ Python exception")
            })?;
        exit_decision(&result).map_err(|decision_error| {
            PythonError::from_pyerr(py, decision_error, "context", "__exit__ decision")
        })
    })
    .map_err(PythonError::runtime)?
}

fn call_exit_sifr_cause(
    object: &ObjectHandle,
    cause: &SifrExitCause,
) -> Result<PythonExitDecision, PythonError> {
    super::attach(|py| {
        let receiver = clone_handle(py, object)?;
        let error_type = boundary_error_type(py)?;
        let error_value = error_type
            .call1((cause.kind.label(), &cause.sifr_type, &cause.message))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "context", "SifrBoundaryError construction")
            })?;
        let result = receiver
            .bind(py)
            .call_method1("__exit__", (error_type, &error_value, py.None()))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "context", "__exit__ Sifr cause")
            })?;
        exit_decision(&result)
            .map_err(|error| PythonError::from_pyerr(py, error, "context", "__exit__ decision"))
    })
    .map_err(PythonError::runtime)?
}

fn finish_context_exit(
    object: &ObjectHandle,
    result: Result<PythonExitDecision, PythonError>,
) -> Result<PythonExitDecision, PythonError> {
    match result {
        Ok(decision) => {
            object.close();
            Ok(decision)
        }
        Err(error) => {
            object.poison();
            Err(error)
        }
    }
}

fn exit_decision(value: &Bound<'_, PyAny>) -> PyResult<PythonExitDecision> {
    value.is_truthy().map(|truthy| {
        if truthy {
            PythonExitDecision::Suppress
        } else {
            PythonExitDecision::Propagate
        }
    })
}

fn boundary_error_type(py: Python<'_>) -> Result<Bound<'_, PyType>, PythonError> {
    BOUNDARY_ERROR_TYPE
        .get()
        .map(|error_type| error_type.bind(py).clone())
        .ok_or_else(|| {
            PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
                "SifrBoundaryError is not registered".to_string(),
            ))
        })
}

fn boundary_registration_error(error: PyErr) -> PythonRuntimeError {
    PythonRuntimeError::PythonOperationFailed(format!(
        "failed to register SifrBoundaryError: {error}"
    ))
}

fn cleanup_evidence() -> std::sync::MutexGuard<'static, Vec<ContextCleanupEvidence>> {
    match CLEANUP_EVIDENCE.lock() {
        Ok(evidence) => evidence,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
pub(super) fn reset_context_state_for_tests() {
    cleanup_evidence().clear();
}
