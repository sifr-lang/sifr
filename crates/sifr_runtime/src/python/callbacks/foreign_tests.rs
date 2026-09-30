use super::*;
use crate::python::{initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard};
use pyo3::types::PyAnyMethods;
use std::sync::Barrier;
use std::sync::atomic::{AtomicBool, AtomicUsize};

struct DropProbe(Arc<AtomicUsize>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn uncommitted_retained_callback_invalidates_escaped_shell_without_closing_group() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-callback-rollback"))
        .expect("runtime should initialize");
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let callback = foreign_callback_with_owner(
        owner.clone(),
        44,
        1,
        ForeignCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        |_, value| Ok(value),
        crate::python::from_int,
    )
    .expect("callback should create");
    let escaped =
        crate::python::attach(|py| crate::python::object_ops::clone_handle(py, callback.object()))
            .expect("runtime should attach")
            .expect("callable should clone");
    drop(callback);

    assert_eq!(owner.status(), super::super::CallbackOwnerStatus::Open);
    let exception = crate::python::attach(|py| match escaped.bind(py).call1((1_i64,)) {
        Ok(_) => Ok(None),
        Err(error) => error
            .get_type(py)
            .getattr("__name__")
            .and_then(|name| name.extract::<String>())
            .map(Some),
    })
    .expect("runtime should attach")
    .expect("exception name should resolve");
    assert_eq!(exception.as_deref(), Some("SifrCallbackClosedError"));
    owner
        .shutdown_from_runtime()
        .expect("group should remain independently closeable");
}

#[test]
fn escaped_retained_callable_rejects_before_validating_after_owner_close() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-callback-after-close"))
        .expect("runtime should initialize");
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let callback = foreign_callback_with_owner(
        owner.clone(),
        31,
        1,
        ForeignCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        |_, value| Ok(value),
        crate::python::from_int,
    )
    .expect("callback should create");
    callback
        .retain_in_owner()
        .expect("callable should be retained by owner");
    let escaped =
        crate::python::attach(|py| crate::python::object_ops::clone_handle(py, callback.object()))
            .expect("runtime should attach")
            .expect("callable should clone");

    let unregister = owner
        .begin_owner_unregister()
        .expect("unregister should begin");
    drop(unregister);
    owner
        .close_after_owner_unregister()
        .expect("owner should close");
    for outcome in crate::python::attach(|py| {
        [escaped.bind(py).call1((1_i64,)), escaped.bind(py).call0()]
            .into_iter()
            .map(|outcome| match outcome {
                Ok(_) => Ok(None),
                Err(error) => error
                    .get_type(py)
                    .getattr("__name__")
                    .and_then(|name| name.extract::<String>())
                    .map(Some),
            })
            .collect::<Vec<_>>()
    })
    .expect("runtime should attach")
    {
        let exception_name = outcome
            .expect("exception name should resolve")
            .expect("escaped callable must reject entry after close");
        assert_eq!(exception_name, "SifrCallbackClosedError");
    }
}

#[test]
fn retained_foreign_callbacks_allocate_distinct_owner_local_identities() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-foreign-callback-identities"))
        .expect("runtime should initialize");
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let nested_slot = Arc::new(Mutex::new(None::<ObjectHandle>));

    let nested = foreign_callback_with_owner(
        owner.clone(),
        1,
        1,
        ForeignCallbackConcurrency::Serial,
        |args| crate::python::to_int(&args[0]),
        |_, value| Ok(value + 1),
        crate::python::from_int,
    )
    .expect("nested callback should create");
    *nested_slot.lock().expect("nested slot") = Some(nested.object().clone());

    let nested_for_handler = Arc::clone(&nested_slot);
    let outer = foreign_callback_with_owner(
        owner.clone(),
        1,
        1,
        ForeignCallbackConcurrency::Serial,
        |args| crate::python::to_int(&args[0]),
        move |_, value| {
            let nested = nested_for_handler
                .lock()
                .expect("nested slot")
                .clone()
                .expect("nested callback should be installed");
            let argument = crate::python::from_int(value)?;
            crate::python::call_object_owned(&nested, &[argument], &[])
                .and_then(|result| crate::python::to_int(&result))
                .map_err(CallbackExecutionError::from)
        },
        crate::python::from_int,
    )
    .expect("outer callback should create");

    let argument = crate::python::from_int(41).expect("argument should convert");
    let result = crate::python::call_object_owned(outer.object(), &[argument], &[])
        .and_then(|value| crate::python::to_int(&value))
        .expect("distinct callbacks must not look recursively reentrant");
    assert_eq!(result, 42);

    drop(outer);
    drop(nested);
    owner
        .shutdown_from_runtime()
        .expect("owner should remain independently closeable");
}

#[tokio::test(flavor = "current_thread")]
async fn async_call_scope_close_yields_while_foreign_invocation_drains() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("foreign-async-drain")).expect("runtime should initialize");
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let entered_by_handler = Arc::clone(&entered);
    let release_by_handler = Arc::clone(&release);
    let callback = foreign_callback(
        1,
        1,
        ForeignCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value| {
            entered_by_handler.wait();
            release_by_handler.wait();
            Ok(value)
        },
        crate::python::from_int,
    )
    .expect("callback should create");
    let callable = callback.object().clone();
    let invocation = std::thread::spawn(move || {
        let argument = crate::python::from_int(42).expect("argument should convert");
        crate::python::call_object_owned(&callable, &[argument], &[])
            .expect("foreign invocation should finish");
    });
    entered.wait();

    let progressed = Arc::new(AtomicBool::new(false));
    let progressed_by_release = Arc::clone(&progressed);
    let release_after_yield = async {
        tokio::task::yield_now().await;
        progressed_by_release.store(true, Ordering::SeqCst);
        release.wait();
    };
    let (close, ()) = tokio::join!(callback.close_call_scope_async(), release_after_yield);
    close.expect("async close should drain without blocking the executor");
    assert!(progressed.load(Ordering::SeqCst));
    invocation.join().expect("invocation thread should join");
}

#[test]
fn escaped_retained_callable_does_not_retain_handler_captures_after_owner_close() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-callback-capture-release"))
        .expect("runtime should initialize");
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let drops = Arc::new(AtomicUsize::new(0));
    let probe = DropProbe(Arc::clone(&drops));
    let callback = foreign_callback_with_owner(
        owner.clone(),
        32,
        1,
        ForeignCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value| {
            let _capture = &probe;
            Ok(value)
        },
        crate::python::from_int,
    )
    .expect("callback should create");
    callback
        .retain_in_owner()
        .expect("callable should be retained by owner");
    let escaped =
        crate::python::attach(|py| crate::python::object_ops::clone_handle(py, callback.object()))
            .expect("runtime should attach")
            .expect("callable should clone");

    let unregister = owner
        .begin_owner_unregister()
        .expect("unregister should begin");
    drop(unregister);
    owner
        .close_after_owner_unregister()
        .expect("owner should close");

    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(
        crate::python::attach(|py| escaped.bind(py).call1((1_i64,)).is_err())
            .expect("runtime should attach")
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn serial_conversion_failure_does_not_strand_fifo_admission() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("foreign-serial-conversion"))
        .expect("runtime should initialize");
    let callback = foreign_callback(
        45,
        1,
        ForeignCallbackConcurrency::Serial,
        |args| crate::python::to_int(&args[0]),
        |_, value| Ok(value),
        crate::python::from_int,
    )
    .expect("callback should create");
    let wrong = crate::python::from_str("wrong").expect("argument should create");
    crate::python::call_object_owned(callback.object(), &[wrong], &[])
        .expect_err("conversion should fail before FIFO admission");
    let valid = crate::python::from_int(7).expect("argument should create");
    let result = crate::python::call_object_owned(callback.object(), &[valid], &[])
        .and_then(|value| crate::python::to_int(&value))
        .expect("later serial invocation must not be stranded");
    assert_eq!(result, 7);
    callback.close_call_scope().expect("callback should close");
}

#[test]
fn call_scope_close_drains_borrowed_target_while_decoding() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("foreign-decode-drain")).expect("runtime should initialize");
    let entered_decode = Arc::new(Barrier::new(2));
    let release_decode = Arc::new(Barrier::new(2));
    let borrowed_decode_count = AtomicUsize::new(0);
    let entered_for_decode = Arc::clone(&entered_decode);
    let release_for_decode = Arc::clone(&release_decode);
    // SAFETY: private wrapper drains decoding before borrowed_decode_count ends.
    let callback = unsafe {
        foreign_callback_scoped_with_owner(
            CallbackOwnerState::new_call_scoped().expect("owner"),
            46,
            1,
            ForeignCallbackConcurrency::Parallel,
            |args| {
                borrowed_decode_count.fetch_add(1, Ordering::SeqCst);
                entered_for_decode.wait();
                release_for_decode.wait();
                crate::python::to_int(&args[0])
            },
            |_, value| Ok(value),
            crate::python::from_int,
        )
    }
    .expect("callback should create");
    let escaped = callback.object().clone();
    let invocation = std::thread::spawn(move || {
        let argument = crate::python::from_int(9).expect("argument should create");
        crate::python::call_object_owned(&escaped, &[argument], &[])
            .and_then(|result| crate::python::to_int(&result))
            .expect("callback should finish")
    });
    entered_decode.wait();

    let owner = callback.owner().clone();
    let close_finished = Arc::new(AtomicBool::new(false));
    let close_finished_in_thread = Arc::clone(&close_finished);
    let close = std::thread::spawn(move || {
        owner.close_call_scope().expect("close should drain decode");
        close_finished_in_thread.store(true, Ordering::SeqCst);
    });
    while callback.owner().status() == super::super::CallbackOwnerStatus::Open {
        std::thread::yield_now();
    }
    assert!(!close_finished.load(Ordering::SeqCst));
    assert_eq!(borrowed_decode_count.load(Ordering::SeqCst), 1);

    release_decode.wait();
    close.join().expect("close thread should join");
    drop(callback);
    assert_eq!(invocation.join().expect("invocation should join"), 9);
}
