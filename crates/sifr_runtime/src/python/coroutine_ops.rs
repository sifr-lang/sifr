use super::object_ops::{clone_handle, store_object};
use super::{ObjectHandle, PythonError};

pub fn run_coroutine_blocking(coroutine: &ObjectHandle) -> Result<ObjectHandle, PythonError> {
    super::async_runtime::ensure_started().map_err(PythonError::runtime)?;
    super::attach(|py| {
        let coroutine = clone_handle(py, coroutine)?;
        super::async_runtime::run_coroutine_blocking(py, &coroutine).and_then(store_object)
    })
    .map_err(PythonError::runtime)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        async_runtime_diagnostics, call_object, close_object, from_float, from_int, get_attr,
        import_module, initialize_runtime, reset_runtime_state_for_tests, shutdown_diagnostics,
        test_config, test_guard, to_int, PythonAsyncRuntimeDiagnostics, PythonRuntimeDiagnostics,
    };
    use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyModule};
    use std::sync::{Arc, Barrier};

    #[test]
    fn run_coroutine_blocking_uses_the_application_owned_event_loop() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("run-coroutine")).expect("init should succeed");

        let asyncio = import_module("asyncio").expect("asyncio module should import");
        let sleep = get_attr(&asyncio, "sleep").expect("sleep should resolve");
        let delay = from_float(0.0).expect("delay should store");
        let expected = from_int(41).expect("result should store");
        let coroutine = call_object(&sleep, &[delay.clone()], &[("result", expected.clone())])
            .expect("sleep coroutine should be created");
        close_object(delay).expect("delay should close after coroutine creation");
        close_object(expected).expect("expected value should close after coroutine creation");
        let value = run_coroutine_blocking(&coroutine).expect("coroutine should complete");

        assert_eq!(to_int(&value).expect("coroutine result should convert"), 41);

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
        super::super::async_runtime::shutdown().expect("owned loop should stop");
        assert_eq!(
            async_runtime_diagnostics().expect("async diagnostics should be available"),
            PythonAsyncRuntimeDiagnostics::default()
        );
    }

    #[test]
    fn concurrent_raw_coroutines_share_one_owned_loop_and_thread() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let mut config = test_config("shared-loop");
        config
            .required_import_roots
            .push("__sifr_async_identity__".to_string());
        config
            .trusted_import_roots
            .push("__sifr_async_identity__".to_string());
        initialize_runtime(config).expect("init should preserve lazy owned-loop startup");
        install_identity_module();

        let start_barrier = Arc::new(Barrier::new(3));
        let workers = (0..2)
            .map(|_| {
                let start_barrier = Arc::clone(&start_barrier);
                std::thread::spawn(move || {
                    start_barrier.wait();
                    run_identity_coroutine()
                })
            })
            .collect::<Vec<_>>();
        start_barrier.wait();
        let identities = workers
            .into_iter()
            .map(|worker| worker.join().expect("identity worker should not panic"))
            .collect::<Vec<_>>();

        assert_eq!(identities[0], identities[1]);
        assert_eq!(
            async_runtime_diagnostics().expect("async diagnostics should be available"),
            PythonAsyncRuntimeDiagnostics {
                running: true,
                stopping: false,
                loop_threads: 1,
                active_submissions: 0,
                pending_submissions: 0,
            }
        );
        super::super::async_runtime::shutdown().expect("owned loop should stop");
        assert_eq!(
            async_runtime_diagnostics().expect("async diagnostics should be available"),
            PythonAsyncRuntimeDiagnostics::default()
        );
    }

    #[test]
    fn raw_coroutine_python_failure_returns_checked_error_on_owned_loop() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let mut config = test_config("raw-failure");
        config
            .required_import_roots
            .push("__sifr_async_identity__".to_string());
        config
            .trusted_import_roots
            .push("__sifr_async_identity__".to_string());
        initialize_runtime(config).expect("init should preserve owned-loop startup");
        install_identity_module();

        let module = import_module("__sifr_async_identity__").expect("module should import");
        let fail = get_attr(&module, "fail").expect("failure function should resolve");
        let coroutine = call_object(&fail, &[], &[]).expect("coroutine should be created");
        let error =
            run_coroutine_blocking(&coroutine).expect_err("Python failure should be checked");

        assert_eq!(error.kind, "await");
        assert_eq!(error.exception_type, "ValueError");
        assert!(error.message.contains("raw failure"));
        drop(error);
        for handle in [module, fail, coroutine] {
            close_object(handle).expect("failure handle should close");
        }
        super::super::async_runtime::shutdown().expect("owned loop should stop");
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn shutdown_cancels_and_joins_an_in_flight_raw_coroutine() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let mut config = test_config("shutdown-in-flight");
        config.start_async_loop = true;
        config
            .required_import_roots
            .push("__sifr_async_identity__".to_string());
        config
            .trusted_import_roots
            .push("__sifr_async_identity__".to_string());
        initialize_runtime(config).expect("init should start the owned loop");
        install_identity_module();

        let worker = std::thread::spawn(run_waiting_coroutine);
        let registered = (0..5_000).any(|_| {
            let registered = async_runtime_diagnostics()
                .expect("async diagnostics should be available")
                .active_submissions
                == 1;
            if !registered {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            registered
        });
        if !registered {
            let _ignored = super::super::async_runtime::shutdown();
            let _ignored = worker.join();
            panic!("worker submission should be registered before shutdown");
        }

        super::super::async_runtime::shutdown().expect("owned loop should stop");
        assert!(
            worker.join().expect("waiting worker should not panic"),
            "shutdown should surface cancellation to the blocking raw caller"
        );
        assert_eq!(
            async_runtime_diagnostics().expect("async diagnostics should be available"),
            PythonAsyncRuntimeDiagnostics::default()
        );
    }

    fn install_identity_module() {
        super::super::attach(|py| {
            let module = PyModule::from_code(
                py,
                c"import asyncio\nimport threading\n\nasync def identity():\n    return f'{id(asyncio.get_running_loop())}:{threading.get_ident()}'\n\nasync def fail():\n    raise ValueError('raw failure')\n\nasync def wait():\n    await asyncio.sleep(60)\n",
                c"<sifr-async-identity>",
                c"__sifr_async_identity__",
            )
            .expect("identity module should compile");
            let sys = py.import("sys").expect("sys should import");
            let modules = sys
                .getattr("modules")
                .expect("sys.modules should resolve")
                .cast_into::<PyDict>()
                .expect("sys.modules should be a dict");
            modules
                .set_item("__sifr_async_identity__", module)
                .expect("identity module should install");
        })
        .expect("runtime should attach");
    }

    fn run_identity_coroutine() -> String {
        let module = import_module("__sifr_async_identity__").expect("module should import");
        let identity = get_attr(&module, "identity").expect("identity should resolve");
        let coroutine = call_object(&identity, &[], &[]).expect("coroutine should be created");
        let value = run_coroutine_blocking(&coroutine).expect("coroutine should complete");
        let identity_value = crate::python::to_str(&value).expect("identity should be text");
        for handle in [module, identity, coroutine, value] {
            close_object(handle).expect("identity handle should close");
        }
        identity_value
    }

    fn run_waiting_coroutine() -> bool {
        let module = import_module("__sifr_async_identity__").expect("module should import");
        let wait = get_attr(&module, "wait").expect("wait should resolve");
        let coroutine = call_object(&wait, &[], &[]).expect("coroutine should be created");
        let was_cancelled = run_coroutine_blocking(&coroutine).is_err();
        for handle in [module, wait, coroutine] {
            close_object(handle).expect("waiting handle should close");
        }
        was_cancelled
    }
}
