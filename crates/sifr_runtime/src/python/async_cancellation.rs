use super::async_runtime::SubmissionCancellationBridge;
use super::async_terminal::PythonTerminalError;
use super::{PythonError, PythonRuntimeError};
use crate::cancellation::{CancellationCarrier, CancellationResume};
use pyo3::prelude::*;
use std::future::poll_fn;
use std::task::Poll;

pub(super) fn classify_task_error(
    py: Python<'_>,
    error: PyErr,
    cancellation: &SubmissionCancellationBridge,
    context: &str,
) -> PythonTerminalError {
    if cancellation.was_requested() && is_cancelled_error(py, &error) {
        PythonTerminalError::ActiveCancellation
    } else {
        PythonTerminalError::Mapped(Box::new(PythonError::from_pyerr(
            py, error, "await", context,
        )))
    }
}

pub(super) async fn propagate<T>(carrier: Option<&CancellationCarrier>) -> Result<T, PythonError> {
    let Some(carrier) = carrier else {
        return Err(propagation_error("active cancellation has no task carrier"));
    };
    match carrier.resume_fallback_after_claim() {
        CancellationResume::Invoked | CancellationResume::AlreadyResumed => yield_once().await,
        outcome => {
            return Err(propagation_error(&format!(
                "active cancellation could not resume its native task fallback: {outcome:?}"
            )));
        }
    }
    Err(propagation_error(
        "active cancellation fallback returned without terminating the native task",
    ))
}

fn is_cancelled_error(py: Python<'_>, error: &PyErr) -> bool {
    py.import("asyncio")
        .and_then(|asyncio| asyncio.getattr("CancelledError"))
        .is_ok_and(|cancelled| error.is_instance(py, &cancelled))
}

async fn yield_once() {
    let mut yielded = false;
    poll_fn(move |context| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}

fn propagation_error(message: &str) -> PythonError {
    PythonError::runtime(PythonRuntimeError::AsyncRuntimeFailed(message.to_string()))
}
