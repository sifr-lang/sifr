use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyTuple};
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

const DLTENSOR_NAME: &CStr = c"dltensor";
const USED_DLTENSOR_NAME: &CStr = c"used_dltensor";
const DEVICE_CPU: i32 = 1;

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

#[repr(C)]
#[derive(Clone, Copy)]
struct DLDevice {
    device_type: i32,
    device_id: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DLDataType {
    code: u8,
    bits: u8,
    lanes: u16,
}

#[repr(C)]
struct DLTensor {
    data: *mut c_void,
    device: DLDevice,
    ndim: i32,
    dtype: DLDataType,
    shape: *mut i64,
    strides: *mut i64,
    byte_offset: u64,
}

#[repr(C)]
struct DLManagedTensor {
    dl_tensor: DLTensor,
    manager_ctx: *mut c_void,
    deleter: Option<unsafe extern "C" fn(*mut DLManagedTensor)>,
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
}

struct TrackedDlpackTensor {
    tensor_ptr: usize,
    deleter: Option<unsafe extern "C" fn(*mut DLManagedTensor)>,
    released: bool,
    _owner: Py<PyAny>,
    _capsule: Py<PyAny>,
}

impl Drop for TrackedDlpackTensor {
    fn drop(&mut self) {
        if !self.released {
            if let Some(deleter) = self.deleter.take() {
                let tensor = self.tensor_ptr as *mut DLManagedTensor;
                unsafe { deleter(tensor) };
            }
            self.released = true;
        }
        let _ignored = super::update_object_count(-1);
    }
}

pub fn dlpack_tensor(object: ObjectHandle) -> Result<PythonDlpackTensorMetadata, PythonError> {
    super::attach(|py| {
        let owner = clone_handle(py, object)?;
        let device = dlpack_device(py, owner.bind(py))?;
        if device.device_type != DEVICE_CPU {
            return Err(unsupported_device_error(device));
        }
        let capsule = owner
            .bind(py)
            .call_method0("__dlpack__")
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "__dlpack__"))?;
        consume_capsule(py, owner, &capsule)
    })
    .map_err(PythonError::runtime)?
}

pub fn release_dlpack((handle, token): DlpackHandle) -> Result<(), PythonError> {
    super::attach(|_py| {
        let mut store = dlpack_store()?;
        if store
            .tensors
            .get(&handle)
            .is_some_and(|entry| entry.token == token)
        {
            let entry = store.tensors.remove(&handle);
            drop(entry);
            Ok(())
        } else {
            Err(closed_error(handle))
        }
    })
    .map_err(PythonError::runtime)?
}

fn consume_capsule(
    py: Python<'_>,
    owner: Py<PyAny>,
    capsule: &Bound<'_, PyAny>,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    let capsule = capsule.cast::<PyCapsule>().map_err(|_| {
        dlpack_error("DLPack exporter returned a non-PyCapsule value; expected dltensor")
    })?;
    validate_capsule_name(capsule, DLTENSOR_NAME, "__dlpack__")?;
    let pointer = capsule
        .pointer_checked(Some(DLTENSOR_NAME))
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "__dlpack__"))?;
    let tensor = pointer.as_ptr().cast::<DLManagedTensor>();
    let metadata = metadata_for_tensor(tensor)?;
    let rename_result =
        unsafe { ffi::PyCapsule_SetName(capsule.as_ptr(), USED_DLTENSOR_NAME.as_ptr()) };
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
            tensor_ptr: tensor as usize,
            deleter: unsafe { (*tensor).deleter },
            released: false,
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

fn metadata_for_tensor(
    tensor: *mut DLManagedTensor,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    if tensor.is_null() {
        return Err(dlpack_error("DLPack capsule pointer is null"));
    }
    let tensor_ref = unsafe { &*tensor };
    let dl_tensor = &tensor_ref.dl_tensor;
    let dimensions = checked_i64_i32(dl_tensor.ndim, "DLPack dimensions")?;
    let len = usize::try_from(dl_tensor.ndim)
        .map_err(|_| dlpack_error("DLPack dimensions exceed Sifr list range"))?;
    if dl_tensor.shape.is_null() && len > 0 {
        return Err(dlpack_error("DLPack tensor shape pointer is null"));
    }
    let shape = if dl_tensor.shape.is_null() {
        Vec::new()
    } else {
        unsafe { slice_to_vec(dl_tensor.shape, len) }
    };
    let strides = if dl_tensor.strides.is_null() {
        Vec::new()
    } else {
        unsafe { slice_to_vec(dl_tensor.strides, len) }
    };
    let dtype = dtype_name(dl_tensor.dtype)?;
    let byte_offset = i64::try_from(dl_tensor.byte_offset)
        .map_err(|_| dlpack_error("DLPack byte offset exceeds Sifr int range"))?;
    Ok(PythonDlpackTensorMetadata {
        handle: -1,
        token: 0,
        dtype_code: i64::from(dl_tensor.dtype.code),
        dtype_bits: i64::from(dl_tensor.dtype.bits),
        dtype_lanes: i64::from(dl_tensor.dtype.lanes),
        dtype,
        device_type: i64::from(dl_tensor.device.device_type),
        device_id: i64::from(dl_tensor.device.device_id),
        dimensions,
        shape,
        strides,
        byte_offset,
        has_deleter: tensor_ref.deleter.is_some(),
        stream_sync_required: dl_tensor.device.device_type != DEVICE_CPU,
    })
}

unsafe fn slice_to_vec(pointer: *mut i64, len: usize) -> Vec<i64> {
    unsafe { std::slice::from_raw_parts(pointer.cast_const(), len) }.to_vec()
}

fn dtype_name(dtype: DLDataType) -> Result<String, PythonError> {
    let lanes = dtype.lanes;
    if lanes != 1 {
        return Err(unsupported_dtype_error(dtype));
    }
    let name = match (dtype.code, dtype.bits) {
        (0, 8) => "int8",
        (0, 16) => "int16",
        (0, 32) => "int32",
        (0, 64) => "int64",
        (1, 8) => "uint8",
        (1, 16) => "uint16",
        (1, 32) => "uint32",
        (1, 64) => "uint64",
        (2, 16) => "float16",
        (2, 32) => "float32",
        (2, 64) => "float64",
        (4, 16) => "bfloat16",
        (6, 1) | (6, 8) => "bool",
        _ => return Err(unsupported_dtype_error(dtype)),
    };
    Ok(name.to_string())
}

fn store_tensor(
    tensor: TrackedDlpackTensor,
    mut metadata: PythonDlpackTensorMetadata,
) -> Result<PythonDlpackTensorMetadata, PythonError> {
    let mut store = dlpack_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    super::update_object_count(1).map_err(PythonError::runtime)?;
    metadata.handle = handle;
    metadata.token = token;
    store.tensors.insert(
        handle,
        DlpackEntry {
            token,
            _tensor: tensor,
        },
    );
    Ok(metadata)
}

fn validate_capsule_name(
    capsule: &Bound<'_, PyCapsule>,
    expected_name: &'static CStr,
    context: &'static str,
) -> Result<(), PythonError> {
    let actual_name = unsafe { ffi::PyCapsule_GetName(capsule.as_ptr()) };
    if actual_name.is_null() {
        return Err(dlpack_error(format!(
            "{context} capsule has no name; expected {}",
            expected_name.to_string_lossy()
        )));
    }
    let actual_name = unsafe { CStr::from_ptr(actual_name) };
    if actual_name != expected_name {
        return Err(dlpack_error(format!(
            "{context} capsule has name '{}'; expected '{}'",
            actual_name.to_string_lossy(),
            expected_name.to_string_lossy()
        )));
    }
    Ok(())
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

fn checked_i64_i32(value: i32, context: &'static str) -> Result<i64, PythonError> {
    if value < 0 {
        return Err(dlpack_error(format!("{context} must be non-negative")));
    }
    Ok(i64::from(value))
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
    }
}

fn closed_error(handle: i64) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonClosedDlpackTensor".to_string(),
        message: format!("Python DLPack tensor handle {handle} is closed"),
        traceback: String::new(),
        context: "DLPack tensor handle lookup".to_string(),
    }
}

fn dlpack_error(message: impl Into<String>) -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "SifrPythonDlpackError".to_string(),
        message: message.into(),
        traceback: String::new(),
        context: "DLPack validation".to_string(),
    }
}

fn dlpack_store() -> Result<MutexGuard<'static, DlpackStore>, PythonError> {
    DLPACK_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python DLPack tensor store is unavailable".to_string(),
        traceback: String::new(),
        context: "DLPack tensor store".to_string(),
    })
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
        let tensor = dlpack_tensor(object).expect("DLPack tensor should export");

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
        let tensor = dlpack_tensor(object).expect("scalar tensor should export");

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
        let tensor = dlpack_tensor(object).expect("first consume should succeed");
        let consumed = dlpack_tensor(object).expect_err("second consume should fail");
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
            dlpack_tensor(invalid_name).expect_err("used capsule should be rejected");
        assert_eq!(invalid_name_error.exception_type, "SifrPythonDlpackError");

        let unsupported_dtype = exporter(DEVICE_CPU, 0, dtype(99, 32, 1), DLTENSOR_NAME)
            .expect("unsupported-dtype exporter should be stored");
        let dtype_error =
            dlpack_tensor(unsupported_dtype).expect_err("unsupported dtype should fail");
        assert_eq!(
            dtype_error.exception_type,
            "SifrPythonDlpackUnsupportedDtype"
        );

        let unsupported_device = exporter(2, 0, dtype(2, 32, 1), DLTENSOR_NAME)
            .expect("unsupported-device exporter should be stored");
        let device_error =
            dlpack_tensor(unsupported_device).expect_err("unsupported device should fail");
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

    def __dlpack__(self):
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

    def __dlpack__(self):
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
