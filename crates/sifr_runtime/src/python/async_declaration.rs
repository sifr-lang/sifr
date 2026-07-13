use super::async_runtime::{
    finish_submission, register_submission, release_pending_submission, reserve_submission,
    terminal_for_submission, SubmissionCancellationBridge,
};
use super::async_terminal::{
    PythonTerminal, PythonTerminalError, PythonTerminalOutcome, PythonTerminalValue,
};
use super::async_value::{materialize, PythonAsyncRequest, PythonAsyncTarget, PythonAsyncValue};
use super::object_ops::clone_handle;
use super::{PythonError, PythonRuntimeError};
use crate::cancellation::CancellationCarrier;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

/// Submit one generated typed declaration operation to the application-owned
/// asyncio loop. This API is compiler glue, not a user-visible interop surface.
#[doc(hidden)]
pub async fn submit_async_declaration(
    request: PythonAsyncRequest,
    carrier: Option<&CancellationCarrier>,
) -> Result<PythonAsyncValue, PythonError> {
    super::async_runtime::ensure_started().map_err(PythonError::runtime)?;
    validate_keywords(&request)?;
    let terminal =
        super::attach(|py| submit_typed(py, request, carrier)).map_err(PythonError::runtime)??;
    match terminal.await {
        Ok(PythonTerminalValue::Typed(value)) => Ok(value),
        Ok(PythonTerminalValue::Raw(_)) => Err(PythonError::runtime(
            PythonRuntimeError::AsyncRuntimeFailed(
                "typed asyncio submission produced a raw terminal value".to_string(),
            ),
        )),
        Err(PythonTerminalError::Mapped(error)) => Err(*error),
        Err(PythonTerminalError::Runtime(error)) => Err(PythonError::runtime(error)),
        Err(PythonTerminalError::Python(error)) => Python::try_attach(|py| {
            Err(PythonError::from_pyerr(
                py,
                error,
                "await",
                "typed Python declaration",
            ))
        })
        .unwrap_or_else(|| Err(PythonError::runtime(PythonRuntimeError::NotInitialized))),
    }
}

fn submit_typed(
    py: Python<'_>,
    request: PythonAsyncRequest,
    carrier: Option<&CancellationCarrier>,
) -> Result<PythonTerminal, PythonError> {
    let (terminal, cancellation) = terminal_for_submission(carrier)?;
    let (submission_id, loop_object) =
        reserve_submission(py, &terminal).map_err(PythonError::runtime)?;
    let context = request_context(&request);
    let request = Arc::new(request);
    let done_callback = build_done_callback(
        py,
        submission_id,
        Arc::clone(&request),
        context.clone(),
        &terminal,
    )
    .map_err(|error| {
        request.finish_semantic_close(false);
        release_pending_submission(submission_id);
        PythonError::from_pyerr(py, error, "call", "typed asyncio completion callback")
    })?;
    let setup_callback = build_setup_callback(
        py,
        submission_id,
        Arc::clone(&request),
        &loop_object,
        &done_callback,
        &terminal,
        &cancellation,
        context,
    )
    .map_err(|error| {
        request.finish_semantic_close(false);
        release_pending_submission(submission_id);
        PythonError::from_pyerr(py, error, "call", "typed asyncio setup callback")
    })?;
    loop_object
        .bind(py)
        .call_method1("call_soon_threadsafe", (setup_callback,))
        .map_err(|error| {
            request.finish_semantic_close(false);
            release_pending_submission(submission_id);
            PythonError::from_pyerr(py, error, "call", "typed asyncio submission")
        })?;
    Ok(terminal)
}

fn build_done_callback(
    py: Python<'_>,
    submission_id: u64,
    request: Arc<PythonAsyncRequest>,
    context: String,
    terminal: &PythonTerminal,
) -> PyResult<Py<PyAny>> {
    let terminal = terminal.clone();
    PyCFunction::new_closure(
        py,
        Some(c"__sifr_python_typed_task_done"),
        None,
        move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<()> {
            let py = args.py();
            let outcome = catch_unwind(AssertUnwindSafe(|| -> PythonTerminalOutcome {
                let task = args.get_item(0).map_err(|error| {
                    mapped_error(PythonError::from_pyerr(py, error, "await", &context))
                })?;
                let value = task.call_method0("result").map_err(|error| {
                    mapped_error(PythonError::from_pyerr(py, error, "await", &context))
                })?;
                super::async_value::convert_output(py, &value, &request.output, &context)
                    .map(PythonTerminalValue::Typed)
                    .map_err(mapped_error)
            }))
            .unwrap_or_else(|_| {
                Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::AsyncRuntimeFailed(
                        "typed asyncio terminal callback panicked".to_string(),
                    ),
                ))
            });
            request.finish_semantic_close(outcome.is_ok());
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
    request: Arc<PythonAsyncRequest>,
    loop_object: &Py<PyAny>,
    done_callback: &Py<PyAny>,
    terminal: &PythonTerminal,
    cancellation: &Arc<SubmissionCancellationBridge>,
    context: String,
) -> PyResult<Py<PyAny>> {
    let loop_object = loop_object.clone_ref(py);
    let done_callback = done_callback.clone_ref(py);
    let terminal = terminal.clone();
    let cancellation = Arc::clone(cancellation);
    PyCFunction::new_closure(
        py,
        Some(c"__sifr_python_typed_task_setup"),
        None,
        move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| -> PyResult<()> {
            let py = args.py();
            let mut registered = false;
            let mut exact_task: Option<Py<PyAny>> = None;
            let setup = catch_unwind(AssertUnwindSafe(|| -> Result<(), PythonTerminalError> {
                let callable =
                    resolve_callable(py, &request.target, &context).map_err(mapped_error)?;
                let positional = request
                    .args
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        materialize(py, value, &format!("{context}.arg[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(mapped_error)?;
                let positional = PyTuple::new(py, positional.iter())
                    .map_err(|error| mapped_pyerr(py, error, "conversion", &context))?;
                let keywords = PyDict::new(py);
                for (name, value) in &request.kwargs {
                    let value = materialize(py, value, &format!("{context}.{name}"))
                        .map_err(mapped_error)?;
                    keywords
                        .set_item(name, value.bind(py))
                        .map_err(|error| mapped_pyerr(py, error, "conversion", &context))?;
                }
                let _call_depth = super::enter_python_call();
                let awaitable = callable
                    .bind(py)
                    .call(positional, Some(&keywords))
                    .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                let inspect = py
                    .import("inspect")
                    .map_err(|error| mapped_pyerr(py, error, "import", &context))?;
                let is_awaitable = inspect
                    .call_method1("isawaitable", (&awaitable,))
                    .and_then(|value| value.extract::<bool>())
                    .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                if !is_awaitable {
                    return Err(mapped_error(PythonError::without_replay(
                        "call",
                        "TypeError",
                        "Python coroutine declaration returned a non-awaitable value",
                        String::new(),
                        context.clone(),
                    )));
                }
                let asyncio = py
                    .import("asyncio")
                    .map_err(|error| mapped_pyerr(py, error, "import", &context))?;
                let ensure_kwargs = PyDict::new(py);
                ensure_kwargs
                    .set_item("loop", loop_object.bind(py))
                    .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                let task = asyncio
                    .getattr("ensure_future")
                    .and_then(|ensure| ensure.call((awaitable,), Some(&ensure_kwargs)))
                    .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                exact_task = Some(task.clone().unbind());
                task.call_method1("add_done_callback", (done_callback.bind(py),))
                    .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                register_submission(py, submission_id, &loop_object, &task.clone().unbind())
                    .map_err(PythonTerminalError::Runtime)?;
                registered = true;
                if cancellation.publish(submission_id) {
                    task.call_method0("cancel")
                        .map_err(|error| mapped_pyerr(py, error, "call", &context))?;
                }
                Ok(())
            }))
            .unwrap_or_else(|_| {
                Err(PythonTerminalError::Runtime(
                    PythonRuntimeError::AsyncRuntimeFailed(
                        "typed asyncio setup callback panicked".to_string(),
                    ),
                ))
            });

            if let Err(error) = setup {
                request.finish_semantic_close(false);
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

fn resolve_callable(
    py: Python<'_>,
    target: &PythonAsyncTarget,
    context: &str,
) -> Result<Py<PyAny>, PythonError> {
    match target {
        PythonAsyncTarget::Function(path) => {
            let target = super::object_ops::resolve_target(path)?;
            clone_handle(py, &target)
        }
        PythonAsyncTarget::Method { receiver, member } => receiver
            .clone_ref(py)?
            .bind(py)
            .getattr(member.as_str())
            .map(Bound::unbind)
            .map_err(|error| PythonError::from_pyerr(py, error, "attribute", context)),
    }
}

fn validate_keywords(request: &PythonAsyncRequest) -> Result<(), PythonError> {
    let mut names = HashSet::with_capacity(request.kwargs.len());
    for (name, _) in &request.kwargs {
        if !names.insert(name.as_str()) {
            return Err(PythonError::without_replay(
                "call",
                "TypeError",
                format!("multiple values for keyword argument '{name}'"),
                String::new(),
                request_context(request),
            ));
        }
    }
    Ok(())
}

fn request_context(request: &PythonAsyncRequest) -> String {
    match &request.target {
        PythonAsyncTarget::Function(path) => path.join("."),
        PythonAsyncTarget::Method { member, .. } => format!("Self.{member}"),
    }
}

fn mapped_pyerr(
    py: Python<'_>,
    error: PyErr,
    kind: &'static str,
    context: &str,
) -> PythonTerminalError {
    mapped_error(PythonError::from_pyerr(py, error, kind, context))
}

fn mapped_error(error: PythonError) -> PythonTerminalError {
    PythonTerminalError::Mapped(Box::new(error))
}
