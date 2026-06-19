use super::{runtime_config, Object, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashMap;
use std::fmt;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

static OBJECT_STORE: LazyLock<Mutex<ObjectStore>> =
    LazyLock::new(|| Mutex::new(ObjectStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type ObjectHandle = (i64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonError {
    pub kind: String,
    pub exception_type: String,
    pub message: String,
    pub traceback: String,
    pub context: String,
}

impl PythonError {
    fn runtime(error: PythonRuntimeError) -> Self {
        Self {
            kind: "runtime".to_string(),
            exception_type: "SifrPythonRuntimeError".to_string(),
            message: error.to_string(),
            traceback: String::new(),
            context: String::new(),
        }
    }

    fn trust(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            kind: "trust".to_string(),
            exception_type: "SIFR-PYTRUST".to_string(),
            message: message.into(),
            traceback: String::new(),
            context: context.into(),
        }
    }

    fn closed(handle: i64) -> Self {
        Self {
            kind: "resource".to_string(),
            exception_type: "SifrPythonClosedObject".to_string(),
            message: format!("Python object handle {handle} is closed"),
            traceback: String::new(),
            context: "object handle lookup".to_string(),
        }
    }

    fn from_pyerr(
        py: Python<'_>,
        error: PyErr,
        kind: &'static str,
        context: impl Into<String>,
    ) -> Self {
        let exception_type = error
            .get_type(py)
            .name()
            .map_or_else(|_| "PythonError".to_string(), |name| name.to_string());
        let traceback = format_traceback(py, &error);
        Self {
            kind: kind.to_string(),
            exception_type,
            message: error.to_string(),
            traceback,
            context: context.into(),
        }
    }
}

impl fmt::Display for PythonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.context.is_empty() {
            write!(f, "{}: {}", self.exception_type, self.message)
        } else {
            write!(
                f,
                "{} during {}: {}",
                self.exception_type, self.context, self.message
            )
        }
    }
}

impl std::error::Error for PythonError {}

#[derive(Default)]
struct ObjectStore {
    next_handle: i64,
    next_nonce: u64,
    objects: HashMap<i64, ObjectEntry>,
}

struct ObjectEntry {
    token: i64,
    object: Object,
}

pub fn import_module(name: &str) -> Result<ObjectHandle, PythonError> {
    validate_import_policy(name)?;
    super::attach(|py| {
        py.import(name)
            .map_err(|error| PythonError::from_pyerr(py, error, "import", name))
            .and_then(|module| store_object(module.unbind().into()))
    })
    .map_err(PythonError::runtime)?
}

pub fn get_attr(object: ObjectHandle, name: &str) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object
            .bind(py)
            .getattr(name)
            .map_err(|error| PythonError::from_pyerr(py, error, "attribute", name))
            .and_then(|value| store_object(value.unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn get_item_str(object: ObjectHandle, key: &str) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object
            .bind(py)
            .get_item(key)
            .map_err(|error| PythonError::from_pyerr(py, error, "item", key))
            .and_then(|value| store_object(value.unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn call_object(
    object: ObjectHandle,
    args: &[ObjectHandle],
    kwargs: &[(&str, ObjectHandle)],
) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let callable = clone_handle(py, object)?;
        let tuple_args = args
            .iter()
            .map(|arg| clone_handle(py, *arg))
            .collect::<Result<Vec<_>, _>>()?;
        let tuple = PyTuple::new(py, tuple_args.iter())
            .map_err(|error| PythonError::from_pyerr(py, error, "conversion", "call args"))?;
        let kw_dict = PyDict::new(py);
        for (key, value_handle) in kwargs {
            let value = clone_handle(py, *value_handle)?;
            kw_dict
                .set_item(*key, value.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "conversion", *key))?;
        }
        callable
            .bind(py)
            .call(tuple, Some(&kw_dict))
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "call object"))
            .and_then(|value| store_object(value.unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn call_attr(
    object: ObjectHandle,
    name: &str,
    args: &[ObjectHandle],
    kwargs: &[(&str, ObjectHandle)],
) -> Result<ObjectHandle, PythonError> {
    let callable = get_attr(object, name)?;
    let result = call_object(callable, args, kwargs);
    let close_result = close_object(callable);
    match (result, close_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) | (Err(error), _) => Err(error),
    }
}

pub fn enter_context(object: ObjectHandle) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object
            .bind(py)
            .call_method0("__enter__")
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "__enter__"))
            .and_then(|value| store_object(value.unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn exit_context(object: ObjectHandle) -> Result<(), PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object
            .bind(py)
            .call_method1("__exit__", (py.None(), py.None(), py.None()))
            .map(|_| ())
            .map_err(|error| PythonError::from_pyerr(py, error, "call", "__exit__"))
    })
    .map_err(PythonError::runtime)?
}

pub fn close_object((handle, token): ObjectHandle) -> Result<(), PythonError> {
    let mut store = object_store()?;
    if store
        .objects
        .get(&handle)
        .is_some_and(|entry| entry.token == token)
    {
        store.objects.remove(&handle);
        Ok(())
    } else {
        Err(PythonError::closed(handle))
    }
}

fn validate_import_policy(name: &str) -> Result<(), PythonError> {
    let root = name
        .split('.')
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| PythonError::trust("Python import name is empty", "import root"))?;
    let config = runtime_config().map_err(PythonError::runtime)?;
    if !contains_root(&config.allowed_import_roots, root) {
        return Err(PythonError::trust(
            format!("Python import root '{root}' is not listed in [python].allow-imports"),
            name,
        ));
    }
    if !contains_root(&config.trusted_import_roots, root) {
        return Err(PythonError::trust(
            format!("Python import root '{root}' is not listed in [trust].python"),
            name,
        ));
    }
    if contains_root(&config.native_import_roots, root)
        && !contains_root(&config.trusted_native_roots, root)
    {
        return Err(PythonError::trust(
            format!("native Python import root '{root}' is not listed in [trust].python-native"),
            name,
        ));
    }
    Ok(())
}

fn contains_root(roots: &[String], root: &str) -> bool {
    roots
        .iter()
        .any(|candidate| candidate == root || candidate == "*")
}

fn store_object(object: Py<PyAny>) -> Result<ObjectHandle, PythonError> {
    let object = Object::new(object).map_err(PythonError::runtime)?;
    let mut store = object_store()?;
    store.next_handle = store.next_handle.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python object handle space exhausted".to_string(),
        ))
    })?;
    store.next_nonce = store.next_nonce.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python object token space exhausted".to_string(),
        ))
    })?;
    let handle = store.next_handle;
    let token = token_for(handle, store.next_nonce);
    store.objects.insert(handle, ObjectEntry { token, object });
    Ok((handle, token))
}

fn clone_handle(py: Python<'_>, (handle, token): ObjectHandle) -> Result<Py<PyAny>, PythonError> {
    object_store()?
        .objects
        .get(&handle)
        .filter(|entry| entry.token == token)
        .ok_or_else(|| PythonError::closed(handle))?
        .object
        .clone_ref(py)
        .map_err(PythonError::runtime)
}

fn token_for(handle: i64, nonce: u64) -> i64 {
    let hash = TOKEN_HASHER.hash_one((handle, nonce));
    i64::from_ne_bytes(hash.to_ne_bytes())
}

fn object_store() -> Result<MutexGuard<'static, ObjectStore>, PythonError> {
    OBJECT_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python object store is unavailable".to_string(),
        traceback: String::new(),
        context: "object store".to_string(),
    })
}

fn format_traceback(py: Python<'_>, error: &PyErr) -> String {
    py.import("traceback")
        .and_then(|traceback| {
            traceback.call_method1(
                "format_exception",
                (error.get_type(py), error.value(py), error.traceback(py)),
            )
        })
        .and_then(|formatted| formatted.extract::<Vec<String>>())
        .map(|parts| parts.join(""))
        .unwrap_or_default()
}

#[cfg(test)]
pub(super) fn reset_object_store_for_tests() {
    let mut store = object_store().expect("object store should be available");
    *store = ObjectStore::default();
}
