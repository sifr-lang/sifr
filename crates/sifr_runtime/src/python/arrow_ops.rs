use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyTuple};
use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

const ARROW_SCHEMA_NAME: &CStr = c"arrow_schema";
const ARROW_ARRAY_NAME: &CStr = c"arrow_array";
const ARROW_STREAM_NAME: &CStr = c"arrow_array_stream";

static ARROW_STORE: LazyLock<Mutex<ArrowStore>> =
    LazyLock::new(|| Mutex::new(ArrowStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type ArrowHandle = (i64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonArrowCapsuleMetadata {
    pub handle: i64,
    pub token: i64,
    pub kind: String,
    pub capsule_names: Vec<String>,
    pub producer_module: String,
    pub producer_type: String,
    pub copy_possible: bool,
}

#[derive(Default)]
struct ArrowStore {
    next_handle: i64,
    next_nonce: u64,
    capsules: HashMap<i64, ArrowEntry>,
}

struct ArrowEntry {
    token: i64,
    _capsules: TrackedArrowCapsules,
    capsule_names: Vec<String>,
}

struct TrackedArrowCapsules {
    capsules: Vec<Py<PyAny>>,
}

impl Drop for TrackedArrowCapsules {
    fn drop(&mut self) {
        self.capsules.clear();
        let _ignored = super::update_object_count(-1);
    }
}

pub fn arrow_array(object: &ObjectHandle) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Array)
}

pub fn arrow_stream(object: &ObjectHandle) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Stream)
}

pub fn arrow_schema(object: &ObjectHandle) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Schema)
}

pub fn release_arrow((handle, token): ArrowHandle) -> Result<(), PythonError> {
    let entry = {
        let mut store = arrow_store()?;
        if store
            .capsules
            .get(&handle)
            .is_some_and(|entry| entry.token == token)
        {
            store.capsules.remove(&handle)
        } else {
            return Err(closed_error(handle));
        }
    };
    super::attach(|_py| drop(entry)).map_err(PythonError::runtime)?;
    Ok(())
}

pub fn arrow_capsule_names(handle: ArrowHandle) -> Result<Vec<String>, PythonError> {
    let store = arrow_store()?;
    store
        .capsules
        .get(&handle.0)
        .filter(|entry| entry.token == handle.1)
        .map(|entry| entry.capsule_names.clone())
        .ok_or_else(|| closed_error(handle.0))
}

#[derive(Clone, Copy)]
enum ArrowKind {
    Array,
    Stream,
    Schema,
}

impl ArrowKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Array => "array",
            Self::Stream => "stream",
            Self::Schema => "schema",
        }
    }

    const fn method(self) -> &'static str {
        match self {
            Self::Array => "__arrow_c_array__",
            Self::Stream => "__arrow_c_stream__",
            Self::Schema => "__arrow_c_schema__",
        }
    }
}

fn acquire_arrow_capsules(
    object: &ObjectHandle,
    kind: ArrowKind,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let producer = producer_info(object.bind(py));
        let exported = call_export_method(py, object.bind(py), kind)
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", kind.method()))?;
        let capsules = extract_capsules(py, &exported, kind)?;
        let capsule_names = capsule_names(kind);
        store_arrow_capsules(capsules, kind, capsule_names, producer)
    })
    .map_err(PythonError::runtime)?
}

fn call_export_method<'py>(
    py: Python<'py>,
    object: &Bound<'py, PyAny>,
    kind: ArrowKind,
) -> PyResult<Bound<'py, PyAny>> {
    match kind {
        ArrowKind::Array | ArrowKind::Stream => object.call_method1(kind.method(), (py.None(),)),
        ArrowKind::Schema => object.call_method0(kind.method()),
    }
}

fn extract_capsules(
    py: Python<'_>,
    exported: &Bound<'_, PyAny>,
    kind: ArrowKind,
) -> Result<Vec<Py<PyAny>>, PythonError> {
    match kind {
        ArrowKind::Array => {
            let tuple = exported.cast::<PyTuple>().map_err(|_| {
                arrow_error("__arrow_c_array__ must return (arrow_schema, arrow_array)")
            })?;
            if tuple.len() != 2 {
                return Err(arrow_error(
                    "__arrow_c_array__ must return exactly two capsules",
                ));
            }
            let schema = tuple.get_item(0).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "__arrow_c_array__ schema")
            })?;
            let array = tuple.get_item(1).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "__arrow_c_array__ array")
            })?;
            validate_capsule(py, &schema, ARROW_SCHEMA_NAME, "__arrow_c_array__ schema")?;
            validate_capsule(py, &array, ARROW_ARRAY_NAME, "__arrow_c_array__ array")?;
            Ok(vec![schema.unbind(), array.unbind()])
        }
        ArrowKind::Stream => {
            validate_capsule(py, exported, ARROW_STREAM_NAME, "__arrow_c_stream__")?;
            Ok(vec![exported.clone().unbind()])
        }
        ArrowKind::Schema => {
            validate_capsule(py, exported, ARROW_SCHEMA_NAME, "__arrow_c_schema__")?;
            Ok(vec![exported.clone().unbind()])
        }
    }
}

fn validate_capsule(
    py: Python<'_>,
    object: &Bound<'_, PyAny>,
    expected_name: &'static CStr,
    context: &'static str,
) -> Result<(), PythonError> {
    let capsule = object.cast::<PyCapsule>().map_err(|_| {
        arrow_error(format!(
            "{context} returned a non-PyCapsule value; expected {}",
            expected_name.to_string_lossy()
        ))
    })?;
    let actual_name = unsafe { ffi::PyCapsule_GetName(capsule.as_ptr()) };
    if actual_name.is_null() {
        return Err(arrow_error(format!(
            "{context} capsule has no name; expected {}",
            expected_name.to_string_lossy()
        )));
    }
    let actual_name = unsafe { CStr::from_ptr(actual_name) };
    if actual_name != expected_name {
        return Err(arrow_error(format!(
            "{context} capsule has name '{}'; expected '{}'",
            actual_name.to_string_lossy(),
            expected_name.to_string_lossy()
        )));
    }
    capsule
        .pointer_checked(Some(expected_name))
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", context))?;
    let destructor = unsafe { ffi::PyCapsule_GetDestructor(capsule.as_ptr()) };
    if destructor.is_none() {
        let _stale_error = PyErr::take(py);
        return Err(arrow_error(format!(
            "{context} capsule '{}' has no destructor",
            expected_name.to_string_lossy()
        )));
    }
    Ok(())
}

fn store_arrow_capsules(
    capsules: Vec<Py<PyAny>>,
    kind: ArrowKind,
    capsule_names: Vec<String>,
    producer: ProducerInfo,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    let mut store = arrow_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    super::update_object_count(1).map_err(PythonError::runtime)?;
    store.capsules.insert(
        handle,
        ArrowEntry {
            token,
            _capsules: TrackedArrowCapsules { capsules },
            capsule_names: capsule_names.clone(),
        },
    );
    Ok(PythonArrowCapsuleMetadata {
        handle,
        token,
        kind: kind.label().to_string(),
        capsule_names,
        producer_module: producer.module,
        producer_type: producer.name,
        copy_possible: producer.copy_possible,
    })
}

struct ProducerInfo {
    module: String,
    name: String,
    copy_possible: bool,
}

fn producer_info(object: &Bound<'_, PyAny>) -> ProducerInfo {
    let type_object = object.get_type();
    let module = type_object
        .getattr("__module__")
        .and_then(|value| value.extract::<String>())
        .unwrap_or_default();
    let name = type_object
        .name()
        .map_or_else(|_| "object".to_string(), |name| name.to_string());
    let copy_possible = !is_proven_zero_copy_module(&module);
    ProducerInfo {
        module,
        name,
        copy_possible,
    }
}

fn is_proven_zero_copy_module(module: &str) -> bool {
    // Only audited zero-copy producers belong here.
    module == "pyarrow"
        || module.starts_with("pyarrow.")
        || module == "polars"
        || module.starts_with("polars.")
}

fn capsule_names(kind: ArrowKind) -> Vec<String> {
    match kind {
        ArrowKind::Array => vec!["arrow_schema".to_string(), "arrow_array".to_string()],
        ArrowKind::Stream => vec!["arrow_array_stream".to_string()],
        ArrowKind::Schema => vec!["arrow_schema".to_string()],
    }
}

fn reserve_handle(store: &mut ArrowStore) -> Result<ArrowHandle, PythonError> {
    store.next_handle = store.next_handle.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python Arrow handle space exhausted".to_string(),
        ))
    })?;
    store.next_nonce = store.next_nonce.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python Arrow token space exhausted".to_string(),
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
        exception_type: "SifrPythonClosedArrowCapsule".to_string(),
        message: format!("Python Arrow capsule handle {handle} is closed"),
        traceback: String::new(),
        context: "Arrow capsule handle lookup".to_string(),
        replay: None,
    }
}

fn arrow_error(message: impl Into<String>) -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "SifrPythonArrowCapsuleError".to_string(),
        message: message.into(),
        traceback: String::new(),
        context: "Arrow PyCapsule validation".to_string(),
        replay: None,
    }
}

fn arrow_store() -> Result<MutexGuard<'static, ArrowStore>, PythonError> {
    ARROW_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python Arrow capsule store is unavailable".to_string(),
        traceback: String::new(),
        context: "Arrow capsule store".to_string(),
        replay: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        close_object, initialize_runtime, reset_runtime_state_for_tests, resource_diagnostics,
        test_config, test_guard, PythonResourceDiagnostics,
    };
    use pyo3::types::{PyCapsule, PyDict};

    #[test]
    fn arrow_array_stream_schema_track_metadata_and_release() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("arrow-capsules")).expect("init should succeed");

        let object = exporter().expect("exporter should be stored");
        let array = arrow_array(&object).expect("array capsules should export");
        let stream = arrow_stream(&object).expect("stream capsule should export");
        let schema = arrow_schema(&object).expect("schema capsule should export");

        assert_eq!(array.kind, "array");
        assert_eq!(array.capsule_names, ["arrow_schema", "arrow_array"]);
        assert!(!array.copy_possible);
        assert_eq!(stream.capsule_names, ["arrow_array_stream"]);
        assert_eq!(schema.capsule_names, ["arrow_schema"]);
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 4,
                leaked_objects: 0,
            }
        );

        release_arrow((array.handle, array.token)).expect("array should release");
        release_arrow((stream.handle, stream.token)).expect("stream should release");
        release_arrow((schema.handle, schema.token)).expect("schema should release");
        close_object(object).expect("object should close");
    }

    #[test]
    fn arrow_marks_pandas_like_producers_copy_possible() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("arrow-copy-possible")).expect("init should succeed");

        let object = pandas_exporter().expect("exporter should be stored");
        let stream = arrow_stream(&object).expect("stream capsule should export");

        assert!(stream.copy_possible);
        release_arrow((stream.handle, stream.token)).expect("stream should release");
        close_object(object).expect("object should close");
    }

    #[test]
    fn arrow_rejects_malformed_capsule_and_double_release() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("arrow-errors")).expect("init should succeed");

        let malformed = malformed_exporter().expect("malformed exporter should be stored");
        let error = arrow_array(&malformed).expect_err("wrong capsule names must fail");
        assert_eq!(error.kind, "zero-copy");
        assert_eq!(error.exception_type, "SifrPythonArrowCapsuleError");

        let object = exporter().expect("exporter should be stored");
        let stream = arrow_stream(&object).expect("stream capsule should export");
        release_arrow((stream.handle, stream.token)).expect("stream should release");
        let closed =
            release_arrow((stream.handle, stream.token)).expect_err("second release should fail");
        assert_eq!(closed.kind, "resource");
        assert_eq!(closed.exception_type, "SifrPythonClosedArrowCapsule");

        close_object(malformed).expect("malformed object should close");
        close_object(object).expect("object should close");
    }

    #[test]
    fn arrow_rejects_capsules_without_destructors() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("arrow-no-destructor")).expect("init should succeed");

        let malformed = exporter_without_destructor().expect("exporter should be stored");
        let error = arrow_schema(&malformed).expect_err("missing destructor must fail");

        assert_eq!(error.kind, "zero-copy");
        assert_eq!(error.exception_type, "SifrPythonArrowCapsuleError");
        assert!(error.message.contains("has no destructor"));
        close_object(malformed).expect("malformed object should close");
    }

    fn exporter() -> Result<ObjectHandle, PythonError> {
        exporter_with_schema("pyarrow.lib", ARROW_SCHEMA_NAME)
    }

    fn pandas_exporter() -> Result<ObjectHandle, PythonError> {
        super::super::attach(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item("STREAM", capsule_any(py, ARROW_STREAM_NAME)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test stream"))?;
            py.run(
                cr#"
class PandasExporter:
    __module__ = "pandas.core.frame"

    def __arrow_c_stream__(self, requested_schema=None):
        return STREAM

obj = PandasExporter()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test pandas exporter")
            })?;
            let object = globals
                .get_item("obj")
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "test pandas exporter")
                })?
                .ok_or_else(|| arrow_error("test pandas exporter did not create obj"))?;
            super::super::object_ops::store_object(object.unbind())
        })
        .map_err(PythonError::runtime)?
    }

    fn malformed_exporter() -> Result<ObjectHandle, PythonError> {
        exporter_with_schema("pyarrow.lib", ARROW_STREAM_NAME)
    }

    fn exporter_without_destructor() -> Result<ObjectHandle, PythonError> {
        super::super::attach(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item("SCHEMA", capsule_without_destructor(py)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test schema"))?;
            py.run(
                cr#"
class ArrowExporter:
    __module__ = "pyarrow.lib"

    def __arrow_c_schema__(self):
        return SCHEMA

obj = ArrowExporter()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test destructorless exporter")
            })?;
            let object = globals
                .get_item("obj")
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "test destructorless exporter")
                })?
                .ok_or_else(|| arrow_error("test destructorless exporter did not create obj"))?;
            super::super::object_ops::store_object(object.unbind())
        })
        .map_err(PythonError::runtime)?
    }

    fn exporter_with_schema(
        module_name: &str,
        schema_name: &'static CStr,
    ) -> Result<ObjectHandle, PythonError> {
        super::super::attach(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item("MODULE_NAME", module_name)
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test module"))?;
            globals
                .set_item("SCHEMA", capsule_any(py, schema_name)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test schema"))?;
            globals
                .set_item("ARRAY", capsule_any(py, ARROW_ARRAY_NAME)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test array"))?;
            globals
                .set_item("STREAM", capsule_any(py, ARROW_STREAM_NAME)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test stream"))?;
            py.run(
                cr#"
class ArrowExporter:
    __module__ = MODULE_NAME

    def __arrow_c_array__(self, requested_schema=None):
        return (SCHEMA, ARRAY)

    def __arrow_c_stream__(self, requested_schema=None):
        return STREAM

    def __arrow_c_schema__(self):
        return SCHEMA

obj = ArrowExporter()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?;
            let object = globals
                .get_item("obj")
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?
                .ok_or_else(|| arrow_error("test exporter did not create obj"))?;
            super::super::object_ops::store_object(object.unbind())
        })
        .map_err(PythonError::runtime)?
    }

    fn capsule<'py>(py: Python<'py>, name: &'static CStr) -> PyResult<Bound<'py, PyCapsule>> {
        PyCapsule::new_with_value(py, 1_u8, name)
    }

    fn capsule_any(py: Python<'_>, name: &'static CStr) -> Result<Py<PyAny>, PythonError> {
        capsule(py, name)
            .map(|capsule| capsule.into_any().unbind())
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))
    }

    fn capsule_without_destructor(py: Python<'_>) -> Result<Py<PyAny>, PythonError> {
        let leaked = Box::leak(Box::new(1_u8));
        let pointer = std::ptr::NonNull::from(leaked).cast();
        let capsule = unsafe { PyCapsule::new_with_pointer(py, pointer, ARROW_SCHEMA_NAME) }
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test destructorless capsule")
            })?;
        Ok(capsule.into_any().unbind())
    }
}
