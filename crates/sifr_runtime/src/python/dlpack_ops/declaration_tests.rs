use super::abi::{
    DLDataType, DLDevice, DLManagedTensor, DLManagedTensorVersioned, DLPackVersion, DLTensor,
    DLTENSOR_NAME, DLTENSOR_VERSIONED_NAME, USED_DLTENSOR_NAME,
};
use super::{
    acquire_dlpack_tensor, dlpack_stream, prepare_dlpack_argument, release_dlpack,
    PythonDlpackStreamMetadata, PythonError, DEVICE_CPU,
};
use crate::python::object_ops::{clone_handle, store_object};
use crate::python::{
    close_object, initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
    ObjectHandle,
};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyDict};
use std::ffi::{c_void, CStr};
use std::sync::atomic::{AtomicUsize, Ordering};

static LEGACY_RELEASES: AtomicUsize = AtomicUsize::new(0);
static VERSIONED_RELEASES: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
struct OwnedLegacy {
    managed: DLManagedTensor,
    shape: Vec<i64>,
    strides: Vec<i64>,
    data: Vec<u8>,
}

#[repr(C)]
struct OwnedVersioned {
    managed: DLManagedTensorVersioned,
    shape: Vec<i64>,
    strides: Vec<i64>,
    data: Vec<u8>,
}

#[test]
fn versioned_capsule_is_accepted_and_released_once() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-versioned")).expect("init should succeed");

    let object =
        exporter(versioned_capsule(0, 1, 3), DEVICE_CPU, 0).expect("exporter should exist");
    let tensor = acquire_dlpack_tensor(&object, "cpu", None, Some("float64"))
        .expect("versioned tensor should be acquired");
    assert_eq!(tensor.dtype, "float64");
    release_dlpack((tensor.handle, tensor.token)).expect("tensor should release");
    assert_eq!(VERSIONED_RELEASES.load(Ordering::SeqCst), 1);
    close_object(object).expect("exporter should close");
}

#[test]
fn versioned_copied_flag_is_rejected_without_leaking() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-copied")).expect("init should succeed");

    let object =
        exporter(versioned_capsule(1 << 1, 1, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let error = acquire_dlpack_tensor(&object, "cpu", None, Some("float64"))
        .expect_err("copied tensor must be rejected");
    assert!(error.message.contains("copied tensor"));
    close_object(object).expect("exporter should close");
    assert_eq!(VERSIONED_RELEASES.load(Ordering::SeqCst), 1);
}

#[test]
fn versioned_capsule_rejects_incompatible_major_version_without_leaking() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-major-version")).expect("init should succeed");

    let object =
        exporter(versioned_capsule(0, 2, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let error = acquire_dlpack_tensor(&object, "cpu", None, Some("float64"))
        .expect_err("incompatible major version must be rejected");
    assert!(error.message.contains("major version 2.0"));
    close_object(object).expect("exporter should close");
    assert_eq!(VERSIONED_RELEASES.load(Ordering::SeqCst), 1);
}

#[test]
fn acquisition_uses_full_signature_once_without_legacy_retry() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("dlpack-no-retry")).expect("init should succeed");
    let object = rejecting_exporter().expect("exporter should exist");

    let error = acquire_dlpack_tensor(&object, "cpu", None, Some("float64"))
        .expect_err("producer error should propagate");
    assert_eq!(error.exception_type, "TypeError");
    assert_eq!(attribute_i64(&object, "calls"), 1);
    close_object(object).expect("exporter should close");
}

#[test]
fn cuda_and_any_require_a_matching_explicit_stream() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-stream-policy")).expect("init should succeed");

    let no_stream = exporter(legacy_capsule(2, 3), 2, 3).expect("exporter should exist");
    let missing = acquire_dlpack_tensor(&no_stream, "any", None, Some("float64"))
        .expect_err("non-CPU any acquisition requires a stream");
    assert!(missing.message.contains("explicit consumer stream"));
    assert_eq!(attribute_i64(&no_stream, "calls"), 0);
    close_object(no_stream).expect("exporter should close");

    let mismatch = exporter(legacy_capsule(2, 3), 2, 3).expect("exporter should exist");
    let wrong_stream = PythonDlpackStreamMetadata {
        device_type: 2,
        device_id: 4,
        stream_token: 91,
    };
    let error = acquire_dlpack_tensor(&mismatch, "cuda", Some(&wrong_stream), Some("float64"))
        .expect_err("mismatched device id must fail");
    assert!(error.message.contains("family/id"));
    assert_eq!(attribute_i64(&mismatch, "calls"), 0);
    close_object(mismatch).expect("exporter should close");

    let matched = exporter(legacy_capsule(2, 3), 2, 3).expect("exporter should exist");
    let stream = PythonDlpackStreamMetadata {
        device_type: 2,
        device_id: 3,
        stream_token: 92,
    };
    let tensor = acquire_dlpack_tensor(&matched, "cuda", Some(&stream), Some("float64"))
        .expect("matching stream should acquire");
    assert_eq!(attribute_i64(&matched, "seen_stream"), 92);
    release_dlpack((tensor.handle, tensor.token)).expect("tensor should release");
    close_object(matched).expect("exporter should close");

    let cpu =
        exporter(legacy_capsule(DEVICE_CPU, 0), DEVICE_CPU, 0).expect("CPU exporter should exist");
    let cpu_stream = PythonDlpackStreamMetadata {
        device_type: i64::from(DEVICE_CPU),
        device_id: 0,
        stream_token: 93,
    };
    let tensor = acquire_dlpack_tensor(&cpu, "any", Some(&cpu_stream), Some("float64"))
        .expect("matching CPU stream metadata should acquire");
    assert_eq!(attribute_i64(&cpu, "seen_stream"), -1);
    release_dlpack((tensor.handle, tensor.token)).expect("CPU tensor should release");
    close_object(cpu).expect("CPU exporter should close");
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 4);
}

#[test]
fn capsule_device_mismatch_releases_the_acquired_tensor() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-capsule-device")).expect("init should succeed");

    let object = exporter(legacy_capsule(DEVICE_CPU, 0), 2, 3).expect("exporter should exist");
    let stream = PythonDlpackStreamMetadata {
        device_type: 2,
        device_id: 3,
        stream_token: 77,
    };
    let error = acquire_dlpack_tensor(&object, "cuda", Some(&stream), Some("float64"))
        .expect_err("capsule device mismatch must fail");
    assert!(error.message.contains("capsule device"));
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 1);
    close_object(object).expect("exporter should close");
}

#[test]
fn test_runtime_reset_releases_an_outstanding_tensor_once() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-reset")).expect("init should succeed");

    let object =
        exporter(legacy_capsule(DEVICE_CPU, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let _tensor = acquire_dlpack_tensor(&object, "cpu", None, Some("float64"))
        .expect("tensor should acquire");
    reset_runtime_state_for_tests();
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 1);
}

#[test]
fn normalized_stream_metadata_is_closed_and_checked() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("dlpack-stream-value")).expect("init should succeed");

    let stream = tuple_object((2_i64, 7_i64, 123_i64)).expect("tuple should be stored");
    let metadata = dlpack_stream(&stream, "cuda").expect("CUDA tuple should normalize");
    assert_eq!(metadata.device_type, 2);
    assert_eq!(metadata.device_id, 7);
    assert_eq!(metadata.stream_token, 123);
    close_object(stream).expect("tuple should close");

    let wrong = tuple_object((1_i64, 0_i64, 0_i64)).expect("tuple should be stored");
    let error = dlpack_stream(&wrong, "cuda").expect_err("wrong family should fail");
    assert!(error.message.contains("device family"));
    close_object(wrong).expect("tuple should close");

    let ambiguous = tuple_object((2_i64, 0_i64, 0_i64)).expect("tuple should be stored");
    let error = dlpack_stream(&ambiguous, "cuda").expect_err("zero CUDA stream should fail");
    assert!(error.message.contains("token 0"));
    close_object(ambiguous).expect("tuple should close");
}

#[test]
fn unconsumed_argument_releases_once_during_reconciliation() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-unconsumed-argument")).expect("init should succeed");

    let exporter =
        exporter(legacy_capsule(DEVICE_CPU, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let tensor = acquire_dlpack_tensor(&exporter, "cpu", None, Some("float64"))
        .expect("tensor should acquire");
    let argument =
        prepare_dlpack_argument((tensor.handle, tensor.token)).expect("argument should prepare");
    let temporary = argument.object().expect("argument object should exist");
    close_object(temporary).expect("temporary argument should close");
    argument.finish().expect("argument should reconcile");
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 1);
    close_object(exporter).expect("exporter should close");
}

#[test]
fn consumed_argument_transfers_deleter_ownership_exactly_once() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-consumed-argument")).expect("init should succeed");

    let exporter =
        exporter(legacy_capsule(DEVICE_CPU, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let tensor = acquire_dlpack_tensor(&exporter, "cpu", None, Some("float64"))
        .expect("tensor should acquire");
    let argument =
        prepare_dlpack_argument((tensor.handle, tensor.token)).expect("argument should prepare");
    let temporary = argument.object().expect("argument object should exist");
    let pointer = mark_argument_consumed(&temporary);
    close_object(temporary).expect("temporary argument should close");
    argument.finish().expect("argument should reconcile");
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 0);

    unsafe { legacy_deleter(pointer.cast()) };
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 1);
    close_object(exporter).expect("exporter should close");
}

#[test]
fn attach_failure_leaves_the_deleter_with_the_capsule_owner() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    reset_releases();
    initialize_runtime(test_config("dlpack-finalize-after-reset")).expect("init should succeed");

    let exporter =
        exporter(legacy_capsule(DEVICE_CPU, 0), DEVICE_CPU, 0).expect("exporter should exist");
    let tensor = acquire_dlpack_tensor(&exporter, "cpu", None, Some("float64"))
        .expect("tensor should acquire");
    let argument =
        prepare_dlpack_argument((tensor.handle, tensor.token)).expect("argument should prepare");

    reset_runtime_state_for_tests();
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 0);
    let error = argument
        .finish()
        .expect_err("a stopped runtime must reject finalization");
    assert!(error.message.contains("not been initialized"), "{error:?}");
    // The producer-named capsule remains the sole owner. In particular, the
    // rejected attach closure must not run the entry's parallel deleter.
    assert_eq!(LEGACY_RELEASES.load(Ordering::SeqCst), 0);
}

fn reset_releases() {
    LEGACY_RELEASES.store(0, Ordering::SeqCst);
    VERSIONED_RELEASES.store(0, Ordering::SeqCst);
}

fn exporter(
    capsule: Result<Py<PyAny>, PythonError>,
    device_type: i32,
    device_id: i32,
) -> Result<ObjectHandle, PythonError> {
    let capsule = capsule?;
    super::super::attach(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("CAPSULE", capsule.bind(py))
            .map_err(test_pyerr(py, "capsule"))?;
        globals
            .set_item("DEVICE", (device_type, device_id))
            .map_err(test_pyerr(py, "device"))?;
        py.run(
            cr#"
class Exporter:
    def __init__(self):
        self.calls = 0
        self.seen_stream = -1
        self.device = DEVICE
        self.capsule = CAPSULE

    def __dlpack_device__(self):
        return self.device

    def __dlpack__(self, *, stream, max_version, copy):
        self.calls += 1
        self.seen_stream = -1 if stream is None else stream
        assert max_version == (1, 0)
        assert copy is False
        return self.capsule

obj = Exporter()
del CAPSULE, DEVICE
"#,
            Some(&globals),
            None,
        )
        .map_err(test_pyerr(py, "exporter"))?;
        let object = globals
            .get_item("obj")
            .map_err(test_pyerr(py, "exporter"))?
            .ok_or_else(|| super::dlpack_error("test exporter did not create obj"))?
            .unbind();
        globals
            .del_item("obj")
            .map_err(test_pyerr(py, "exporter cleanup"))?;
        store_object(object)
    })
    .map_err(PythonError::runtime)?
}

fn rejecting_exporter() -> Result<ObjectHandle, PythonError> {
    super::super::attach(|py| {
        let globals = PyDict::new(py);
        py.run(
            cr#"
class Exporter:
    def __init__(self):
        self.calls = 0

    def __dlpack_device__(self):
        return (1, 0)

    def __dlpack__(self, **kwargs):
        self.calls += 1
        raise TypeError("producer rejected the versioned request")

obj = Exporter()
"#,
            Some(&globals),
            None,
        )
        .map_err(test_pyerr(py, "rejecting exporter"))?;
        let object = globals
            .get_item("obj")
            .map_err(test_pyerr(py, "rejecting exporter"))?
            .ok_or_else(|| super::dlpack_error("test exporter did not create obj"))?
            .unbind();
        globals
            .del_item("obj")
            .map_err(test_pyerr(py, "rejecting exporter cleanup"))?;
        store_object(object)
    })
    .map_err(PythonError::runtime)?
}

fn tuple_object(value: (i64, i64, i64)) -> Result<ObjectHandle, PythonError> {
    super::super::attach(|py| {
        value
            .into_pyobject(py)
            .map_err(test_pyerr(py, "stream tuple"))
            .and_then(|tuple| store_object(tuple.into_any().unbind()))
    })
    .map_err(PythonError::runtime)?
}

fn attribute_i64(object: &ObjectHandle, name: &str) -> i64 {
    super::super::attach(|py| {
        clone_handle(py, object)?
            .bind(py)
            .getattr(name)
            .and_then(|value| value.extract::<i64>())
            .map_err(test_pyerr(py, "test attribute"))
    })
    .expect("runtime should be attached")
    .expect("attribute should be readable")
}

fn mark_argument_consumed(object: &ObjectHandle) -> *mut c_void {
    super::super::attach(|py| {
        let object = clone_handle(py, object)?;
        let capsule = object
            .bind(py)
            .cast::<PyCapsule>()
            .map_err(|_| super::dlpack_error("argument was not a capsule"))?;
        let pointer =
            unsafe { ffi::PyCapsule_GetPointer(capsule.as_ptr(), DLTENSOR_NAME.as_ptr()) };
        assert!(!pointer.is_null());
        let renamed =
            unsafe { ffi::PyCapsule_SetName(capsule.as_ptr(), USED_DLTENSOR_NAME.as_ptr()) };
        assert_eq!(renamed, 0);
        Ok::<*mut c_void, PythonError>(pointer)
    })
    .expect("runtime should be attached")
    .expect("argument should be consumable")
}

fn legacy_capsule(device_type: i32, device_id: i32) -> Result<Py<PyAny>, PythonError> {
    super::super::attach(|py| {
        let mut owned = Box::new(OwnedLegacy {
            managed: DLManagedTensor {
                dl_tensor: tensor(device_type, device_id),
                manager_ctx: std::ptr::null_mut(),
                deleter: Some(legacy_deleter),
            },
            shape: vec![2, 3],
            strides: vec![3, 1],
            data: vec![0; 48],
        });
        wire_tensor(
            &mut owned.managed.dl_tensor,
            &mut owned.shape,
            &mut owned.strides,
            &mut owned.data,
        );
        capsule_from_pointer(
            py,
            Box::into_raw(owned).cast(),
            DLTENSOR_NAME,
            legacy_capsule_destructor,
        )
    })
    .map_err(PythonError::runtime)?
}

fn versioned_capsule(
    flags: u64,
    version_major: u32,
    version_minor: u32,
) -> Result<Py<PyAny>, PythonError> {
    super::super::attach(|py| {
        let mut owned = Box::new(OwnedVersioned {
            managed: DLManagedTensorVersioned {
                version: DLPackVersion {
                    major: version_major,
                    minor: version_minor,
                },
                manager_ctx: std::ptr::null_mut(),
                deleter: Some(versioned_deleter),
                flags,
                dl_tensor: tensor(DEVICE_CPU, 0),
            },
            shape: vec![2, 3],
            strides: vec![3, 1],
            data: vec![0; 48],
        });
        wire_tensor(
            &mut owned.managed.dl_tensor,
            &mut owned.shape,
            &mut owned.strides,
            &mut owned.data,
        );
        capsule_from_pointer(
            py,
            Box::into_raw(owned).cast(),
            DLTENSOR_VERSIONED_NAME,
            versioned_capsule_destructor,
        )
    })
    .map_err(PythonError::runtime)?
}

fn tensor(device_type: i32, device_id: i32) -> DLTensor {
    DLTensor {
        data: std::ptr::null_mut(),
        device: DLDevice {
            device_type,
            device_id,
        },
        ndim: 2,
        dtype: DLDataType {
            code: 2,
            bits: 64,
            lanes: 1,
        },
        shape: std::ptr::null_mut(),
        strides: std::ptr::null_mut(),
        byte_offset: 0,
    }
}

fn wire_tensor(tensor: &mut DLTensor, shape: &mut [i64], strides: &mut [i64], data: &mut [u8]) {
    tensor.data = data.as_mut_ptr().cast();
    tensor.shape = shape.as_mut_ptr();
    tensor.strides = strides.as_mut_ptr();
}

fn capsule_from_pointer(
    py: Python<'_>,
    pointer: *mut c_void,
    name: &'static CStr,
    destructor: unsafe extern "C" fn(*mut ffi::PyObject),
) -> Result<Py<PyAny>, PythonError> {
    let pointer = std::ptr::NonNull::new(pointer)
        .ok_or_else(|| super::dlpack_error("test tensor pointer was null"))?;
    let capsule =
        unsafe { PyCapsule::new_with_pointer_and_destructor(py, pointer, name, Some(destructor)) }
            .map_err(test_pyerr(py, "test capsule"))?;
    Ok(capsule.into_any().unbind())
}

fn test_pyerr<'py>(
    py: Python<'py>,
    context: &'static str,
) -> impl FnOnce(PyErr) -> PythonError + 'py {
    move |error| PythonError::from_pyerr(py, error, "zero-copy", context)
}

unsafe extern "C" fn legacy_capsule_destructor(capsule: *mut ffi::PyObject) {
    release_capsule(capsule, DLTENSOR_NAME, |pointer| unsafe {
        legacy_deleter(pointer.cast())
    });
}

unsafe extern "C" fn versioned_capsule_destructor(capsule: *mut ffi::PyObject) {
    release_capsule(capsule, DLTENSOR_VERSIONED_NAME, |pointer| unsafe {
        versioned_deleter(pointer.cast())
    });
}

fn release_capsule(
    capsule: *mut ffi::PyObject,
    expected: &'static CStr,
    release: impl FnOnce(*mut c_void),
) {
    let name = unsafe { ffi::PyCapsule_GetName(capsule) };
    if name.is_null() || unsafe { CStr::from_ptr(name) } != expected {
        return;
    }
    let pointer = unsafe { ffi::PyCapsule_GetPointer(capsule, expected.as_ptr()) };
    if !pointer.is_null() {
        release(pointer);
    }
}

unsafe extern "C" fn legacy_deleter(tensor: *mut DLManagedTensor) {
    LEGACY_RELEASES.fetch_add(1, Ordering::SeqCst);
    if !tensor.is_null() {
        let _owned = unsafe { Box::from_raw(tensor.cast::<OwnedLegacy>()) };
    }
}

unsafe extern "C" fn versioned_deleter(tensor: *mut DLManagedTensorVersioned) {
    VERSIONED_RELEASES.fetch_add(1, Ordering::SeqCst);
    if !tensor.is_null() {
        let _owned = unsafe { Box::from_raw(tensor.cast::<OwnedVersioned>()) };
    }
}
