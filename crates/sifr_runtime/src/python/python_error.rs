use super::{ForeignObject, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict};
use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct PythonExceptionReplay {
    triple: ForeignObject,
}

impl PythonExceptionReplay {
    fn capture(py: Python<'_>, error: &PyErr) -> Option<Self> {
        let triple = PyDict::new(py);
        triple.set_item("type", error.get_type(py)).ok()?;
        triple.set_item("value", error.value(py)).ok()?;
        match error.traceback(py) {
            Some(traceback) => triple.set_item("traceback", traceback).ok()?,
            None => triple.set_item("traceback", py.None()).ok()?,
        }
        Some(Self {
            triple: ForeignObject::new(triple.unbind().into()).ok()?,
        })
    }

    pub(super) fn resolve(
        &self,
        py: Python<'_>,
    ) -> Result<(Py<PyAny>, Py<PyAny>, Py<PyAny>), PythonError> {
        let triple = self.triple.clone_ref(py).map_err(PythonError::runtime)?;
        let dict = triple.bind(py).cast::<PyDict>().map_err(|_| {
            PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
                "Python exception replay capability is malformed".to_string(),
            ))
        })?;
        let resolve = |key: &str| {
            dict.get_item(key)
                .map_err(|error| PythonError::from_pyerr(py, error, "context", "exception replay"))?
                .ok_or_else(|| {
                    PythonError::runtime(PythonRuntimeError::PythonOperationFailed(format!(
                        "Python exception replay capability is missing {key}"
                    )))
                })
                .map(Bound::unbind)
        };
        Ok((resolve("type")?, resolve("value")?, resolve("traceback")?))
    }
}

#[derive(Clone, Debug)]
pub struct PythonError {
    pub kind: String,
    pub exception_type: String,
    pub message: String,
    pub traceback: String,
    pub context: String,
    pub(crate) replay: Option<Box<PythonExceptionReplay>>,
}

impl PythonError {
    pub(super) fn runtime(error: PythonRuntimeError) -> Self {
        Self::without_replay(
            "runtime",
            "SifrPythonRuntimeError",
            error.to_string(),
            String::new(),
            String::new(),
        )
    }

    pub(super) fn trust(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::without_replay(
            "trust",
            "SIFR-PYTRUST",
            message.into(),
            String::new(),
            context.into(),
        )
    }

    pub(super) fn closed() -> Self {
        Self::without_replay(
            "resource",
            "SifrPythonClosedObject",
            "Python object identity is closed",
            String::new(),
            "object handle lookup",
        )
    }

    pub(super) fn from_pyerr(
        py: Python<'_>,
        error: PyErr,
        kind: &'static str,
        context: impl Into<String>,
    ) -> Self {
        let exception_type = error
            .get_type(py)
            .name()
            .map_or_else(|_| "PythonError".to_string(), |name| name.to_string());
        let traceback = format_traceback(py, &error);
        let replay = PythonExceptionReplay::capture(py, &error).map(Box::new);
        Self {
            kind: kind.to_string(),
            exception_type,
            message: error.to_string(),
            traceback,
            context: context.into(),
            replay,
        }
    }

    #[doc(hidden)]
    pub fn without_replay(
        kind: impl Into<String>,
        exception_type: impl Into<String>,
        message: impl Into<String>,
        traceback: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            exception_type: exception_type.into(),
            message: message.into(),
            traceback: traceback.into(),
            context: context.into(),
            replay: None,
        }
    }

    pub(super) fn replay(
        &self,
        py: Python<'_>,
    ) -> Result<(Py<PyAny>, Py<PyAny>, Py<PyAny>), PythonError> {
        self.replay
            .as_ref()
            .ok_or_else(|| {
                Self::runtime(PythonRuntimeError::PythonOperationFailed(
                    "Python error has no live exception replay capability".to_string(),
                ))
            })?
            .resolve(py)
    }
}

impl PartialEq for PythonError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.exception_type == other.exception_type
            && self.message == other.message
            && self.traceback == other.traceback
            && self.context == other.context
    }
}

impl Eq for PythonError {}

impl fmt::Display for PythonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.context.is_empty() {
            write!(formatter, "{}: {}", self.exception_type, self.message)
        } else {
            write!(
                formatter,
                "{} during {}: {}",
                self.exception_type, self.context, self.message
            )
        }
    }
}

impl std::error::Error for PythonError {}

fn format_traceback(py: Python<'_>, error: &PyErr) -> String {
    py.import("traceback")
        .and_then(|traceback| {
            traceback.call_method1(
                "format_exception",
                (error.get_type(py), error.value(py), error.traceback(py)),
            )
        })
        .and_then(|formatted| formatted.extract::<Vec<String>>())
        .map(|parts| parts.join(""))
        .unwrap_or_default()
}
