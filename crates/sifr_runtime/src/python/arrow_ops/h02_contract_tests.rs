use super::*;
use crate::python::{
    close_object, initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
};
use std::mem::{align_of, offset_of, size_of};
use std::sync::atomic::{AtomicUsize, Ordering};

static SCHEMAS: AtomicUsize = AtomicUsize::new(0);
static ARRAYS: AtomicUsize = AtomicUsize::new(0);
static STREAMS: AtomicUsize = AtomicUsize::new(0);
static DEVICE_STREAMS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn release_schema(value: *mut abi::ArrowSchema) {
    // SAFETY: called once for an initialized test-owned header.
    unsafe { (*value).release = None };
    SCHEMAS.fetch_add(1, Ordering::SeqCst);
}
unsafe extern "C" fn release_array(value: *mut abi::ArrowArray) {
    // SAFETY: called once for an initialized test-owned header.
    unsafe { (*value).release = None };
    ARRAYS.fetch_add(1, Ordering::SeqCst);
}
fn schema() -> abi::ArrowSchema {
    abi::ArrowSchema {
        format: c"i".as_ptr(),
        name: std::ptr::null(),
        metadata: std::ptr::null(),
        flags: 0,
        n_children: 0,
        children: std::ptr::null_mut(),
        dictionary: std::ptr::null_mut(),
        release: Some(release_schema),
        private_data: std::ptr::null_mut(),
    }
}
fn array() -> abi::ArrowArray {
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
        private_data: std::ptr::null_mut(),
    }
}
fn device() -> abi::ArrowDeviceArray {
    abi::ArrowDeviceArray {
        array: array(),
        device_id: 0,
        device_type: 1,
        sync_event: std::ptr::null_mut(),
        reserved: [0; 3],
    }
}
unsafe extern "C" fn get_schema(
    _stream: *mut abi::ArrowArrayStream,
    output: *mut abi::ArrowSchema,
) -> i32 {
    // SAFETY: consumer provides writable, initialized-size output storage.
    unsafe { output.write(schema()) };
    0
}
unsafe extern "C" fn get_next(
    _stream: *mut abi::ArrowArrayStream,
    output: *mut abi::ArrowArray,
) -> i32 {
    // SAFETY: consumer provides writable, initialized-size output storage.
    unsafe { output.write(array()) };
    0
}
unsafe extern "C" fn last_error(_stream: *mut abi::ArrowArrayStream) -> *const std::ffi::c_char {
    std::ptr::null()
}
unsafe extern "C" fn release_stream(value: *mut abi::ArrowArrayStream) {
    // SAFETY: called once for an initialized test-owned header.
    unsafe { (*value).release = None };
    STREAMS.fetch_add(1, Ordering::SeqCst);
}
unsafe extern "C" fn device_get_schema(
    _stream: *mut abi::ArrowDeviceArrayStream,
    output: *mut abi::ArrowSchema,
) -> i32 {
    // SAFETY: consumer provides writable, initialized-size output storage.
    unsafe { output.write(schema()) };
    0
}
unsafe extern "C" fn device_get_next(
    _stream: *mut abi::ArrowDeviceArrayStream,
    output: *mut abi::ArrowDeviceArray,
) -> i32 {
    // SAFETY: consumer provides writable, initialized-size output storage.
    unsafe { output.write(device()) };
    0
}
unsafe extern "C" fn device_last_error(
    _stream: *mut abi::ArrowDeviceArrayStream,
) -> *const std::ffi::c_char {
    std::ptr::null()
}
unsafe extern "C" fn release_device_stream(value: *mut abi::ArrowDeviceArrayStream) {
    // SAFETY: called once for an initialized test-owned header.
    unsafe { (*value).release = None };
    DEVICE_STREAMS.fetch_add(1, Ordering::SeqCst);
}
fn stream() -> abi::ArrowArrayStream {
    abi::ArrowArrayStream {
        get_schema: Some(get_schema),
        get_next: Some(get_next),
        get_last_error: Some(last_error),
        release: Some(release_stream),
        private_data: std::ptr::null_mut(),
    }
}
fn device_stream() -> abi::ArrowDeviceArrayStream {
    abi::ArrowDeviceArrayStream {
        device_type: 1,
        get_schema: Some(device_get_schema),
        get_next: Some(device_get_next),
        get_last_error: Some(device_last_error),
        release: Some(release_device_stream),
        private_data: std::ptr::null_mut(),
    }
}
fn init(name: &str) {
    reset_runtime_state_for_tests();
    initialize_runtime(test_config(name)).expect("runtime");
    for counter in [&SCHEMAS, &ARRAYS, &STREAMS, &DEVICE_STREAMS] {
        counter.store(0, Ordering::SeqCst);
    }
}
fn capsule(py: Python<'_>, kind: ArrowKind) -> Py<PyAny> {
    macro_rules! cap {
        ($value:expr, $name:expr, $release:expr) => {
            PyCapsule::new_with_value_and_destructor(py, $value, $name, |mut value, _| {
                ($release)(&mut value);
            })
            .expect("test capsule")
            .into_any()
            .unbind()
        };
    }
    match kind {
        ArrowKind::Schema => cap!(
            schema(),
            ARROW_SCHEMA_NAME,
            |value: &mut abi::ArrowSchema| {
                if let Some(callback) = value.release {
                    // SAFETY: the capsule owns this initialized header.
                    unsafe { callback(value) };
                }
            }
        ),
        ArrowKind::Array => cap!(array(), ARROW_ARRAY_NAME, |value: &mut abi::ArrowArray| {
            if let Some(callback) = value.release {
                // SAFETY: the capsule owns this initialized header.
                unsafe { callback(value) };
            }
        }),
        ArrowKind::DeviceArray => cap!(
            device(),
            ARROW_DEVICE_ARRAY_NAME,
            |value: &mut abi::ArrowDeviceArray| {
                if let Some(callback) = value.array.release {
                    // SAFETY: embedded array is initialized and capsule-owned.
                    unsafe { callback(&mut value.array) };
                }
            }
        ),
        ArrowKind::Stream => cap!(
            stream(),
            ARROW_STREAM_NAME,
            |value: &mut abi::ArrowArrayStream| {
                if let Some(callback) = value.release {
                    // SAFETY: the capsule owns this initialized header.
                    unsafe { callback(value) };
                }
            }
        ),
        ArrowKind::DeviceStream => cap!(
            device_stream(),
            ARROW_DEVICE_STREAM_NAME,
            |value: &mut abi::ArrowDeviceArrayStream| {
                if let Some(callback) = value.release {
                    // SAFETY: the capsule owns this initialized header.
                    unsafe { callback(value) };
                }
            }
        ),
    }
}
fn store(py: Python<'_>, kind: ArrowKind) -> PythonArrowCapsuleMetadata {
    let mut capsules = vec![];
    if matches!(kind, ArrowKind::Array | ArrowKind::DeviceArray) {
        capsules.push(capsule(py, ArrowKind::Schema));
    }
    capsules.push(capsule(py, kind));
    store_arrow_capsules(
        capsules,
        kind,
        capsule_names(kind),
        ProducerInfo {
            module: "contract".into(),
            name: "Exporter".into(),
            copy_possible: true,
        },
    )
    .expect("store")
}
fn handle(metadata: &PythonArrowCapsuleMetadata) -> ArrowHandle {
    (metadata.handle, metadata.token)
}
fn assert_layout() {
    // Arrow C headers on the qualified 64-bit host: every field offset is checked,
    // including nullable function-pointer representation and device-stream padding.
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(
            (
                size_of::<abi::ArrowSchema>(),
                align_of::<abi::ArrowSchema>()
            ),
            (72, 8)
        );
        assert_eq!(
            [
                offset_of!(abi::ArrowSchema, format),
                offset_of!(abi::ArrowSchema, name),
                offset_of!(abi::ArrowSchema, metadata),
                offset_of!(abi::ArrowSchema, flags),
                offset_of!(abi::ArrowSchema, n_children),
                offset_of!(abi::ArrowSchema, children),
                offset_of!(abi::ArrowSchema, dictionary),
                offset_of!(abi::ArrowSchema, release),
                offset_of!(abi::ArrowSchema, private_data),
            ],
            [0, 8, 16, 24, 32, 40, 48, 56, 64]
        );
        assert_eq!(
            (size_of::<abi::ArrowArray>(), align_of::<abi::ArrowArray>()),
            (80, 8)
        );
        assert_eq!(
            [
                offset_of!(abi::ArrowArray, length),
                offset_of!(abi::ArrowArray, null_count),
                offset_of!(abi::ArrowArray, offset),
                offset_of!(abi::ArrowArray, n_buffers),
                offset_of!(abi::ArrowArray, n_children),
                offset_of!(abi::ArrowArray, buffers),
                offset_of!(abi::ArrowArray, children),
                offset_of!(abi::ArrowArray, dictionary),
                offset_of!(abi::ArrowArray, release),
                offset_of!(abi::ArrowArray, private_data),
            ],
            [0, 8, 16, 24, 32, 40, 48, 56, 64, 72]
        );
        assert_eq!(
            (
                size_of::<abi::ArrowArrayStream>(),
                align_of::<abi::ArrowArrayStream>()
            ),
            (40, 8)
        );
        assert_eq!(
            [
                offset_of!(abi::ArrowArrayStream, get_schema),
                offset_of!(abi::ArrowArrayStream, get_next),
                offset_of!(abi::ArrowArrayStream, get_last_error),
                offset_of!(abi::ArrowArrayStream, release),
                offset_of!(abi::ArrowArrayStream, private_data),
            ],
            [0, 8, 16, 24, 32]
        );
        assert_eq!(
            (
                size_of::<abi::ArrowDeviceArray>(),
                align_of::<abi::ArrowDeviceArray>()
            ),
            (128, 8)
        );
        assert_eq!(
            [
                offset_of!(abi::ArrowDeviceArray, array),
                offset_of!(abi::ArrowDeviceArray, device_id),
                offset_of!(abi::ArrowDeviceArray, device_type),
                offset_of!(abi::ArrowDeviceArray, sync_event),
                offset_of!(abi::ArrowDeviceArray, reserved),
            ],
            [0, 80, 88, 96, 104]
        );
        assert_eq!(
            (
                size_of::<abi::ArrowDeviceArrayStream>(),
                align_of::<abi::ArrowDeviceArrayStream>()
            ),
            (48, 8)
        );
        assert_eq!(
            [
                offset_of!(abi::ArrowDeviceArrayStream, device_type),
                offset_of!(abi::ArrowDeviceArrayStream, get_schema),
                offset_of!(abi::ArrowDeviceArrayStream, get_next),
                offset_of!(abi::ArrowDeviceArrayStream, get_last_error),
                offset_of!(abi::ArrowDeviceArrayStream, release),
                offset_of!(abi::ArrowDeviceArrayStream, private_data),
            ],
            [0, 8, 16, 24, 32, 40]
        );
    }
}

#[test]
fn abi_layout_and_capsule_transfer() {
    let _guard = test_guard();
    init("h02f-layout-transfer");
    assert_layout();
    super::super::attach(|py| {
        // Borrowed metadata observes one capsule; prepared ownership is exclusive.
        let original = store(py, ArrowKind::Array);
        let alias_caps = {
            let entries = arrow_store().expect("store");
            entries.capsules[&original.handle]
                .capsules
                .capsules
                .iter()
                .map(|value| value.clone_ref(py))
                .collect()
        };
        let alias = store_arrow_capsules(
            alias_caps,
            ArrowKind::Array,
            capsule_names(ArrowKind::Array),
            ProducerInfo {
                module: "contract".into(),
                name: "Alias".into(),
                copy_possible: true,
            },
        )
        .expect("borrowed alias");
        let argument = prepare_arrow_argument(handle(&original)).expect("first transfer");
        assert!(release_arrow(handle(&original)).is_err());
        let error = prepare_arrow_argument(handle(&alias))
            .err()
            .expect("alias must decline");
        assert!(error.message.contains("active owned transfer"));
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 0);
        assert_eq!(ARRAYS.load(Ordering::SeqCst), 0);
        argument.finish().expect("unconsumed transfer");
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 1);
        assert_eq!(ARRAYS.load(Ordering::SeqCst), 1);

        // External capsule mutation is rejected using stored identity before ABI read.
        let changed = store(py, ArrowKind::Schema);
        let entries = arrow_store().expect("store");
        let capsule = entries.capsules[&changed.handle].capsules.capsules[0]
            .bind(py)
            .clone();
        let original_ptr =
            unsafe { ffi::PyCapsule_GetPointer(capsule.as_ptr(), ARROW_SCHEMA_NAME.as_ptr()) };
        // SAFETY: test changes identity only; restore before any native destructor.
        assert_eq!(
            unsafe {
                ffi::PyCapsule_SetPointer(capsule.as_ptr(), std::ptr::dangling_mut::<u8>().cast())
            },
            0
        );
        drop(entries);
        let error = prepare_arrow_argument(handle(&changed))
            .err()
            .expect("changed identity");
        assert!(error.message.contains("pointer identity changed"));
        // The local capsule reference pins its allocation through rejected admission.
        assert_eq!(
            unsafe { ffi::PyCapsule_SetPointer(capsule.as_ptr(), original_ptr) },
            0
        );
        drop(capsule);

        // A retained shell cannot initiate a transfer after argument finalization.
        let retained = store(py, ArrowKind::Schema);
        let argument = prepare_arrow_argument(handle(&retained)).expect("argument");
        let retained_object = argument.object().expect("retained shell");
        argument.finish().expect("finish");
        let native = clone_handle(py, &retained_object).expect("retained native");
        assert!(native.bind(py).call_method0("__arrow_c_schema__").is_err());
        drop(native);
        close_object(retained_object).expect("retained close");
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 3);

        // Borrowed consumption invalidates later admission before creating a proxy.
        let consumed = store(py, ArrowKind::Schema);
        {
            let entries = arrow_store().expect("store");
            let capsule = entries.capsules[&consumed.handle].capsules.capsules[0].bind(py);
            let pointer = capsule
                .cast::<PyCapsule>()
                .expect("capsule")
                .pointer_checked(Some(ARROW_SCHEMA_NAME))
                .expect("pointer")
                .cast::<abi::ArrowSchema>();
            // SAFETY: test capsule owns a valid, unconsumed schema.
            unsafe { release_schema(pointer.as_ptr()) };
        }
        assert!(prepare_arrow_argument(handle(&consumed)).is_err());
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 4);

        // Missing callback and malformed shape are checked without invoking pointers.
        let mut bad = array();
        bad.null_count = 2;
        assert!(abi::validate_array(std::ptr::NonNull::from(&mut bad).cast(), "test").is_err());
        bad.null_count = 0;
        bad.length = i64::MAX;
        bad.offset = 1;
        assert!(abi::validate_array(std::ptr::NonNull::from(&mut bad).cast(), "test").is_err());
        bad.length = 1;
        bad.offset = 0;
        bad.release = None;
        assert!(abi::validate_array(std::ptr::NonNull::from(&mut bad).cast(), "test").is_err());

        // Device array retains its capsule after producer disappearance and drops once.
        let device = store(py, ArrowKind::DeviceArray);
        drop(prepare_arrow_argument(handle(&device)).expect("device transfer"));
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 5);
        assert_eq!(ARRAYS.load(Ordering::SeqCst), 2);
        for full in [true, false] {
            let device = store(py, ArrowKind::DeviceArray);
            let argument = prepare_arrow_argument(handle(&device)).expect("device pair");
            let object = argument.object().expect("device proxy");
            let native = clone_handle(py, &object).expect("native");
            let exported = native
                .bind(py)
                .call_method0("__arrow_c_device_array__")
                .expect("export");
            let pair = exported.cast::<PyTuple>().expect("pair");
            let schema = pair.get_item(0).expect("schema");
            let schema = schema
                .cast::<PyCapsule>()
                .expect("capsule")
                .pointer_checked(Some(ARROW_SCHEMA_NAME))
                .expect("pointer")
                .cast::<abi::ArrowSchema>();
            // SAFETY: capsule pins the initialized schema; clear release on consumption.
            unsafe { release_schema(schema.as_ptr()) };
            if full {
                let array = pair.get_item(1).expect("device");
                let array = array
                    .cast::<PyCapsule>()
                    .expect("capsule")
                    .pointer_checked(Some(ARROW_DEVICE_ARRAY_NAME))
                    .expect("pointer")
                    .cast::<abi::ArrowDeviceArray>();
                // SAFETY: capsule pins the initialized embedded array.
                unsafe { release_array(&raw mut (*array.as_ptr()).array) };
            }
            drop(exported);
            drop(native);
            close_object(object).expect("close proxy");
            let result = argument.finish();
            if full {
                result.expect("full device consumption");
            } else {
                assert!(
                    result
                        .expect_err("partial device consumption")
                        .message
                        .contains("partially consumed")
                );
            }
        }
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 7);
        assert_eq!(ARRAYS.load(Ordering::SeqCst), 4);
    })
    .expect("attachment");
}

#[test]
fn stream_callbacks_release_exactly_once() {
    let _guard = test_guard();
    init("h02f-stream-release");
    super::super::attach(|py| {
        for kind in [ArrowKind::Stream, ArrowKind::DeviceStream] {
            for mode in 0..4 {
                let metadata = store(py, kind);
                if mode == 0 {
                    release_arrow(handle(&metadata)).expect("explicit release");
                    assert!(release_arrow(handle(&metadata)).is_err());
                    continue;
                }
                let argument = prepare_arrow_argument(handle(&metadata)).expect("stream argument");
                if mode == 1 {
                    let object = argument.object().expect("consumer object");
                    let native = clone_handle(py, &object).expect("object");
                    let exported = native.bind(py).call_method0(kind.method()).expect("export");
                    let pointer = exported
                        .cast::<PyCapsule>()
                        .expect("capsule")
                        .pointer_checked(Some(if kind == ArrowKind::Stream {
                            ARROW_STREAM_NAME
                        } else {
                            ARROW_DEVICE_STREAM_NAME
                        }))
                        .expect("pointer");
                    let mut output_schema = schema();
                    let mut output_array = array();
                    let mut output_device = device();
                    // SAFETY: live test capsules provide matching initialized headers
                    // and callbacks write only to owned output structs.
                    unsafe {
                        if kind == ArrowKind::Stream {
                            let stream = pointer.cast::<abi::ArrowArrayStream>().as_ptr();
                            assert_eq!(
                                ((*stream).get_schema.expect("schema"))(stream, &mut output_schema),
                                0
                            );
                            assert_eq!(
                                ((*stream).get_next.expect("next"))(stream, &mut output_array),
                                0
                            );
                            assert!(((*stream).get_last_error.expect("error"))(stream).is_null());
                            ((*stream).release.expect("release"))(stream);
                            release_array(&mut output_array);
                        } else {
                            let stream = pointer.cast::<abi::ArrowDeviceArrayStream>().as_ptr();
                            assert_eq!(
                                ((*stream).get_schema.expect("schema"))(stream, &mut output_schema),
                                0
                            );
                            assert_eq!(
                                ((*stream).get_next.expect("next"))(stream, &mut output_device),
                                0
                            );
                            assert!(((*stream).get_last_error.expect("error"))(stream).is_null());
                            ((*stream).release.expect("release"))(stream);
                            release_array(&mut output_device.array);
                        }
                        release_schema(&mut output_schema);
                    }
                    drop(exported);
                    drop(native);
                    close_object(object).expect("close clone");
                    argument.finish().expect("consumed finish");
                } else if mode == 2 {
                    argument.finish().expect("unconsumed finish");
                } else {
                    let object = argument.object().expect("failed consumer");
                    let native = clone_handle(py, &object).expect("native");
                    assert!(
                        native
                            .bind(py)
                            .call_method0("missing_consumer_operation")
                            .is_err()
                    );
                    drop(native);
                    close_object(object).expect("failed consumer clone");
                    drop(argument);
                }
            }
        }
        assert_eq!(STREAMS.load(Ordering::SeqCst), 4);
        assert_eq!(DEVICE_STREAMS.load(Ordering::SeqCst), 4);
        assert_eq!(SCHEMAS.load(Ordering::SeqCst), 2);
        assert_eq!(ARRAYS.load(Ordering::SeqCst), 2);

        for missing in 0..4 {
            let mut regular = stream();
            let mut device = device_stream();
            match missing {
                0 => {
                    regular.get_schema = None;
                    device.get_schema = None;
                }
                1 => {
                    regular.get_next = None;
                    device.get_next = None;
                }
                2 => {
                    regular.get_last_error = None;
                    device.get_last_error = None;
                }
                _ => {
                    regular.release = None;
                    device.release = None;
                }
            }
            assert!(
                abi::validate_stream(std::ptr::NonNull::from(&mut regular).cast(), "test").is_err()
            );
            assert!(
                abi::validate_device_stream(std::ptr::NonNull::from(&mut device).cast(), "test")
                    .is_err()
            );
        }
        // Reset releases stored streams and does not repeat callbacks on stale handles.
        let a = store(py, ArrowKind::Stream);
        let b = store(py, ArrowKind::DeviceStream);
        reset_arrow_store_for_tests();
        assert_eq!(STREAMS.load(Ordering::SeqCst), 5);
        assert_eq!(DEVICE_STREAMS.load(Ordering::SeqCst), 5);
        assert!(release_arrow(handle(&a)).is_err());
        assert!(release_arrow(handle(&b)).is_err());
    })
    .expect("attachment");
}
