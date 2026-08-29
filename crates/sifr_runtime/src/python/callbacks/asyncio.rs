use super::asyncio_entry::AsyncioCallbackEntry;
use super::execution::{
    CallbackExecutionError, collect_args, execution_error, python_error, result_object,
    validate_call_shape,
};
use super::{CallbackInvocationLease, CallbackOwnerState, errors};
use crate::cancellation::CancellationCarrier;
use crate::python::{ObjectHandle, PythonError, PythonRuntimeError, object_ops};
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use tokio::sync::Notify;

type BoxCallbackFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Box<dyn AsyncioOutput + 'a>, CallbackExecutionError>>
            + Send
            + 'a,
    >,
>;

trait AsyncioTarget<'a>: Send + Sync {
    fn prepare(
        &self,
        args: Vec<ObjectHandle>,
    ) -> Result<Box<dyn AsyncioPrepared<'a> + 'a>, PythonError>;
}

trait AsyncioPrepared<'a>: Send {
    fn invoke(
        self: Box<Self>,
        entry_sequence: u64,
        cancellation: CancellationCarrier,
    ) -> BoxCallbackFuture<'a>;
}

trait AsyncioOutput: Send {
    fn encode(self: Box<Self>) -> Result<ObjectHandle, PythonError>;
}

struct TypedAsyncioTarget<Decode, Handler, Encode> {
    decode: Arc<Decode>,
    handler: Arc<Handler>,
    encode: Arc<Encode>,
}

struct TypedAsyncioPrepared<A, R, Handler, Encode> {
    value: A,
    handler: Arc<Handler>,
    encode: Arc<Encode>,
    _result: PhantomData<R>,
}

struct TypedAsyncioOutput<R, Encode> {
    value: R,
    encode: Arc<Encode>,
}

impl<'a, A, R, Decode, Handler, Encode, HandlerFuture> AsyncioTarget<'a>
    for TypedAsyncioTarget<Decode, Handler, Encode>
where
    A: Send + 'a,
    R: Send + 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'a,
    Handler: Fn(u64, A, CancellationCarrier) -> HandlerFuture + Send + Sync + 'a,
    HandlerFuture: Future<Output = Result<R, CallbackExecutionError>> + Send + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    fn prepare(
        &self,
        args: Vec<ObjectHandle>,
    ) -> Result<Box<dyn AsyncioPrepared<'a> + 'a>, PythonError> {
        Ok(Box::new(TypedAsyncioPrepared {
            value: (self.decode)(args)?,
            handler: Arc::clone(&self.handler),
            encode: Arc::clone(&self.encode),
            _result: PhantomData,
        }))
    }
}

impl<'a, A, R, Handler, Encode, HandlerFuture> AsyncioPrepared<'a>
    for TypedAsyncioPrepared<A, R, Handler, Encode>
where
    A: Send + 'a,
    R: Send + 'a,
    Handler: Fn(u64, A, CancellationCarrier) -> HandlerFuture + Send + Sync + 'a,
    HandlerFuture: Future<Output = Result<R, CallbackExecutionError>> + Send + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    fn invoke(
        self: Box<Self>,
        entry_sequence: u64,
        cancellation: CancellationCarrier,
    ) -> BoxCallbackFuture<'a> {
        Box::pin(async move {
            let value = (self.handler)(entry_sequence, self.value, cancellation).await?;
            Ok(Box::new(TypedAsyncioOutput {
                value,
                encode: self.encode,
            }) as Box<dyn AsyncioOutput + 'a>)
        })
    }
}

impl<R, Encode> AsyncioOutput for TypedAsyncioOutput<R, Encode>
where
    R: Send,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync,
{
    fn encode(self: Box<Self>) -> Result<ObjectHandle, PythonError> {
        (self.encode)(self.value)
    }
}

#[derive(Clone, Copy)]
struct AsyncioTargetPtr(*const (dyn AsyncioTarget<'static> + 'static));

#[allow(clippy::transmute_ptr_to_ptr)]
#[allow(unsafe_code)]
/// Erase the callback target lifetime while the owning callback retains it.
///
/// # Safety
///
/// The returned pointer must never outlive the boxed target in `AsyncioCallback`.
unsafe fn erase_target_lifetime(target: *const (dyn AsyncioTarget<'_> + '_)) -> AsyncioTargetPtr {
    // SAFETY: the caller stores the pointer together with the owning boxed
    // target and closes all admitted invocations before dropping that owner.
    AsyncioTargetPtr(unsafe {
        std::mem::transmute::<
            *const (dyn AsyncioTarget<'_> + '_),
            *const (dyn AsyncioTarget<'static> + 'static),
        >(target)
    })
}

// SAFETY: access is synchronized by callback admission/ownership. The pointed
// target is Send + Sync and remains live until all admitted work completes.
#[allow(unsafe_code)]
unsafe impl Send for AsyncioTargetPtr {}
// SAFETY: access is synchronized by callback admission/ownership. The pointed
// target is Send + Sync and remains live until all admitted work completes.
#[allow(unsafe_code)]
unsafe impl Sync for AsyncioTargetPtr {}

/// Erase a prepared future lifetime while its callback target remains owned.
///
/// # Safety
///
/// The future must complete or be cancelled before the callback target drops.
#[allow(unsafe_code)]
unsafe fn erase_future_lifetime(future: BoxCallbackFuture<'_>) -> BoxCallbackFuture<'static> {
    // SAFETY: callback admission and close wait for every invocation future;
    // the target and captured values therefore outlive the erased future.
    unsafe { std::mem::transmute::<BoxCallbackFuture<'_>, BoxCallbackFuture<'static>>(future) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncioCallbackConcurrency {
    Serial,
    Parallel,
}

pub struct AsyncioCallback<'a> {
    object: ObjectHandle,
    owner: CallbackOwnerState,
    callback_id: u64,
    target: Mutex<Option<Box<dyn AsyncioTarget<'a> + 'a>>>,
    admission: Arc<AsyncioAdmission>,
    retained: AtomicBool,
}

#[derive(Default)]
struct AsyncioAdmission {
    state: Mutex<AsyncioAdmissionState>,
    changed: Notify,
}

#[derive(Default)]
struct AsyncioAdmissionState {
    open: bool,
    active_setups: usize,
}

struct AsyncioAdmissionLease {
    admission: Arc<AsyncioAdmission>,
    active: bool,
}

impl AsyncioAdmission {
    fn open() -> Self {
        Self {
            state: Mutex::new(AsyncioAdmissionState {
                open: true,
                active_setups: 0,
            }),
            changed: Notify::new(),
        }
    }

    fn enter(admission: &Arc<Self>) -> Result<AsyncioAdmissionLease, ()> {
        let mut state = admission
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.open {
            return Err(());
        }
        state.active_setups = state.active_setups.checked_add(1).ok_or(())?;
        drop(state);
        Ok(AsyncioAdmissionLease {
            admission: Arc::clone(admission),
            active: true,
        })
    }

    fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .open = false;
    }

    async fn close_and_wait(&self) {
        self.close();
        loop {
            let notified = self.changed.notified();
            if self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .active_setups
                == 0
            {
                return;
            }
            notified.await;
        }
    }
}

impl Drop for AsyncioAdmissionLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut state = self
            .admission
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active_setups = state.active_setups.saturating_sub(1);
        drop(state);
        self.admission.changed.notify_waiters();
    }
}

impl AsyncioCallback<'_> {
    #[must_use]
    pub fn object(&self) -> &ObjectHandle {
        &self.object
    }

    #[must_use]
    pub fn owner(&self) -> &CallbackOwnerState {
        &self.owner
    }

    pub async fn close_call_scope(&self) -> Result<(), PythonError> {
        self.admission.close_and_wait().await;
        self.owner.close_call_scope_async().await?;
        object_ops::close_object(self.object.clone())
    }

    pub async fn close_after_owner_unregister(&self) -> Result<(), PythonError> {
        self.admission.close_and_wait().await;
        self.owner.close_after_owner_unregister_async().await?;
        object_ops::close_object(self.object.clone())
    }

    pub async fn rollback_provisional(&self) -> Result<(), PythonError> {
        if self.retained.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        self.admission.close_and_wait().await;
        self.owner.cancel_callback_entries(self.callback_id).await;
        drop(
            self.target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
        );
        object_ops::close_object(self.object.clone())
    }
}

impl AsyncioCallback<'static> {
    pub fn retain_in_owner(&self) -> Result<(), PythonError> {
        if self.retained.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let target = self
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or_else(|| errors::unavailable("asyncio retained target"))?;
        let object = self.object.clone();
        self.owner.retain_capture(move || {
            drop(target);
            super::ownership::release_callable(object);
        })
    }
}

impl Drop for AsyncioCallback<'_> {
    fn drop(&mut self) {
        if self.retained.load(Ordering::Acquire) {
            return;
        }
        if self.owner.active_calls() == 0 {
            let _ignored = self.owner.close_call_scope();
            self.admission.close();
            let _ignored = object_ops::close_object(self.object.clone());
        } else if let Some(target) = self
            .target
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            Box::leak(target);
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(unsafe_code)]
pub fn asyncio_callback_scoped_with_owner<'a, A, R, Decode, Handler, Encode, HandlerFuture>(
    owner: CallbackOwnerState,
    _callback_id: u64,
    expected_arity: usize,
    concurrency: AsyncioCallbackConcurrency,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<AsyncioCallback<'a>, PythonError>
where
    A: Send + 'a,
    R: Send + 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'a,
    Handler: Fn(u64, A, CancellationCarrier) -> HandlerFuture + Send + Sync + 'a,
    HandlerFuture: Future<Output = Result<R, CallbackExecutionError>> + Send + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    let callback_id = owner.allocate_callback_id()?;
    crate::python::async_runtime::ensure_started().map_err(PythonError::runtime)?;
    let runtime = tokio::runtime::Handle::try_current().map_err(|error| {
        PythonError::runtime(PythonRuntimeError::AsyncRuntimeFailed(format!(
            "asyncio callback requires an active Sifr executor: {error}"
        )))
    })?;
    let target: Box<dyn AsyncioTarget<'a> + 'a> = Box::new(TypedAsyncioTarget {
        decode: Arc::new(decode),
        handler: Arc::new(handler),
        encode: Arc::new(encode),
    });
    let target_ptr: *const (dyn AsyncioTarget<'a> + 'a) = &*target;
    // SAFETY: target is moved into the returned callback owner. Admission is
    // closed and active invocations finish before that owner releases target.
    let target_ptr = Arc::new(unsafe { erase_target_lifetime(target_ptr) });
    let fifo = Arc::new(AsyncFifo::default());
    let admission = Arc::new(AsyncioAdmission::open());
    let shell_admission = Arc::clone(&admission);
    let shell_owner = owner.clone();
    let object = crate::python::attach(|py| {
        let callback = PyCFunction::new_closure(
            py,
            Some(c"sifr_asyncio_callback"),
            None,
            move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
                let _admission = AsyncioAdmission::enter(&shell_admission)
                    .map_err(|()| python_error(errors::closed(shell_owner.owner_id())))?;
                validate_call_shape(args, kwargs, expected_arity)?;
                let py = args.py();
                let asyncio = py.import("asyncio")?;
                let loop_object = asyncio.call_method0("get_running_loop")?;
                if !crate::python::async_runtime::is_owned_loop(py, &loop_object)
                    .map_err(|error| python_error(PythonError::runtime(error)))?
                {
                    return Err(python_error(errors::wrong_asyncio_loop(
                        shell_owner.owner_id(),
                        callback_id,
                    )));
                }
                if concurrency == AsyncioCallbackConcurrency::Serial
                    && errors::python_callback_origin(py).map_err(python_error)?
                        == Some((shell_owner.owner_id(), callback_id))
                {
                    return Err(python_error(errors::reentrant(
                        shell_owner.owner_id(),
                        callback_id,
                    )));
                }
                let invocation = shell_owner
                    .accept(callback_id, false)
                    .map_err(python_error)?;
                let entry_sequence = invocation.entry_sequence();
                let values = collect_args(args).map_err(python_error)?;
                // SAFETY: target_ptr is kept alive by the callback-owned target;
                // this admitted invocation completes before close can drop it.
                let prepared = unsafe { (&*target_ptr.0).prepare(values) }.map_err(python_error)?;
                let cancellation = CancellationCarrier::new();
                // SAFETY: the registered invocation owns completion/cancellation
                // and callback close waits for it before dropping target state.
                let prepared = unsafe {
                    erase_future_lifetime(prepared.invoke(entry_sequence, cancellation.clone()))
                };
                let future = loop_object.call_method0("create_future")?;
                let entry = AsyncioCallbackEntry::register(
                    &shell_owner,
                    callback_id,
                    entry_sequence,
                    cancellation,
                    loop_object.clone().unbind(),
                    future.clone().unbind(),
                )
                .map_err(python_error)?;
                let callback_entry = Arc::clone(&entry);
                let cancel_callback = match PyCFunction::new_closure(
                    py,
                    Some(c"__sifr_asyncio_callback_cancel"),
                    None,
                    move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| {
                        let cancelled = args
                            .get_item(0)?
                            .call_method0("cancelled")?
                            .extract::<bool>()?;
                        callback_entry.python_finished(cancelled);
                        Ok::<(), PyErr>(())
                    },
                ) {
                    Ok(callback) => callback,
                    Err(error) => {
                        entry.setup_failed();
                        return Err(error);
                    }
                };
                if let Err(error) = future.call_method1("add_done_callback", (cancel_callback,)) {
                    entry.setup_failed();
                    return Err(error);
                }
                let serial_ticket = if concurrency == AsyncioCallbackConcurrency::Serial {
                    match AsyncFifo::reserve(&fifo) {
                        Ok(ticket) => Some(ticket),
                        Err(error) => {
                            entry.setup_failed();
                            return Err(python_error(error));
                        }
                    }
                } else {
                    None
                };
                let task_future = InvocationFuture {
                    invocation: Some(invocation),
                    future: prepared,
                };
                let task_loop = loop_object.clone().unbind();
                let task_python_future = future.clone().unbind();
                let task_owner = shell_owner.clone();
                let worker = runtime.spawn(async move {
                    let _permit = match serial_ticket {
                        Some(ticket) => Some(ticket.await),
                        None => None,
                    };
                    task_future.await
                });
                entry.install_worker_abort(worker.abort_handle());
                let supervisor_entry = Arc::clone(&entry);
                let _supervisor = runtime.spawn(async move {
                    match worker.await {
                        Ok(completion) => {
                            if schedule_completion(
                                task_loop,
                                task_python_future,
                                task_owner,
                                completion,
                            ) {
                                supervisor_entry.task_finished();
                            } else {
                                supervisor_entry.task_aborted();
                            }
                        }
                        Err(_join_error) => supervisor_entry.task_aborted(),
                    }
                });
                Ok(future.unbind())
            },
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "asyncio callback"))?;
        object_ops::store_object(callback.into_any().unbind())
    })
    .map_err(PythonError::runtime)??;
    Ok(AsyncioCallback {
        object,
        owner,
        callback_id,
        target: Mutex::new(Some(target)),
        admission,
        retained: AtomicBool::new(false),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn asyncio_callback_with_owner<A, R, Decode, Handler, Encode, HandlerFuture>(
    owner: CallbackOwnerState,
    callback_id: u64,
    expected_arity: usize,
    concurrency: AsyncioCallbackConcurrency,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<AsyncioCallback<'static>, PythonError>
where
    A: Send + 'static,
    R: Send + 'static,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'static,
    Handler: Fn(u64, A, CancellationCarrier) -> HandlerFuture + Send + Sync + 'static,
    HandlerFuture: Future<Output = Result<R, CallbackExecutionError>> + Send + 'static,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static,
{
    asyncio_callback_scoped_with_owner(
        owner,
        callback_id,
        expected_arity,
        concurrency,
        decode,
        handler,
        encode,
    )
}

struct InvocationCompletion {
    invocation: CallbackInvocationLease,
    outcome: Result<Box<dyn AsyncioOutput + 'static>, CallbackExecutionError>,
}

struct InvocationFuture {
    invocation: Option<CallbackInvocationLease>,
    future: BoxCallbackFuture<'static>,
}

impl Future for InvocationFuture {
    type Output = InvocationCompletion;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let invocation = self
            .invocation
            .as_ref()
            .expect("callback invocation must exist while its future is pending");
        let _active = invocation.enter_poll();
        match self.future.as_mut().poll(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(outcome) => Poll::Ready(InvocationCompletion {
                invocation: self
                    .invocation
                    .take()
                    .expect("ready callback invocation must exist"),
                outcome,
            }),
        }
    }
}

fn schedule_completion(
    loop_object: Py<PyAny>,
    future: Py<PyAny>,
    owner: CallbackOwnerState,
    completion: InvocationCompletion,
) -> bool {
    let state = Arc::new(Mutex::new(Some(completion)));
    let attached = Python::try_attach(|py| {
        let callback_state = Arc::clone(&state);
        let callback_future = future.clone_ref(py);
        let callback_owner = owner.clone();
        let callback = PyCFunction::new_closure(
            py,
            Some(c"__sifr_asyncio_callback_complete"),
            None,
            move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| {
                let Some(completion) = callback_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                else {
                    return Ok::<(), PyErr>(());
                };
                let py = args.py();
                if callback_future
                    .bind(py)
                    .call_method0("done")?
                    .extract::<bool>()?
                {
                    drop(completion);
                    return Ok::<(), PyErr>(());
                }
                let entry_sequence = completion.invocation.entry_sequence();
                match completion.outcome {
                    Ok(output) => {
                        match output.encode().and_then(|value| result_object(py, value)) {
                            Ok(value) => {
                                callback_future
                                    .bind(py)
                                    .call_method1("set_result", (value,))?;
                            }
                            Err(error) => {
                                let error = python_error(error);
                                callback_future
                                    .bind(py)
                                    .call_method1("set_exception", (error.into_value(py),))?;
                            }
                        }
                    }
                    Err(error) => {
                        let error = execution_error(py, &callback_owner, entry_sequence, error);
                        callback_future
                            .bind(py)
                            .call_method1("set_exception", (error.into_value(py),))?;
                    }
                }
                drop(completion.invocation);
                Ok::<(), PyErr>(())
            },
        )?;
        loop_object
            .bind(py)
            .call_method1("call_soon_threadsafe", (callback,))?;
        Ok::<(), PyErr>(())
    });
    if !matches!(attached, Some(Ok(()))) {
        drop(
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
        );
        return false;
    }
    true
}

#[derive(Default)]
struct AsyncFifo {
    state: Mutex<AsyncFifoState>,
}

#[derive(Default)]
struct AsyncFifoState {
    next_ticket: u64,
    serving: u64,
    cancelled: BTreeSet<u64>,
    waiters: BTreeMap<u64, Waker>,
}

struct AsyncFifoTicket {
    fifo: Arc<AsyncFifo>,
    ticket: u64,
    pending: bool,
}

struct AsyncFifoPermit {
    fifo: Arc<AsyncFifo>,
}

impl AsyncFifo {
    fn reserve(fifo: &Arc<Self>) -> Result<AsyncFifoTicket, PythonError> {
        let mut state = fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ticket = state.next_ticket;
        state.next_ticket = state
            .next_ticket
            .checked_add(1)
            .ok_or_else(|| errors::unavailable("asyncio serial ticket space"))?;
        Ok(AsyncFifoTicket {
            fifo: Arc::clone(fifo),
            ticket,
            pending: true,
        })
    }
}

impl Future for AsyncFifoTicket {
    type Output = AsyncFifoPermit;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let ready = {
            let mut state = self
                .fifo
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.serving == self.ticket {
                state.waiters.remove(&self.ticket);
                true
            } else {
                state.waiters.insert(self.ticket, context.waker().clone());
                false
            }
        };
        if ready {
            self.pending = false;
            Poll::Ready(AsyncFifoPermit {
                fifo: Arc::clone(&self.fifo),
            })
        } else {
            Poll::Pending
        }
    }
}

impl Drop for AsyncFifoTicket {
    fn drop(&mut self) {
        if !self.pending {
            return;
        }
        let mut state = self
            .fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.waiters.remove(&self.ticket);
        state.cancelled.insert(self.ticket);
        advance_fifo(&mut state);
    }
}

impl Drop for AsyncFifoPermit {
    fn drop(&mut self) {
        let mut state = self
            .fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.serving = state.serving.saturating_add(1);
        advance_fifo(&mut state);
    }
}

fn advance_fifo(state: &mut AsyncFifoState) {
    while state.cancelled.remove(&state.serving) {
        state.serving = state.serving.saturating_add(1);
    }
    if let Some(waker) = state.waiters.remove(&state.serving) {
        waker.wake();
    }
}
