use super::{PythonError, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;
use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

const LOOP_THREAD_NAME: &str = "sifr-python-asyncio";

static ASYNC_STATE: Mutex<AsyncRuntimeState> = Mutex::new(AsyncRuntimeState::new());
static ASYNC_STATE_CHANGED: Condvar = Condvar::new();
#[cfg(test)]
static FORCE_LOOP_SETUP_FAILURE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AsyncLifecycle {
    Disabled,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

struct AsyncRuntimeState {
    lifecycle: AsyncLifecycle,
    loop_object: Option<Py<PyAny>>,
    loop_thread: Option<JoinHandle<()>>,
    next_submission_id: u64,
    pending_submissions: usize,
    submissions: BTreeMap<u64, Py<PyAny>>,
    failure: Option<String>,
}

impl AsyncRuntimeState {
    const fn new() -> Self {
        Self {
            lifecycle: AsyncLifecycle::Disabled,
            loop_object: None,
            loop_thread: None,
            next_submission_id: 1,
            pending_submissions: 0,
            submissions: BTreeMap::new(),
            failure: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PythonAsyncRuntimeDiagnostics {
    pub running: bool,
    pub stopping: bool,
    pub loop_threads: usize,
    pub active_submissions: usize,
    pub pending_submissions: usize,
}

pub(super) fn start() -> Result<(), PythonRuntimeError> {
    {
        let mut state = lock_state()?;
        loop {
            match state.lifecycle {
                AsyncLifecycle::Running => return Ok(()),
                AsyncLifecycle::Starting => state = wait_for_change(state)?,
                AsyncLifecycle::Stopping => {
                    return Err(PythonRuntimeError::AsyncRuntimeStopping);
                }
                AsyncLifecycle::Disabled | AsyncLifecycle::Stopped | AsyncLifecycle::Failed => {
                    state.lifecycle = AsyncLifecycle::Starting;
                    state.failure = None;
                    break;
                }
            }
        }
    }

    let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
    let loop_thread = std::thread::Builder::new()
        .name(LOOP_THREAD_NAME.to_string())
        .spawn(move || run_loop_thread(ready_sender))
        .map_err(|error| fail_start(format!("loop thread could not start: {error}")))?;

    match ready_receiver.recv() {
        Ok(Ok(loop_object)) => {
            let mut state = lock_state()?;
            state.loop_object = Some(loop_object);
            state.loop_thread = Some(loop_thread);
            state.lifecycle = AsyncLifecycle::Running;
            ASYNC_STATE_CHANGED.notify_all();
            Ok(())
        }
        Ok(Err(message)) => {
            let _ignored = loop_thread.join();
            Err(fail_start(message))
        }
        Err(error) => {
            let _ignored = loop_thread.join();
            Err(fail_start(format!(
                "loop thread exited before publishing readiness: {error}"
            )))
        }
    }
}

pub(super) fn ensure_started() -> Result<(), PythonRuntimeError> {
    start()
}

pub(super) fn run_coroutine_blocking(
    py: Python<'_>,
    coroutine: &Py<PyAny>,
) -> Result<Py<PyAny>, PythonError> {
    let (submission_id, loop_object) = reserve_submission(py).map_err(PythonError::runtime)?;
    let scheduled = py
        .import("asyncio")
        .and_then(|asyncio| {
            asyncio.call_method1(
                "run_coroutine_threadsafe",
                (coroutine.bind(py), loop_object.bind(py)),
            )
        })
        .map(Bound::unbind)
        .map_err(|error| {
            release_pending_submission();
            PythonError::from_pyerr(py, error, "call", "owned asyncio submission")
        })?;
    if let Err(error) = register_submission(py, submission_id, &scheduled) {
        let _ignored = scheduled.bind(py).call_method0("cancel");
        release_pending_submission();
        return Err(PythonError::runtime(error));
    }

    let result = scheduled
        .bind(py)
        .call_method0("result")
        .map(Bound::unbind)
        .map_err(|error| PythonError::from_pyerr(py, error, "await", "owned asyncio coroutine"));
    finish_submission(submission_id);
    result
}

pub(super) fn shutdown() -> Result<(), PythonRuntimeError> {
    let (loop_object, loop_thread) = {
        let mut state = lock_state()?;
        match state.lifecycle {
            AsyncLifecycle::Disabled | AsyncLifecycle::Stopped => return Ok(()),
            AsyncLifecycle::Failed => state.lifecycle = AsyncLifecycle::Stopping,
            AsyncLifecycle::Starting => return Err(PythonRuntimeError::AsyncRuntimeNotRunning),
            AsyncLifecycle::Running => state.lifecycle = AsyncLifecycle::Stopping,
            AsyncLifecycle::Stopping => {}
        }
        while state.pending_submissions > 0 {
            state = wait_for_change(state)?;
        }
        (state.loop_object.take(), state.loop_thread.take())
    };

    cancel_registered_submissions()?;
    {
        let mut state = lock_state()?;
        while !state.submissions.is_empty() {
            state = wait_for_change(state)?;
        }
    }
    let stop_result = loop_object.map_or(Ok(()), |loop_object| {
        Python::try_attach(|py| {
            let stop = loop_object.bind(py).getattr("stop")?;
            loop_object
                .bind(py)
                .call_method1("call_soon_threadsafe", (stop,))?;
            Ok::<(), PyErr>(())
        })
        .ok_or(PythonRuntimeError::NotInitialized)?
        .map_err(|error| PythonRuntimeError::AsyncRuntimeFailed(error.to_string()))
    });
    let join_result = loop_thread.map_or(Ok(()), |loop_thread| {
        loop_thread
            .join()
            .map_err(|_| PythonRuntimeError::AsyncRuntimeFailed("loop thread panicked".to_string()))
    });
    let mut state = lock_state()?;
    state.lifecycle = AsyncLifecycle::Stopped;
    state.loop_object = None;
    let failure = state.failure.take();
    ASYNC_STATE_CHANGED.notify_all();
    drop(state);
    if let Some(message) = failure {
        return Err(PythonRuntimeError::AsyncRuntimeFailed(message));
    }
    stop_result.and(join_result)
}

pub fn async_runtime_diagnostics() -> Result<PythonAsyncRuntimeDiagnostics, PythonRuntimeError> {
    let state = lock_state()?;
    Ok(PythonAsyncRuntimeDiagnostics {
        running: state.lifecycle == AsyncLifecycle::Running,
        stopping: state.lifecycle == AsyncLifecycle::Stopping,
        loop_threads: usize::from(matches!(
            state.lifecycle,
            AsyncLifecycle::Starting | AsyncLifecycle::Running | AsyncLifecycle::Stopping
        )),
        active_submissions: state.submissions.len(),
        pending_submissions: state.pending_submissions,
    })
}

fn run_loop_thread(ready_sender: std::sync::mpsc::SyncSender<Result<Py<PyAny>, String>>) {
    let mut ready_sender = Some(ready_sender);
    let result = Python::try_attach(|py| -> Result<(), String> {
        #[cfg(test)]
        if FORCE_LOOP_SETUP_FAILURE.swap(false, Ordering::SeqCst) {
            return Err("forced loop setup failure".to_string());
        }
        let asyncio = py.import("asyncio").map_err(|error| error.to_string())?;
        let loop_object = asyncio
            .call_method0("new_event_loop")
            .map_err(|error| error.to_string())?
            .unbind();
        asyncio
            .call_method1("set_event_loop", (loop_object.bind(py),))
            .map_err(|error| error.to_string())?;
        ready_sender
            .take()
            .ok_or_else(|| "loop readiness sender is unavailable".to_string())?
            .send(Ok(loop_object.clone_ref(py)))
            .map_err(|error| error.to_string())?;
        loop_object
            .bind(py)
            .call_method0("run_forever")
            .map_err(|error| error.to_string())?;
        let shutdown_asyncgens = loop_object
            .bind(py)
            .call_method0("shutdown_asyncgens")
            .map_err(|error| error.to_string())?;
        loop_object
            .bind(py)
            .call_method1("run_until_complete", (shutdown_asyncgens,))
            .map_err(|error| error.to_string())?;
        loop_object
            .bind(py)
            .call_method0("close")
            .map_err(|error| error.to_string())?;
        Ok(())
    });
    let failure = match result {
        Some(Ok(())) => return,
        Some(Err(message)) => message,
        None => "CPython was unavailable on the loop thread".to_string(),
    };
    if let Some(sender) = ready_sender {
        let _ignored = sender.send(Err(failure));
    } else if let Ok(mut state) = ASYNC_STATE.lock() {
        state.failure = Some(failure);
        state.lifecycle = AsyncLifecycle::Failed;
        ASYNC_STATE_CHANGED.notify_all();
    }
}

fn reserve_submission(py: Python<'_>) -> Result<(u64, Py<PyAny>), PythonRuntimeError> {
    let mut state = lock_state()?;
    if state.lifecycle != AsyncLifecycle::Running {
        return Err(if state.lifecycle == AsyncLifecycle::Stopping {
            PythonRuntimeError::AsyncRuntimeStopping
        } else {
            PythonRuntimeError::AsyncRuntimeNotRunning
        });
    }
    let loop_object = state
        .loop_object
        .as_ref()
        .ok_or(PythonRuntimeError::AsyncRuntimeNotRunning)?
        .clone_ref(py);
    let submission_id = state.next_submission_id;
    state.next_submission_id = state.next_submission_id.saturating_add(1);
    state.pending_submissions = state.pending_submissions.saturating_add(1);
    Ok((submission_id, loop_object))
}

fn register_submission(
    py: Python<'_>,
    submission_id: u64,
    future: &Py<PyAny>,
) -> Result<(), PythonRuntimeError> {
    let mut state = lock_state()?;
    state.pending_submissions = state.pending_submissions.saturating_sub(1);
    state
        .submissions
        .insert(submission_id, future.clone_ref(py));
    ASYNC_STATE_CHANGED.notify_all();
    Ok(())
}

fn release_pending_submission() {
    if let Ok(mut state) = ASYNC_STATE.lock() {
        state.pending_submissions = state.pending_submissions.saturating_sub(1);
        ASYNC_STATE_CHANGED.notify_all();
    }
}

fn finish_submission(submission_id: u64) {
    if let Ok(mut state) = ASYNC_STATE.lock() {
        state.submissions.remove(&submission_id);
        ASYNC_STATE_CHANGED.notify_all();
    }
}

fn cancel_registered_submissions() -> Result<(), PythonRuntimeError> {
    Python::try_attach(|py| {
        let futures = {
            let state = lock_state()?;
            state
                .submissions
                .values()
                .map(|future| future.clone_ref(py))
                .collect::<Vec<_>>()
        };
        for future in futures {
            future
                .bind(py)
                .call_method0("cancel")
                .map_err(|error| PythonRuntimeError::AsyncRuntimeFailed(error.to_string()))?;
        }
        Ok(())
    })
    .ok_or(PythonRuntimeError::NotInitialized)?
}

fn fail_start(message: String) -> PythonRuntimeError {
    if let Ok(mut state) = ASYNC_STATE.lock() {
        state.lifecycle = AsyncLifecycle::Failed;
        state.failure = Some(message.clone());
        state.loop_object = None;
        state.loop_thread = None;
        ASYNC_STATE_CHANGED.notify_all();
    }
    PythonRuntimeError::AsyncRuntimeFailed(message)
}

fn lock_state() -> Result<MutexGuard<'static, AsyncRuntimeState>, PythonRuntimeError> {
    ASYNC_STATE
        .lock()
        .map_err(|_| PythonRuntimeError::StateUnavailable)
}

fn wait_for_change(
    state: MutexGuard<'static, AsyncRuntimeState>,
) -> Result<MutexGuard<'static, AsyncRuntimeState>, PythonRuntimeError> {
    ASYNC_STATE_CHANGED
        .wait(state)
        .map_err(|_| PythonRuntimeError::StateUnavailable)
}

#[cfg(test)]
pub(super) fn reset_for_tests() {
    if let Ok(mut state) = ASYNC_STATE.lock() {
        *state = AsyncRuntimeState::new();
        ASYNC_STATE_CHANGED.notify_all();
    }
    FORCE_LOOP_SETUP_FAILURE.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
    };

    #[test]
    fn loop_setup_failure_is_joined_and_leaves_no_live_thread() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("async-loop-failure"))
            .expect("CPython should initialize without the loop");
        FORCE_LOOP_SETUP_FAILURE.store(true, Ordering::SeqCst);

        let error = start().expect_err("forced loop setup should fail");

        assert!(matches!(error, PythonRuntimeError::AsyncRuntimeFailed(_)));
        assert_eq!(
            async_runtime_diagnostics().expect("diagnostics should remain available"),
            PythonAsyncRuntimeDiagnostics::default()
        );
        assert!(matches!(
            shutdown(),
            Err(PythonRuntimeError::AsyncRuntimeFailed(_))
        ));
        assert_eq!(
            async_runtime_diagnostics().expect("failed runtime should normalize to stopped"),
            PythonAsyncRuntimeDiagnostics::default()
        );
    }
}
