use super::state::CallbackAsyncEntryLease;
use super::{CallbackOwnerState, errors};
use crate::cancellation::{CancellationBind, CancellationCarrier};
use crate::python::PythonError;
use pyo3::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::AbortHandle;

const TASK_TERMINAL: u8 = 1;
const PYTHON_TERMINAL: u8 = 2;
const ENTRY_TERMINAL: u8 = 4;

pub(super) struct AsyncioCallbackEntry {
    cancellation: CancellationCarrier,
    terminal: AtomicU8,
    state: Mutex<EntryState>,
}

struct EntryState {
    owner_lease: Option<CallbackAsyncEntryLease>,
    loop_object: Option<Py<PyAny>>,
    python_future: Option<Py<PyAny>>,
    worker_abort: Option<AbortHandle>,
    python_cancel_queued: bool,
    worker_abort_requested: bool,
}

impl AsyncioCallbackEntry {
    pub(super) fn register(
        owner: &CallbackOwnerState,
        callback_id: u64,
        entry_sequence: u64,
        cancellation: CancellationCarrier,
        loop_object: Py<PyAny>,
        python_future: Py<PyAny>,
    ) -> Result<Arc<Self>, PythonError> {
        let entry = Arc::new(Self {
            cancellation,
            terminal: AtomicU8::new(0),
            state: Mutex::new(EntryState {
                owner_lease: None,
                loop_object: Some(loop_object),
                python_future: Some(python_future),
                worker_abort: None,
                python_cancel_queued: false,
                worker_abort_requested: false,
            }),
        });
        let weak_entry = Arc::downgrade(&entry);
        match entry.cancellation.bind_fallback(Arc::new(move || {
            if let Some(entry) = weak_entry.upgrade() {
                entry.cancel_python_and_abort_worker();
            }
        })) {
            CancellationBind::Bound => {}
            CancellationBind::InvokedPendingCancellation => {
                return Err(errors::unavailable(
                    "asyncio callback cancellation before entry registration",
                ));
            }
            CancellationBind::AlreadyBound | CancellationBind::StateUnavailable => {
                return Err(errors::unavailable(
                    "asyncio callback cancellation fallback",
                ));
            }
        }

        let weak_entry = Arc::downgrade(&entry);
        let owner_lease = owner.register_async_entry(
            callback_id,
            entry_sequence,
            Arc::new(move || {
                if let Some(entry) = weak_entry.upgrade() {
                    entry.cancel_from_owner();
                }
            }),
        )?;
        entry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .owner_lease = Some(owner_lease);
        Ok(entry)
    }

    pub(super) fn install_worker_abort(&self, abort: AbortHandle) {
        let abort_now = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.worker_abort_requested {
                true
            } else {
                state.worker_abort = Some(abort.clone());
                false
            }
        };
        if abort_now {
            abort.abort();
        }
    }

    pub(super) fn python_finished(&self, cancelled: bool) {
        if cancelled {
            let _outcome = self.cancellation.request_cancel();
        }
        self.mark_terminal(PYTHON_TERMINAL);
    }

    pub(super) fn task_finished(&self) {
        self.mark_terminal(TASK_TERMINAL);
    }

    pub(super) fn task_aborted(&self) {
        self.queue_python_cancel();
        self.mark_terminal(TASK_TERMINAL);
    }

    pub(super) fn setup_failed(&self) {
        self.queue_python_cancel();
        self.mark_terminal(TASK_TERMINAL);
        self.mark_terminal(PYTHON_TERMINAL);
    }

    fn cancel_from_owner(&self) {
        self.queue_python_cancel();
        let _outcome = self.cancellation.request_cancel();
        self.abort_worker();
    }

    fn cancel_python_and_abort_worker(&self) {
        self.queue_python_cancel();
        self.abort_worker();
    }

    fn queue_python_cancel(&self) {
        let should_queue = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.python_cancel_queued {
                return;
            }
            state.python_cancel_queued = true;
            state.loop_object.is_some() && state.python_future.is_some()
        };
        if !should_queue {
            return;
        }
        let queued = Python::try_attach(|py| {
            let handles = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state
                    .loop_object
                    .as_ref()
                    .zip(state.python_future.as_ref())
                    .map(|(loop_object, future)| (loop_object.clone_ref(py), future.clone_ref(py)))
            };
            let Some((loop_object, future)) = handles else {
                return Ok::<(), PyErr>(());
            };
            let cancel = future.bind(py).getattr("cancel")?;
            loop_object
                .bind(py)
                .call_method1("call_soon_threadsafe", (cancel,))?;
            Ok::<(), PyErr>(())
        });
        if !matches!(queued, Some(Ok(()))) {
            self.mark_terminal(PYTHON_TERMINAL);
        }
    }

    fn abort_worker(&self) {
        let abort = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.worker_abort_requested = true;
            state.worker_abort.clone()
        };
        if let Some(abort) = abort {
            abort.abort();
        }
    }

    fn mark_terminal(&self, terminal: u8) {
        let previous = self.terminal.fetch_or(terminal, Ordering::AcqRel);
        if (previous | terminal) != (TASK_TERMINAL | PYTHON_TERMINAL) {
            return;
        }
        if self
            .terminal
            .compare_exchange(
                TASK_TERMINAL | PYTHON_TERMINAL,
                TASK_TERMINAL | PYTHON_TERMINAL | ENTRY_TERMINAL,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return;
        }
        let resources = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (
                state.owner_lease.take(),
                state.loop_object.take(),
                state.python_future.take(),
                state.worker_abort.take(),
            )
        };
        let _attached = Python::try_attach(move |_py| drop(resources));
    }
}
