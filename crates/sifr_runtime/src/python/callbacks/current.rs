use super::execution::{
    CallbackExecutionError, collect_args, execution_error, python_error, result_object,
    validate_call_shape,
};
use super::{CallbackOwnerState, errors};
use crate::python::{ObjectHandle, PythonError, object_ops};
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

type CurrentTarget<'a> = dyn for<'py> Fn(
        u64,
        Python<'py>,
        &Bound<'py, PyTuple>,
        Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Py<PyAny>>
    + 'a;

#[derive(Clone, Copy)]
struct CurrentTargetPtr(*const CurrentTarget<'static>);

#[allow(clippy::transmute_ptr_to_ptr)]
unsafe fn erase_target_lifetime(target: *const CurrentTarget<'_>) -> CurrentTargetPtr {
    // SAFETY: callers must keep the target alive until this pointer is removed
    // from the thread-local registry and all admitted invocations have drained.
    CurrentTargetPtr(unsafe {
        std::mem::transmute::<*const CurrentTarget<'_>, *const CurrentTarget<'static>>(target)
    })
}

thread_local! {
    static CURRENT_TARGETS: RefCell<HashMap<u64, CurrentTargetPtr>> = RefCell::new(HashMap::new());
}

pub struct CurrentCallback<'a> {
    object: ObjectHandle,
    owner: CallbackOwnerState,
    target: Option<Box<CurrentTarget<'a>>>,
    token: u64,
    callback_id: u64,
    creator: std::thread::ThreadId,
    closed: Cell<bool>,
    _not_send: PhantomData<Rc<()>>,
}

static NEXT_CURRENT_TOKEN: AtomicU64 = AtomicU64::new(0);

impl CurrentCallback<'_> {
    #[must_use]
    pub fn object(&self) -> &ObjectHandle {
        &self.object
    }

    #[must_use]
    pub fn owner(&self) -> &CallbackOwnerState {
        &self.owner
    }

    pub fn close(&self) -> Result<(), PythonError> {
        if self.closed.get() {
            return Ok(());
        }
        if std::thread::current().id() != self.creator {
            return Err(errors::wrong_thread(
                self.owner.owner_id(),
                self.callback_id,
            ));
        }
        self.owner.close_call_scope()?;
        CURRENT_TARGETS.with(|targets| {
            targets.borrow_mut().remove(&self.token);
        });
        self.closed.set(true);
        object_ops::close_object(self.object.clone())
    }
}

impl Drop for CurrentCallback<'_> {
    fn drop(&mut self) {
        if self.close().is_err() && !self.closed.get() {
            // A safe caller can arrange for the wrapper to be dropped
            // reentrantly from its own non-Send handler through shared local
            // state. Closing correctly rejects that operation, but Drop must
            // then keep the currently executing target alive. Leaking only on
            // this contract violation preserves memory safety and leaves the
            // shell guarded by its still-open owner.
            if let Some(target) = self.target.take() {
                Box::leak(target);
            }
        }
    }
}

pub fn current_callback<'a, A, R, Decode, Handler, Encode>(
    callback_id: u64,
    expected_arity: usize,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<CurrentCallback<'a>, PythonError>
where
    A: 'a,
    R: 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + 'a,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + 'a,
{
    let owner = CallbackOwnerState::new_call_scoped()?;
    current_callback_with_owner(owner, callback_id, expected_arity, decode, handler, encode)
}

pub fn current_callback_with_owner<'a, A, R, Decode, Handler, Encode>(
    owner: CallbackOwnerState,
    callback_id: u64,
    expected_arity: usize,
    decode: Decode,
    handler: Handler,
    encode: Encode,
) -> Result<CurrentCallback<'a>, PythonError>
where
    A: 'a,
    R: 'a,
    Decode: Fn(Vec<ObjectHandle>) -> Result<A, PythonError> + 'a,
    Handler: Fn(u64, A) -> Result<R, CallbackExecutionError> + 'a,
    Encode: Fn(R) -> Result<ObjectHandle, PythonError> + 'a,
{
    let token = NEXT_CURRENT_TOKEN
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| errors::unavailable("current callback token space"))?
        .checked_add(1)
        .ok_or_else(|| errors::unavailable("current callback token space"))?;
    let target_owner = owner.clone();
    let target: Box<CurrentTarget<'a>> = Box::new(move |entry_sequence, py, args, kwargs| {
        validate_call_shape(args, kwargs, expected_arity)?;
        let args = collect_args(args).map_err(python_error)?;
        let decoded = decode(args).map_err(python_error)?;
        let result = handler(entry_sequence, decoded)
            .map_err(|error| execution_error(py, &target_owner, entry_sequence, error))?;
        let result = encode(result).map_err(python_error)?;
        result_object(py, result).map_err(python_error)
    });
    let target_ptr: *const CurrentTarget<'a> = &*target;
    // `CurrentCallback` owns the boxed target and removes this pointer from the
    // creator thread's registry before the box is dropped. Dereferencing the
    // lifetime-erased pointer remains confined to the guarded shell below.
    // SAFETY: the owning callback removes the pointer before dropping `target`.
    let target_ptr = unsafe { erase_target_lifetime(target_ptr) };
    CURRENT_TARGETS.with(|targets| {
        targets.borrow_mut().insert(token, target_ptr);
    });

    let creator = std::thread::current().id();
    let shell_creator = creator;
    let shell_owner = owner.clone();
    let object = crate::python::attach(|py| {
        let callback = PyCFunction::new_closure(
            py,
            Some(c"sifr_current_callback"),
            None,
            move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
                if std::thread::current().id() != shell_creator {
                    return Err(python_error(errors::wrong_thread(
                        shell_owner.owner_id(),
                        callback_id,
                    )));
                }
                let invocation = shell_owner
                    .accept(callback_id, false)
                    .map_err(python_error)?;
                let entry_sequence = invocation.entry_sequence();
                let _guard = invocation.enter().map_err(python_error)?;
                let target = CURRENT_TARGETS.with(|targets| targets.borrow().get(&token).copied());
                let Some(target) = target else {
                    return Err(python_error(errors::closed(shell_owner.owner_id())));
                };
                // The registry borrow must end before invoking user code: a
                // handler may create or close another current-thread callback.
                // SAFETY: registry membership is bounded by the owning
                // `CurrentCallback` as documented above.
                unsafe { (&*target.0)(entry_sequence, args.py(), args, kwargs) }
            },
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "current callback"))?;
        object_ops::store_object(callback.into_any().unbind())
    })
    .map_err(PythonError::runtime)
    .and_then(|outcome| outcome);
    let object = match object {
        Ok(object) => object,
        Err(error) => {
            CURRENT_TARGETS.with(|targets| {
                targets.borrow_mut().remove(&token);
            });
            return Err(error);
        }
    };

    Ok(CurrentCallback {
        object,
        owner,
        target: Some(target),
        token,
        callback_id,
        creator,
        closed: Cell::new(false),
        _not_send: PhantomData,
    })
}
