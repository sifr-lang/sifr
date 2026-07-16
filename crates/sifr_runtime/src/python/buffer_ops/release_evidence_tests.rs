use super::*;
use crate::python::{
    close_object, initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
    PythonResourceIdentity,
};
use pyo3::types::{PyAnyMethods, PyModule};

#[test]
fn instrumented_exporter_explicit_release_is_exact_once_and_pointer_identical() {
    let _guard = test_guard();
    let object = setup_instrumented_exporter("buffer-release-evidence-explicit");
    let exporter_pointer = exporter_metric(&object, "data_pointer");
    let view = buffer_u8(&object, true).expect("instrumented buffer should acquire");
    let key = (view.handle, view.token);

    assert_eq!(admitted_pointer(view.handle), exporter_pointer);
    assert_eq!(exporter_metric(&object, "acquisitions"), 1);
    assert_eq!(exporter_metric(&object, "releases"), 0);
    release_buffer(key).expect("explicit release should succeed");
    assert_eq!(exporter_metric(&object, "releases"), 1);
    release_buffer(key).expect_err("double release should fail before touching the exporter");
    assert_eq!(exporter_metric(&object, "releases"), 1);

    close_object(object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_automatic_resource_drop_releases_exactly_once() {
    let _guard = test_guard();
    let object = setup_instrumented_exporter("buffer-release-evidence-drop");
    let view = buffer_u8(&object, false).expect("instrumented buffer should acquire");

    drop(PythonResourceIdentity::buffer((view.handle, view.token)));

    assert_eq!(exporter_metric(&object, "acquisitions"), 1);
    assert_eq!(exporter_metric(&object, "releases"), 1);
    close_object(object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_validation_failure_rolls_back_exactly_once() {
    let _guard = test_guard();
    let object = setup_instrumented_exporter("buffer-release-evidence-validation");
    let error = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I16,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect_err("uint8 exporter must reject an int16 declaration");

    assert_eq!(error.context, "buffer metadata validation");
    assert_eq!(exporter_metric(&object, "acquisitions"), 1);
    assert_eq!(exporter_metric(&object, "releases"), 1);
    close_object(object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_admission_conflict_releases_rejected_view_exactly_once() {
    let _guard = test_guard();
    let object = setup_instrumented_exporter("buffer-release-evidence-admission");
    let reader = buffer_u8(&object, false).expect("first view should acquire");
    let error = buffer_u8(&object, true).expect_err("overlapping writer must be rejected");

    assert_eq!(error.context, "buffer exporter access admission");
    assert_eq!(exporter_metric(&object, "acquisitions"), 2);
    assert_eq!(exporter_metric(&object, "releases"), 1);
    release_buffer((reader.handle, reader.token)).expect("reader should release");
    assert_eq!(exporter_metric(&object, "releases"), 2);
    close_object(object).expect("exporter should close");
}

#[test]
fn instrumented_exporter_store_failure_rolls_back_exactly_once() {
    let _guard = test_guard();
    let object = setup_instrumented_exporter("buffer-release-evidence-store");
    let previous_handle = {
        let mut store = buffer_store().expect("buffer store should lock");
        std::mem::replace(&mut store.next_handle, i64::MAX)
    };

    let error = buffer_u8(&object, false).expect_err("exhausted handle space must reject storage");

    buffer_store()
        .expect("buffer store should lock")
        .next_handle = previous_handle;
    assert_eq!(error.kind, "runtime");
    assert_eq!(exporter_metric(&object, "acquisitions"), 1);
    assert_eq!(exporter_metric(&object, "releases"), 1);
    close_object(object).expect("exporter should close");
}

fn setup_instrumented_exporter(config_name: &str) -> ObjectHandle {
    reset_runtime_state_for_tests();
    initialize_runtime(test_config(config_name)).expect("init should succeed");
    super::super::attach(|py| {
        let module = PyModule::from_code(
            py,
            c"import ctypes\nclass Exporter:\n    def __init__(self):\n        self.storage = bytearray(b'sifr')\n        self.acquisitions = 0\n        self.releases = 0\n        self.data_pointer = ctypes.addressof(ctypes.c_ubyte.from_buffer(self.storage))\n    def __buffer__(self, _flags):\n        self.acquisitions += 1\n        return memoryview(self.storage)\n    def __release_buffer__(self, _view):\n        self.releases += 1\n",
            c"buffer_release_evidence.py",
            c"buffer_release_evidence",
        )?;
        let exporter = module.getattr("Exporter")?.call0()?;
        super::super::object_ops::store_object(exporter.unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("instrumented exporter should create")
}

fn exporter_metric(object: &ObjectHandle, name: &str) -> usize {
    super::super::attach(|py| {
        let exporter = clone_handle(py, object)
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        exporter.bind(py).getattr(name)?.extract::<usize>()
    })
    .expect("attach should succeed")
    .expect("instrumented exporter metric should resolve")
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
