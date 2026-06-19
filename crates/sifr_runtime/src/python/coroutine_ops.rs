use super::object_ops::{clone_handle, store_object};
use super::{ObjectHandle, PythonError};
use pyo3::types::PyAnyMethods;

pub fn run_coroutine_blocking(coroutine: ObjectHandle) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let coroutine = clone_handle(py, coroutine)?;
        let asyncio = py
            .import("asyncio")
            .map_err(|error| PythonError::from_pyerr(py, error, "import", "asyncio"))?;
        let value = asyncio
            .call_method1("run", (coroutine.bind(py),))
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "asyncio.run coroutine"))?;
        store_object(value.unbind())
    })
    .map_err(PythonError::runtime)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        call_object, close_object, from_float, from_int, get_attr, import_module,
        initialize_runtime, reset_runtime_state_for_tests, shutdown_diagnostics, test_config,
        test_guard, to_int, PythonRuntimeDiagnostics,
    };

    #[test]
    fn run_coroutine_blocking_runs_python_owned_event_loop() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("run-coroutine")).expect("init should succeed");

        let asyncio = import_module("asyncio").expect("asyncio module should import");
        let sleep = get_attr(asyncio, "sleep").expect("sleep should resolve");
        let delay = from_float(0.0).expect("delay should store");
        let expected = from_int(41).expect("result should store");
        let coroutine = call_object(sleep, &[delay], &[("result", expected)])
            .expect("sleep coroutine should be created");
        close_object(delay).expect("delay should close after coroutine creation");
        close_object(expected).expect("expected value should close after coroutine creation");
        let value = run_coroutine_blocking(coroutine).expect("coroutine should complete");

        assert_eq!(to_int(value).expect("coroutine result should convert"), 41);

        for handle in [asyncio, sleep, coroutine, value] {
            close_object(handle).expect("object should close");
        }
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }
}
