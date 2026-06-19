use super::{runtime_config, Object, PythonRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyTuple};
use pyo3::IntoPyObjectExt;
use std::collections::HashMap;
use std::fmt;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

static OBJECT_STORE: LazyLock<Mutex<ObjectStore>> =
    LazyLock::new(|| Mutex::new(ObjectStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type ObjectHandle = (i64, i64);

macro_rules! typed_container_conversions {
    ($($list:ident, $tuple:ident, $dict:ident => $ty:ty, $expected:literal);+ $(;)?) => {
        $(
            pub fn $list(object: ObjectHandle) -> Result<Vec<$ty>, PythonError> {
                copy_list(object, $expected, stringify!($list))
            }

            pub fn $tuple(object: ObjectHandle) -> Result<Vec<$ty>, PythonError> {
                copy_tuple(object, $expected, stringify!($tuple))
            }

            pub fn $dict(object: ObjectHandle) -> Result<HashMap<String, $ty>, PythonError> {
                copy_dict_str(object, $expected, stringify!($dict))
            }
        )+
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonError {
    pub kind: String,
    pub exception_type: String,
    pub message: String,
    pub traceback: String,
    pub context: String,
}

impl PythonError {
    pub(super) fn runtime(error: PythonRuntimeError) -> Self {
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

    pub(super) fn from_pyerr(
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

pub fn from_none() -> Result<ObjectHandle, PythonError> {
    super::attach(|py| store_object(py.None())).map_err(PythonError::runtime)?
}

pub fn from_bool(value: bool) -> Result<ObjectHandle, PythonError> {
    store_primitive(value, "from bool")
}

pub fn from_int(value: i64) -> Result<ObjectHandle, PythonError> {
    store_primitive(value, "from int")
}

pub fn from_float(value: f64) -> Result<ObjectHandle, PythonError> {
    store_primitive(value, "from float")
}

pub fn from_str(value: &str) -> Result<ObjectHandle, PythonError> {
    store_primitive(value, "from str")
}

pub fn from_bytes(value: &[u8]) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| store_object(PyBytes::new(py, value).into_any().unbind()))
        .map_err(PythonError::runtime)?
}

pub fn from_list(values: &[ObjectHandle]) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let values = clone_handles(py, values)?;
        PyList::new(py, values.iter())
            .map_err(|error| PythonError::from_pyerr(py, error, "conversion", "from_list"))
            .and_then(|list| store_object(list.into_any().unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn from_tuple(values: &[ObjectHandle]) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let values = clone_handles(py, values)?;
        PyTuple::new(py, values.iter())
            .map_err(|error| PythonError::from_pyerr(py, error, "conversion", "from_tuple"))
            .and_then(|tuple| store_object(tuple.into_any().unbind()))
    })
    .map_err(PythonError::runtime)?
}

pub fn from_dict_str(values: &[(&str, ObjectHandle)]) -> Result<ObjectHandle, PythonError> {
    store_dict(values, "from_dict_str")
}

pub fn from_record(values: &[(&str, ObjectHandle)]) -> Result<ObjectHandle, PythonError> {
    store_dict(values, "from_record")
}

pub fn to_none(object: ObjectHandle) -> Result<(), PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        if object.bind(py).is_none() {
            Ok(())
        } else {
            Err(conversion_error("expected Python None", "to_none"))
        }
    })
    .map_err(PythonError::runtime)?
}

pub fn to_bool(object: ObjectHandle) -> Result<bool, PythonError> {
    extract_handle(object, "bool", "to_bool")
}

pub fn to_int(object: ObjectHandle) -> Result<i64, PythonError> {
    extract_handle(object, "int", "to_int")
}

pub fn to_i8(object: ObjectHandle) -> Result<i8, PythonError> {
    extract_handle(object, "int8", "to_i8")
}

pub fn to_i16(object: ObjectHandle) -> Result<i16, PythonError> {
    extract_handle(object, "int16", "to_i16")
}

pub fn to_i32(object: ObjectHandle) -> Result<i32, PythonError> {
    extract_handle(object, "int32", "to_i32")
}

pub fn to_i64(object: ObjectHandle) -> Result<i64, PythonError> {
    extract_handle(object, "int64", "to_i64")
}

pub fn to_u8(object: ObjectHandle) -> Result<u8, PythonError> {
    extract_handle(object, "uint8", "to_u8")
}

pub fn to_u16(object: ObjectHandle) -> Result<u16, PythonError> {
    extract_handle(object, "uint16", "to_u16")
}

pub fn to_u32(object: ObjectHandle) -> Result<u32, PythonError> {
    extract_handle(object, "uint32", "to_u32")
}

pub fn to_u64(object: ObjectHandle) -> Result<u64, PythonError> {
    extract_handle(object, "uint64", "to_u64")
}

pub fn to_isize(object: ObjectHandle) -> Result<isize, PythonError> {
    extract_handle(object, "isize", "to_isize")
}

pub fn to_usize(object: ObjectHandle) -> Result<usize, PythonError> {
    extract_handle(object, "usize", "to_usize")
}

pub fn to_float(object: ObjectHandle) -> Result<f64, PythonError> {
    extract_handle(object, "float", "to_float")
}

pub fn to_str(object: ObjectHandle) -> Result<String, PythonError> {
    extract_handle(object, "str", "to_str")
}

pub fn to_bytes(object: ObjectHandle) -> Result<Vec<u8>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object
            .bind(py)
            .cast::<PyBytes>()
            .map(|bytes| bytes.as_bytes().to_vec())
            .map_err(|_| conversion_error("expected Python bytes", "to_bytes: expected bytes"))
    })
    .map_err(PythonError::runtime)?
}

typed_container_conversions! {
    copy_list_bool, copy_tuple_bool, copy_dict_str_bool => bool, "bool";
    copy_list_int, copy_tuple_int, copy_dict_str_int => i64, "int";
    copy_list_i32, copy_tuple_i32, copy_dict_str_i32 => i32, "int32";
    copy_list_u8, copy_tuple_u8, copy_dict_str_u8 => u8, "uint8";
    copy_list_float, copy_tuple_float, copy_dict_str_float => f64, "float";
    copy_list_str, copy_tuple_str, copy_dict_str_str => String, "str"
}

pub fn copy_list_bytes(object: ObjectHandle) -> Result<Vec<Vec<u8>>, PythonError> {
    copy_list_exact_bytes(object, "copy_list_bytes")
}

pub fn copy_tuple_bytes(object: ObjectHandle) -> Result<Vec<Vec<u8>>, PythonError> {
    copy_tuple_exact_bytes(object, "copy_tuple_bytes")
}

pub fn copy_dict_str_bytes(object: ObjectHandle) -> Result<HashMap<String, Vec<u8>>, PythonError> {
    copy_dict_str_exact_bytes(object, "copy_dict_str_bytes")
}

pub fn copy_record_fields(
    object: ObjectHandle,
    fields: &[&str],
) -> Result<Vec<(String, ObjectHandle)>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let object = object.bind(py);
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            let value = object.getattr(*field).or_else(|_| object.get_item(*field));
            let value = value.map_err(|_| {
                conversion_error(
                    format!("expected Python record field '{field}'"),
                    format!("copy_record_fields.{field}"),
                )
            })?;
            values.push(((*field).to_string(), value.unbind()));
        }
        let (names, objects): (Vec<_>, Vec<_>) = values.into_iter().unzip();
        let handles = store_objects(objects)?;
        Ok(names.into_iter().zip(handles).collect())
    })
    .map_err(PythonError::runtime)?
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

pub(super) fn store_object(object: Py<PyAny>) -> Result<ObjectHandle, PythonError> {
    let object = Object::new(object).map_err(PythonError::runtime)?;
    let mut store = object_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    store.objects.insert(handle, ObjectEntry { token, object });
    Ok((handle, token))
}

fn store_objects(objects: Vec<Py<PyAny>>) -> Result<Vec<ObjectHandle>, PythonError> {
    let objects = objects
        .into_iter()
        .map(Object::new)
        .collect::<Result<Vec<_>, _>>()
        .map_err(PythonError::runtime)?;
    let mut handles = Vec::with_capacity(objects.len());
    let mut entries = Vec::with_capacity(objects.len());
    let mut store = object_store()?;
    for object in objects {
        let (handle, token) = reserve_handle(&mut store)?;
        handles.push((handle, token));
        entries.push((handle, ObjectEntry { token, object }));
    }
    store.objects.extend(entries);
    Ok(handles)
}

fn reserve_handle(store: &mut ObjectStore) -> Result<ObjectHandle, PythonError> {
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
    Ok((
        store.next_handle,
        token_for(store.next_handle, store.next_nonce),
    ))
}

fn store_primitive<T>(value: T, context: &'static str) -> Result<ObjectHandle, PythonError>
where
    T: for<'py> IntoPyObjectExt<'py>,
{
    super::attach(|py| {
        value
            .into_py_any(py)
            .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))
            .and_then(store_object)
    })
    .map_err(PythonError::runtime)?
}

fn store_dict(
    values: &[(&str, ObjectHandle)],
    context: &'static str,
) -> Result<ObjectHandle, PythonError> {
    super::attach(|py| {
        let dict = PyDict::new(py);
        for (key, value_handle) in values {
            let value = clone_handle(py, *value_handle)?;
            dict.set_item(*key, value.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "conversion", *key))?;
        }
        store_object(dict.into_any().unbind())
    })
    .map_err(PythonError::runtime)?
    .map_err(|mut error| {
        if error.context.is_empty() {
            error.context = context.to_string();
        }
        error
    })
}

fn extract_handle<T>(
    object: ObjectHandle,
    expected: &'static str,
    context: &'static str,
) -> Result<T, PythonError>
where
    for<'py> T: FromPyObject<'py, 'py>,
{
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        object.bind(py).extract::<T>().map_err(|error| {
            let mut converted = PythonError::from_pyerr(py, error.into(), "conversion", context);
            converted.context = format!("{context}: expected {expected}");
            converted
        })
    })
    .map_err(PythonError::runtime)?
}

fn extract_py_value<'a, 'py, T>(
    py: Python<'py>,
    value: &'a Bound<'py, PyAny>,
    expected: &'static str,
    context: String,
) -> Result<T, PythonError>
where
    T: FromPyObject<'a, 'py>,
{
    value.extract::<T>().map_err(|error| {
        let mut converted = PythonError::from_pyerr(py, error.into(), "conversion", &context);
        converted.context = format!("{context}: expected {expected}");
        converted
    })
}

fn copy_list<T>(
    object: ObjectHandle,
    expected: &'static str,
    context: &'static str,
) -> Result<Vec<T>, PythonError>
where
    for<'a, 'py> T: FromPyObject<'a, 'py>,
{
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let list = object
            .bind(py)
            .cast::<PyList>()
            .map_err(|_| conversion_error("expected Python list", context))?;
        let mut converted = Vec::with_capacity(list.len());
        for (index, value) in list.iter().enumerate() {
            converted.push(extract_py_value(
                py,
                &value,
                expected,
                format!("{context}[{index}]"),
            )?);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn copy_tuple<T>(
    object: ObjectHandle,
    expected: &'static str,
    context: &'static str,
) -> Result<Vec<T>, PythonError>
where
    for<'a, 'py> T: FromPyObject<'a, 'py>,
{
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let tuple = object
            .bind(py)
            .cast::<PyTuple>()
            .map_err(|_| conversion_error("expected Python tuple", context))?;
        let mut converted = Vec::with_capacity(tuple.len());
        for (index, value) in tuple.iter().enumerate() {
            converted.push(extract_py_value(
                py,
                &value,
                expected,
                format!("{context}[{index}]"),
            )?);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn copy_dict_str<T>(
    object: ObjectHandle,
    expected: &'static str,
    context: &'static str,
) -> Result<HashMap<String, T>, PythonError>
where
    for<'a, 'py> T: FromPyObject<'a, 'py>,
{
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let dict = object
            .bind(py)
            .cast::<PyDict>()
            .map_err(|_| conversion_error("expected Python dict", context))?;
        let mut converted = HashMap::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key = extract_py_value::<String>(py, &key, "str", format!("{context}.<key>"))?;
            let value = extract_py_value(py, &value, expected, format!("{context}[{key:?}]"))?;
            converted.insert(key, value);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn copy_list_exact_bytes(
    object: ObjectHandle,
    context: &'static str,
) -> Result<Vec<Vec<u8>>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let list = object
            .bind(py)
            .cast::<PyList>()
            .map_err(|_| conversion_error("expected Python list", context))?;
        let mut converted = Vec::with_capacity(list.len());
        for (index, value) in list.iter().enumerate() {
            converted.push(extract_exact_bytes(value, format!("{context}[{index}]"))?);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn copy_tuple_exact_bytes(
    object: ObjectHandle,
    context: &'static str,
) -> Result<Vec<Vec<u8>>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let tuple = object
            .bind(py)
            .cast::<PyTuple>()
            .map_err(|_| conversion_error("expected Python tuple", context))?;
        let mut converted = Vec::with_capacity(tuple.len());
        for (index, value) in tuple.iter().enumerate() {
            converted.push(extract_exact_bytes(value, format!("{context}[{index}]"))?);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn copy_dict_str_exact_bytes(
    object: ObjectHandle,
    context: &'static str,
) -> Result<HashMap<String, Vec<u8>>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let dict = object
            .bind(py)
            .cast::<PyDict>()
            .map_err(|_| conversion_error("expected Python dict", context))?;
        let mut converted = HashMap::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key = extract_py_value::<String>(py, &key, "str", format!("{context}.<key>"))?;
            let value = extract_exact_bytes(value, format!("{context}[{key:?}]"))?;
            converted.insert(key, value);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

fn extract_exact_bytes(value: Bound<'_, PyAny>, context: String) -> Result<Vec<u8>, PythonError> {
    value
        .cast::<PyBytes>()
        .map(|bytes| bytes.as_bytes().to_vec())
        .map_err(|_| conversion_error("expected Python bytes", context))
}

fn conversion_error(message: impl Into<String>, context: impl Into<String>) -> PythonError {
    PythonError {
        kind: "conversion".to_string(),
        exception_type: "SifrPythonTypeConversionError".to_string(),
        message: message.into(),
        traceback: String::new(),
        context: context.into(),
    }
}

fn clone_handles(py: Python<'_>, values: &[ObjectHandle]) -> Result<Vec<Py<PyAny>>, PythonError> {
    values
        .iter()
        .map(|value| clone_handle(py, *value))
        .collect::<Result<Vec<_>, _>>()
}

pub(super) fn clone_handle(
    py: Python<'_>,
    (handle, token): ObjectHandle,
) -> Result<Py<PyAny>, PythonError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        initialize_runtime, reset_runtime_state_for_tests, shutdown_diagnostics, test_config,
        test_guard, PythonRuntimeDiagnostics,
    };

    #[test]
    fn primitive_conversion_round_trips_and_rejects_fixed_width_overflow() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("primitive-conversion")).expect("init should succeed");

        let none = from_none().expect("None object should be stored");
        to_none(none).expect("None should convert to None");
        close_object(none).expect("None object should close");

        let flag = from_bool(true).expect("bool object should be stored");
        assert!(to_bool(flag).expect("bool should convert"));
        close_object(flag).expect("bool object should close");

        let integer = from_int(127).expect("int object should be stored");
        assert_eq!(to_int(integer).expect("int should convert"), 127);
        assert_eq!(to_i8(integer).expect("int8 should convert"), 127);
        assert_eq!(to_u8(integer).expect("uint8 should convert"), 127);
        close_object(integer).expect("int object should close");

        let too_wide = from_int(256).expect("wide int object should be stored");
        let overflow = to_u8(too_wide).expect_err("uint8 overflow should fail");
        assert_eq!(overflow.kind, "conversion");
        assert!(overflow.context.contains("uint8"));
        close_object(too_wide).expect("wide int object should close");

        let float = from_float(1.25).expect("float object should be stored");
        assert_eq!(to_float(float).expect("float should convert"), 1.25);
        close_object(float).expect("float object should close");

        let text = from_str("sifr").expect("str object should be stored");
        assert_eq!(to_str(text).expect("str should convert"), "sifr");
        close_object(text).expect("str object should close");

        let bytes = from_bytes(&[1, 2, 3]).expect("bytes object should be stored");
        assert_eq!(
            to_bytes(bytes).expect("bytes should convert"),
            vec![1, 2, 3]
        );
        close_object(bytes).expect("bytes object should close");

        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn explicit_container_copy_conversions_preserve_nested_paths() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("container-conversion")).expect("init should succeed");

        let first = from_int(1).expect("int object should be stored");
        let second = from_int(2).expect("int object should be stored");
        let list = from_list(&[first, second]).expect("list should be stored");
        assert_eq!(copy_list_int(list).expect("list should copy"), vec![1, 2]);

        let tuple = from_tuple(&[first, second]).expect("tuple should be stored");
        assert_eq!(
            copy_tuple_i32(tuple).expect("tuple should copy"),
            vec![1, 2]
        );

        let too_wide = from_int(256).expect("wide int object should be stored");
        let bad_list = from_list(&[first, too_wide]).expect("bad list should be stored");
        let overflow = copy_list_u8(bad_list).expect_err("nested overflow should fail");
        assert_eq!(overflow.kind, "conversion");
        assert!(overflow.context.contains("copy_list_u8[1]"));
        assert!(overflow.context.contains("uint8"));

        let dict =
            from_dict_str(&[("first", first), ("second", second)]).expect("dict should be stored");
        let copied = copy_dict_str_int(dict).expect("dict should copy");
        assert_eq!(copied.get("first"), Some(&1));
        assert_eq!(copied.get("second"), Some(&2));

        let record = from_record(&[("answer", second)]).expect("record should be stored");
        let fields = copy_record_fields(record, &["answer"]).expect("record should copy fields");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "answer");
        assert_eq!(to_int(fields[0].1).expect("field should convert"), 2);

        for handle in [
            first,
            second,
            list,
            tuple,
            too_wide,
            bad_list,
            dict,
            record,
            fields[0].1,
        ] {
            close_object(handle).expect("object should close");
        }

        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }
}
