use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyDict, PyTuple};
use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

mod abi;

const ARROW_SCHEMA_NAME: &CStr = c"arrow_schema";
const ARROW_ARRAY_NAME: &CStr = c"arrow_array";
const ARROW_STREAM_NAME: &CStr = c"arrow_array_stream";
const ARROW_DEVICE_ARRAY_NAME: &CStr = c"arrow_device_array";
const ARROW_DEVICE_STREAM_NAME: &CStr = c"arrow_device_array_stream";

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
    kind: ArrowKind,
    capsules: TrackedArrowCapsules,
    capsule_names: Vec<String>,
}

pub struct PythonArrowArgument {
    entry: Option<ArrowEntry>,
    object: Option<ObjectHandle>,
}

impl PythonArrowArgument {
    pub fn object(&self) -> Result<ObjectHandle, PythonError> {
        let object = self
            .object
            .as_ref()
            .ok_or_else(|| arrow_error("Python Arrow argument is already finalized"))?;
        super::object_ops::temporary_argument_handle(object)
    }

    pub fn finish(mut self) -> Result<(), PythonError> {
        let entry = self
            .entry
            .take()
            .ok_or_else(|| arrow_error("Python Arrow argument is already finalized"))?;
        let object = self.object.take();
        let state = super::attach(move |py| {
            let state = consumption_state(py, &entry)?;
            drop(object);
            drop(entry);
            super::foreign_object::drain_pending_releases(py);
            Ok(state)
        })
        .map_err(PythonError::runtime)??;
        if state == abi::ConsumptionState::Partial {
            Err(arrow_error(
                "Python Arrow consumer partially consumed a paired schema/data resource",
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for PythonArrowArgument {
    fn drop(&mut self) {
        let Some(entry) = self.entry.take() else {
            return;
        };
        let object = self.object.take();
        let _ignored = super::attach(move |py| {
            drop(object);
            drop(entry);
            super::foreign_object::drain_pending_releases(py);
        });
    }
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
    acquire_arrow_capsules(object, ArrowKind::Array, None)
}

pub fn arrow_array_with_schema(
    object: &ObjectHandle,
    schema: ArrowHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Array, Some(schema))
}

pub fn arrow_stream(object: &ObjectHandle) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Stream, None)
}

pub fn arrow_stream_with_schema(
    object: &ObjectHandle,
    schema: ArrowHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Stream, Some(schema))
}

pub fn arrow_schema(object: &ObjectHandle) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::Schema, None)
}

pub fn arrow_device_array(
    object: &ObjectHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::DeviceArray, None)
}

pub fn arrow_device_array_with_schema(
    object: &ObjectHandle,
    schema: ArrowHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::DeviceArray, Some(schema))
}

pub fn arrow_device_stream(
    object: &ObjectHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::DeviceStream, None)
}

pub fn arrow_device_stream_with_schema(
    object: &ObjectHandle,
    schema: ArrowHandle,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    acquire_arrow_capsules(object, ArrowKind::DeviceStream, Some(schema))
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

pub fn require_arrow_certification(
    metadata: &PythonArrowCapsuleMetadata,
    target: &str,
) -> Result<(), PythonError> {
    let config = super::runtime_config().map_err(PythonError::runtime)?;
    if config.arrow_certifications.iter().any(|certification| {
        certification.target == target
            && certification.kind == metadata.kind
            && certification.producer_module == metadata.producer_module
            && certification.producer_type == metadata.producer_type
    }) {
        Ok(())
    } else {
        Err(arrow_error(format!(
            "target '{target}' returning '{}.{}' as '{}' has no exact executable no-copy certification for this environment",
            metadata.producer_module, metadata.producer_type, metadata.kind
        )))
    }
}

pub fn prepare_arrow_argument(handle: ArrowHandle) -> Result<PythonArrowArgument, PythonError> {
    let entry = {
        let mut store = arrow_store()?;
        if store
            .capsules
            .get(&handle.0)
            .is_some_and(|entry| entry.token == handle.1)
        {
            store.capsules.remove(&handle.0)
        } else {
            return Err(closed_error(handle.0));
        }
    }
    .ok_or_else(|| closed_error(handle.0))?;
    let (entry, object) = super::attach(move |py| match build_argument_proxy(py, &entry) {
        Ok(object) => Ok((entry, object)),
        Err(error) => {
            drop(entry);
            super::foreign_object::drain_pending_releases(py);
            Err(error)
        }
    })
    .map_err(PythonError::runtime)??;
    Ok(PythonArrowArgument {
        entry: Some(entry),
        object: Some(object),
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrowKind {
    Array,
    Stream,
    Schema,
    DeviceArray,
    DeviceStream,
}

impl ArrowKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Array => "array",
            Self::Stream => "stream",
            Self::Schema => "schema",
            Self::DeviceArray => "device_array",
            Self::DeviceStream => "device_stream",
        }
    }

    const fn method(self) -> &'static str {
        match self {
            Self::Array => "__arrow_c_array__",
            Self::Stream => "__arrow_c_stream__",
            Self::Schema => "__arrow_c_schema__",
            Self::DeviceArray => "__arrow_c_device_array__",
            Self::DeviceStream => "__arrow_c_device_stream__",
        }
    }
}

fn acquire_arrow_capsules(
    object: &ObjectHandle,
    kind: ArrowKind,
    requested_schema: Option<ArrowHandle>,
) -> Result<PythonArrowCapsuleMetadata, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let producer = producer_info(object.bind(py));
        let requested_schema = requested_schema.map(take_requested_schema).transpose()?;
        let exported = call_export_method(
            object.bind(py),
            kind,
            requested_schema.as_ref().and_then(|entry| {
                entry
                    .capsules
                    .capsules
                    .first()
                    .map(|schema| schema.bind(py))
            }),
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", kind.method()))?;
        let capsules = extract_capsules(py, &exported, kind)?;
        drop(requested_schema);
        let capsule_names = capsule_names(kind);
        store_arrow_capsules(capsules, kind, capsule_names, producer)
    })
    .map_err(PythonError::runtime)?
}

fn call_export_method<'py>(
    object: &Bound<'py, PyAny>,
    kind: ArrowKind,
    requested_schema: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    match kind {
        ArrowKind::Array | ArrowKind::Stream | ArrowKind::DeviceArray | ArrowKind::DeviceStream => {
            match requested_schema {
                Some(schema) => object.call_method1(kind.method(), (schema,)),
                None => object.call_method0(kind.method()),
            }
        }
        ArrowKind::Schema => object.call_method0(kind.method()),
    }
}

fn take_requested_schema(handle: ArrowHandle) -> Result<ArrowEntry, PythonError> {
    let mut store = arrow_store()?;
    let entry = store
        .capsules
        .get(&handle.0)
        .filter(|entry| entry.token == handle.1)
        .ok_or_else(|| closed_error(handle.0))?;
    if entry.kind != ArrowKind::Schema {
        return Err(arrow_error(
            "requested_schema must be an owned python.ArrowSchema resource",
        ));
    }
    if entry.capsules.capsules.len() != 1 {
        return Err(arrow_error(
            "requested ArrowSchema resource must own exactly one capsule",
        ));
    }
    store
        .capsules
        .remove(&handle.0)
        .ok_or_else(|| closed_error(handle.0))
}

fn build_argument_proxy(py: Python<'_>, entry: &ArrowEntry) -> Result<ObjectHandle, PythonError> {
    let globals = PyDict::new(py);
    let capsules = PyTuple::new(
        py,
        entry
            .capsules
            .capsules
            .iter()
            .map(|capsule| capsule.clone_ref(py)),
    )
    .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?;
    globals
        .set_item("CAPSULES", capsules)
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?;
    globals
        .set_item("ARROW_KIND", entry.kind.method())
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?;
    py.run(
        cr#"
class _SifrOwnedArrowArgument:
    def __init__(self, capsules, arrow_kind):
        self._capsules = capsules
        self._arrow_kind = arrow_kind
        self._exported = False
    def _require(self, arrow_kind):
        if self._exported:
            raise RuntimeError("owned Arrow resource was already exported")
        self._exported = True
    def _reject_requested_schema(self, requested_schema):
        if requested_schema is not None:
            raise ValueError("owned Arrow transfer cannot satisfy a new requested schema")

def _export_pair(self, requested_schema=None):
    self._reject_requested_schema(requested_schema)
    self._require(self._arrow_kind)
    return self._capsules

def _export_single(self, requested_schema=None):
    self._reject_requested_schema(requested_schema)
    self._require(self._arrow_kind)
    return self._capsules[0]

def _export_schema(self):
    self._require(self._arrow_kind)
    return self._capsules[0]

if ARROW_KIND == "__arrow_c_schema__":
    setattr(_SifrOwnedArrowArgument, ARROW_KIND, _export_schema)
elif ARROW_KIND in ("__arrow_c_array__", "__arrow_c_device_array__"):
    setattr(_SifrOwnedArrowArgument, ARROW_KIND, _export_pair)
else:
    setattr(_SifrOwnedArrowArgument, ARROW_KIND, _export_single)
obj = _SifrOwnedArrowArgument(CAPSULES, ARROW_KIND)
"#,
        Some(&globals),
        None,
    )
    .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?;
    let object = globals
        .get_item("obj")
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?
        .ok_or_else(|| arrow_error("failed to create Python Arrow argument proxy"))?
        .unbind();
    globals
        .del_item("CAPSULES")
        .and_then(|()| globals.del_item("ARROW_KIND"))
        .and_then(|()| globals.del_item("obj"))
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument"))?;
    super::foreign_object::ForeignObject::new(object).map_err(PythonError::runtime)
}

fn consumption_state(
    py: Python<'_>,
    entry: &ArrowEntry,
) -> Result<abi::ConsumptionState, PythonError> {
    let pointer = |index: usize, name: &'static CStr| {
        let object = entry
            .capsules
            .capsules
            .get(index)
            .ok_or_else(|| arrow_error("Python Arrow argument capsule is missing"))?;
        object
            .bind(py)
            .cast::<PyCapsule>()
            .map_err(|_| arrow_error("Python Arrow argument capsule identity changed"))?
            .pointer_checked(Some(name))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "Arrow argument finalization")
            })
    };
    match entry.kind {
        ArrowKind::Array => Ok(abi::array_pair_consumption(
            pointer(0, ARROW_SCHEMA_NAME)?,
            pointer(1, ARROW_ARRAY_NAME)?,
        )),
        ArrowKind::Schema => Ok(abi::schema_consumption(pointer(0, ARROW_SCHEMA_NAME)?)),
        ArrowKind::Stream => Ok(abi::stream_consumption(pointer(0, ARROW_STREAM_NAME)?)),
        ArrowKind::DeviceArray => Ok(abi::device_array_pair_consumption(
            pointer(0, ARROW_SCHEMA_NAME)?,
            pointer(1, ARROW_DEVICE_ARRAY_NAME)?,
        )),
        ArrowKind::DeviceStream => Ok(abi::device_stream_consumption(pointer(
            0,
            ARROW_DEVICE_STREAM_NAME,
        )?)),
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
            let schema_pointer =
                validate_capsule(py, &schema, ARROW_SCHEMA_NAME, "__arrow_c_array__ schema")?;
            let array_pointer =
                validate_capsule(py, &array, ARROW_ARRAY_NAME, "__arrow_c_array__ array")?;
            abi::validate_array_pair(schema_pointer, array_pointer, "__arrow_c_array__")?;
            Ok(vec![schema.unbind(), array.unbind()])
        }
        ArrowKind::Stream => {
            let pointer = validate_capsule(py, exported, ARROW_STREAM_NAME, "__arrow_c_stream__")?;
            abi::validate_stream(pointer, "__arrow_c_stream__")?;
            Ok(vec![exported.clone().unbind()])
        }
        ArrowKind::Schema => {
            let pointer = validate_capsule(py, exported, ARROW_SCHEMA_NAME, "__arrow_c_schema__")?;
            abi::validate_schema(pointer, "__arrow_c_schema__")?;
            Ok(vec![exported.clone().unbind()])
        }
        ArrowKind::DeviceArray => {
            let tuple = exported.cast::<PyTuple>().map_err(|_| {
                arrow_error(
                    "__arrow_c_device_array__ must return (arrow_schema, arrow_device_array)",
                )
            })?;
            if tuple.len() != 2 {
                return Err(arrow_error(
                    "__arrow_c_device_array__ must return exactly two capsules",
                ));
            }
            let schema = tuple.get_item(0).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "__arrow_c_device_array__ schema")
            })?;
            let array = tuple.get_item(1).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "__arrow_c_device_array__ array")
            })?;
            let schema_pointer = validate_capsule(
                py,
                &schema,
                ARROW_SCHEMA_NAME,
                "__arrow_c_device_array__ schema",
            )?;
            let array_pointer = validate_capsule(
                py,
                &array,
                ARROW_DEVICE_ARRAY_NAME,
                "__arrow_c_device_array__ array",
            )?;
            abi::validate_device_array_pair(
                schema_pointer,
                array_pointer,
                "__arrow_c_device_array__",
            )?;
            Ok(vec![schema.unbind(), array.unbind()])
        }
        ArrowKind::DeviceStream => {
            let pointer = validate_capsule(
                py,
                exported,
                ARROW_DEVICE_STREAM_NAME,
                "__arrow_c_device_stream__",
            )?;
            abi::validate_device_stream(pointer, "__arrow_c_device_stream__")?;
            Ok(vec![exported.clone().unbind()])
        }
    }
}

fn validate_capsule(
    py: Python<'_>,
    object: &Bound<'_, PyAny>,
    expected_name: &'static CStr,
    context: &'static str,
) -> Result<std::ptr::NonNull<std::ffi::c_void>, PythonError> {
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
    let pointer = capsule
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
    Ok(pointer)
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
            kind,
            capsules: TrackedArrowCapsules { capsules },
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
    ProducerInfo {
        module,
        name,
        // The Arrow protocol alone never proves representation-preserving
        // export. Declaration activation requires package-authored executable
        // certification, so the raw runtime substrate remains conservative.
        copy_possible: true,
    }
}

fn capsule_names(kind: ArrowKind) -> Vec<String> {
    match kind {
        ArrowKind::Array => vec!["arrow_schema".to_string(), "arrow_array".to_string()],
        ArrowKind::Stream => vec!["arrow_array_stream".to_string()],
        ArrowKind::Schema => vec!["arrow_schema".to_string()],
        ArrowKind::DeviceArray => {
            vec!["arrow_schema".to_string(), "arrow_device_array".to_string()]
        }
        ArrowKind::DeviceStream => vec!["arrow_device_array_stream".to_string()],
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
pub(super) fn reset_arrow_store_for_tests() {
    if let Ok(mut store) = ARROW_STORE.lock() {
        *store = ArrowStore::default();
    }
}

#[cfg(test)]
mod tests;
