use super::async_terminal::{
    terminal_error_to_python, PythonTerminal, PythonTerminalError, PythonTerminalOutcome,
    PythonTerminalValue,
};
use super::{PythonError, PythonRuntimeError};
use crate::cancellation::{CancellationCarrier, CancellationClaimError, CancellationHook};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyCFunction, PyDict, PyTuple};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

const LOOP_THREAD_NAME: &str = "sifr-python-asyncio";

static ASYNC_STATE: Mutex<AsyncRuntimeState> = Mutex::new(AsyncRuntimeState::new());
static ASYNC_STATE_CHANGED: Condvar = Condvar::new();
#[cfg(test)]
static FORCE_LOOP_SETUP_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FORCE_SUBMISSION_QUEUE_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FORCE_TERMINAL_CALLBACK_PANIC: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FORCE_SUBMISSION_CANCEL_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static SHUTDOWN_PHASE_TRACE: Mutex<Vec<ShutdownPhase>> = Mutex::new(Vec::new());

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
    pending_submissions: BTreeMap<u64, PythonTerminal>,
    submissions: BTreeMap<u64, RegisteredSubmission>,
    failure: Option<String>,
}

struct RegisteredSubmission {
    loop_object: Py<PyAny>,
    exact_task: Py<PyAny>,
    terminal: PythonTerminal,
}

#[derive(Default)]
struct SubmissionCancellationState {
    requested: bool,
    submission_id: Option<u64>,
}

#[derive(Default)]
pub(super) struct SubmissionCancellationBridge {
    state: Mutex<SubmissionCancellationState>,
}

impl AsyncRuntimeState {
    const fn new() -> Self {
        Self {
            lifecycle: AsyncLifecycle::Disabled,
            loop_object: None,
            loop_thread: None,
            next_submission_id: 1,
            pending_submissions: BTreeMap::new(),
            submissions: BTreeMap::new(),
            failure: None,
        }
    }
}

impl SubmissionCancellationBridge {
    pub(super) fn publish(&self, submission_id: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return true;
        };
        state.submission_id = Some(submission_id);
        state.requested
    }

    fn request(&self) {
        let submission_id = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if state.requested {
                return;
            }
            state.requested = true;
            state.submission_id
        };
        if let Some(submission_id) = submission_id {
            let _ignored = cancel_submission(submission_id);
        }
    }

    pub(super) fn was_requested(&self) -> bool {
        self.state.lock().map_or(true, |state| state.requested)
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

pub(super) fn is_owned_loop(
    py: Python<'_>,
    candidate: &Bound<'_, PyAny>,
) -> Result<bool, PythonRuntimeError> {
    let state = lock_state()?;
    let loop_object = state
        .loop_object
        .as_ref()
        .ok_or(PythonRuntimeError::AsyncRuntimeNotRunning)?;
    Ok(candidate.is(loop_object.bind(py)))
}

pub(super) fn run_coroutine_blocking(
    py: Python<'_>,
    coroutine: &Py<PyAny>,
) -> Result<Py<PyAny>, PythonError> {
    let terminal = submit_coroutine(py, coroutine, None)?;
    // `run_coroutine_blocking` is classified as blocking_io and must be explicitly
    // offloaded by async Sifr code. Releasing the GIL here lets the loop thread run.
    let outcome = super::detach(py, || terminal.wait());
    match outcome.map_err(|error| terminal_error_to_python(py, error, "owned asyncio coroutine"))? {
        PythonTerminalValue::Raw(value) => Ok(value),
        PythonTerminalValue::Typed(_) => Err(PythonError::runtime(
            PythonRuntimeError::AsyncRuntimeFailed(
                "raw asyncio submission produced a typed terminal value".to_string(),
            ),
        )),
        PythonTerminalValue::ExitDecision(_) => Err(PythonError::runtime(
            PythonRuntimeError::AsyncRuntimeFailed(
                "raw asyncio submission produced an exit-decision terminal value".to_string(),
            ),
        )),
    }
}

pub(super) fn submit_coroutine(
    py: Python<'_>,
    coroutine: &Py<PyAny>,
    carrier: Option<&CancellationCarrier>,
) -> Result<PythonTerminal, PythonError> {
    let (terminal, cancellation) = terminal_for_submission(carrier)
        .map_err(|error| terminal_error_to_python(py, error, "owned asyncio submission"))?;
    let (submission_id, loop_object) =
        reserve_submission(py, &terminal).map_err(PythonError::runtime)?;
    let done_callback = build_done_callback(py, submission_id, &terminal).map_err(|error| {
        release_pending_submission(submission_id);
        PythonError::from_pyerr(py, error, "call", "owned asyncio completion callback")
    })?;
    let setup_callback = build_setup_callback(
        py,
        submission_id,
        coroutine,
        &loop_object,
        &done_callback,
        &terminal,
        &cancellation,
    )
    .map_err(|error| {
        release_pending_submission(submission_id);
        PythonError::from_pyerr(py, error, "call", "owned asyncio setup callback")
    })?;
    #[cfg(test)]
    if FORCE_SUBMISSION_QUEUE_FAILURE.swap(false, Ordering::SeqCst) {
        release_pending_submission(submission_id);
        return Err(PythonError::runtime(
            PythonRuntimeError::AsyncRuntimeFailed(
                "forced owned asyncio submission queue failure".to_string(),
            ),
        ));
    }
    loop_object
        .bind(py)
        .call_method1("call_soon_threadsafe", (setup_callback,))
        .map_err(|error| {
            release_pending_submission(submission_id);
            PythonError::from_pyerr(py, error, "call", "owned asyncio submission")
        })?;
    Ok(terminal)
}

pub(super) fn terminal_for_submission(
    carrier: Option<&CancellationCarrier>,
) -> Result<(PythonTerminal, Arc<SubmissionCancellationBridge>), PythonTerminalError> {
    let cancellation = Arc::new(SubmissionCancellationBridge::default());
    let cancellation_claim = if let Some(carrier) = carrier {
        match carrier.claim(cancellation_hook(&cancellation)) {
            Ok(claim) => Some(claim),
            Err(CancellationClaimError::CancelledBeforeClaim) => {
                return Err(PythonTerminalError::ActiveCancellation);
            }
            Err(CancellationClaimError::AlreadyClaimed) => {
                return Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::AsyncCancellationAlreadyClaimed,
                ));
            }
            Err(CancellationClaimError::StateUnavailable) => {
                return Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::StateUnavailable,
                ));
            }
        }
    } else {
        None
    };

    Ok((
        PythonTerminal::with_cancellation_claim(cancellation_claim),
        cancellation,
    ))
}

fn build_done_callback(
    py: Python<'_>,
    submission_id: u64,
    terminal: &PythonTerminal,
) -> PyResult<Py<PyAny>> {
    let terminal = terminal.clone();
    PyCFunction::new_closure(
        py,
        Some(c"__sifr_python_task_done"),
        None,
        move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<()> {
            let outcome = catch_unwind(AssertUnwindSafe(|| -> PythonTerminalOutcome {
                #[cfg(test)]
                assert!(
                    !FORCE_TERMINAL_CALLBACK_PANIC.swap(false, Ordering::SeqCst),
                    "forced terminal callback panic"
                );
                let task = args.get_item(0).map_err(PythonTerminalError::Python)?;
                task.call_method0("result")
                    .map(|value| PythonTerminalValue::Raw(value.unbind()))
                    .map_err(PythonTerminalError::Python)
            }))
            .unwrap_or_else(|_| {
                Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::AsyncRuntimeFailed(
                        "owned asyncio terminal callback panicked".to_string(),
                    ),
                ))
            });
            let outcome = match finish_submission(submission_id) {
                Ok(()) => outcome,
                Err(error) => Err(PythonTerminalError::Runtime(error)),
            };
            let _completed = terminal.complete(outcome);
            Ok(())
        },
    )
    .map(|callback| callback.into_any().unbind())
}

#[allow(clippy::too_many_arguments)]
fn build_setup_callback(
    py: Python<'_>,
    submission_id: u64,
    coroutine: &Py<PyAny>,
    loop_object: &Py<PyAny>,
    done_callback: &Py<PyAny>,
    terminal: &PythonTerminal,
    cancellation: &Arc<SubmissionCancellationBridge>,
) -> PyResult<Py<PyAny>> {
    let coroutine = coroutine.clone_ref(py);
    let loop_object = loop_object.clone_ref(py);
    let done_callback = done_callback.clone_ref(py);
    let terminal = terminal.clone();
    let cancellation = Arc::clone(cancellation);
    PyCFunction::new_closure(
        py,
        Some(c"__sifr_python_task_setup"),
        None,
        move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<()> {
            let py = args.py();
            let mut registered = false;
            let mut exact_task: Option<Py<PyAny>> = None;
            let setup = catch_unwind(AssertUnwindSafe(|| -> Result<(), PythonTerminalError> {
                let task = loop_object
                    .bind(py)
                    .call_method1("create_task", (coroutine.bind(py),))
                    .map_err(PythonTerminalError::Python)?;
                exact_task = Some(task.clone().unbind());
                task.call_method1("add_done_callback", (done_callback.bind(py),))
                    .map_err(PythonTerminalError::Python)?;
                register_submission(py, submission_id, &loop_object, &task.clone().unbind())
                    .map_err(PythonTerminalError::Runtime)?;
                registered = true;
                if cancellation.publish(submission_id) {
                    task.call_method0("cancel")
                        .map_err(PythonTerminalError::Python)?;
                }
                Ok(())
            }))
            .unwrap_or_else(|_| {
                Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::AsyncRuntimeFailed(
                        "owned asyncio setup callback panicked".to_string(),
                    ),
                ))
            });

            if let Err(error) = setup {
                if registered {
                    let _ignored = finish_submission(submission_id);
                } else {
                    release_pending_submission(submission_id);
                }
                if let Some(task) = exact_task {
                    let _ignored = task.bind(py).call_method0("cancel");
                }
                let _completed = terminal.complete(Err(error));
            }
            Ok(())
        },
    )
    .map(|callback| callback.into_any().unbind())
}

fn cancellation_hook(bridge: &Arc<SubmissionCancellationBridge>) -> CancellationHook {
    let bridge = Arc::clone(bridge);
    Arc::new(move || bridge.request())
}

pub(super) fn shutdown() -> Result<(), PythonRuntimeError> {
    let resources = {
        let mut state = lock_state()?;
        match state.lifecycle {
            AsyncLifecycle::Disabled | AsyncLifecycle::Stopped => return Ok(()),
            AsyncLifecycle::Failed => state.lifecycle = AsyncLifecycle::Stopping,
            AsyncLifecycle::Starting => return Err(PythonRuntimeError::AsyncRuntimeNotRunning),
            AsyncLifecycle::Running => state.lifecycle = AsyncLifecycle::Stopping,
            AsyncLifecycle::Stopping => {}
        }
        ASYNC_STATE_CHANGED.notify_all();
        while !state.pending_submissions.is_empty() && state.lifecycle != AsyncLifecycle::Failed {
            state = wait_for_change(state)?;
        }
        ShutdownResources {
            loop_object: state.loop_object.take(),
            loop_thread: state.loop_thread.take(),
            failure: state.failure.take(),
        }
    };
    record_shutdown_phase(ShutdownPhase::AdmissionsStopped);

    let mut first_error = resources
        .failure
        .map(PythonRuntimeError::AsyncRuntimeFailed);
    record_shutdown_phase(ShutdownPhase::CallbackShutdown);
    retain_first_error(
        &mut first_error,
        super::shutdown_hooks::shutdown_registered_callbacks(),
    );
    record_shutdown_phase(ShutdownPhase::AsyncCleanup);
    retain_first_error(
        &mut first_error,
        super::shutdown_hooks::run_registered_async_cleanup(),
    );

    record_shutdown_phase(ShutdownPhase::SubmissionCancellation);
    if let Err(error) = cancel_registered_submissions() {
        drain_outstanding_submissions("owned asyncio task cancellation could not be queued");
        retain_first_error(&mut first_error, Err(error));
    } else if let Err(error) = wait_for_submissions_to_drain() {
        drain_outstanding_submissions("owned asyncio loop failed while draining submissions");
        retain_first_error(&mut first_error, Err(error));
    }

    record_shutdown_phase(ShutdownPhase::LoopStop);
    let stop_result = resources.loop_object.map_or(Ok(()), |loop_object| {
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
    retain_first_error(&mut first_error, stop_result);

    record_shutdown_phase(ShutdownPhase::LoopJoin);
    let join_result = resources.loop_thread.map_or(Ok(()), |loop_thread| {
        loop_thread
            .join()
            .map_err(|_| PythonRuntimeError::AsyncRuntimeFailed("loop thread panicked".to_string()))
    });
    retain_first_error(&mut first_error, join_result);

    let runtime_failure = {
        let mut state = lock_state()?;
        state.lifecycle = AsyncLifecycle::Stopped;
        state.loop_object = None;
        state.loop_thread = None;
        let failure = state.failure.take();
        ASYNC_STATE_CHANGED.notify_all();
        failure
    };
    if let Some(message) = runtime_failure {
        retain_first_error(
            &mut first_error,
            Err(PythonRuntimeError::AsyncRuntimeFailed(message)),
        );
    }
    first_error.map_or(Ok(()), Err)
}

struct ShutdownResources {
    loop_object: Option<Py<PyAny>>,
    loop_thread: Option<JoinHandle<()>>,
    failure: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShutdownPhase {
    AdmissionsStopped,
    CallbackShutdown,
    AsyncCleanup,
    SubmissionCancellation,
    LoopStop,
    LoopJoin,
}

fn retain_first_error(
    first_error: &mut Option<PythonRuntimeError>,
    result: Result<(), PythonRuntimeError>,
) {
    if first_error.is_none() {
        if let Err(error) = result {
            *first_error = Some(error);
        }
    }
}

fn record_shutdown_phase(phase: ShutdownPhase) {
    #[cfg(test)]
    if let Ok(mut phases) = SHUTDOWN_PHASE_TRACE.lock() {
        phases.push(phase);
    }
    let _ = phase;
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
        pending_submissions: state.pending_submissions.len(),
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
    } else {
        fail_live_runtime(&failure);
    }
}

pub(super) fn reserve_submission(
    py: Python<'_>,
    terminal: &PythonTerminal,
) -> Result<(u64, Py<PyAny>), PythonRuntimeError> {
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
    state
        .pending_submissions
        .insert(submission_id, terminal.clone());
    Ok((submission_id, loop_object))
}

pub(super) fn register_submission(
    py: Python<'_>,
    submission_id: u64,
    loop_object: &Py<PyAny>,
    exact_task: &Py<PyAny>,
) -> Result<(), PythonRuntimeError> {
    let mut state = lock_state()?;
    let terminal = state
        .pending_submissions
        .remove(&submission_id)
        .ok_or_else(|| {
            PythonRuntimeError::AsyncRuntimeFailed(format!(
                "owned asyncio submission {submission_id} lost its pending terminal"
            ))
        })?;
    state.submissions.insert(
        submission_id,
        RegisteredSubmission {
            loop_object: loop_object.clone_ref(py),
            exact_task: exact_task.clone_ref(py),
            terminal,
        },
    );
    ASYNC_STATE_CHANGED.notify_all();
    Ok(())
}

pub(super) fn release_pending_submission(submission_id: u64) {
    let terminal = {
        let Ok(mut state) = ASYNC_STATE.lock() else {
            return;
        };
        let terminal = state.pending_submissions.remove(&submission_id);
        ASYNC_STATE_CHANGED.notify_all();
        terminal
    };
    drop(terminal);
}

pub(super) fn finish_submission(submission_id: u64) -> Result<(), PythonRuntimeError> {
    let removed = {
        let mut state = lock_state()?;
        let removed = state.submissions.remove(&submission_id);
        ASYNC_STATE_CHANGED.notify_all();
        removed
    };
    drop(removed);
    Ok(())
}

fn wait_for_submissions_to_drain() -> Result<(), PythonRuntimeError> {
    let mut state = lock_state()?;
    loop {
        if state.submissions.is_empty() {
            return Ok(());
        }
        if state.lifecycle == AsyncLifecycle::Failed {
            return Err(PythonRuntimeError::AsyncRuntimeFailed(
                state
                    .failure
                    .clone()
                    .unwrap_or_else(|| "owned asyncio loop failed while draining".to_string()),
            ));
        }
        state = wait_for_change(state)?;
    }
}

fn drain_outstanding_submissions(message: &str) {
    let (pending, active) = {
        let Ok(mut state) = ASYNC_STATE.lock() else {
            return;
        };
        let pending = std::mem::take(&mut state.pending_submissions)
            .into_values()
            .collect::<Vec<_>>();
        let active = std::mem::take(&mut state.submissions)
            .into_values()
            .collect::<Vec<_>>();
        ASYNC_STATE_CHANGED.notify_all();
        (pending, active)
    };
    complete_drained_submissions(pending, active, message);
}

fn fail_live_runtime(message: &str) {
    let (pending, active) = {
        let Ok(mut state) = ASYNC_STATE.lock() else {
            return;
        };
        state.failure = Some(message.to_string());
        state.lifecycle = AsyncLifecycle::Failed;
        let pending = std::mem::take(&mut state.pending_submissions)
            .into_values()
            .collect::<Vec<_>>();
        let active = std::mem::take(&mut state.submissions)
            .into_values()
            .collect::<Vec<_>>();
        ASYNC_STATE_CHANGED.notify_all();
        (pending, active)
    };
    complete_drained_submissions(pending, active, message);
}

fn complete_drained_submissions(
    pending: Vec<PythonTerminal>,
    active: Vec<RegisteredSubmission>,
    message: &str,
) {
    for terminal in pending {
        let _completed = terminal.complete(Err(PythonTerminalError::Runtime(
            PythonRuntimeError::AsyncRuntimeFailed(message.to_string()),
        )));
    }
    for submission in &active {
        let _completed = submission
            .terminal
            .complete(Err(PythonTerminalError::Runtime(
                PythonRuntimeError::AsyncRuntimeFailed(message.to_string()),
            )));
    }
    let _attached = Python::try_attach(move |_py| drop(active));
}

fn cancel_registered_submissions() -> Result<(), PythonRuntimeError> {
    #[cfg(test)]
    if FORCE_SUBMISSION_CANCEL_FAILURE.swap(false, Ordering::SeqCst) {
        return Err(PythonRuntimeError::AsyncRuntimeFailed(
            "forced owned asyncio cancellation failure".to_string(),
        ));
    }
    Python::try_attach(|py| {
        let submissions = {
            let state = lock_state()?;
            state
                .submissions
                .values()
                .map(|submission| {
                    (
                        submission.loop_object.clone_ref(py),
                        submission.exact_task.clone_ref(py),
                    )
                })
                .collect::<Vec<_>>()
        };
        for (loop_object, exact_task) in submissions {
            queue_exact_task_cancel(py, &loop_object, &exact_task)?;
        }
        Ok(())
    })
    .ok_or(PythonRuntimeError::NotInitialized)?
}

fn cancel_submission(submission_id: u64) -> Result<(), PythonRuntimeError> {
    Python::try_attach(|py| {
        let submission = {
            let state = lock_state()?;
            state.submissions.get(&submission_id).map(|submission| {
                (
                    submission.loop_object.clone_ref(py),
                    submission.exact_task.clone_ref(py),
                )
            })
        };
        if let Some((loop_object, exact_task)) = submission {
            queue_exact_task_cancel(py, &loop_object, &exact_task)?;
        }
        Ok(())
    })
    .ok_or(PythonRuntimeError::NotInitialized)?
}

fn queue_exact_task_cancel(
    py: Python<'_>,
    loop_object: &Py<PyAny>,
    exact_task: &Py<PyAny>,
) -> Result<(), PythonRuntimeError> {
    let cancel = exact_task
        .bind(py)
        .getattr("cancel")
        .map_err(|error| PythonRuntimeError::AsyncRuntimeFailed(error.to_string()))?;
    loop_object
        .bind(py)
        .call_method1("call_soon_threadsafe", (cancel,))
        .map_err(|error| PythonRuntimeError::AsyncRuntimeFailed(error.to_string()))?;
    Ok(())
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
    FORCE_SUBMISSION_QUEUE_FAILURE.store(false, Ordering::SeqCst);
    FORCE_TERMINAL_CALLBACK_PANIC.store(false, Ordering::SeqCst);
    FORCE_SUBMISSION_CANCEL_FAILURE.store(false, Ordering::SeqCst);
    super::shutdown_hooks::reset_for_tests();
    if let Ok(mut phases) = SHUTDOWN_PHASE_TRACE.lock() {
        phases.clear();
    }
}

#[cfg(test)]
#[path = "async_runtime_tests.rs"]
mod tests;
