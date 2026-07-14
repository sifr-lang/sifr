use super::*;
use crate::python::{
    close_object, from_bytes, from_int, initialize_runtime, reset_runtime_state_for_tests,
    resource_diagnostics, test_config, test_guard, PythonResourceDiagnostics,
    PythonResourceIdentity,
};
use pyo3::types::PyAnyMethods;

#[test]
fn buffer_view_tracks_metadata_copy_and_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-view")).expect("init should succeed");

    let object = from_bytes(&[1, 2, 3]).expect("bytes should be stored");
    let view = buffer_u8(&object, false).expect("bytes should expose u8 buffer");

    assert_eq!(view.element, PythonBufferElement::U8);
    assert_eq!(view.len_bytes, 3);
    assert_eq!(view.item_size, 1);
    assert!(view.readonly);
    assert!(view.c_contiguous);
    assert_eq!(
        buffer_read_u8((view.handle, view.token), 1).expect("read"),
        2
    );
    assert_eq!(
        copy_buffer_slice_u8((view.handle, view.token), 1, 2).expect("slice should copy"),
        vec![2, 3]
    );
    assert_eq!(
        copy_buffer_u8((view.handle, view.token)).expect("buffer should copy"),
        vec![1, 2, 3]
    );
    assert_eq!(
        resource_diagnostics().expect("diagnostics should be available"),
        PythonResourceDiagnostics {
            initialized: true,
            live_objects: 2,
            leaked_objects: 0,
        }
    );
    release_buffer((view.handle, view.token)).expect("buffer should release");
    close_object(object).expect("object should close");
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
fn buffer_double_release_and_use_after_release_are_deterministic() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-double-release")).expect("init should succeed");

    let object = from_bytes(&[1]).expect("bytes should be stored");
    let view = buffer_u8(&object, false).expect("bytes should expose u8 buffer");
    release_buffer((view.handle, view.token)).expect("buffer should release");
    let error = release_buffer((view.handle, view.token)).expect_err("second release fails");
    let copy_error =
        copy_buffer_u8((view.handle, view.token)).expect_err("copy after release fails");

    assert_eq!(error.kind, "resource");
    assert_eq!(error.exception_type, "SifrPythonClosedBuffer");
    assert_eq!(copy_error.kind, "resource");
    assert_eq!(copy_error.exception_type, "SifrPythonClosedBuffer");
    close_object(object).expect("object should close");
}

#[test]
fn sealed_resource_identity_drop_releases_buffer_and_metadata() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-identity-drop")).expect("init should succeed");

    let object = from_bytes(&[1, 2]).expect("bytes should be stored");
    let view = buffer_u8(&object, false).expect("bytes should expose u8 buffer");
    let key = (view.handle, view.token);
    let identity = PythonResourceIdentity::buffer(key);
    assert_eq!(buffer_shape(key).expect("metadata should be live"), vec![2]);

    drop(identity);

    let error = buffer_shape(key).expect_err("metadata drops with the resource");
    assert_eq!(error.exception_type, "SifrPythonClosedBuffer");
    close_object(object).expect("object should close");
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
fn buffer_rejects_wrong_dtype_layout_bounds_and_readonly_write() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-rejects")).expect("init should succeed");

    let object = from_bytes(&[1]).expect("bytes should be stored");
    let writable = buffer_u8(&object, true).expect_err("bytes are readonly");
    assert_eq!(writable.kind, "zero-copy");
    let wrong_type = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I16,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::CContiguous,
        },
    )
    .expect_err("bytes are not int16");
    assert_eq!(wrong_type.kind, "zero-copy");

    let strided = super::super::attach(|py| {
        let builtins = py.import("builtins")?;
        let bytes = builtins.getattr("bytearray")?.call1(([1_u8, 2, 3, 4],))?;
        let view = builtins.getattr("memoryview")?.call1((bytes,))?;
        let slice = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 0, 4, 2),))?;
        super::super::object_ops::store_object(slice.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("strided view should create");
    let wrong_layout = acquire_buffer(
        &strided,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::CContiguous,
        },
    )
    .expect_err("strided view is not C contiguous");
    assert_eq!(wrong_layout.exception_type, "BufferError");

    let view = buffer_u8(&object, false).expect("bytes view");
    let bounds = buffer_read_u8((view.handle, view.token), 1).expect_err("bounds fail");
    assert_eq!(bounds.exception_type, "IndexError");
    release_buffer((view.handle, view.token)).expect("release");

    let integer = from_int(1).expect("int should be stored");
    let unsupported = buffer_u8(&integer, false).expect_err("int has no u8 buffer");
    assert_eq!(unsupported.kind, "zero-copy");

    close_object(object).expect("object should close");
    close_object(strided).expect("strided view should close");
    close_object(integer).expect("integer should close");
}

#[test]
fn typed_writable_buffer_supports_checked_read_write_and_slice_copy() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-typed-write")).expect("init should succeed");

    let object = super::super::attach(|py| {
        let array = py
            .import("array")?
            .getattr("array")?
            .call1(("i", [10_i32, 20, 30]))?;
        super::super::object_ops::store_object(array.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("array should create");
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I32,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::CContiguous,
        },
    )
    .expect("typed buffer should acquire");
    let key = (view.handle, view.token);

    assert_eq!(buffer_read_i32(key, 1).expect("read"), 20);
    buffer_write_i32(key, 1, 25).expect("write");
    assert_eq!(buffer_read_i32(key, 1).expect("read after write"), 25);
    assert_eq!(
        copy_buffer_slice_i32(key, 0, 3).expect("slice"),
        vec![10, 25, 30]
    );

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
}
