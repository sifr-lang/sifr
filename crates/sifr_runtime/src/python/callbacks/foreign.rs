use super::CallbackOwnerState;
use super::execution::{
    CallbackExecutionError, collect_args, execution_error, python_error, result_object,
    validate_call_shape,
};
use crate::python::{ObjectHandle, PythonError, object_ops};
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};

trait ForeignTarget<'a>: Send + Sync {
    fn prepare(
        &self,
        args: Vec<ObjectHandle>,
    ) -> Result<Box<dyn ForeignPrepared<'a> + 'a>, PythonError>;
}

trait ForeignPrepared<'a>: Send {
    fn invoke(
        self: Box<Self>,
        entry_sequence: u64,
    ) -> Result<Box<dyn ForeignOutput + 'a>, CallbackExecutionError>;
}

trait ForeignOutput: Send {
    fn encode(self: Box<Self>) -> Result<ObjectHandle, PythonError>;
}

struct TypedForeignTarget<Decode, Handler, Encode> {
    decode: Arc<Decode>,
    handler: Arc<Handler>,
    encode: Arc<Encode>,
}

struct TypedForeignPrepared<A, R, Handler, Encode> {
    value: A,
    handler: Arc<Handler>,
    encode: Arc<Encode>,
    _result: PhantomData<R>,
}

struct TypedForeignOutput<R, Encode> {
    value: R,
    encode: Arc<Encode>,
}

struct RetainedForeignTarget {
    target: Mutex<Option<Arc<dyn ForeignTarget<'static> + 'static>>>,
}

impl RetainedForeignTarget {
    fn new(target: Arc<dyn ForeignTarget<'static> + 'static>) -> Self {
        Self {
            target: Mutex::new(Some(target)),
        }
    }

    fn target(&self) -> Option<Arc<dyn ForeignTarget<'static> + 'static>> {
        self.target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn release(&self) {
        self.target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

impl<'a, A, R, Decode, Handler, Encode> ForeignTarget<'a>
    for TypedForeignTarget<Decode, Handler, Encode>
where
    A: Send + 'a,
    R: Send + 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'a,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + Send + Sync + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    fn prepare(
        &self,
        args: Vec<ObjectHandle>,
    ) -> Result<Box<dyn ForeignPrepared<'a> + 'a>, PythonError> {
        Ok(Box::new(TypedForeignPrepared {
            value: (self.decode)(args)?,
            handler: Arc::clone(&self.handler),
            encode: Arc::clone(&self.encode),
            _result: PhantomData,
        }))
    }
}

impl<'a, A, R, Handler, Encode> ForeignPrepared<'a> for TypedForeignPrepared<A, R, Handler, Encode>
where
    A: Send + 'a,
    R: Send + 'a,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + Send + Sync + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    fn invoke(
        self: Box<Self>,
        entry_sequence: u64,
    ) -> Result<Box<dyn ForeignOutput + 'a>, CallbackExecutionError> {
        let value = (self.handler)(entry_sequence, self.value)?;
        Ok(Box::new(TypedForeignOutput {
            value,
            encode: self.encode,
        }))
    }
}

impl<R, Encode> ForeignOutput for TypedForeignOutput<R, Encode>
where
    R: Send,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync,
{
    fn encode(self: Box<Self>) -> Result<ObjectHandle, PythonError> {
        (self.encode)(self.value)
    }
}

#[derive(Clone, Copy)]
struct ForeignTargetPtr(*const (dyn ForeignTarget<'static> + 'static));

#[allow(clippy::transmute_ptr_to_ptr)]
unsafe fn erase_target_lifetime(target: *const (dyn ForeignTarget<'_> + '_)) -> ForeignTargetPtr {
    // SAFETY: callers must keep the target alive until owner admission closes
    // and every accepted invocation has drained.
    ForeignTargetPtr(unsafe {
        std::mem::transmute::<
            *const (dyn ForeignTarget<'_> + '_),
            *const (dyn ForeignTarget<'static> + 'static),
        >(target)
    })
}

// SAFETY: the pointer is only dereferenced after owner admission. Call-scoped
// close rejects new entries and drains all accepted calls before its owning box
// can be dropped.
unsafe impl Send for ForeignTargetPtr {}
// SAFETY: `ForeignTarget` is `Sync`, and its owner provides the lifetime proof.
unsafe impl Sync for ForeignTargetPtr {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignCallbackConcurrency {
    Serial,
    Parallel,
}

pub struct ForeignCallback<'a> {
    object: ObjectHandle,
    owner: CallbackOwnerState,
    target: Option<Box<dyn ForeignTarget<'a> + 'a>>,
    retained_target: RefCell<Option<Arc<RetainedForeignTarget>>>,
    closed: Cell<bool>,
    retained: Cell<bool>,
    admission: Arc<AtomicBool>,
    _not_send: PhantomData<Rc<()>>,
}

impl ForeignCallback<'_> {
    #[must_use]
    pub fn object(&self) -> &ObjectHandle {
        &self.object
    }

    #[must_use]
    pub fn owner(&self) -> &CallbackOwnerState {
        &self.owner
    }

    pub fn close_call_scope(&self) -> Result<(), PythonError> {
        if self.closed.get() {
            return Ok(());
        }
        self.owner.close_call_scope()?;
        self.admission.store(false, Ordering::Release);
        self.closed.set(true);
        object_ops::close_object(self.object.clone())
    }

    pub async fn close_call_scope_async(&self) -> Result<(), PythonError> {
        if self.closed.get() {
            return Ok(());
        }
        self.owner.close_call_scope_async().await?;
        self.admission.store(false, Ordering::Release);
        self.closed.set(true);
        object_ops::close_object(self.object.clone())
    }

    pub fn close_after_owner_unregister(&self) -> Result<(), PythonError> {
        self.owner.close_after_owner_unregister()?;
        self.admission.store(false, Ordering::Release);
        self.closed.set(true);
        object_ops::close_object(self.object.clone())
    }

    pub fn retain_in_owner(&self) -> Result<(), PythonError> {
        if self.retained.get() {
            return Ok(());
        }
        let object = self.object.clone();
        let retained_target = self.retained_target.borrow().clone();
        self.owner.retain_capture(move || {
            if let Some(target) = retained_target {
                target.release();
            }
            super::ownership::release_callable(object);
        })?;
        self.retained_target.borrow_mut().take();
        self.retained.set(true);
        Ok(())
    }
}

impl Drop for ForeignCallback<'_> {
    fn drop(&mut self) {
        if self.target.is_some() && self.close_call_scope().is_err() && !self.closed.get() {
            // See `CurrentCallback::drop`: a reentrant ownership violation
            // must not deallocate the target while its shell is executing.
            if let Some(target) = self.target.take() {
                Box::leak(target);
            }
        }
        if self.target.is_none() && !self.retained.get() && !self.closed.get() {
            self.admission.store(false, Ordering::Release);
            let _ignored = object_ops::close_object(self.object.clone());
        }
    }
}

pub fn foreign_callback<A, R, Decode, Handler, Encode>(
    callback_id: u64,
    expected_arity: usize,
    concurrency: ForeignCallbackConcurrency,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<ForeignCallback<'static>, PythonError>
where
    A: Send + 'static,
    R: Send + 'static,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'static,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + Send + Sync + 'static,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static,
{
    let owner = CallbackOwnerState::new_call_scoped()?;
    // SAFETY: every capture and converted value is owned and static.
    unsafe {
        foreign_callback_scoped_with_owner(
            owner,
            callback_id,
            expected_arity,
            concurrency,
            decode,
            handler,
            encode,
        )
    }
}

/// Constructs a compiler-owned callback borrowing its enclosing call frame.
///
/// # Safety
/// Keep the captures alive until this wrapper is dropped; never forget it or
/// drop it from an admitted invocation. Generated callers use a private local
/// whose drop precedes the handler binding's drop on every exit. Drop closes
/// owner admission and drains decoding, execution and encoding before freeing
/// the target. Send/Sync targets permit shared foreign-thread calls; mutable
/// access must be synchronized by the capture's own safe types.
pub unsafe fn foreign_callback_scoped_with_owner<'a, A, R, Decode, Handler, Encode>(
    owner: CallbackOwnerState,
    callback_id: u64,
    expected_arity: usize,
    concurrency: ForeignCallbackConcurrency,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<ForeignCallback<'a>, PythonError>
where
    A: Send + 'a,
    R: Send + 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'a,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + Send + Sync + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'a,
{
    owner.require_call_scope()?;
    let target: Box<dyn ForeignTarget<'a> + 'a> = Box::new(TypedForeignTarget {
        decode: Arc::new(decode),
        handler: Arc::new(handler),
        encode: Arc::new(encode),
    });
    let target_ptr: *const (dyn ForeignTarget<'a> + 'a) = &*target;
    // The returned callback owns `target`, and `close_call_scope` drains every
    // invocation before the lifetime-erased pointer is dereferenced or dropped.
    // SAFETY: `close_call_scope` drains the owner before `_target` is dropped.
    let target_ptr = Arc::new(unsafe { erase_target_lifetime(target_ptr) });
    let fifo = Arc::new(FifoSerial::default());
    let admission = Arc::new(Mutex::new(()));
    let callback_admission = Arc::new(AtomicBool::new(true));
    let shell_callback_admission = Arc::clone(&callback_admission);
    let shell_owner = owner.clone();
    let object = crate::python::attach(|py| {
        let callback = PyCFunction::new_closure(
            py,
            Some(c"sifr_foreign_callback"),
            None,
            move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
                let py = args.py();
                if !shell_callback_admission.load(Ordering::Acquire) {
                    return Err(python_error(super::errors::closed(shell_owner.owner_id())));
                }
                let (invocation, serial_ticket) =
                    admit(&shell_owner, callback_id, concurrency, &admission, &fifo)?;
                let entry_sequence = invocation.entry_sequence();
                let invocation = invocation.enter().map_err(python_error)?;
                validate_call_shape(args, kwargs, expected_arity)?;
                let args = collect_args(args).map_err(python_error)?;
                // SAFETY: owner admission now spans checked Python decoding,
                // detached handler execution, and result encoding. Closing
                // cannot drop the borrowed target until this invocation drains.
                let prepared = unsafe { (&*target_ptr.0).prepare(args) }.map_err(python_error)?;
                let outcome = py.detach(move || {
                    let _permit = serial_ticket.map(FifoTicket::acquire);
                    prepared
                        .invoke(entry_sequence)
                        .map(|result| (invocation, result))
                });
                let (invocation, result) = outcome
                    .map_err(|error| execution_error(py, &shell_owner, entry_sequence, error))?;
                let result = result.encode().map_err(python_error)?;
                let result = result_object(py, result).map_err(python_error);
                drop(invocation);
                result
            },
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "foreign callback"))?;
        object_ops::store_object(callback.into_any().unbind())
    })
    .map_err(PythonError::runtime)??;
    Ok(ForeignCallback {
        object,
        owner,
        target: Some(target),
        retained_target: RefCell::new(None),
        closed: Cell::new(false),
        retained: Cell::new(false),
        admission: callback_admission,
        _not_send: PhantomData,
    })
}

pub fn foreign_callback_with_owner<A, R, Decode, Handler, Encode>(
    owner: CallbackOwnerState,
    _callback_id: u64,
    expected_arity: usize,
    concurrency: ForeignCallbackConcurrency,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<ForeignCallback<'static>, PythonError>
where
    A: Send + 'static,
    R: Send + 'static,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + Send + Sync + 'static,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + Send + Sync + 'static,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static,
{
    let callback_id = owner.allocate_callback_id()?;
    let target: Arc<dyn ForeignTarget<'static> + 'static> = Arc::new(TypedForeignTarget {
        decode: Arc::new(decode),
        handler: Arc::new(handler),
        encode: Arc::new(encode),
    });
    let retained_target = Arc::new(RetainedForeignTarget::new(target));
    let shell_target: Weak<RetainedForeignTarget> = Arc::downgrade(&retained_target);
    let fifo = Arc::new(FifoSerial::default());
    let admission = Arc::new(Mutex::new(()));
    let callback_admission = Arc::new(AtomicBool::new(true));
    let shell_callback_admission = Arc::clone(&callback_admission);
    let shell_owner = owner.clone();
    let object = crate::python::attach(|py| {
        let callback = PyCFunction::new_closure(
            py,
            Some(c"sifr_foreign_callback"),
            None,
            move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
                let py = args.py();
                if !shell_callback_admission.load(Ordering::Acquire) {
                    return Err(python_error(super::errors::closed(shell_owner.owner_id())));
                }
                let (invocation, serial_ticket) =
                    admit(&shell_owner, callback_id, concurrency, &admission, &fifo)?;
                let entry_sequence = invocation.entry_sequence();
                let invocation = invocation.enter().map_err(python_error)?;
                validate_call_shape(args, kwargs, expected_arity)?;
                let args = collect_args(args).map_err(python_error)?;
                let target = shell_target
                    .upgrade()
                    .and_then(|target| target.target())
                    .ok_or_else(|| python_error(super::errors::closed(shell_owner.owner_id())))?;
                let prepared = target.prepare(args).map_err(python_error)?;
                let outcome = py.detach(move || {
                    let _permit = serial_ticket.map(FifoTicket::acquire);
                    prepared
                        .invoke(entry_sequence)
                        .map(|result| (invocation, result))
                });
                let (invocation, result) = outcome
                    .map_err(|error| execution_error(py, &shell_owner, entry_sequence, error))?;
                let result = result.encode().map_err(python_error)?;
                let result = result_object(py, result).map_err(python_error);
                drop(invocation);
                result
            },
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "foreign callback"))?;
        object_ops::store_object(callback.into_any().unbind())
    })
    .map_err(PythonError::runtime)??;
    Ok(ForeignCallback {
        object,
        owner,
        target: None,
        retained_target: RefCell::new(Some(retained_target)),
        closed: Cell::new(false),
        retained: Cell::new(false),
        admission: callback_admission,
        _not_send: PhantomData,
    })
}

fn admit(
    owner: &CallbackOwnerState,
    callback_id: u64,
    concurrency: ForeignCallbackConcurrency,
    admission: &Mutex<()>,
    fifo: &Arc<FifoSerial>,
) -> PyResult<(super::CallbackInvocationLease, Option<FifoTicket>)> {
    if concurrency == ForeignCallbackConcurrency::Serial {
        owner
            .reject_serial_reentrancy(callback_id)
            .map_err(python_error)?;
        let _admission = admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let invocation = owner.accept(callback_id, true).map_err(python_error)?;
        Ok((invocation, Some(FifoSerial::reserve(fifo))))
    } else {
        owner
            .accept(callback_id, false)
            .map(|invocation| (invocation, None))
            .map_err(python_error)
    }
}

#[derive(Default)]
struct FifoSerial {
    state: Mutex<FifoState>,
    changed: Condvar,
}

#[derive(Default)]
struct FifoState {
    next_ticket: u64,
    serving: u64,
    cancelled: BTreeSet<u64>,
}

struct FifoTicket {
    fifo: Arc<FifoSerial>,
    ticket: u64,
    pending: bool,
}

struct FifoPermit {
    fifo: Arc<FifoSerial>,
}

impl FifoSerial {
    fn reserve(fifo: &Arc<Self>) -> FifoTicket {
        let mut state = fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ticket = state.next_ticket;
        state.next_ticket = state.next_ticket.saturating_add(1);
        FifoTicket {
            fifo: Arc::clone(fifo),
            ticket,
            pending: true,
        }
    }

    fn cancel(&self, ticket: u64) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if ticket == state.serving {
            state.serving = state.serving.saturating_add(1);
            advance_cancelled(&mut state);
            self.changed.notify_all();
        } else {
            state.cancelled.insert(ticket);
        }
    }
}

impl FifoTicket {
    fn acquire(mut self) -> FifoPermit {
        let mut state = self
            .fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while state.serving != self.ticket {
            state = self
                .fifo
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        drop(state);
        self.pending = false;
        FifoPermit {
            fifo: Arc::clone(&self.fifo),
        }
    }
}

impl Drop for FifoTicket {
    fn drop(&mut self) {
        if self.pending {
            self.fifo.cancel(self.ticket);
        }
    }
}

impl Drop for FifoPermit {
    fn drop(&mut self) {
        let mut state = self
            .fifo
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.serving = state.serving.saturating_add(1);
        advance_cancelled(&mut state);
        self.fifo.changed.notify_all();
    }
}

fn advance_cancelled(state: &mut FifoState) {
    while state.cancelled.remove(&state.serving) {
        state.serving = state.serving.saturating_add(1);
    }
}

#[cfg(test)]
#[path = "foreign_tests.rs"]
mod tests;
