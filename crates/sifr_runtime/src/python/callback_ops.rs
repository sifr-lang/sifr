use super::object_ops::{clone_handle, close_object, store_object};
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

static CALLBACK_STORE: LazyLock<Mutex<CallbackStore>> =
    LazyLock::new(|| Mutex::new(CallbackStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type CallbackHandle = (i64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonCallbackMetadata {
    pub handle: i64,
    pub token: i64,
    pub object_handle: i64,
    pub object_token: i64,
    pub kind: String,
}

#[derive(Default)]
struct CallbackStore {
    next_handle: i64,
    next_nonce: u64,
    callbacks: HashMap<i64, CallbackEntry>,
}

struct CallbackEntry {
    token: i64,
    kind: CallbackKind,
    object: ObjectHandle,
    target: CallbackTarget,
    invocations: usize,
}

#[derive(Clone, Copy)]
enum CallbackKind {
    Local,
    Threadsafe,
}

type SifrCallback =
    Arc<dyn Fn(ObjectHandle) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static>;

#[derive(Clone)]
enum CallbackTarget {
    Echo,
    Sifr(SifrCallback),
}

impl CallbackKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Threadsafe => "threadsafe",
        }
    }
}

pub fn local_callback_echo() -> Result<PythonCallbackMetadata, PythonError> {
    create_callback(CallbackKind::Local, CallbackTarget::Echo)
}

pub fn threadsafe_callback_echo() -> Result<PythonCallbackMetadata, PythonError> {
    create_callback(CallbackKind::Threadsafe, CallbackTarget::Echo)
}

pub fn local_callback<F>(callback: F) -> Result<PythonCallbackMetadata, PythonError>
where
    F: Fn(ObjectHandle) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static,
{
    create_callback(
        CallbackKind::Local,
        CallbackTarget::Sifr(Arc::new(callback)),
    )
}

pub fn threadsafe_callback<F>(callback: F) -> Result<PythonCallbackMetadata, PythonError>
where
    F: Fn(ObjectHandle) -> Result<ObjectHandle, PythonError> + Send + Sync + 'static,
{
    create_callback(
        CallbackKind::Threadsafe,
        CallbackTarget::Sifr(Arc::new(callback)),
    )
}

pub fn close_callback((handle, token): CallbackHandle) -> Result<(), PythonError> {
    let entry = super::attach(|_py| {
        let mut store = callback_store()?;
        if store
            .callbacks
            .get(&handle)
            .is_some_and(|entry| entry.token == token)
        {
            Ok(store.callbacks.remove(&handle))
        } else {
            Err(closed_error(handle))
        }
    })
    .map_err(PythonError::runtime)??;
    if let Some(entry) = entry {
        super::update_object_count(-1).map_err(PythonError::runtime)?;
        close_object(entry.object)?;
    }
    Ok(())
}

fn create_callback(
    kind: CallbackKind,
    target: CallbackTarget,
) -> Result<PythonCallbackMetadata, PythonError> {
    let (handle, token) = {
        let mut store = callback_store()?;
        reserve_handle(&mut store)?
    };
    let object = super::attach(|py| {
        let callback = PyCFunction::new_closure(
            py,
            Some(c"sifr_callback"),
            None,
            move |args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>| {
                invoke_callback(args.py(), (handle, token), args)
            },
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "callback", "create callback"))?;
        store_object(callback.into_any().unbind())
    })
    .map_err(PythonError::runtime)??;
    super::update_object_count(1).map_err(PythonError::runtime)?;
    let mut store = callback_store()?;
    store.callbacks.insert(
        handle,
        CallbackEntry {
            token,
            kind,
            object,
            target,
            invocations: 0,
        },
    );
    Ok(PythonCallbackMetadata {
        handle,
        token,
        object_handle: object.0,
        object_token: object.1,
        kind: kind.label().to_string(),
    })
}

fn invoke_callback(
    py: Python<'_>,
    (handle, token): CallbackHandle,
    args: &Bound<'_, PyTuple>,
) -> PyResult<Py<PyAny>> {
    let target = {
        let mut store = CALLBACK_STORE
            .lock()
            .map_err(|_| PyRuntimeError::new_err("Sifr callback store is unavailable"))?;
        let entry = store
            .callbacks
            .get_mut(&handle)
            .filter(|entry| entry.token == token)
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!("Sifr callback handle {handle} is closed"))
            })?;
        entry.invocations = entry
            .invocations
            .checked_add(1)
            .ok_or_else(|| PyRuntimeError::new_err("Sifr callback invocation count overflowed"))?;
        if matches!(entry.kind, CallbackKind::Local) && super::python_call_depth() == 0 {
            return Err(PyRuntimeError::new_err(
                "Sifr local callback escaped its active call scope",
            ));
        }
        entry.target.clone()
    };
    match target {
        CallbackTarget::Echo => echo_first_arg(py, args),
        CallbackTarget::Sifr(callback) => invoke_sifr_callback(py, args, &callback),
    }
}

fn echo_first_arg(py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
    if args.is_empty() {
        Ok(py.None())
    } else {
        args.get_item(0).map(Bound::unbind)
    }
}

fn invoke_sifr_callback(
    py: Python<'_>,
    args: &Bound<'_, PyTuple>,
    callback: &SifrCallback,
) -> PyResult<Py<PyAny>> {
    let arg_handle = if args.is_empty() {
        store_object(py.None()).map_err(py_runtime_error)?
    } else {
        store_object(args.get_item(0)?.unbind()).map_err(py_runtime_error)?
    };
    let result_handle = match callback(arg_handle) {
        Ok(result_handle) => result_handle,
        Err(error) => {
            let _ignored = close_object(arg_handle);
            return Err(py_runtime_error(error));
        }
    };
    let result = match clone_handle(py, result_handle) {
        Ok(result) => result,
        Err(error) => {
            if result_handle != arg_handle {
                let _ignored = close_object(arg_handle);
            }
            let _ignored = close_object(result_handle);
            return Err(py_runtime_error(error));
        }
    };
    if result_handle != arg_handle {
        let _ignored = close_object(arg_handle);
    }
    let _ignored = close_object(result_handle);
    Ok(result)
}

fn py_runtime_error(error: PythonError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn reserve_handle(store: &mut CallbackStore) -> Result<CallbackHandle, PythonError> {
    store.next_handle = store.next_handle.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python callback handle space exhausted".to_string(),
        ))
    })?;
    store.next_nonce = store.next_nonce.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python callback token space exhausted".to_string(),
        ))
    })?;
    Ok((
        store.next_handle,
        token_for(store.next_handle, store.next_nonce),
    ))
}

fn token_for(handle: i64, nonce: u64) -> i64 {
    let hash = TOKEN_HASHER.hash_one((handle, nonce));
    i64::from_ne_bytes(hash.to_ne_bytes())
}

fn closed_error(handle: i64) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonClosedCallback".to_string(),
        message: format!("Python callback handle {handle} is closed"),
        traceback: String::new(),
        context: "callback handle lookup".to_string(),
    }
}

fn callback_store() -> Result<MutexGuard<'static, CallbackStore>, PythonError> {
    CALLBACK_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python callback store is unavailable".to_string(),
        traceback: String::new(),
        context: "callback store".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        call_object, close_object, from_int, initialize_runtime, reset_runtime_state_for_tests,
        resource_diagnostics, test_config, test_guard, to_int, PythonResourceDiagnostics,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn local_callback_allows_same_stack_reentry_and_rejects_escape() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("local-callback")).expect("init should succeed");

        let callback = local_callback_echo().expect("callback should create");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 2,
                leaked_objects: 0,
            }
        );
        let arg = from_int(7).expect("arg should store");
        for _ in 0..2 {
            let result = call_object((callback.object_handle, callback.object_token), &[arg], &[])
                .expect("same-stack local callback call should succeed");
            assert_eq!(to_int(result).expect("result should convert"), 7);
            close_object(result).expect("result should close");
        }
        super::super::attach(|py| {
            let callable = super::super::object_ops::clone_handle(
                py,
                (callback.object_handle, callback.object_token),
            )?;
            callable
                .bind(py)
                .call1((7_i64,))
                .map(|_| ())
                .map_err(|error| PythonError::from_pyerr(py, error, "call", "escaped local"))
        })
        .map_err(PythonError::runtime)
        .and_then(|result| result)
        .expect_err("local callback escape should fail");
        close_object(arg).expect("arg should close");
        close_callback((callback.handle, callback.token)).expect("callback should close");
    }

    #[test]
    fn registered_callback_invokes_sifr_handler() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("registered-callback")).expect("init should succeed");

        let callback = local_callback(|arg| {
            let value = to_int(arg)?;
            close_object(arg)?;
            from_int(value + 1)
        })
        .expect("callback should create");
        let arg = from_int(7).expect("arg should store");
        let result = call_object((callback.object_handle, callback.object_token), &[arg], &[])
            .expect("registered callback call should succeed");
        assert_eq!(to_int(result).expect("result should convert"), 8);
        close_object(result).expect("result should close");
        close_object(arg).expect("arg should close");
        close_callback((callback.handle, callback.token)).expect("callback should close");
    }

    #[test]
    fn malformed_callback_result_releases_temporary_argument() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("callback-stale-result")).expect("init should succeed");
        let stale = from_int(99).expect("stale result should store");
        close_object(stale).expect("stale result should close");
        let callback = local_callback(move |_arg| Ok(stale)).expect("callback should create");
        let arg = from_int(7).expect("arg should store");
        let before = resource_diagnostics().expect("diagnostics should be available");

        call_object((callback.object_handle, callback.object_token), &[arg], &[])
            .expect_err("stale callback result should fail");

        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            before
        );
        close_object(arg).expect("arg should close");
        close_callback((callback.handle, callback.token)).expect("callback should close");
    }

    #[test]
    fn threadsafe_callback_survives_repeated_and_background_calls() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("threadsafe-callback")).expect("init should succeed");

        let callback = threadsafe_callback_echo().expect("callback should create");
        let arg = from_int(11).expect("arg should store");
        for _ in 0..2 {
            let result = call_object((callback.object_handle, callback.object_token), &[arg], &[])
                .expect("threadsafe callback should repeat");
            assert_eq!(to_int(result).expect("result should convert"), 11);
            close_object(result).expect("result should close");
        }
        super::super::attach(|py| {
            let callable = super::super::object_ops::clone_handle(
                py,
                (callback.object_handle, callback.object_token),
            )?;
            let globals = PyDict::new(py);
            globals
                .set_item("CALLBACK", callable.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "callback", "thread setup"))?;
            py.run(
                cr#"
import threading
result = []
def run():
    result.append(CALLBACK(19))
thread = threading.Thread(target=run)
thread.start()
thread.join()
assert len(result) == 1
assert result[0] == 19
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "callback", "thread call"))?;
            Ok(())
        })
        .map_err(PythonError::runtime)
        .and_then(|result| result)
        .expect("background call should work");
        close_object(arg).expect("arg should close");
        close_callback((callback.handle, callback.token)).expect("callback should close");
    }

    #[test]
    fn callback_close_and_after_close_are_deterministic() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("callback-close")).expect("init should succeed");

        let callback = threadsafe_callback_echo().expect("callback should create");
        close_callback((callback.handle, callback.token)).expect("callback should close");
        let closed = close_callback((callback.handle, callback.token))
            .expect_err("second close should fail");
        assert_eq!(closed.kind, "resource");
        assert_eq!(closed.exception_type, "SifrPythonClosedCallback");
        let arg = from_int(1).expect("arg should store");
        let after_close = call_object((callback.object_handle, callback.object_token), &[arg], &[])
            .expect_err("calling closed callback object should fail");
        assert_eq!(after_close.kind, "resource");
        close_object(arg).expect("arg should close");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn object_destructor_can_reenter_callback_and_object_stores() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("destructor-reentry")).expect("init should succeed");
        let invocations = Arc::new(AtomicUsize::new(0));
        let invocation_counter = Arc::clone(&invocations);
        let callback = threadsafe_callback(move |arg| {
            invocation_counter.fetch_add(1, Ordering::SeqCst);
            close_object(arg)?;
            super::super::from_none()
        })
        .expect("callback should create");
        let object = super::super::attach(|py| {
            let callable = super::super::object_ops::clone_handle(
                py,
                (callback.object_handle, callback.object_token),
            )?;
            let globals = PyDict::new(py);
            globals
                .set_item("CALLBACK", callable.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "callback", "destructor"))?;
            py.run(
                cr#"
class ReentrantDestructor:
    def __del__(self):
        CALLBACK(1)
value = ReentrantDestructor()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "callback", "destructor"))?;
            let value = globals
                .get_item("value")
                .map_err(|error| PythonError::from_pyerr(py, error, "callback", "destructor"))?
                .ok_or_else(|| {
                    PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
                        "destructor fixture did not create a value".to_string(),
                    ))
                })?;
            let object = super::super::object_ops::store_object(value.unbind())?;
            globals
                .del_item("value")
                .map_err(|error| PythonError::from_pyerr(py, error, "callback", "destructor"))?;
            Ok(object)
        })
        .map_err(PythonError::runtime)
        .and_then(|result| result)
        .expect("destructor object should be stored");

        close_object(object).expect("destructor object should close");

        assert_eq!(invocations.load(Ordering::SeqCst), 1);
        close_callback((callback.handle, callback.token)).expect("callback should close");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }
}
