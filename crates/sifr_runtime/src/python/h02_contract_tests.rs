use super::*;
use pyo3::types::PyModule;

#[test]
#[allow(unsafe_code)]
fn initialization_and_gil_entry_contract() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    assert_eq!(attach(|_| ()), Err(PythonRuntimeError::NotInitialized));
    assert_eq!(
        std::thread::spawn(|| attach(|_| ()))
            .join()
            .expect("foreign entry"),
        Err(PythonRuntimeError::NotInitialized)
    );
    let config = test_config("h02-core-initialization");
    let mut invalid = config.clone();
    invalid.sys_path.push("/bad\0path".to_string());
    assert!(matches!(
        initialize_runtime(invalid),
        Err(PythonRuntimeError::PythonOperationFailed(_))
    ));
    assert_eq!(runtime_state().expect("state").config, None);
    assert!(!shutdown_diagnostics().expect("diagnostics").initialized);
    assert_eq!(
        initialize_runtime(config.clone()),
        Ok(PythonRuntimeInitStatus::Initialized)
    );
    assert_eq!(
        initialize_runtime(config.clone()),
        Ok(PythonRuntimeInitStatus::AlreadyInitialized)
    );
    let mut conflicting = config;
    conflicting.probe_digest.push_str("-different");
    assert!(matches!(
        initialize_runtime(conflicting),
        Err(PythonRuntimeError::ConflictingEnvironment { .. })
    ));
    let foreign = std::thread::spawn(|| {
        attach(|py| {
            // SAFETY: only observes this attached thread's state, never dereferences
            // foreign data. Detach must release entry until PyO3 restores it.
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            let value = detach(py, || {
                assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 0);
                42
            });
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            assert_eq!(
                py.eval(c"40 + 2", None, None)
                    .expect("foreign Python eval")
                    .extract::<i32>()
                    .expect("result"),
                value
            );
            value
        })
    })
    .join()
    .expect("foreign join")
    .expect("foreign attach");
    assert_eq!(foreign, 42);

    // Verification failure must never expose a partially initialized runtime.
    reset_runtime_state_for_tests();
    let mut wrong = test_config("h02-core-failed-verification");
    wrong.cpython_version_tuple = vec![99, 99];
    assert!(matches!(
        initialize_runtime(wrong.clone()),
        Err(PythonRuntimeError::InterpreterVersionMismatch { .. })
    ));
    assert_eq!(attach(|_| ()), Err(PythonRuntimeError::NotInitialized));
    assert!(matches!(
        initialize_runtime(wrong),
        Err(PythonRuntimeError::InterpreterVersionMismatch { .. })
    ));
    assert!(matches!(
        initialize_runtime(test_config("replacement")),
        Err(PythonRuntimeError::ConflictingEnvironment { .. })
    ));
    reset_runtime_state_for_tests();
}

#[test]
fn foreign_object_thread_lifetime_and_release_contract() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ForeignObject>();
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("h02-core-object")).expect("initialize");
    let (object, module) = attach(|py| {
        let module = PyModule::from_code(py, c"releases = 0\nclass Tracked:\n    def __del__(self):\n        global releases\n        releases += 1\n", c"h02_lifetime.py", c"__h02_lifetime__").expect("module");
        let object = ForeignObject::new(module.getattr("Tracked").expect("class").call0().expect("object").unbind()).expect("foreign object");
        (object, module.unbind())
    }).expect("attach");
    let alias = object.clone();
    let lease = object.lease().expect("in-flight pin");
    let closing = object
        .begin_semantic_close()
        .expect("semantic close authority");
    assert!(object.lease().is_err());
    assert!(object.begin_semantic_close().is_err());
    attach(|py| {
        assert!(alias.clone_ref(py).is_err());
        assert!(
            lease.clone_ref(py).is_ok(),
            "only existing in-flight pins remain resolvable"
        );
    })
    .expect("check closing");
    object.finish_semantic_close(true);
    drop(closing);
    assert_eq!(
        attach(|py| module
            .bind(py)
            .getattr("releases")
            .expect("counter")
            .extract::<usize>()
            .expect("counter type"))
        .expect("attach"),
        0
    );
    assert_eq!(shutdown_diagnostics().expect("diagnostics").live_objects, 1);
    std::thread::spawn(move || drop(lease))
        .join()
        .expect("detached release");
    assert_eq!(foreign_object::pending_release_count(), 1);
    assert_eq!(
        shutdown_diagnostics()
            .expect("pending counted")
            .live_objects,
        1
    );
    assert_eq!(
        attach(|py| {
            assert!(alias.clone_ref(py).is_err());
            module
                .bind(py)
                .getattr("releases")
                .expect("counter")
                .extract::<usize>()
                .expect("counter type")
        })
        .expect("drain attached"),
        1
    );
    object.close();
    alias.close();
    drop(object);
    drop(alias);
    assert_eq!(foreign_object::pending_release_count(), 0);
    assert_eq!(
        shutdown_diagnostics()
            .expect("final diagnostics")
            .live_objects,
        0
    );
    assert_eq!(
        attach(|py| module
            .bind(py)
            .getattr("releases")
            .expect("counter")
            .extract::<usize>()
            .expect("counter type"))
        .expect("check exact once"),
        1
    );
    // Public semantic cleanup must claim exclusive authority before invoking
    // Python, including when Python releases the GIL during cleanup.
    let (closing_object, cleanup_module) = attach(|py| {
        let module = PyModule::from_code(py, c"import threading\nentered = threading.Event()\nfinish = threading.Event()\ncalls = 0\nclass Cleanup:\n    def close(self):\n        global calls\n        calls += 1\n        entered.set()\n        assert finish.wait(5)\n", c"h02_cleanup.py", c"__h02_cleanup__").expect("cleanup module");
        let object = ForeignObject::new(module.getattr("Cleanup").expect("cleanup class").call0().expect("cleanup object").unbind()).expect("foreign cleanup");
        (object, module.unbind())
    }).expect("setup cleanup");
    let thread_object = closing_object.clone();
    let closer = std::thread::spawn(move || semantic_close(thread_object, "close"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let entered = attach(|py| {
            cleanup_module
                .bind(py)
                .getattr("entered")
                .expect("event")
                .call_method0("is_set")
                .expect("is_set")
                .extract::<bool>()
                .expect("bool")
        })
        .expect("wait cleanup");
        if entered {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "semantic close entered"
        );
        std::thread::yield_now();
    }
    attach(|py| assert!(closing_object.clone_ref(py).is_err())).expect("reject concurrent access");
    assert!(
        semantic_close(closing_object.clone(), "close").is_err(),
        "reject duplicate semantic close"
    );
    attach(|py| {
        cleanup_module
            .bind(py)
            .getattr("finish")
            .expect("event")
            .call_method0("set")
            .expect("set");
    })
    .expect("finish cleanup");
    closer
        .join()
        .expect("close thread")
        .expect("semantic close");
    assert_eq!(
        attach(|py| cleanup_module
            .bind(py)
            .getattr("calls")
            .expect("calls")
            .extract::<usize>()
            .expect("count"))
        .expect("read calls"),
        1
    );
    assert_eq!(
        shutdown_diagnostics()
            .expect("released cleanup")
            .live_objects,
        0
    );
    drop(closing_object);
    drop(cleanup_module);
    drop(module);
    reset_runtime_state_for_tests();
}
