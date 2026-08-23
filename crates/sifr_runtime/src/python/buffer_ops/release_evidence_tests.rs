use super::*;
use crate::python::{
    PythonResourceIdentity, close_object, initialize_runtime, reset_runtime_state_for_tests,
    test_config, test_guard,
};
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const BUFFER_FORMAT: &[u8; 2] = b"B\0";

#[derive(Default)]
struct ExporterMetrics {
    acquisitions: AtomicUsize,
    releases: AtomicUsize,
}

#[pyclass]
struct InstrumentedExporter {
    storage: Vec<u8>,
    metrics: Arc<ExporterMetrics>,
}

#[pymethods]
impl InstrumentedExporter {
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        if view.is_null() {
            return Err(PyBufferError::new_err("buffer view is null"));
        }

        let exporter = slf.borrow();
        let data_pointer = exporter.storage.as_ptr().cast::<c_void>().cast_mut();
        let length = isize::try_from(exporter.storage.len())
            .map_err(|_| PyBufferError::new_err("buffer length exceeds Py_ssize_t"))?;
        exporter
            .metrics
            .acquisitions
            .fetch_add(1, Ordering::Relaxed);
        drop(exporter);

        unsafe {
            (*view).obj = slf.into_any().into_ptr();
            (*view).buf = data_pointer;
            (*view).len = length;
            (*view).readonly = 0;
            (*view).itemsize = 1;
            (*view).format = if flags & ffi::PyBUF_FORMAT == ffi::PyBUF_FORMAT {
                BUFFER_FORMAT.as_ptr().cast::<c_char>().cast_mut()
            } else {
                ptr::null_mut()
            };
            (*view).ndim = 1;
            (*view).shape = if flags & ffi::PyBUF_ND == ffi::PyBUF_ND {
                &raw mut (*view).len
            } else {
                ptr::null_mut()
            };
            (*view).strides = if flags & ffi::PyBUF_STRIDES == ffi::PyBUF_STRIDES {
                &raw mut (*view).itemsize
            } else {
                ptr::null_mut()
            };
            (*view).suboffsets = ptr::null_mut();
            (*view).internal = ptr::null_mut();
        }
        Ok(())
    }

    unsafe fn __releasebuffer__(&self, _view: *mut ffi::Py_buffer) {
        self.metrics.releases.fetch_add(1, Ordering::Relaxed);
    }
}

struct InstrumentedEvidence {
    object: ObjectHandle,
    metrics: Arc<ExporterMetrics>,
    data_pointer: usize,
}

#[test]
fn instrumented_exporter_explicit_release_is_exact_once_and_pointer_identical() {
    let _guard = test_guard();
    let evidence = setup_instrumented_exporter("buffer-release-evidence-explicit");
    let view = buffer_u8(&evidence.object, true).expect("instrumented buffer should acquire");
    let key = (view.handle, view.token);

    assert_eq!(admitted_pointer(view.handle), evidence.data_pointer);
    assert_eq!(evidence.acquisitions(), 1);
    assert_eq!(evidence.releases(), 0);
    release_buffer(key).expect("explicit release should succeed");
    assert_eq!(evidence.releases(), 1);
    release_buffer(key).expect_err("double release should fail before touching the exporter");
    assert_eq!(evidence.releases(), 1);

    close_object(evidence.object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_automatic_resource_drop_releases_exactly_once() {
    let _guard = test_guard();
    let evidence = setup_instrumented_exporter("buffer-release-evidence-drop");
    let view = buffer_u8(&evidence.object, false).expect("instrumented buffer should acquire");

    drop(PythonResourceIdentity::buffer((view.handle, view.token)));

    assert_eq!(evidence.acquisitions(), 1);
    assert_eq!(evidence.releases(), 1);
    close_object(evidence.object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_validation_failure_rolls_back_exactly_once() {
    let _guard = test_guard();
    let evidence = setup_instrumented_exporter("buffer-release-evidence-validation");
    let error = acquire_buffer(
        &evidence.object,
        PythonBufferRequest {
            element: PythonBufferElement::I16,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect_err("uint8 exporter must reject an int16 declaration");

    assert_eq!(error.context, "buffer metadata validation");
    assert_eq!(evidence.acquisitions(), 1);
    assert_eq!(evidence.releases(), 1);
    close_object(evidence.object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_admission_conflict_releases_rejected_view_exactly_once() {
    let _guard = test_guard();
    let evidence = setup_instrumented_exporter("buffer-release-evidence-admission");
    let reader = buffer_u8(&evidence.object, false).expect("first view should acquire");
    let error = buffer_u8(&evidence.object, true).expect_err("overlapping writer must be rejected");

    assert_eq!(error.context, "buffer exporter access admission");
    assert_eq!(evidence.acquisitions(), 2);
    assert_eq!(evidence.releases(), 1);
    release_buffer((reader.handle, reader.token)).expect("reader should release");
    assert_eq!(evidence.releases(), 2);
    close_object(evidence.object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_store_failure_rolls_back_exactly_once() {
    let _guard = test_guard();
    let evidence = setup_instrumented_exporter("buffer-release-evidence-store");
    let previous_handle = {
        let mut store = buffer_store().expect("buffer store should lock");
        std::mem::replace(&mut store.next_handle, i64::MAX)
    };

    let error =
        buffer_u8(&evidence.object, false).expect_err("exhausted handle space must reject storage");

    buffer_store()
        .expect("buffer store should lock")
        .next_handle = previous_handle;
    assert_eq!(error.kind, "runtime");
    assert_eq!(evidence.acquisitions(), 1);
    assert_eq!(evidence.releases(), 1);
    close_object(evidence.object).expect("exporter should close");
}

impl InstrumentedEvidence {
    fn acquisitions(&self) -> usize {
        self.metrics.acquisitions.load(Ordering::Relaxed)
    }

    fn releases(&self) -> usize {
        self.metrics.releases.load(Ordering::Relaxed)
    }
}

fn setup_instrumented_exporter(config_name: &str) -> InstrumentedEvidence {
    reset_runtime_state_for_tests();
    initialize_runtime(test_config(config_name)).expect("init should succeed");
    super::super::attach(|py| {
        let storage = b"sifr".to_vec();
        let data_pointer = storage.as_ptr() as usize;
        let metrics = Arc::new(ExporterMetrics::default());
        let exporter = Py::new(
            py,
            InstrumentedExporter {
                storage,
                metrics: Arc::clone(&metrics),
            },
        )?;
        let object = super::super::object_ops::store_object(exporter.into_any())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        Ok::<_, PyErr>(InstrumentedEvidence {
            object,
            metrics,
            data_pointer,
        })
    })
    .expect("attach should succeed")
    .expect("instrumented exporter should create")
}

fn admitted_pointer(handle: i64) -> usize {
    let store = buffer_store().expect("buffer store should lock");
    let admission = store
        .admissions
        .get(&handle)
        .expect("buffer admission should exist");
    let BufferFootprint::Direct { ranges } = &admission.footprint else {
        panic!("non-empty instrumented buffer must have a direct footprint");
    };
    ranges.first().expect("buffer range should exist").start
}
