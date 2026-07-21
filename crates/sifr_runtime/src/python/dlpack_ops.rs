use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyTuple};
use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

mod abi;
mod argument;
#[cfg(test)]
mod declaration_tests;

pub use argument::{prepare_dlpack_argument, PythonDlpackArgument};

use abi::{
    metadata_for_managed_tensor, DLDataType, DLDevice, ManagedTensor, DLTENSOR_NAME,
    DLTENSOR_VERSIONED_NAME,
};
#[cfg(test)]
use abi::{DLManagedTensor, DLTensor, USED_DLTENSOR_NAME};
#[cfg(test)]
use std::ffi::c_void;

const DEVICE_CPU: i32 = 1;
const DEVICE_CUDA: i32 = 2;

static DLPACK_STORE: LazyLock<Mutex<DlpackStore>> =
    LazyLock::new(|| Mutex::new(DlpackStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type DlpackHandle = (i64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonDlpackTensorMetadata {
    pub handle: i64,
    pub token: i64,
    pub dtype_code: i64,
    pub dtype_bits: i64,
    pub dtype_lanes: i64,
    pub dtype: String,
    pub device_type: i64,
    pub device_id: i64,
    pub dimensions: i64,
    pub shape: Vec<i64>,
    pub strides: Vec<i64>,
    pub byte_offset: i64,
    pub has_deleter: bool,
    pub stream_sync_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonDlpackStreamMetadata {
    pub device_type: i64,
    pub device_id: i64,
    pub stream_token: i64,
}

#[derive(Default)]
struct DlpackStore {
    next_handle: i64,
    next_nonce: u64,
    tensors: HashMap<i64, DlpackEntry>,
}

struct DlpackEntry {
    token: i64,
    _tensor: TrackedDlpackTensor,
    shape: Vec<i64>,
    strides: Vec<i64>,
}

struct TrackedDlpackTensor {
    tensor: ManagedTensor,
    released: bool,
    counted: bool,
    _owner: Py<PyAny>,
    _capsule: Py<PyAny>,
}

impl Drop for TrackedDlpackTensor {
    fn drop(&mut self) {
        if !self.released {
            unsafe { self.tensor.release() };
            self.released = true;
        }
        if self.counted {
            let _ignored = super::update_object_count(-1);
        }
    }
}

pub fn dlpack_tensor(object: &ObjectHandle) -> Result<PythonDlpackTensorMetadata, PythonError> {
    acquire_dlpack_tensor(object, "cpu", None, None)
}

pub fn acquire_dlpack_tensor(
    object: &ObjectHandle,
    expected_device: &str,
    stream: Option<&PythonDlpackStreamMetadata>,
    expected_dtype: Option<&str>,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    super::attach(|py| {
        let owner = clone_handle(py, object)?;
        let device = dlpack_device(py, owner.bind(py))?;
        validate_device_policy(device, expected_device, stream)?;
        let kwargs = pyo3::types::PyDict::new(py);
        if device.device_type == DEVICE_CPU {
            kwargs.set_item("stream", py.None()).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream")
            })?;
        } else if let Some(stream) = stream {
            kwargs
                .set_item("stream", stream.stream_token)
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream")
                })?;
        } else {
            kwargs.set_item("stream", py.None()).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream")
            })?;
        }
        kwargs
            .set_item("max_version", (1_u32, 0_u32))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "DLPack max_version")
            })?;
        kwargs.set_item("copy", false).map_err(|error| {
            PythonError::from_pyerr(py, error, "zero-copy", "DLPack copy policy")
        })?;
        let capsule = owner
            .bind(py)
            .call_method("__dlpack__", (), Some(&kwargs))
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "__dlpack__"))?;
        let metadata = consume_capsule(py, owner, &capsule)?;
        if metadata.device_type != i64::from(device.device_type)
            || metadata.device_id != i64::from(device.device_id)
        {
            let _ignored = release_dlpack((metadata.handle, metadata.token));
            return Err(dlpack_error(
                "DLPack capsule device does not match `__dlpack_device__`",
            ));
        }
        if expected_dtype.is_some_and(|expected| metadata.dtype != expected) {
            let actual = metadata.dtype.clone();
            let _ignored = release_dlpack((metadata.handle, metadata.token));
            return Err(dlpack_error(format!(
                "DLPack dtype mismatch: declaration requires '{}', producer returned '{}'",
                expected_dtype.unwrap_or_default(),
                actual
            )));
        }
        Ok(metadata)
    })
    .map_err(PythonError::runtime)?
}

pub fn dlpack_stream(
    object: &ObjectHandle,
    expected_device: &str,
) -> Result<PythonDlpackStreamMetadata, PythonError> {
    super::attach(|py| {
        let owner = clone_handle(py, object)?;
        let bound = owner.bind(py);
        let (device_type, device_id, stream_token) = if let Ok(tuple) = bound.cast::<PyTuple>() {
            if tuple.len() != 3 {
                return Err(dlpack_error(
                    "normalized DLPack stream tuple must contain device type, device id, and token",
                ));
            }
            (
                extract_stream_i64(
                    py,
                    &tuple.get_item(0).map_err(|error| {
                        PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream tuple")
                    })?,
                    "device type",
                )?,
                extract_stream_i64(
                    py,
                    &tuple.get_item(1).map_err(|error| {
                        PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream tuple")
                    })?,
                    "device id",
                )?,
                extract_stream_i64(
                    py,
                    &tuple.get_item(2).map_err(|error| {
                        PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream tuple")
                    })?,
                    "stream token",
                )?,
            )
        } else {
            let device = bound.getattr("device").map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream device")
            })?;
            let family = device
                .getattr("type")
                .and_then(|value| value.extract::<String>())
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream device type")
                })?;
            let device_type = i64::from(device_code(&family)?);
            let device_id = device
                .getattr("index")
                .and_then(|value| value.extract::<Option<i64>>())
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream device id")
                })?
                .unwrap_or(0);
            let token = bound
                .getattr(if family == "cuda" {
                    "cuda_stream"
                } else {
                    "ptr"
                })
                .and_then(|value| value.extract::<i64>())
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "DLPack stream token")
                })?;
            (device_type, device_id, token)
        };
        if device_type != i64::from(device_code(expected_device)?) {
            return Err(dlpack_error(format!(
                "DLPack stream device family does not match declared device '{expected_device}'"
            )));
        }
        if device_id < 0 || stream_token < 0 {
            return Err(dlpack_error(
                "DLPack stream device id and token must be non-negative",
            ));
        }
        if device_type == i64::from(DEVICE_CUDA) && stream_token == 0 {
            return Err(dlpack_error(
                "CUDA DLPack stream token 0 is ambiguous and unsupported",
            ));
        }
        Ok(PythonDlpackStreamMetadata {
            device_type,
            device_id,
            stream_token,
        })
    })
    .map_err(PythonError::runtime)?
}

pub fn release_dlpack((handle, token): DlpackHandle) -> Result<(), PythonError> {
    let entry = {
        let mut store = dlpack_store()?;
        if store
            .tensors
            .get(&handle)
            .is_some_and(|entry| entry.token == token)
        {
            store.tensors.remove(&handle)
        } else {
            return Err(closed_error(handle));
        }
    };
    super::attach(|_py| drop(entry)).map_err(PythonError::runtime)?;
    Ok(())
}

pub fn dlpack_shape(handle: DlpackHandle) -> Result<Vec<i64>, PythonError> {
    dlpack_metadata(handle).map(|(shape, _)| shape)
}

pub fn dlpack_strides(handle: DlpackHandle) -> Result<Vec<i64>, PythonError> {
    dlpack_metadata(handle).map(|(_, strides)| strides)
}

fn consume_capsule(
    py: Python<'_>,
    owner: Py<PyAny>,
    capsule: &Bound<'_, PyAny>,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    let capsule = capsule.cast::<PyCapsule>().map_err(|_| {
        dlpack_error("DLPack exporter returned a non-PyCapsule value; expected dltensor")
    })?;
    let actual_name = capsule_name(capsule, "__dlpack__")?;
    if actual_name != DLTENSOR_NAME && actual_name != DLTENSOR_VERSIONED_NAME {
        return Err(dlpack_error(format!(
            "__dlpack__ capsule has name '{}'; expected 'dltensor' or 'dltensor_versioned'",
            actual_name.to_string_lossy()
        )));
    }
    let pointer = capsule
        .pointer_checked(Some(actual_name))
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "__dlpack__"))?;
    let tensor = ManagedTensor::from_capsule_name(pointer.as_ptr(), actual_name)?;
    let metadata = metadata_for_managed_tensor(tensor)?;
    let rename_result =
        unsafe { ffi::PyCapsule_SetName(capsule.as_ptr(), tensor.used_capsule_name().as_ptr()) };
    if rename_result != 0 {
        return Err(PythonError::from_pyerr(
            py,
            PyErr::fetch(py),
            "zero-copy",
            "mark DLPack capsule consumed",
        ));
    }
    store_tensor(
        TrackedDlpackTensor {
            tensor,
            released: false,
            counted: false,
            _owner: owner,
            _capsule: capsule.clone().into_any().unbind(),
        },
        metadata,
    )
}

fn dlpack_device(py: Python<'_>, object: &Bound<'_, PyAny>) -> Result<DLDevice, PythonError> {
    let raw = object
        .call_method0("__dlpack_device__")
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "__dlpack_device__"))?;
    let tuple = raw
        .cast::<PyTuple>()
        .map_err(|_| dlpack_error("__dlpack_device__ must return (device_type, device_id)"))?;
    if tuple.len() != 2 {
        return Err(dlpack_error(
            "__dlpack_device__ must return exactly two values",
        ));
    }
    let device_type = tuple
        .get_item(0)
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device type"))?
        .extract::<i32>()
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device type"))?;
    let device_id = tuple
        .get_item(1)
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device id"))?
        .extract::<i32>()
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device id"))?;
    Ok(DLDevice {
        device_type,
        device_id,
    })
}

fn store_tensor(
    mut tensor: TrackedDlpackTensor,
    mut metadata: PythonDlpackTensorMetadata,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    let mut store = dlpack_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    super::update_object_count(1).map_err(PythonError::runtime)?;
    tensor.counted = true;
    metadata.handle = handle;
    metadata.token = token;
    store.tensors.insert(
        handle,
        DlpackEntry {
            token,
            _tensor: tensor,
            shape: metadata.shape.clone(),
            strides: metadata.strides.clone(),
        },
    );
    Ok(metadata)
}

fn dlpack_metadata(handle: DlpackHandle) -> Result<(Vec<i64>, Vec<i64>), PythonError> {
    let store = dlpack_store()?;
    store
        .tensors
        .get(&handle.0)
        .filter(|entry| entry.token == handle.1)
        .map(|entry| (entry.shape.clone(), entry.strides.clone()))
        .ok_or_else(|| closed_error(handle.0))
}

fn capsule_name<'a>(
    capsule: &'a Bound<'_, PyCapsule>,
    context: &'static str,
) -> Result<&'a CStr, PythonError> {
    let actual_name = unsafe { ffi::PyCapsule_GetName(capsule.as_ptr()) };
    if actual_name.is_null() {
        return Err(dlpack_error(format!("{context} capsule has no name")));
    }
    Ok(unsafe { CStr::from_ptr(actual_name) })
}

fn validate_device_policy(
    device: DLDevice,
    expected: &str,
    stream: Option<&PythonDlpackStreamMetadata>,
) -> Result<(), PythonError> {
    let expected_code = device_code(expected)?;
    if expected != "any" && device.device_type != expected_code {
        return Err(unsupported_device_error(device));
    }
    if device.device_type != DEVICE_CPU && stream.is_none() {
        return Err(dlpack_error(
            "non-CPU DLPack acquisition requires an explicit consumer stream",
        ));
    }
    if let Some(stream) = stream {
        if stream.device_type != i64::from(device.device_type)
            || stream.device_id != i64::from(device.device_id)
        {
            return Err(dlpack_error(
                "DLPack consumer stream family/id does not match the producer device",
            ));
        }
    }
    Ok(())
}

fn device_code(device: &str) -> Result<i32, PythonError> {
    match device {
        "cpu" => Ok(DEVICE_CPU),
        "cuda" => Ok(2),
        "any" => Ok(0),
        _ => Err(dlpack_error(format!(
            "unsupported DLPack device family '{device}'"
        ))),
    }
}

fn extract_stream_i64(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    context: &'static str,
) -> Result<i64, PythonError> {
    value
        .extract::<i64>()
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", context))
}

fn reserve_handle(store: &mut DlpackStore) -> Result<DlpackHandle, PythonError> {
    store.next_handle = store.next_handle.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python DLPack handle space exhausted".to_string(),
        ))
    })?;
    store.next_nonce = store.next_nonce.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python DLPack token space exhausted".to_string(),
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

fn unsupported_device_error(device: DLDevice) -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "SifrPythonDlpackUnsupportedDevice".to_string(),
        message: format!(
            "DLPack device type {} id {} requires explicit stream/device support",
            device.device_type, device.device_id
        ),
        traceback: String::new(),
        context: "DLPack device validation".to_string(),
        replay: None,
    }
}

fn unsupported_dtype_error(dtype: DLDataType) -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "SifrPythonDlpackUnsupportedDtype".to_string(),
        message: format!(
            "unsupported DLPack dtype code={} bits={} lanes={}",
            dtype.code, dtype.bits, dtype.lanes
        ),
        traceback: String::new(),
        context: "DLPack dtype validation".to_string(),
        replay: None,
    }
}

fn closed_error(handle: i64) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonClosedDlpackTensor".to_string(),
        message: format!("Python DLPack tensor handle {handle} is closed"),
        traceback: String::new(),
        context: "DLPack tensor handle lookup".to_string(),
        replay: None,
    }
}

pub(super) fn dlpack_error(message: impl Into<String>) -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "SifrPythonDlpackError".to_string(),
        message: message.into(),
        traceback: String::new(),
        context: "DLPack validation".to_string(),
        replay: None,
    }
}

fn dlpack_store() -> Result<MutexGuard<'static, DlpackStore>, PythonError> {
    DLPACK_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python DLPack tensor store is unavailable".to_string(),
        traceback: String::new(),
        context: "DLPack tensor store".to_string(),
        replay: None,
    })
}

#[cfg(test)]
pub(super) fn reset_dlpack_store_for_tests() {
    if let Ok(mut store) = DLPACK_STORE.lock() {
        *store = DlpackStore::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        close_object, initialize_runtime, reset_runtime_state_for_tests, resource_diagnostics,
        test_config, test_guard, PythonResourceDiagnostics,
    };
    use pyo3::types::PyDict;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DELETER_CALLS: AtomicUsize = AtomicUsize::new(0);

    #[repr(C)]
    struct TestManagedTensor {
        managed: DLManagedTensor,
        shape: Vec<i64>,
        strides: Vec<i64>,
        data: Vec<u8>,
    }

    #[test]
    fn dlpack_cpu_tensor_tracks_metadata_and_release() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        TEST_DELETER_CALLS.store(0, Ordering::SeqCst);
        initialize_runtime(test_config("dlpack-cpu")).expect("init should succeed");

        let object = exporter(DEVICE_CPU, 0, dtype(2, 32, 1), DLTENSOR_NAME)
            .expect("exporter should be stored");
        let tensor = dlpack_tensor(&object).expect("DLPack tensor should export");

        assert_eq!(tensor.dtype, "float32");
        assert_eq!(tensor.shape, [2, 3]);
        assert_eq!(tensor.strides, [3, 1]);
        assert_eq!(tensor.byte_offset, 0);
        assert!(tensor.has_deleter);
        assert!(!tensor.stream_sync_required);
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 2,
                leaked_objects: 0,
            }
        );

        release_dlpack((tensor.handle, tensor.token)).expect("DLPack tensor should release");
        assert_eq!(TEST_DELETER_CALLS.load(Ordering::SeqCst), 1);
        close_object(object).expect("object should close");
    }

    #[test]
    fn dlpack_scalar_tensor_allows_null_shape_pointer() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        TEST_DELETER_CALLS.store(0, Ordering::SeqCst);
        initialize_runtime(test_config("dlpack-scalar")).expect("init should succeed");

        let object = scalar_exporter().expect("scalar exporter should be stored");
        let tensor = dlpack_tensor(&object).expect("scalar tensor should export");

        assert_eq!(tensor.dimensions, 0);
        assert!(tensor.shape.is_empty());
        assert!(tensor.strides.is_empty());
        release_dlpack((tensor.handle, tensor.token)).expect("scalar should release");
        assert_eq!(TEST_DELETER_CALLS.load(Ordering::SeqCst), 1);
        close_object(object).expect("object should close");
    }

    #[test]
    fn dlpack_rejects_double_consumption_and_double_release() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        TEST_DELETER_CALLS.store(0, Ordering::SeqCst);
        initialize_runtime(test_config("dlpack-double")).expect("init should succeed");

        let object = exporter(DEVICE_CPU, 0, dtype(1, 8, 1), DLTENSOR_NAME)
            .expect("exporter should be stored");
        let tensor = dlpack_tensor(&object).expect("first consume should succeed");
        let consumed = dlpack_tensor(&object).expect_err("second consume should fail");
        assert_eq!(consumed.kind, "zero-copy");
        assert!(consumed.message.contains("used_dltensor"));

        release_dlpack((tensor.handle, tensor.token)).expect("DLPack tensor should release");
        let closed =
            release_dlpack((tensor.handle, tensor.token)).expect_err("second release should fail");
        assert_eq!(closed.kind, "resource");
        assert_eq!(closed.exception_type, "SifrPythonClosedDlpackTensor");
        close_object(object).expect("object should close");
    }

    #[test]
    fn dlpack_rejects_invalid_capsule_name_dtype_and_device() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("dlpack-rejects")).expect("init should succeed");

        let invalid_name = exporter(DEVICE_CPU, 0, dtype(2, 32, 1), USED_DLTENSOR_NAME)
            .expect("invalid-name exporter should be stored");
        let invalid_name_error =
            dlpack_tensor(&invalid_name).expect_err("used capsule should be rejected");
        assert_eq!(invalid_name_error.exception_type, "SifrPythonDlpackError");

        let unsupported_dtype = exporter(DEVICE_CPU, 0, dtype(99, 32, 1), DLTENSOR_NAME)
            .expect("unsupported-dtype exporter should be stored");
        let dtype_error =
            dlpack_tensor(&unsupported_dtype).expect_err("unsupported dtype should fail");
        assert_eq!(
            dtype_error.exception_type,
            "SifrPythonDlpackUnsupportedDtype"
        );

        let unsupported_device = exporter(2, 0, dtype(2, 32, 1), DLTENSOR_NAME)
            .expect("unsupported-device exporter should be stored");
        let device_error =
            dlpack_tensor(&unsupported_device).expect_err("unsupported device should fail");
        assert_eq!(
            device_error.exception_type,
            "SifrPythonDlpackUnsupportedDevice"
        );

        close_object(invalid_name).expect("object should close");
        close_object(unsupported_dtype).expect("object should close");
        close_object(unsupported_device).expect("object should close");
    }

    fn exporter(
        device_type: i32,
        device_id: i32,
        dtype: DLDataType,
        capsule_name: &'static CStr,
    ) -> Result<ObjectHandle, PythonError> {
        super::super::attach(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item(
                    "CAPSULE",
                    capsule(py, device_type, device_id, dtype, capsule_name)?.bind(py),
                )
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))?;
            globals
                .set_item("DEVICE_TYPE", device_type)
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device type"))?;
            globals
                .set_item("DEVICE_ID", device_id)
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "device id"))?;
            py.run(
                cr#"
class DlpackExporter:
    def __dlpack_device__(self):
        return (DEVICE_TYPE, DEVICE_ID)

    def __dlpack__(self, *, stream, max_version, copy):
        assert max_version == (1, 0)
        assert copy is False
        return CAPSULE

obj = DlpackExporter()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?;
            let object = globals
                .get_item("obj")
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?
                .ok_or_else(|| dlpack_error("test exporter did not create obj"))?;
            super::super::object_ops::store_object(object.unbind())
        })
        .map_err(PythonError::runtime)?
    }

    fn scalar_exporter() -> Result<ObjectHandle, PythonError> {
        super::super::attach(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item("CAPSULE", scalar_capsule(py)?.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))?;
            py.run(
                cr#"
class DlpackExporter:
    def __dlpack_device__(self):
        return (1, 0)

    def __dlpack__(self, *, stream, max_version, copy):
        assert max_version == (1, 0)
        assert copy is False
        return CAPSULE

obj = DlpackExporter()
"#,
                Some(&globals),
                None,
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?;
            let object = globals
                .get_item("obj")
                .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test exporter"))?
                .ok_or_else(|| dlpack_error("test exporter did not create obj"))?;
            super::super::object_ops::store_object(object.unbind())
        })
        .map_err(PythonError::runtime)?
    }

    fn capsule(
        py: Python<'_>,
        device_type: i32,
        device_id: i32,
        dtype: DLDataType,
        capsule_name: &'static CStr,
    ) -> Result<Py<PyAny>, PythonError> {
        let mut owned = Box::new(TestManagedTensor {
            managed: DLManagedTensor {
                dl_tensor: DLTensor {
                    data: std::ptr::null_mut(),
                    device: DLDevice {
                        device_type,
                        device_id,
                    },
                    ndim: 2,
                    dtype,
                    shape: std::ptr::null_mut(),
                    strides: std::ptr::null_mut(),
                    byte_offset: 0,
                },
                manager_ctx: std::ptr::null_mut(),
                deleter: Some(test_deleter),
            },
            shape: vec![2, 3],
            strides: vec![3, 1],
            data: vec![0; 6],
        });
        owned.managed.dl_tensor.data = owned.data.as_mut_ptr().cast::<c_void>();
        owned.managed.dl_tensor.shape = owned.shape.as_mut_ptr();
        owned.managed.dl_tensor.strides = owned.strides.as_mut_ptr();
        let pointer = std::ptr::NonNull::new(Box::into_raw(owned).cast::<c_void>())
            .ok_or_else(|| dlpack_error("test tensor pointer was null"))?;
        let capsule = unsafe {
            PyCapsule::new_with_pointer_and_destructor(
                py,
                pointer,
                capsule_name,
                Some(test_capsule_destructor),
            )
        }
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))?;
        Ok(capsule.into_any().unbind())
    }

    fn scalar_capsule(py: Python<'_>) -> Result<Py<PyAny>, PythonError> {
        let mut owned = Box::new(TestManagedTensor {
            managed: DLManagedTensor {
                dl_tensor: DLTensor {
                    data: std::ptr::null_mut(),
                    device: DLDevice {
                        device_type: DEVICE_CPU,
                        device_id: 0,
                    },
                    ndim: 0,
                    dtype: dtype(2, 32, 1),
                    shape: std::ptr::null_mut(),
                    strides: std::ptr::null_mut(),
                    byte_offset: 0,
                },
                manager_ctx: std::ptr::null_mut(),
                deleter: Some(test_deleter),
            },
            shape: Vec::new(),
            strides: Vec::new(),
            data: vec![0; 1],
        });
        owned.managed.dl_tensor.data = owned.data.as_mut_ptr().cast::<c_void>();
        let pointer = std::ptr::NonNull::new(Box::into_raw(owned).cast::<c_void>())
            .ok_or_else(|| dlpack_error("test tensor pointer was null"))?;
        let capsule = unsafe {
            PyCapsule::new_with_pointer_and_destructor(
                py,
                pointer,
                DLTENSOR_NAME,
                Some(test_capsule_destructor),
            )
        }
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))?;
        Ok(capsule.into_any().unbind())
    }

    const fn dtype(code: u8, bits: u8, lanes: u16) -> DLDataType {
        DLDataType { code, bits, lanes }
    }

    unsafe extern "C" fn test_capsule_destructor(capsule: *mut ffi::PyObject) {
        let name = unsafe { ffi::PyCapsule_GetName(capsule) };
        if name.is_null() {
            return;
        }
        let name = unsafe { CStr::from_ptr(name) };
        if name != DLTENSOR_NAME {
            return;
        }
        let pointer = unsafe { ffi::PyCapsule_GetPointer(capsule, DLTENSOR_NAME.as_ptr()) };
        if !pointer.is_null() {
            unsafe { test_deleter(pointer.cast::<DLManagedTensor>()) };
        }
    }

    unsafe extern "C" fn test_deleter(tensor: *mut DLManagedTensor) {
        TEST_DELETER_CALLS.fetch_add(1, Ordering::SeqCst);
        if !tensor.is_null() {
            let _owned = unsafe { Box::from_raw(tensor.cast::<TestManagedTensor>()) };
        }
    }
}
