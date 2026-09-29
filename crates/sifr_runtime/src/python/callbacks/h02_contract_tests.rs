use super::*;
use crate::python::{
    call_object_owned, from_int, initialize_runtime, reset_runtime_state_for_tests, test_config,
    test_guard, to_int,
};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

struct ReleaseProbe(Arc<AtomicUsize>);
impl Drop for ReleaseProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn borrowed_callback_lifetime_and_thread_owner() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("h02-borrowed-thread")).expect("initialize");
    let total = Cell::new(0);
    let owner = CallbackOwnerState::new_call_scoped().expect("owner");
    // SAFETY: the private callback drops before total; it is never forgotten or
    // moved into a handler. Only its guarded Python shell escapes.
    let callback = unsafe {
        current_callback_scoped_with_owner(
            owner.clone(),
            1,
            1,
            |args| to_int(&args[0]),
            |_, value| {
                total.set(total.get() + 1);
                Ok(value)
            },
            from_int,
        )
    }
    .expect("borrowed callback");
    let escaped = callback.object().clone();
    let result = call_object_owned(callback.object(), &[from_int(41).expect("argument")], &[])
        .and_then(|value| to_int(&value))
        .expect("invoke");
    assert_eq!(result, 41);
    assert_eq!(total.get(), 1);
    let foreign_shell = escaped.clone();
    let wrong_thread = std::thread::spawn(move || {
        call_object_owned(&foreign_shell, &[from_int(1).expect("argument")], &[])
            .expect_err("reject foreign thread")
    })
    .join()
    .expect("join");
    assert_eq!(wrong_thread.exception_type, "SifrCallbackThreadError");
    drop(callback);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    let after = call_object_owned(&escaped, &[from_int(1).expect("argument")], &[])
        .expect_err("reject escaped shell");
    assert_eq!(after.exception_type, "SifrCallbackClosedError");
    assert_eq!(total.get(), 1);
}

struct BorrowedPending<'a> {
    polls: &'a AtomicUsize,
    releases: &'a AtomicUsize,
    started: Arc<tokio::sync::Notify>,
}
impl Future for BorrowedPending<'_> {
    type Output = Result<crate::SifrInt, CallbackExecutionError>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        Poll::Pending
    }
}
impl Drop for BorrowedPending<'_> {
    fn drop(&mut self) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn close_cancel_reentrancy_releases_once() {
    let _guard = test_guard();
    super::asyncio_tests::initialize_callback_runtime("h02-drop-cancel");
    let polls = AtomicUsize::new(0);
    let futures_released = AtomicUsize::new(0);
    let captures_released = Arc::new(AtomicUsize::new(0));
    let capture = ReleaseProbe(Arc::clone(&captures_released));
    let started = Arc::new(tokio::sync::Notify::new());
    let owner = CallbackOwnerState::new_call_scoped().expect("owner");
    let reentrant = owner.clone();
    let borrowed_polls = &polls;
    let borrowed_releases = &futures_released;
    // SAFETY: cancellation drops the private wrapper before these borrowed
    // counters expire; the handler cannot access or forget the wrapper.
    let callback = unsafe {
        asyncio_callback_scoped_with_owner(
            owner.clone(),
            2,
            1,
            AsyncioCallbackConcurrency::Parallel,
            |args| to_int(&args[0]),
            move |_, _, _| {
                let _capture = &capture;
                let error = reentrant
                    .close_call_scope()
                    .expect_err("reject close during poll");
                assert_eq!(error.exception_type, "SifrCallbackCloseReentrancyError");
                BorrowedPending {
                    polls: borrowed_polls,
                    releases: borrowed_releases,
                    started: Arc::clone(&started),
                }
            },
            from_int,
        )
    }
    .expect("callback");
    let shell = callback.object().clone();
    let request = super::asyncio_tests::function_request(
        "start_and_fail",
        vec![crate::python::async_from_object(&shell).expect("transport")],
    );
    crate::python::submit_async_declaration(request, None)
        .await
        .expect_err("setup fixture failure");
    while polls.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    let mut closer = Box::pin(callback.close_call_scope());
    let mut context = Context::from_waker(Waker::noop());
    assert!(closer.as_mut().poll(&mut context).is_pending());
    drop(closer);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closing);
    // Generated async-frame cancellation: Drop must revoke without polling the
    // worker executor again, including a current-thread executor.
    drop(callback);
    assert_eq!(futures_released.load(Ordering::SeqCst), 1);
    assert_eq!(captures_released.load(Ordering::SeqCst), 1);
    let polls_at_drop = polls.load(Ordering::SeqCst);
    owner
        .close_call_scope_async()
        .await
        .expect("successor drain");
    tokio::task::yield_now().await;
    assert_eq!(polls.load(Ordering::SeqCst), polls_at_drop);
    assert_eq!(owner.active_calls(), 0);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(futures_released.load(Ordering::SeqCst), 1);
    let error = call_object_owned(&shell, &[from_int(1).expect("argument")], &[])
        .expect_err("escaped shell rejects before touching dead captures");
    assert_eq!(error.exception_type, "SifrCallbackClosedError");
    super::asyncio_invocation::prove_queued_output_revocation();
    // Rejected retained publication must leave the wrapper owning its target;
    // an owner may already have admitted setup when close wins publication.
    let failed_publication_releases = Arc::new(AtomicUsize::new(0));
    let decode_capture = ReleaseProbe(Arc::clone(&failed_publication_releases));
    let retained_owner = CallbackOwnerState::new_retained(|| Ok(())).expect("retained owner");
    let retained = asyncio_callback_with_owner(
        retained_owner.clone(),
        3,
        1,
        AsyncioCallbackConcurrency::Parallel,
        move |args| {
            let _capture = &decode_capture;
            to_int(&args[0])
        },
        |_, value, _| async move { Ok(value) },
        from_int,
    )
    .expect("owned callback");
    let lease = retained_owner.accept(3, false).expect("admitted setup");
    drop(retained_owner.begin_owner_unregister().expect("unregister"));
    let mut closing = Box::pin(retained_owner.close_after_owner_unregister_async());
    assert!(closing.as_mut().poll(&mut context).is_pending());
    assert!(retained.retain_in_owner().is_err());
    assert_eq!(failed_publication_releases.load(Ordering::SeqCst), 0);
    drop(closing);
    drop(retained);
    assert_eq!(failed_publication_releases.load(Ordering::SeqCst), 1);
    drop(lease);
    retained_owner
        .close_after_owner_unregister_async()
        .await
        .expect("drain rejected publication");
    assert_eq!(retained_owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(failed_publication_releases.load(Ordering::SeqCst), 1);
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_closer_handoff_releases_once() {
    let releases = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&releases);
    let owner = CallbackOwnerState::new_call_scoped_with_release(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    })
    .expect("owner");
    let invocation = owner.accept(1, false).expect("admit invocation");
    let mut closer = Box::pin(owner.close_call_scope_async());
    let mut context = Context::from_waker(Waker::noop());
    assert!(closer.as_mut().poll(&mut context).is_pending());
    assert_eq!(owner.status(), CallbackOwnerStatus::Closing);
    drop(closer);
    assert!(owner.accept(2, false).is_err());
    let mut successor = Box::pin(owner.close_call_scope_async());
    assert!(successor.as_mut().poll(&mut context).is_pending());
    assert_eq!(releases.load(Ordering::SeqCst), 0);
    drop(invocation);
    successor.await.expect("successor finishes");
    owner
        .close_call_scope_async()
        .await
        .expect("idempotent close");
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(owner.active_calls(), 0);
    assert!(owner.captures_released());
    assert_eq!(releases.load(Ordering::SeqCst), 1);
}

#[test]
fn borrowed_callback_forget_and_escape_is_rejected_or_drained() {
    // Compile the actual library API, then reject both safe borrowed construction
    // and access to the compiler-only unsafe constructors. The fixture never
    // executes a callback after releasing a borrowed capture.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = std::env::current_exe().expect("test executable");
    let target = executable
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .expect("target directory");
    let build = std::process::Command::new(env!("CARGO"))
        .current_dir(manifest)
        .args([
            "build",
            "--offline",
            "--locked",
            "-p",
            "sifr_runtime",
            "--features",
            "python",
            "--lib",
            "--message-format=json",
        ])
        .arg("--target-dir")
        .arg(target)
        .output()
        .expect("build library for compile-fail proof");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    // Cargo's stable unhashed output belongs to this exact build, avoiding stale
    // artifacts from another feature selection in deps.
    let library = target.join("debug/libsifr_runtime.rlib");
    assert!(library.is_file());
    let directory =
        std::env::temp_dir().join(format!("sifr-h02d0-compile-fail-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("own fixture directory");
    let source = directory.join("borrowed_escape.rs");
    std::fs::write(&source, include_str!("h02_borrowed_escape_fixture.rs.txt"))
        .expect("write fixture");
    let checked = std::process::Command::new("rustc")
        .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
        .arg("--extern")
        .arg(format!("sifr_runtime={}", library.display()))
        .arg("-L")
        .arg(format!(
            "dependency={}",
            target.join("debug/deps").display()
        ))
        .arg("--out-dir")
        .arg(&directory)
        .arg(&source)
        .output()
        .expect("compile fixture");
    let errors = String::from_utf8_lossy(&checked.stderr);
    assert!(!checked.status.success(), "borrowed construction must fail");
    assert!(
        errors.contains("E0133"),
        "unsafe constructor must require authority: {errors}"
    );
    assert!(
        errors.contains("E0373") || errors.contains("E0597"),
        "safe constructor must reject borrowed capture: {errors}"
    );
    for name in [
        "current_callback_scoped_with_owner",
        "foreign_callback_scoped_with_owner",
        "asyncio_callback_scoped_with_owner",
    ] {
        assert!(
            errors.contains(name),
            "missing constructor rejection: {errors}"
        );
    }
    std::fs::remove_dir_all(directory).expect("remove owned fixture directory");
}
