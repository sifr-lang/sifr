use super::*;
use crate::python::{from_int, test_guard, to_int};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Pending<'a> {
    polls: &'a AtomicUsize,
    dropped: &'a AtomicUsize,
    started: Arc<tokio::sync::Notify>,
}
impl std::future::Future for Pending<'_> {
    type Output = Result<crate::SifrInt, CallbackExecutionError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        std::task::Poll::Pending
    }
}
impl Drop for Pending<'_> {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

struct Capture(Arc<AtomicUsize>);
impl Drop for Capture {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

// Run in a separate test process: the historical deadlock blocks the current
// executor thread, so a Tokio timeout in that same thread cannot observe it.
#[test]
#[allow(unsafe_code)]
fn mixed_foreign_asyncio_cancellation_does_not_block_executor() {
    const CHILD: &str = "SIFR_H02D1_MIXED_CALLBACK_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["python::callbacks::h02_core_tests::mixed_foreign_asyncio_cancellation_does_not_block_executor", "--exact", "--nocapture"])
            .env(CHILD, "1").spawn().expect("child");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(status) = child.try_wait().expect("child status") {
                assert!(status.success(), "mixed callback child failed: {status}");
                return;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().expect("stop deadlocked child");
                let _ = child.wait();
                panic!("mixed wrapper teardown blocked its current-thread executor");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    let _guard = test_guard();
    super::asyncio_tests::initialize_callback_runtime("h02-mixed-cancel");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("executor")
        .block_on(async {
            for foreign_first in [true, false] {
                let released = Arc::new(AtomicUsize::new(0));
                let foreign_released = Arc::new(AtomicUsize::new(0));
                let async_released = Arc::new(AtomicUsize::new(0));
                let owner_release = Arc::clone(&released);
                let owner = CallbackOwnerState::new_call_scoped_with_release(move || {
                    owner_release.fetch_add(1, Ordering::SeqCst);
                })
                .expect("owner");
                let foreign_capture = Capture(Arc::clone(&foreign_released));
                let foreign = unsafe {
                    foreign_callback_scoped_with_owner(
                        owner.clone(),
                        100,
                        1,
                        ForeignCallbackConcurrency::Parallel,
                        |args| to_int(&args[0]),
                        move |_, value| {
                            let _capture = &foreign_capture;
                            Ok(value)
                        },
                        from_int,
                    )
                }
                .expect("foreign");
                let async_capture = Capture(Arc::clone(&async_released));
                let started = Arc::new(tokio::sync::Notify::new());
                let seen = Arc::clone(&started);
                let polls = AtomicUsize::new(0);
                let pending_drops = AtomicUsize::new(0);
                let poll_ref = &polls;
                let drop_ref = &pending_drops;
                // SAFETY: wrappers stay private, drop before both borrowed counters,
                // and neither handler can forget or reentrantly drop its wrapper.
                let asynchronous = unsafe {
                    asyncio_callback_scoped_with_owner(
                        owner.clone(),
                        101,
                        1,
                        AsyncioCallbackConcurrency::Parallel,
                        |args| to_int(&args[0]),
                        move |_, _, _| {
                            let _capture = &async_capture;
                            Pending {
                                polls: poll_ref,
                                dropped: drop_ref,
                                started: Arc::clone(&seen),
                            }
                        },
                        from_int,
                    )
                }
                .expect("asyncio");
                let shell = foreign.object().clone();
                let request = super::asyncio_tests::function_request(
                    "start_and_fail",
                    vec![
                        crate::python::async_from_object(asynchronous.object()).expect("transport"),
                    ],
                );
                crate::python::submit_async_declaration(request, None)
                    .await
                    .expect_err("fixture failure");
                started.notified().await;
                assert!(owner.active_calls() > 0);
                if foreign_first {
                    drop(foreign);
                    drop(asynchronous);
                } else {
                    drop(asynchronous);
                    drop(foreign);
                }
                let polls_at_drop = polls.load(Ordering::SeqCst);
                assert_eq!(pending_drops.load(Ordering::SeqCst), 1);
                assert_eq!(foreign_released.load(Ordering::SeqCst), 1);
                assert_eq!(async_released.load(Ordering::SeqCst), 1);
                tokio::time::timeout(std::time::Duration::from_secs(5), async {
                    while owner.status() != CallbackOwnerStatus::Closed {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("owned drain completes without another closer");
                assert_eq!(owner.active_calls(), 0);
                assert_eq!(polls.load(Ordering::SeqCst), polls_at_drop);
                assert_eq!(pending_drops.load(Ordering::SeqCst), 1);
                assert_eq!(released.load(Ordering::SeqCst), 1);
                let error = crate::python::call_object_owned(
                    &shell,
                    &[from_int(1).expect("argument")],
                    &[],
                )
                .expect_err("escaped shell closed");
                assert_eq!(error.exception_type, "SifrCallbackClosedError");
            }
        });
    crate::python::reset_runtime_state_for_tests();
}

#[test]
fn capture_release_reentrant_close_returns_error() {
    let nested = Arc::new(std::sync::Mutex::new(None::<CallbackOwnerState>));
    let release_owner = Arc::clone(&nested);
    let owner = CallbackOwnerState::new_call_scoped_with_release(move || {
        let owner = release_owner
            .lock()
            .expect("release slot")
            .take()
            .expect("owner");
        let error = owner
            .close_call_scope()
            .expect_err("reentrant release close");
        assert_eq!(error.exception_type, "SifrCallbackCloseReentrancyError");
    })
    .expect("owner");
    *nested.lock().expect("slot") = Some(owner.clone());
    owner.close_call_scope().expect("outer close");
    assert!(owner.captures_released());
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
}
