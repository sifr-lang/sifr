use super::*;

#[test]
fn initializes_and_accepts_repeated_same_config() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let config = test_config("same");

    assert_eq!(
        initialize_runtime(config.clone()),
        Ok(PythonRuntimeInitStatus::Initialized)
    );
    assert_eq!(
        initialize_runtime(config),
        Ok(PythonRuntimeInitStatus::AlreadyInitialized)
    );
}

#[test]
fn rejects_reinitialization_with_different_config() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("first")).expect("first init should succeed");

    let error =
        initialize_runtime(test_config("second")).expect_err("different config should fail");

    assert!(matches!(
        error,
        PythonRuntimeError::ConflictingEnvironment { .. }
    ));
}

#[test]
fn attach_requires_runtime_initialization() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();

    let result = attach(|py| py.version_info().major);

    assert_eq!(result, Err(PythonRuntimeError::NotInitialized));
}

#[test]
fn attach_and_detach_run_under_initialized_runtime() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("detach")).expect("init should succeed");

    let result = attach(|py| detach(py, || 41 + 1));

    assert_eq!(result, Ok(42));
}

#[test]
fn embedded_runtime_disables_bytecode_writes() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("no-bytecode")).expect("init should succeed");

    let disabled = attach(|py| {
        py.import("sys")
            .and_then(|sys| sys.getattr("dont_write_bytecode"))
            .and_then(|value| value.extract::<bool>())
    })
    .expect("attach should succeed")
    .expect("sys.dont_write_bytecode should be a bool");

    assert!(disabled);
}

#[test]
fn owned_object_tracking_is_released_on_next_attach() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("object")).expect("init should succeed");
    let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
    assert!(object.is_ok());
    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        PythonRuntimeDiagnostics {
            initialized: true,
            live_objects: 1,
            leaked_objects: 0,
        }
    );

    drop(object);
    attach(|_py| ()).expect("attach should drain the dropped object");

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
fn detached_thread_drop_is_drained_on_next_attach() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("detached-drop")).expect("init should succeed");
    let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
    let object = object.expect("object should be created");

    std::thread::spawn(move || drop(object))
        .join()
        .expect("detached drop thread should finish");

    assert_eq!(foreign_object::pending_release_count(), 1);
    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        PythonRuntimeDiagnostics {
            initialized: true,
            live_objects: 1,
            leaked_objects: 0,
        }
    );
    attach(|_py| ()).expect("attach should drain pending releases");
    assert_eq!(foreign_object::pending_release_count(), 0);
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
fn runtime_guard_drains_pending_releases_at_epilogue() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("epilogue-drop")).expect("init should succeed");
    let runtime_guard = runtime_guard().expect("runtime guard should be available");
    let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
    let object = object.expect("object should be created");
    std::thread::spawn(move || drop(object))
        .join()
        .expect("detached drop thread should finish");
    assert_eq!(foreign_object::pending_release_count(), 1);

    drop(runtime_guard);

    assert_eq!(foreign_object::pending_release_count(), 0);
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
fn shutdown_validation_reports_outstanding_objects() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("shutdown")).expect("init should succeed");
    let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
    assert!(object.is_ok());

    let error = validate_shutdown().expect_err("live object should block shutdown");

    assert_eq!(
        error,
        PythonRuntimeError::OutstandingResources {
            live_objects: 1,
            leaked_objects: 0,
        }
    );
    drop(object);
}

#[test]
fn object_operations_cover_import_attr_item_call_kwargs_and_context() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("ops")).expect("init should succeed");

    let builtins = import_module("builtins").expect("builtins import should succeed");
    let dict = get_attr(&builtins, "dict").expect("dict attr should succeed");
    let math = import_module("math").expect("math import should succeed");
    let kwargs_dict =
        call_object(&dict, &[], &[("module", math.clone())]).expect("kwargs call should succeed");
    let item = get_item_str(&kwargs_dict, "module").expect("item access should succeed");
    close_object(item).expect("item close should succeed");
    close_object(kwargs_dict).expect("dict close should succeed");
    close_object(math).expect("math close should succeed");
    close_object(dict).expect("dict callable close should succeed");

    let contextlib = import_module("contextlib").expect("contextlib import should succeed");
    let nullcontext =
        get_attr(&contextlib, "nullcontext").expect("nullcontext attr should succeed");
    let manager = call_object(&nullcontext, &[], &[]).expect("nullcontext call should succeed");
    let entered = enter_context(&manager).expect("context enter should succeed");
    exit_context(&manager).expect("context exit should succeed");
    close_object(entered).expect("entered context close should succeed");
    close_object(manager).expect("context manager close should succeed");
    close_object(nullcontext).expect("nullcontext close should succeed");
    close_object(contextlib).expect("contextlib close should succeed");
    close_object(builtins).expect("builtins close should succeed");

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
fn import_failure_preserves_python_exception_context() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("import-failure");
    config
        .required_import_roots
        .push("sifr_missing_py3_module".to_string());
    config
        .trusted_import_roots
        .push("sifr_missing_py3_module".to_string());
    initialize_runtime(config).expect("init should succeed");

    let error = import_module("sifr_missing_py3_module")
        .expect_err("missing module should fail as PythonError");

    assert_eq!(error.kind, "import");
    assert_eq!(error.exception_type, "ModuleNotFoundError");
    assert!(error.message.contains("sifr_missing_py3_module"));
    assert!(!error.traceback.is_empty());
}

#[test]
fn trust_policy_rejects_undeclared_import_roots() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("trust")).expect("init should succeed");

    let error = import_module("json").expect_err("untrusted import should fail");

    assert_eq!(error.kind, "trust");
    assert_eq!(error.exception_type, "SIFR-PYTRUST");
    assert!(error.message.contains("json"));
}

#[test]
fn failed_record_field_copy_does_not_leak_partial_handles() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("record-field-leak")).expect("init should succeed");
    let value = from_int(42).expect("field value should be stored");
    let record = from_record(&[("value", value.clone())]).expect("record should be stored");
    let before = shutdown_diagnostics().expect("diagnostics should be available");

    let error =
        copy_record_fields(&record, &["value", "missing"]).expect_err("missing field should fail");

    assert_eq!(error.kind, "conversion");
    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        before
    );
    close_object(record).expect("record should close");
    close_object(value).expect("field value should close");
}

#[test]
fn attribute_and_call_failures_preserve_python_error_families() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("operation-failures")).expect("init should succeed");
    let math = import_module("math").expect("math import should succeed");

    let attr_error = get_attr(&math, "does_not_exist")
        .expect_err("missing attribute should fail as PythonError");
    assert_eq!(attr_error.kind, "attribute");
    assert_eq!(attr_error.exception_type, "AttributeError");

    let builtins = import_module("builtins").expect("builtins import should succeed");
    let len = get_attr(&builtins, "len").expect("len attr should succeed");
    let call_error = call_object(&len, &[], &[]).expect_err("wrong arg count should fail");
    assert_eq!(call_error.kind, "call");
    assert_eq!(call_error.exception_type, "TypeError");

    close_object(len).expect("len close should succeed");
    close_object(builtins).expect("builtins close should succeed");
    close_object(math).expect("math close should succeed");
}
