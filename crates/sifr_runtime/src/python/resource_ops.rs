use super::object_ops::clone_handle;
#[cfg(test)]
use super::object_ops::store_object;
use super::{ObjectHandle, PythonError};
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::{PyAnyMethods, PyModule};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonResourceDiagnostics {
    pub initialized: bool,
    pub live_objects: i64,
    pub leaked_objects: i64,
}

pub fn resource_diagnostics() -> Result<PythonResourceDiagnostics, PythonError> {
    let diagnostics = super::shutdown_diagnostics().map_err(PythonError::runtime)?;
    Ok(PythonResourceDiagnostics {
        initialized: diagnostics.initialized,
        live_objects: i64::try_from(diagnostics.live_objects).map_err(|_| {
            PythonError::runtime(super::PythonRuntimeError::PythonOperationFailed(
                "Python live object count exceeds Sifr int range".to_string(),
            ))
        })?,
        leaked_objects: i64::try_from(diagnostics.leaked_objects).map_err(|_| {
            PythonError::runtime(super::PythonRuntimeError::PythonOperationFailed(
                "Python leaked object count exceeds Sifr int range".to_string(),
            ))
        })?,
    })
}

pub fn exit_context_with_error(
    object: ObjectHandle,
    kind: &str,
    exception_type: &str,
    message: &str,
    traceback: &str,
    context: &str,
) -> Result<(), PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let failure = PyRuntimeError::new_err(format!(
            "Sifr PythonError(kind={kind}, exception_type={exception_type}, context={context}): {message}\n{traceback}"
        ));
        object
            .bind(py)
            .call_method1(
                "__exit__",
                (failure.get_type(py), failure.value(py), py.None()),
            )
            .map(|_| ())
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "__exit__ failure"))
    })
    .map_err(PythonError::runtime)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        close_object, enter_context, from_int, get_attr, initialize_runtime,
        reset_runtime_state_for_tests, test_config, test_guard, to_bool,
    };

    #[test]
    fn resource_diagnostics_reports_live_and_closed_objects() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("resource-diagnostics")).expect("init should succeed");

        let value = from_int(1).expect("object should be stored");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 1,
                leaked_objects: 0,
            }
        );
        close_object(value).expect("object should close");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn double_close_reports_deterministic_resource_error() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("double-close")).expect("init should succeed");

        let value = from_int(1).expect("object should be stored");
        close_object(value).expect("first close should succeed");
        let error = close_object(value).expect_err("second close should fail");

        assert_eq!(error.kind, "resource");
        assert_eq!(error.exception_type, "SifrPythonClosedObject");
        assert!(error.message.contains("closed"));
    }

    #[test]
    fn exit_context_with_error_passes_sifr_error_context_to_python_exit() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("context-exit-with-error")).expect("init should succeed");

        let manager = super::super::attach(|py| {
            let module = PyModule::from_code(
                py,
                c"class RecordingContext:\n    def __enter__(self):\n        return self\n    def __exit__(self, exc_type, exc_value, tb):\n        self.saw_error = exc_type is not None and exc_value is not None\n        self.saw_traceback = tb is not None\n        return False\n",
                c"recording_context.py",
                c"recording_context",
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "recording context"))?;
            let manager = module
                .getattr("RecordingContext")
                .and_then(|class| class.call0())
                .map_err(|error| PythonError::from_pyerr(py, error, "call", "RecordingContext"))?;
            store_object(manager.unbind())
        })
        .map_err(PythonError::runtime)
        .expect("recording context should build")
        .expect("recording context should be stored");
        let entered = enter_context(manager).expect("context should enter");

        exit_context_with_error(
            manager,
            "call",
            "SifrBodyError",
            "body failed",
            "traceback",
            "with_context body",
        )
        .expect("__exit__ should receive Sifr error context");

        let saw_error_attr = get_attr(manager, "saw_error").expect("saw_error should be stored");
        assert!(to_bool(saw_error_attr).expect("saw_error should be true"));
        let saw_traceback_attr =
            get_attr(manager, "saw_traceback").expect("saw_traceback should be stored");
        assert!(!to_bool(saw_traceback_attr).expect("traceback should be None"));

        for handle in [entered, saw_error_attr, saw_traceback_attr, manager] {
            close_object(handle).expect("object should close");
        }
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }
}
