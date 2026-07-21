use super::*;
use crate::python::{
    close_object, get_attr, initialize_runtime, reset_runtime_state_for_tests,
    resource_diagnostics, test_config, test_guard, to_bool, PythonArrowCertification,
    PythonResourceDiagnostics,
};
use pyo3::types::{PyCapsule, PyDict};
use std::sync::atomic::{AtomicUsize, Ordering};

static RELEASE_CALLS: AtomicUsize = AtomicUsize::new(0);
static TRANSFER_RELEASE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn arrow_array_stream_schema_track_metadata_and_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("arrow-capsules")).expect("init should succeed");

    let object = exporter().expect("exporter should be stored");
    let array = arrow_array(&object).expect("array capsules should export");
    let stream = arrow_stream(&object).expect("stream capsule should export");
    let schema = arrow_schema(&object).expect("schema capsule should export");
    let device_array = arrow_device_array(&object).expect("device array should export");
    let device_stream = arrow_device_stream(&object).expect("device stream should export");

    assert_eq!(array.kind, "array");
    assert_eq!(array.capsule_names, ["arrow_schema", "arrow_array"]);
    assert!(array.copy_possible);
    assert_eq!(stream.capsule_names, ["arrow_array_stream"]);
    assert_eq!(schema.capsule_names, ["arrow_schema"]);
    assert_eq!(
        device_array.capsule_names,
        ["arrow_schema", "arrow_device_array"]
    );
    assert_eq!(device_stream.capsule_names, ["arrow_device_array_stream"]);
    assert_eq!(
        resource_diagnostics().expect("diagnostics should be available"),
        PythonResourceDiagnostics {
            initialized: true,
            live_objects: 6,
            leaked_objects: 0,
        }
    );

    release_arrow((array.handle, array.token)).expect("array should release");
    release_arrow((stream.handle, stream.token)).expect("stream should release");
    release_arrow((schema.handle, schema.token)).expect("schema should release");
    release_arrow((device_array.handle, device_array.token)).expect("device array should release");
    release_arrow((device_stream.handle, device_stream.token))
        .expect("device stream should release");
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
fn arrow_declaration_certification_requires_exact_producer_identity() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("arrow-certification");
    config.arrow_certifications = vec![PythonArrowCertification {
        producer_module: "pyarrow.lib".to_string(),
        producer_type: "ArrowExporter".to_string(),
    }];
    initialize_runtime(config).expect("init should succeed");

    let object = exporter().expect("exporter should be stored");
    let array = arrow_array(&object).expect("array should export");
    require_arrow_certification(&array).expect("exact identity should certify");
    let stream = arrow_stream(&object).expect("stream should export");
    let mut wrong = stream.clone();
    wrong.producer_type = "DifferentExporter".to_string();
    assert!(require_arrow_certification(&wrong)
        .expect_err("different producer must fail")
        .message
        .contains("no exact executable no-copy certification"));

    release_arrow((array.handle, array.token)).expect("array should release");
    release_arrow((stream.handle, stream.token)).expect("stream should release");
    close_object(object).expect("object should close");
}

#[test]
fn arrow_requested_schema_passes_the_borrowed_schema_capsule() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("arrow-requested-schema")).expect("init should succeed");

    let object = exporter().expect("exporter should be stored");
    let schema = arrow_schema(&object).expect("schema should export");
    let stream = arrow_stream_with_schema(&object, (schema.handle, schema.token))
        .expect("stream should accept requested schema");
    let seen = get_attr(&object, "requested_schema_seen").expect("marker should exist");
    assert!(to_bool(&seen).expect("marker should be bool"));

    close_object(seen).expect("marker should close");
    release_arrow((stream.handle, stream.token)).expect("stream should release");
    release_arrow((schema.handle, schema.token)).expect("schema should release");
    close_object(object).expect("object should close");
}

#[test]
fn arrow_omitted_schema_calls_the_protocol_without_an_argument() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("arrow-omitted-schema")).expect("init should succeed");

    let object = exporter().expect("exporter should be stored");
    let stream = arrow_stream(&object).expect("stream should omit requested schema");
    let seen = get_attr(&object, "schema_omitted_seen").expect("marker should exist");
    assert!(to_bool(&seen).expect("marker should be bool"));

    close_object(seen).expect("marker should close");
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

#[test]
fn arrow_owned_argument_reconciles_full_partial_and_failed_consumption() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("arrow-transfer")).expect("init should succeed");
    TRANSFER_RELEASE_CALLS.store(0, Ordering::SeqCst);

    let exporter_handle = transfer_exporter().expect("exporter should be stored");
    let unconsumed = arrow_array(&exporter_handle).expect("array should export");
    detach_export_capsules(&exporter_handle).expect("exporter capsules should detach");
    close_object(exporter_handle).expect("exporter should close");
    let argument = prepare_arrow_argument((unconsumed.handle, unconsumed.token))
        .expect("argument should prepare");
    argument
        .finish()
        .expect("unconsumed cleanup should succeed");
    assert_eq!(TRANSFER_RELEASE_CALLS.load(Ordering::SeqCst), 2);

    let exporter_handle = transfer_exporter().expect("exporter should be stored");
    let consumed = arrow_array(&exporter_handle).expect("array should export");
    detach_export_capsules(&exporter_handle).expect("exporter capsules should detach");
    close_object(exporter_handle).expect("exporter should close");
    let argument =
        prepare_arrow_argument((consumed.handle, consumed.token)).expect("argument should prepare");
    consume_argument_pair(&argument, true).expect("consumer should consume both capsules");
    argument.finish().expect("full consumption should succeed");
    assert_eq!(TRANSFER_RELEASE_CALLS.load(Ordering::SeqCst), 4);

    let exporter_handle = transfer_exporter().expect("exporter should be stored");
    let partial = arrow_array(&exporter_handle).expect("array should export");
    detach_export_capsules(&exporter_handle).expect("exporter capsules should detach");
    close_object(exporter_handle).expect("exporter should close");
    let argument =
        prepare_arrow_argument((partial.handle, partial.token)).expect("argument should prepare");
    consume_argument_pair(&argument, false).expect("consumer should consume schema only");
    let error = argument
        .finish()
        .expect_err("partial consumption must fail");
    assert!(error.message.contains("partially consumed"));
    assert_eq!(TRANSFER_RELEASE_CALLS.load(Ordering::SeqCst), 6);
}

#[test]
fn arrow_owned_argument_proxy_is_one_shot() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("arrow-transfer-one-shot")).expect("init should succeed");

    let exporter_handle = exporter().expect("exporter should be stored");
    let resource = arrow_array(&exporter_handle).expect("array should export");
    detach_export_capsules(&exporter_handle).expect("exporter capsules should detach");
    close_object(exporter_handle).expect("exporter should close");
    let argument =
        prepare_arrow_argument((resource.handle, resource.token)).expect("argument should prepare");
    let object = argument.object().expect("proxy should be available");
    let second = super::super::attach(|py| {
        let object = clone_handle(py, &object)?;
        object
            .bind(py)
            .call_method0("__arrow_c_array__")
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow one-shot consumer")
            })?;
        object
            .bind(py)
            .call_method0("__arrow_c_array__")
            .map(|_| ())
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow one-shot consumer")
            })
    })
    .expect("runtime should attach");
    let error = second.expect_err("a second protocol export must fail");
    assert!(error.message.contains("already exported"));
    close_object(object).expect("proxy clone should close");
    argument
        .finish()
        .expect("unconsumed one-shot export should clean up");
}

fn detach_export_capsules(exporter: &ObjectHandle) -> Result<(), PythonError> {
    super::super::attach(|py| {
        let exporter = clone_handle(py, exporter)?;
        let globals = exporter
            .bind(py)
            .getattr("__arrow_c_array__")
            .and_then(|method| method.getattr("__globals__"))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow exporter")
            })?;
        let globals = globals.cast_into::<PyDict>().map_err(|error| {
            arrow_error(format!("test Arrow exporter globals are invalid: {error}"))
        })?;
        globals
            .del_item("SCHEMA")
            .and_then(|()| globals.del_item("ARRAY"))
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test Arrow exporter"))
    })
    .map_err(PythonError::runtime)?
}

fn consume_argument_pair(
    argument: &PythonArrowArgument,
    consume_array: bool,
) -> Result<(), PythonError> {
    let object = argument.object()?;
    super::super::attach(|py| {
        let object = clone_handle(py, &object)?;
        let exported = object
            .bind(py)
            .call_method1("__arrow_c_array__", (py.None(),))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow consumer")
            })?;
        let tuple = exported
            .cast::<PyTuple>()
            .map_err(|_| arrow_error("test Arrow consumer expected tuple"))?;
        let schema_item = tuple.get_item(0).map_err(|error| {
            PythonError::from_pyerr(py, error, "zero-copy", "test Arrow consumer")
        })?;
        let schema = schema_item
            .cast::<PyCapsule>()
            .map_err(|_| arrow_error("test Arrow consumer expected schema capsule"))?;
        let schema_pointer = schema
            .pointer_checked(Some(ARROW_SCHEMA_NAME))
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow consumer")
            })?
            .cast::<abi::ArrowSchema>();
        let schema_release = unsafe { schema_pointer.as_ref() }
            .release
            .ok_or_else(|| arrow_error("schema was already consumed"))?;
        unsafe { schema_release(schema_pointer.as_ptr()) };
        if consume_array {
            let array_item = tuple.get_item(1).map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test Arrow consumer")
            })?;
            let array = array_item
                .cast::<PyCapsule>()
                .map_err(|_| arrow_error("test Arrow consumer expected array capsule"))?;
            let array_pointer = array
                .pointer_checked(Some(ARROW_ARRAY_NAME))
                .map_err(|error| {
                    PythonError::from_pyerr(py, error, "zero-copy", "test Arrow consumer")
                })?
                .cast::<abi::ArrowArray>();
            let array_release = unsafe { array_pointer.as_ref() }
                .release
                .ok_or_else(|| arrow_error("array was already consumed"))?;
            unsafe { array_release(array_pointer.as_ptr()) };
        }
        Ok(())
    })
    .map_err(PythonError::runtime)??;
    close_object(object)
}

fn exporter() -> Result<ObjectHandle, PythonError> {
    exporter_with_schema("pyarrow.lib", ARROW_SCHEMA_NAME, None)
}

fn transfer_exporter() -> Result<ObjectHandle, PythonError> {
    exporter_with_schema(
        "pyarrow.lib",
        ARROW_SCHEMA_NAME,
        Some(&TRANSFER_RELEASE_CALLS),
    )
}

fn pandas_exporter() -> Result<ObjectHandle, PythonError> {
    super::super::attach(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("STREAM", capsule_any(py, ARROW_STREAM_NAME)?.bind(py))
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test stream"))?;
        globals
            .set_item(
                "DEVICE_ARRAY",
                capsule_any(py, ARROW_DEVICE_ARRAY_NAME)?.bind(py),
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test device array")
            })?;
        globals
            .set_item(
                "DEVICE_STREAM",
                capsule_any(py, ARROW_DEVICE_STREAM_NAME)?.bind(py),
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test device stream")
            })?;
        py.run(
            cr#"
class PandasExporter:
    __module__ = "pandas.core.frame"

    def __arrow_c_stream__(self, requested_schema=None):
        self.requested_schema_seen = requested_schema is not None
        return STREAM

obj = PandasExporter()
"#,
            Some(&globals),
            None,
        )
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test pandas exporter"))?;
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
    exporter_with_schema("pyarrow.lib", ARROW_STREAM_NAME, None)
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

    def __arrow_c_device_array__(self, requested_schema=None):
        return (SCHEMA, DEVICE_ARRAY)

    def __arrow_c_device_stream__(self, requested_schema=None):
        return DEVICE_STREAM

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
    release_counter: Option<&'static AtomicUsize>,
) -> Result<ObjectHandle, PythonError> {
    super::super::attach(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("MODULE_NAME", module_name)
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test module"))?;
        globals
            .set_item(
                "SCHEMA",
                capsule_any_with_counter(py, schema_name, release_counter)?.bind(py),
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test schema"))?;
        globals
            .set_item(
                "ARRAY",
                capsule_any_with_counter(py, ARROW_ARRAY_NAME, release_counter)?.bind(py),
            )
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test array"))?;
        globals
            .set_item("STREAM", capsule_any(py, ARROW_STREAM_NAME)?.bind(py))
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test stream"))?;
        globals
            .set_item(
                "DEVICE_ARRAY",
                capsule_any(py, ARROW_DEVICE_ARRAY_NAME)?.bind(py),
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test device array")
            })?;
        globals
            .set_item(
                "DEVICE_STREAM",
                capsule_any(py, ARROW_DEVICE_STREAM_NAME)?.bind(py),
            )
            .map_err(|error| {
                PythonError::from_pyerr(py, error, "zero-copy", "test device stream")
            })?;
        py.run(
            cr#"
class ArrowExporter:
    __module__ = MODULE_NAME

    def __arrow_c_array__(self, requested_schema=None):
        return (SCHEMA, ARRAY)

    def __arrow_c_stream__(self, *requested_schema):
        self.requested_schema_seen = len(requested_schema) == 1 and requested_schema[0] is SCHEMA
        self.schema_omitted_seen = len(requested_schema) == 0
        return STREAM

    def __arrow_c_schema__(self):
        return SCHEMA

    def __arrow_c_device_array__(self, requested_schema=None):
        return (SCHEMA, DEVICE_ARRAY)

    def __arrow_c_device_stream__(self, requested_schema=None):
        return DEVICE_STREAM

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

unsafe extern "C" fn release_schema(schema: *mut abi::ArrowSchema) {
    let counter = release_counter(unsafe { (*schema).private_data });
    unsafe { (*schema).release = None };
    counter.fetch_add(1, Ordering::SeqCst);
}
unsafe extern "C" fn release_array(array: *mut abi::ArrowArray) {
    let counter = release_counter(unsafe { (*array).private_data });
    unsafe { (*array).release = None };
    counter.fetch_add(1, Ordering::SeqCst);
}

fn release_counter(private_data: *mut std::ffi::c_void) -> &'static AtomicUsize {
    if private_data.is_null() {
        &RELEASE_CALLS
    } else {
        unsafe { &*private_data.cast::<AtomicUsize>() }
    }
}
unsafe extern "C" fn stream_get_schema(
    _stream: *mut abi::ArrowArrayStream,
    _schema: *mut abi::ArrowSchema,
) -> i32 {
    0
}
unsafe extern "C" fn stream_get_next(
    _stream: *mut abi::ArrowArrayStream,
    _array: *mut abi::ArrowArray,
) -> i32 {
    0
}
unsafe extern "C" fn stream_get_last_error(
    _stream: *mut abi::ArrowArrayStream,
) -> *const std::ffi::c_char {
    std::ptr::null()
}
unsafe extern "C" fn release_stream(stream: *mut abi::ArrowArrayStream) {
    unsafe { (*stream).release = None };
    RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
}
unsafe extern "C" fn device_stream_get_schema(
    _stream: *mut abi::ArrowDeviceArrayStream,
    _schema: *mut abi::ArrowSchema,
) -> i32 {
    0
}
unsafe extern "C" fn device_stream_get_next(
    _stream: *mut abi::ArrowDeviceArrayStream,
    _array: *mut abi::ArrowDeviceArray,
) -> i32 {
    0
}
unsafe extern "C" fn device_stream_get_last_error(
    _stream: *mut abi::ArrowDeviceArrayStream,
) -> *const std::ffi::c_char {
    std::ptr::null()
}
unsafe extern "C" fn release_device_stream(stream: *mut abi::ArrowDeviceArrayStream) {
    unsafe { (*stream).release = None };
    RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
}

fn valid_schema_with_counter(release_counter: Option<&'static AtomicUsize>) -> abi::ArrowSchema {
    abi::ArrowSchema {
        format: c"i".as_ptr(),
        name: std::ptr::null(),
        metadata: std::ptr::null(),
        flags: 0,
        n_children: 0,
        children: std::ptr::null_mut(),
        dictionary: std::ptr::null_mut(),
        release: Some(release_schema),
        private_data: release_counter.map_or(std::ptr::null_mut(), |counter| {
            std::ptr::from_ref(counter).cast_mut().cast()
        }),
    }
}

fn valid_array() -> abi::ArrowArray {
    valid_array_with_counter(None)
}

fn valid_array_with_counter(release_counter: Option<&'static AtomicUsize>) -> abi::ArrowArray {
    abi::ArrowArray {
        length: 1,
        null_count: 0,
        offset: 0,
        n_buffers: 0,
        n_children: 0,
        buffers: std::ptr::null_mut(),
        children: std::ptr::null_mut(),
        dictionary: std::ptr::null_mut(),
        release: Some(release_array),
        private_data: release_counter.map_or(std::ptr::null_mut(), |counter| {
            std::ptr::from_ref(counter).cast_mut().cast()
        }),
    }
}

fn capsule<'py>(py: Python<'py>, name: &'static CStr) -> PyResult<Bound<'py, PyCapsule>> {
    capsule_with_counter(py, name, None)
}

fn capsule_with_counter<'py>(
    py: Python<'py>,
    name: &'static CStr,
    release_counter: Option<&'static AtomicUsize>,
) -> PyResult<Bound<'py, PyCapsule>> {
    if name == ARROW_SCHEMA_NAME {
        PyCapsule::new_with_value_and_destructor(
            py,
            valid_schema_with_counter(release_counter),
            name,
            |mut value, _| {
                if let Some(release) = value.release {
                    unsafe { release(&raw mut value) };
                }
            },
        )
    } else if name == ARROW_ARRAY_NAME {
        PyCapsule::new_with_value_and_destructor(
            py,
            valid_array_with_counter(release_counter),
            name,
            |mut value, _| {
                if let Some(release) = value.release {
                    unsafe { release(&raw mut value) };
                }
            },
        )
    } else if name == ARROW_STREAM_NAME {
        PyCapsule::new_with_value_and_destructor(
            py,
            abi::ArrowArrayStream {
                get_schema: Some(stream_get_schema),
                get_next: Some(stream_get_next),
                get_last_error: Some(stream_get_last_error),
                release: Some(release_stream),
                private_data: std::ptr::null_mut(),
            },
            name,
            |mut value, _| {
                if let Some(release) = value.release {
                    unsafe { release(&raw mut value) };
                }
            },
        )
    } else if name == ARROW_DEVICE_ARRAY_NAME {
        PyCapsule::new_with_value_and_destructor(
            py,
            abi::ArrowDeviceArray {
                array: valid_array(),
                device_id: 0,
                device_type: 1,
                sync_event: std::ptr::null_mut(),
                reserved: [0; 3],
            },
            name,
            |mut value, _| {
                if let Some(release) = value.array.release {
                    unsafe { release(&raw mut value.array) };
                }
            },
        )
    } else {
        PyCapsule::new_with_value_and_destructor(
            py,
            abi::ArrowDeviceArrayStream {
                device_type: 1,
                get_schema: Some(device_stream_get_schema),
                get_next: Some(device_stream_get_next),
                get_last_error: Some(device_stream_get_last_error),
                release: Some(release_device_stream),
                private_data: std::ptr::null_mut(),
            },
            name,
            |mut value, _| {
                if let Some(release) = value.release {
                    unsafe { release(&raw mut value) };
                }
            },
        )
    }
}

fn capsule_any(py: Python<'_>, name: &'static CStr) -> Result<Py<PyAny>, PythonError> {
    capsule(py, name)
        .map(|capsule| capsule.into_any().unbind())
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))
}

fn capsule_any_with_counter(
    py: Python<'_>,
    name: &'static CStr,
    release_counter: Option<&'static AtomicUsize>,
) -> Result<Py<PyAny>, PythonError> {
    capsule_with_counter(py, name, release_counter)
        .map(|capsule| capsule.into_any().unbind())
        .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "test capsule"))
}

fn capsule_without_destructor(py: Python<'_>) -> Result<Py<PyAny>, PythonError> {
    let leaked = Box::leak(Box::new(1_u8));
    let pointer = std::ptr::NonNull::from(leaked).cast();
    let capsule = unsafe { PyCapsule::new_with_pointer(py, pointer, ARROW_SCHEMA_NAME) }.map_err(
        |error| PythonError::from_pyerr(py, error, "zero-copy", "test destructorless capsule"),
    )?;
    Ok(capsule.into_any().unbind())
}
