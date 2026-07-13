use super::{PythonError, PythonRuntimeError};
use pyo3::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

pub(super) type PythonTerminalOutcome = Result<Py<PyAny>, PythonTerminalError>;

#[derive(Debug)]
pub(super) enum PythonTerminalError {
    Python(PyErr),
    Runtime(PythonRuntimeError),
}

#[derive(Clone)]
pub(super) struct PythonTerminal {
    shared: Arc<PythonTerminalShared>,
}

struct PythonTerminalShared {
    state: Mutex<PythonTerminalState>,
    changed: Condvar,
}

#[derive(Default)]
struct PythonTerminalState {
    outcome: Option<PythonTerminalOutcome>,
    waker: Option<Waker>,
}

impl PythonTerminal {
    pub(super) fn new() -> Self {
        Self {
            shared: Arc::new(PythonTerminalShared {
                state: Mutex::new(PythonTerminalState::default()),
                changed: Condvar::new(),
            }),
        }
    }

    pub(super) fn complete(&self, outcome: PythonTerminalOutcome) -> bool {
        let waker = {
            let mut state = self.lock_state();
            if state.outcome.is_some() {
                return false;
            }
            state.outcome = Some(outcome);
            state.waker.take()
        };
        self.shared.changed.notify_all();
        if let Some(waker) = waker {
            waker.wake();
        }
        true
    }

    pub(super) fn wait(self) -> PythonTerminalOutcome {
        let mut state = self.lock_state();
        loop {
            if let Some(outcome) = state.outcome.take() {
                return outcome;
            }
            state = self
                .shared
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, PythonTerminalState> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Future for PythonTerminal {
    type Output = PythonTerminalOutcome;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.lock_state();
        if let Some(outcome) = state.outcome.take() {
            return Poll::Ready(outcome);
        }
        let replace = state
            .waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(context.waker()));
        if replace {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }
}

pub(super) fn terminal_error_to_python(
    py: Python<'_>,
    error: PythonTerminalError,
    operation: &str,
) -> PythonError {
    match error {
        PythonTerminalError::Python(error) => {
            PythonError::from_pyerr(py, error, "await", operation)
        }
        PythonTerminalError::Runtime(error) => PythonError::runtime(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::{Wake, Waker};

    #[test]
    fn blocking_wait_observes_one_terminal_outcome() {
        let terminal = PythonTerminal::new();
        let completer = terminal.clone();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_worker = Arc::clone(&completed);
        let worker = std::thread::spawn(move || {
            let outcome = completer.complete(Err(PythonTerminalError::Runtime(
                PythonRuntimeError::AsyncRuntimeStopping,
            )));
            completed_worker.store(outcome, Ordering::SeqCst);
        });

        assert!(matches!(
            terminal.wait(),
            Err(PythonTerminalError::Runtime(
                PythonRuntimeError::AsyncRuntimeStopping
            ))
        ));
        worker.join().expect("terminal worker should not panic");
        assert!(completed.load(Ordering::SeqCst));
    }

    struct RecordingWake(AtomicBool);

    impl Wake for RecordingWake {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn future_repoll_replaces_waker_and_observes_terminal_outcome() {
        let terminal = PythonTerminal::new();
        let first_wake = Arc::new(RecordingWake(AtomicBool::new(false)));
        let second_wake = Arc::new(RecordingWake(AtomicBool::new(false)));
        let first_waker = Waker::from(Arc::clone(&first_wake));
        let second_waker = Waker::from(Arc::clone(&second_wake));
        let mut first_context = Context::from_waker(&first_waker);
        let mut second_context = Context::from_waker(&second_waker);
        let mut waiter = terminal.clone();

        assert!(Pin::new(&mut waiter).poll(&mut first_context).is_pending());
        assert!(Pin::new(&mut waiter).poll(&mut second_context).is_pending());
        assert!(terminal.complete(Err(PythonTerminalError::Runtime(
            PythonRuntimeError::AsyncRuntimeStopping,
        ))));

        assert!(!first_wake.0.load(Ordering::SeqCst));
        assert!(second_wake.0.load(Ordering::SeqCst));
        assert!(matches!(
            Pin::new(&mut waiter).poll(&mut second_context),
            Poll::Ready(Err(PythonTerminalError::Runtime(
                PythonRuntimeError::AsyncRuntimeStopping
            )))
        ));
    }
}
