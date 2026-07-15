use super::*;
use crate::python::{
    close_object, from_bytes, from_int, initialize_runtime, reset_runtime_state_for_tests,
    resource_diagnostics, test_config, test_guard, PythonResourceDiagnostics,
    PythonResourceIdentity,
};
use pyo3::types::PyAnyMethods;
use pyo3::{ffi, Bound};
use std::{mem::size_of, ptr};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Barrier,
    },
    time::Duration,
};

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

#[test]
fn read_declaration_rejects_write_on_mutable_exporter() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-read-access")).expect("init should succeed");

    let object = python_i32_array([10_i32, 20, 30]);
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I32,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("read view should acquire from mutable exporter");
    let key = (view.handle, view.token);

    let error = buffer_write_i32(key, 1, 25).expect_err("read declaration blocks mutation");
    assert_eq!(error.exception_type, "BufferError");
    assert_eq!(buffer_read_i32(key, 1).expect("read remains valid"), 20);

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
}

#[test]
fn exporter_admission_allows_shared_reads_and_excludes_writers() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-exporter-admission")).expect("init should succeed");

    let object = python_i32_array([10_i32, 20, 30]);
    let read_request = PythonBufferRequest {
        element: PythonBufferElement::I32,
        access: PythonBufferAccess::Read,
        layout: PythonBufferLayout::Any,
    };
    let write_request = PythonBufferRequest {
        access: PythonBufferAccess::Write,
        ..read_request
    };

    let first_read = acquire_buffer(&object, read_request).expect("first read should acquire");
    let second_read = acquire_buffer(&object, read_request).expect("shared read should acquire");
    let write_error = acquire_buffer(&object, write_request)
        .expect_err("writer must be excluded while readers are live");
    assert_eq!(write_error.exception_type, "BufferError");
    assert_eq!(write_error.context, "buffer exporter access admission");

    release_buffer((first_read.handle, first_read.token)).expect("first read should release");
    let remaining_read_error = acquire_buffer(&object, write_request)
        .expect_err("remaining reader must continue excluding writers");
    assert_eq!(
        remaining_read_error.context,
        "buffer exporter access admission"
    );
    release_buffer((second_read.handle, second_read.token)).expect("second read should release");

    let writer = acquire_buffer(&object, write_request).expect("writer should acquire");
    for request in [read_request, write_request] {
        let error = acquire_buffer(&object, request)
            .expect_err("live writer must exclude every other view");
        assert_eq!(error.context, "buffer exporter access admission");
    }
    release_buffer((writer.handle, writer.token)).expect("writer should release");

    let read_after_release =
        acquire_buffer(&object, read_request).expect("admission should reopen after release");
    release_buffer((read_after_release.handle, read_after_release.token))
        .expect("final read should release");
    close_object(object).expect("close exporter");
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
fn admission_excludes_writable_aliases_from_distinct_views_of_shared_storage() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-shared-storage-admission"))
        .expect("init should succeed");

    let (first, second) = shared_i32_memoryviews();
    let read_request = PythonBufferRequest {
        element: PythonBufferElement::I32,
        access: PythonBufferAccess::Read,
        layout: PythonBufferLayout::Any,
    };
    let write_request = PythonBufferRequest {
        access: PythonBufferAccess::Write,
        ..read_request
    };

    let reader = acquire_buffer(&first, read_request).expect("reader should acquire");
    let error = acquire_buffer(&second, write_request)
        .expect_err("a distinct wrapper over shared storage must not admit a writer");
    assert_eq!(error.context, "buffer exporter access admission");
    release_buffer((reader.handle, reader.token)).expect("reader should release");

    let writer = acquire_buffer(&second, write_request).expect("writer should acquire");
    for request in [read_request, write_request] {
        let error = acquire_buffer(&first, request)
            .expect_err("shared backing storage must remain exclusive while writing");
        assert_eq!(error.context, "buffer exporter access admission");
    }
    release_buffer((writer.handle, writer.token)).expect("writer should release");

    close_object(first).expect("first wrapper should close");
    close_object(second).expect("second wrapper should close");
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
fn admission_allows_disjoint_writable_slices_of_shared_storage() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-disjoint-storage-admission"))
        .expect("init should succeed");

    let (first, second) = disjoint_i32_memoryviews();
    let write_request = PythonBufferRequest {
        element: PythonBufferElement::I32,
        access: PythonBufferAccess::Write,
        layout: PythonBufferLayout::Any,
    };
    let first_writer = acquire_buffer(&first, write_request).expect("first writer should acquire");
    let second_writer =
        acquire_buffer(&second, write_request).expect("disjoint writer should acquire");
    buffer_write_i32((first_writer.handle, first_writer.token), 0, 11)
        .expect("first slice should write");
    buffer_write_i32((second_writer.handle, second_writer.token), 0, 31)
        .expect("second slice should write");
    release_buffer((first_writer.handle, first_writer.token)).expect("first writer should release");
    release_buffer((second_writer.handle, second_writer.token))
        .expect("second writer should release");
    close_object(first).expect("first slice should close");
    close_object(second).expect("second slice should close");
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
fn admission_allows_interleaved_disjoint_writable_strides() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-interleaved-storage-admission"))
        .expect("init should succeed");

    let (even, odd) = interleaved_byte_memoryviews();
    let request = PythonBufferRequest {
        element: PythonBufferElement::U8,
        access: PythonBufferAccess::Write,
        layout: PythonBufferLayout::Any,
    };
    let even_writer = acquire_buffer(&even, request).expect("even writer should acquire");
    let odd_writer = acquire_buffer(&odd, request).expect("odd writer should acquire");
    buffer_write_u8((even_writer.handle, even_writer.token), 1, 8)
        .expect("even stride should write");
    buffer_write_u8((odd_writer.handle, odd_writer.token), 1, 9).expect("odd stride should write");
    release_buffer((even_writer.handle, even_writer.token)).expect("even writer should release");
    release_buffer((odd_writer.handle, odd_writer.token)).expect("odd writer should release");
    close_object(even).expect("even view should close");
    close_object(odd).expect("odd view should close");
}

#[test]
fn any_layout_supports_strided_logical_access() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-any-layout")).expect("init should succeed");

    let object = strided_byte_view();
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("any layout should accept a strided view");
    let key = (view.handle, view.token);

    assert!(!view.c_contiguous);
    assert_eq!(buffer_read_u8(key, 0).expect("first logical item"), 1);
    assert_eq!(buffer_read_u8(key, 1).expect("second logical item"), 3);
    assert_eq!(
        copy_buffer_slice_u8(key, 0, 2).expect("logical copy"),
        vec![1, 3]
    );

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
}

#[test]
fn any_layout_supports_negative_stride_read_write_and_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-negative-stride")).expect("init should succeed");

    let object = negative_strided_byte_view();
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("writable negative-stride view should acquire");
    let key = (view.handle, view.token);

    assert_eq!(view.strides, vec![-1]);
    assert_eq!(
        copy_buffer_slice_u8(key, 0, 4).expect("logical copy"),
        vec![4, 3, 2, 1]
    );
    buffer_write_u8(key, 1, 9).expect("logical write through negative stride");
    assert_eq!(buffer_read_u8(key, 1).expect("read after write"), 9);
    assert_eq!(
        buffer_read_u8(key, 4)
            .expect_err("upper bound must be rejected")
            .exception_type,
        "IndexError"
    );

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
}

#[test]
fn any_layout_supports_indirect_pointer_read_write_and_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-indirect-pointer")).expect("init should succeed");

    let (object, storage) = indirect_byte_memoryview();
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("writable indirect view should acquire");
    let key = (view.handle, view.token);

    assert_eq!(view.shape, vec![2, 2]);
    assert_eq!(view.suboffsets, vec![0, -1]);
    assert_eq!(
        copy_buffer_slice_u8(key, 0, 4).expect("logical copy"),
        vec![1, 2, 3, 4]
    );
    buffer_write_u8(key, 2, 9).expect("write through intermediate row pointer");
    assert_eq!(buffer_read_u8(key, 2).expect("read after write"), 9);
    assert_eq!(
        copy_buffer_slice_u8(key, 1, 2).expect("bounded slice"),
        vec![2, 9]
    );
    assert_eq!(
        buffer_write_u8(key, 4, 0)
            .expect_err("upper bound must be rejected")
            .exception_type,
        "IndexError"
    );

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
    assert_eq!(*storage.first, [1, 2]);
    assert_eq!(*storage.second, [9, 4]);
    drop(storage);
}

#[test]
fn scalar_native_endian_buffer_has_empty_metadata_vectors() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-scalar")).expect("init should succeed");

    let object = super::super::attach(|py| {
        let scalar = py
            .import("ctypes")?
            .getattr("c_int32")?
            .call1((0x0102_0304_i32,))?;
        super::super::object_ops::store_object(scalar.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("ctypes scalar should create");
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I32,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("zero-dimensional scalar should acquire");
    let key = (view.handle, view.token);

    assert_eq!(view.dimensions, 0);
    assert!(view.shape.is_empty());
    assert!(view.strides.is_empty());
    assert!(view.suboffsets.is_empty());
    assert_eq!(buffer_read_i32(key, 0).expect("scalar read"), 0x0102_0304);

    release_buffer(key).expect("release");
    close_object(object).expect("close exporter");
}

#[test]
fn explicit_release_drops_export_even_when_a_runtime_snapshot_exists() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-release-linearization")).expect("init should succeed");

    let object = super::super::attach(|py| {
        let exporter = py
            .import("builtins")?
            .getattr("bytearray")?
            .call1(([1_u8],))?;
        super::super::object_ops::store_object(exporter.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("bytearray should create");
    let view = buffer_u8(&object, false).expect("view should acquire");
    let key = (view.handle, view.token);
    let (snapshot, _) = buffer_snapshot(key).expect("runtime snapshot should exist");

    release_buffer(key).expect("explicit release should drain and release");
    super::super::attach(|py| {
        let exporter = clone_handle(py, &object)
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        exporter.bind(py).call_method1("append", (2_u8,))?;
        Ok::<(), pyo3::PyErr>(())
    })
    .expect("attach should succeed")
    .expect("resize proves PyBuffer_Release completed");

    drop(snapshot);
    close_object(object).expect("close exporter");
}

#[test]
fn error_conversion_runs_after_unlock_during_concurrent_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-error-release-race")).expect("init should succeed");

    let object = from_bytes(&[1]).expect("bytes should be stored");
    let view = buffer_u8(&object, false).expect("view should acquire");
    let key = (view.handle, view.token);
    let barrier = Arc::new(Barrier::new(2));
    let release_barrier = Arc::clone(&barrier);
    let (released_tx, released_rx) = mpsc::sync_channel(1);
    let release_thread = std::thread::spawn(move || {
        release_barrier.wait();
        let result = release_buffer(key);
        released_tx
            .send(result)
            .expect("error conversion observer should remain live");
    });
    let release_completed_during_conversion = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&release_completed_during_conversion);

    let error = with_live_buffer_and_converter(
        key,
        PythonBufferElement::U8,
        false,
        |_value| Err::<(), _>(BufferAccessError::Index("forced bounds failure")),
        |error| {
            barrier.wait();
            let converted = super::super::attach(|py| error.into_python(py));
            if let Ok(result) = released_rx.recv_timeout(Duration::from_secs(2)) {
                result.expect("concurrent release should succeed");
                observed.store(true, Ordering::SeqCst);
            }
            converted
        },
    )
    .expect_err("forced access failure should be converted");

    release_thread.join().expect("release thread should finish");
    assert!(release_completed_during_conversion.load(Ordering::SeqCst));
    assert_eq!(error.exception_type, "IndexError");
    close_object(object).expect("exporter should close");
}

fn python_i32_array(values: [i32; 3]) -> ObjectHandle {
    super::super::attach(|py| {
        let array = py.import("array")?.getattr("array")?.call1(("i", values))?;
        super::super::object_ops::store_object(array.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("array should create")
}

fn shared_i32_memoryviews() -> (ObjectHandle, ObjectHandle) {
    super::super::attach(|py| {
        let array = py
            .import("array")?
            .getattr("array")?
            .call1(("i", [10_i32, 20, 30]))?;
        let memoryview = py.import("builtins")?.getattr("memoryview")?;
        let first = memoryview.call1((&array,))?;
        let second = memoryview.call1((&array,))?;
        let first = super::super::object_ops::store_object(first.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let second = super::super::object_ops::store_object(second.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        Ok::<_, pyo3::PyErr>((first, second))
    })
    .expect("attach should succeed")
    .expect("shared memoryviews should create")
}

fn disjoint_i32_memoryviews() -> (ObjectHandle, ObjectHandle) {
    super::super::attach(|py| {
        let array = py
            .import("array")?
            .getattr("array")?
            .call1(("i", [10_i32, 20, 30, 40]))?;
        let view = py
            .import("builtins")?
            .getattr("memoryview")?
            .call1((&array,))?;
        let first = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 0, 2, 1),))?;
        let second = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 2, 4, 1),))?;
        let first = super::super::object_ops::store_object(first.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let second = super::super::object_ops::store_object(second.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        Ok::<_, pyo3::PyErr>((first, second))
    })
    .expect("attach should succeed")
    .expect("disjoint memoryviews should create")
}

fn interleaved_byte_memoryviews() -> (ObjectHandle, ObjectHandle) {
    super::super::attach(|py| {
        let builtins = py.import("builtins")?;
        let bytes = builtins.getattr("bytearray")?.call1(([1_u8, 2, 3, 4],))?;
        let view = builtins.getattr("memoryview")?.call1((bytes,))?;
        let even = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 0, 4, 2),))?;
        let odd = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 1, 4, 2),))?;
        let even = super::super::object_ops::store_object(even.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let odd = super::super::object_ops::store_object(odd.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        Ok::<_, pyo3::PyErr>((even, odd))
    })
    .expect("attach should succeed")
    .expect("interleaved memoryviews should create")
}

fn strided_byte_view() -> ObjectHandle {
    super::super::attach(|py| {
        let builtins = py.import("builtins")?;
        let bytes = builtins.getattr("bytearray")?.call1(([1_u8, 2, 3, 4],))?;
        let view = builtins.getattr("memoryview")?.call1((bytes,))?;
        let slice = view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 0, 4, 2),))?;
        super::super::object_ops::store_object(slice.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("strided view should create")
}

fn negative_strided_byte_view() -> ObjectHandle {
    super::super::attach(|py| {
        let builtins = py.import("builtins")?;
        let bytes = builtins.getattr("bytearray")?.call1(([1_u8, 2, 3, 4],))?;
        let view = builtins.getattr("memoryview")?.call1((bytes,))?;
        let slice =
            view.call_method1("__getitem__", (pyo3::types::PySlice::new(py, 3, -5, -1),))?;
        super::super::object_ops::store_object(slice.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("negative-stride view should create")
}

struct IndirectByteStorage {
    first: Box<[u8; 2]>,
    second: Box<[u8; 2]>,
    rows: Box<[*mut u8; 2]>,
    shape: Box<[isize; 2]>,
    strides: Box<[isize; 2]>,
    suboffsets: Box<[isize; 2]>,
}

fn indirect_byte_memoryview() -> (ObjectHandle, IndirectByteStorage) {
    let mut first = Box::new([1_u8, 2]);
    let mut second = Box::new([3_u8, 4]);
    let rows = Box::new([first.as_mut_ptr(), second.as_mut_ptr()]);
    let mut storage = IndirectByteStorage {
        first,
        second,
        rows,
        shape: Box::new([2_isize, 2]),
        strides: Box::new([
            isize::try_from(size_of::<*mut u8>()).expect("pointer width fits Py_ssize_t"),
            1,
        ]),
        suboffsets: Box::new([0_isize, -1]),
    };
    let object = super::super::attach(|py| {
        let view = ffi::Py_buffer {
            buf: storage.rows.as_mut_ptr().cast(),
            obj: ptr::null_mut(),
            len: 4,
            itemsize: 1,
            readonly: 0,
            ndim: 2,
            format: c"B".as_ptr().cast_mut(),
            shape: storage.shape.as_mut_ptr(),
            strides: storage.strides.as_mut_ptr(),
            suboffsets: storage.suboffsets.as_mut_ptr(),
            internal: ptr::null_mut(),
        };
        // SAFETY: `storage` owns stable boxed data and metadata until after the
        // returned memoryview and acquired buffer are explicitly closed.
        let memoryview =
            unsafe { Bound::from_owned_ptr_or_err(py, ffi::PyMemoryView_FromBuffer(&view))? };
        super::super::object_ops::store_object(memoryview.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("indirect memoryview should create");
    (object, storage)
}
