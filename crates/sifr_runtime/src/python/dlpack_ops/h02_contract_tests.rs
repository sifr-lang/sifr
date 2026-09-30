use super::abi::{DLManagedTensorVersioned, DLPackVersion, USED_DLTENSOR_VERSIONED_NAME};
use super::declaration_tests as fixtures;
use super::*;
use crate::python::{
    close_object, initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
};
use std::mem::{align_of, offset_of, size_of};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[test]
fn legacy_versioned_layout_and_transfer() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    fixtures::reset_releases();
    initialize_runtime(test_config("h02g-transfer")).expect("initialize");
    assert_abi_layout();

    for versioned in [false, true] {
        for consumed in [false, true] {
            let before = releases(versioned);
            let object = exporter(versioned);
            let metadata = dlpack_tensor(&object).expect("acquire");
            let handle = (metadata.handle, metadata.token);
            assert_eq!(metadata.shape, [2, 3]);
            assert_eq!(metadata.strides, [3, 1]);
            assert!(dlpack_tensor(&object).is_err(), "one-shot producer capsule");
            let argument = prepare_dlpack_argument(handle).expect("prepare");
            assert!(
                dlpack_shape(handle).is_err(),
                "ownership removed from store"
            );
            let temporary = argument.object().expect("temporary");
            let retained = crate::python::attach(|py| clone_handle(py, &temporary))
                .expect("attached")
                .expect("pin capsule");
            close_object(temporary).expect("semantic close");
            close_object(object).expect("close producer handle; entry still pins it");
            assert_eq!(releases(versioned), before);
            if consumed {
                // The callee consumes AND frees the header before finish. The
                // reconciliation must inspect capsule state, never that header.
                consume_and_release(&retained, versioned);
                assert_eq!(releases(versioned), before + 1);
            }
            argument
                .finish()
                .expect("finish despite exposed handle close");
            assert_eq!(releases(versioned), before + 1);
            crate::python::attach(|py| {
                let capsule = retained.bind(py).cast::<PyCapsule>().expect("capsule");
                let used = if versioned {
                    USED_DLTENSOR_VERSIONED_NAME
                } else {
                    USED_DLTENSOR_NAME
                };
                assert_eq!(capsule_name(capsule, "retained shell").expect("name"), used);
                assert!(
                    capsule
                        .pointer_checked(Some(if versioned {
                            DLTENSOR_VERSIONED_NAME
                        } else {
                            DLTENSOR_NAME
                        }))
                        .is_err()
                );
            })
            .expect("attached");
            drop(retained);
            crate::python::attach(|_py| {}).expect("drain");
            assert_eq!(releases(versioned), before + 1);
        }
    }

    // Nullable deleters are ABI-valid: release is a no-op, not a call through
    // a null function pointer. The stack keeps allocation ownership throughout.
    let mut managed = DLManagedTensorVersioned {
        version: DLPackVersion { major: 1, minor: 0 },
        manager_ctx: std::ptr::null_mut(),
        deleter: None,
        flags: 0,
        dl_tensor: DLTensor {
            data: std::ptr::null_mut(),
            device: DLDevice {
                device_type: 1,
                device_id: 0,
            },
            ndim: 1,
            dtype: DLDataType {
                code: 2,
                bits: 64,
                lanes: 1,
            },
            shape: std::ptr::null_mut(),
            strides: std::ptr::null_mut(),
            byte_offset: 0,
        },
    };
    let mut shape = [0];
    managed.dl_tensor.shape = shape.as_mut_ptr();
    let tensor = ManagedTensor::Versioned(&raw mut managed);
    assert!(
        !metadata_for_managed_tensor(tensor)
            .expect("empty tensor")
            .has_deleter
    );
    // SAFETY: stack allocation is live and uniquely owned; null callback.
    unsafe { tensor.release() };
}

#[test]
fn rejection_and_reset_release_exactly_once() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    fixtures::reset_releases();
    initialize_runtime(test_config("h02g-rejection")).expect("initialize");

    for (flags, major, message) in [(2, 1, "copied tensor"), (0, 2, "major version")] {
        let before = releases(true);
        let object =
            fixtures::exporter(fixtures::versioned_capsule(flags, major, 0), DEVICE_CPU, 0)
                .expect("exporter");
        assert!(
            dlpack_tensor(&object)
                .expect_err("reject version/copy")
                .message
                .contains(message)
        );
        assert_eq!(
            releases(true),
            before + 1,
            "rejection releases before exporter closes"
        );
        assert!(
            dlpack_tensor(&object).is_err(),
            "rejected capsule already consumed"
        );
        close_object(object).expect("close");
        assert_eq!(releases(true), before + 1);
    }

    // Malformed shape/stride pointers are rejected before slice construction.
    for field in ["shape", "strides"] {
        let before = releases(false);
        let capsule = fixtures::legacy_capsule(1, 0).expect("capsule");
        crate::python::attach(|py| {
            let capsule = capsule.bind(py).cast::<PyCapsule>().expect("capsule");
            let pointer = capsule
                .pointer_checked(Some(DLTENSOR_NAME))
                .expect("pointer")
                .as_ptr()
                .cast::<DLManagedTensor>();
            // SAFETY: owned fixture header is live and not exposed to a consumer.
            let tensor = unsafe { &mut (*pointer).dl_tensor };
            if field == "shape" {
                tensor.shape = std::ptr::without_provenance_mut(1);
            } else {
                tensor.strides = std::ptr::without_provenance_mut(1);
            }
        })
        .expect("attached");
        let object = fixtures::exporter(Ok(capsule), 1, 0).expect("exporter");
        assert!(
            dlpack_tensor(&object)
                .expect_err("misaligned metadata")
                .message
                .contains("misaligned")
        );
        assert_eq!(releases(false), before + 1);
        close_object(object).expect("close");
        assert_eq!(releases(false), before + 1);
    }
    assert!(ManagedTensor::from_capsule_name(std::ptr::null_mut(), DLTENSOR_NAME).is_err());
    assert!(
        ManagedTensor::from_capsule_name(std::ptr::without_provenance_mut(1), DLTENSOR_NAME)
            .is_err()
    );
    assert!(
        ManagedTensor::from_capsule_name(
            std::ptr::without_provenance_mut(1),
            DLTENSOR_VERSIONED_NAME
        )
        .is_err()
    );

    for (device, id, stream) in [
        (
            2,
            0,
            Some(PythonDlpackStreamMetadata {
                device_type: 2,
                device_id: 0,
                stream_token: 0,
            }),
        ),
        (
            2,
            0,
            Some(PythonDlpackStreamMetadata {
                device_type: 2,
                device_id: 0,
                stream_token: -1,
            }),
        ),
        (
            2,
            -1,
            Some(PythonDlpackStreamMetadata {
                device_type: 2,
                device_id: -1,
                stream_token: 1,
            }),
        ),
        (
            99,
            0,
            Some(PythonDlpackStreamMetadata {
                device_type: 99,
                device_id: 0,
                stream_token: 1,
            }),
        ),
    ] {
        let object =
            fixtures::exporter(fixtures::legacy_capsule(device, id), device, id).expect("exporter");
        assert!(acquire_dlpack_tensor(&object, "any", stream.as_ref(), None).is_err());
        assert_eq!(
            fixtures::attribute_i64(&object, "calls"),
            0,
            "reject before calling producer"
        );
        close_object(object).expect("close");
    }

    for versioned in [false, true] {
        let before = releases(versioned);
        let object = exporter(versioned);
        let metadata = dlpack_tensor(&object).expect("acquire");
        let handle = (metadata.handle, metadata.token);
        reset_dlpack_store_for_tests();
        assert_eq!(releases(versioned), before + 1);
        assert!(release_dlpack(handle).is_err());
        close_object(object).expect("close");
        assert_eq!(releases(versioned), before + 1);

        let object = exporter(versioned);
        let metadata = dlpack_tensor(&object).expect("acquire");
        assert_ne!(
            metadata.handle, handle.0,
            "reset does not revive identities"
        );
        let argument = prepare_dlpack_argument((metadata.handle, metadata.token)).expect("prepare");
        reset_runtime_state_for_tests();
        drop(argument); // detached/semantically stopped cleanup still attaches
        assert_eq!(releases(versioned), before + 2);
        initialize_runtime(test_config("h02g-rejection")).expect("reinitialize");
        drop(object);
        crate::python::attach(|_py| {}).expect("drain");
        assert_eq!(releases(versioned), before + 2);
    }
    reset_reentrant_release();
}

fn exporter(versioned: bool) -> ObjectHandle {
    let capsule = if versioned {
        fixtures::versioned_capsule(0, 1, 3)
    } else {
        fixtures::legacy_capsule(1, 0)
    };
    fixtures::exporter(capsule, 1, 0).expect("exporter")
}
fn releases(versioned: bool) -> usize {
    if versioned {
        &fixtures::VERSIONED_RELEASES
    } else {
        &fixtures::LEGACY_RELEASES
    }
    .load(Ordering::SeqCst)
}
fn consume_and_release(object: &Py<PyAny>, versioned: bool) {
    crate::python::attach(|py| {
        let capsule = object.bind(py).cast::<PyCapsule>().expect("capsule");
        let name = if versioned {
            DLTENSOR_VERSIONED_NAME
        } else {
            DLTENSOR_NAME
        };
        let pointer = capsule.pointer_checked(Some(name)).expect("pointer");
        let tensor = ManagedTensor::from_capsule_name(pointer.as_ptr(), name).expect("tensor");
        // SAFETY: fixture consumer performs the protocol's one-shot transfer.
        assert_eq!(
            unsafe {
                ffi::PyCapsule_SetName(capsule.as_ptr(), tensor.used_capsule_name().as_ptr())
            },
            0
        );
        // SAFETY: consumer owns the allocation after renaming and releases once.
        unsafe { tensor.release() };
    })
    .expect("attached");
}

fn assert_abi_layout() {
    // DLPack v1.2 include/dlpack/dlpack.h, qualified x86_64 C layout.
    assert_eq!(size_of::<usize>(), 8);
    assert_eq!((size_of::<DLDevice>(), align_of::<DLDevice>()), (8, 4));
    assert_eq!(
        [
            offset_of!(DLDevice, device_type),
            offset_of!(DLDevice, device_id)
        ],
        [0, 4]
    );
    assert_eq!((size_of::<DLDataType>(), align_of::<DLDataType>()), (4, 2));
    assert_eq!(
        [
            offset_of!(DLDataType, code),
            offset_of!(DLDataType, bits),
            offset_of!(DLDataType, lanes)
        ],
        [0, 1, 2]
    );
    assert_eq!(
        (size_of::<DLPackVersion>(), align_of::<DLPackVersion>()),
        (8, 4)
    );
    assert_eq!(
        [
            offset_of!(DLPackVersion, major),
            offset_of!(DLPackVersion, minor)
        ],
        [0, 4]
    );
    assert_eq!((size_of::<DLTensor>(), align_of::<DLTensor>()), (48, 8));
    assert_eq!(
        [
            offset_of!(DLTensor, data),
            offset_of!(DLTensor, device),
            offset_of!(DLTensor, ndim),
            offset_of!(DLTensor, dtype),
            offset_of!(DLTensor, shape),
            offset_of!(DLTensor, strides),
            offset_of!(DLTensor, byte_offset)
        ],
        [0, 8, 16, 20, 24, 32, 40]
    );
    assert_eq!(
        (size_of::<DLManagedTensor>(), align_of::<DLManagedTensor>()),
        (64, 8)
    );
    assert_eq!(
        [
            offset_of!(DLManagedTensor, dl_tensor),
            offset_of!(DLManagedTensor, manager_ctx),
            offset_of!(DLManagedTensor, deleter)
        ],
        [0, 48, 56]
    );
    assert_eq!(
        (
            size_of::<DLManagedTensorVersioned>(),
            align_of::<DLManagedTensorVersioned>()
        ),
        (80, 8)
    );
    assert_eq!(
        [
            offset_of!(DLManagedTensorVersioned, version),
            offset_of!(DLManagedTensorVersioned, manager_ctx),
            offset_of!(DLManagedTensorVersioned, deleter),
            offset_of!(DLManagedTensorVersioned, flags),
            offset_of!(DLManagedTensorVersioned, dl_tensor)
        ],
        [0, 8, 16, 24, 32]
    );
}

static REENTRANT_RELEASES: AtomicUsize = AtomicUsize::new(0);
static RELEASE_ATTACHED: AtomicBool = AtomicBool::new(false);
static STORE_UNLOCKED: AtomicBool = AtomicBool::new(false);
unsafe extern "C" fn reentrant_deleter(pointer: *mut DLManagedTensor) {
    // SAFETY: callback runs once for the Box allocated by reset_reentrant_release.
    RELEASE_ATTACHED.store(unsafe { ffi::PyGILState_Check() } != 0, Ordering::SeqCst);
    STORE_UNLOCKED.store(DLPACK_STORE.try_lock().is_ok(), Ordering::SeqCst);
    REENTRANT_RELEASES.fetch_add(1, Ordering::SeqCst);
    // SAFETY: this callback owns the uniquely allocated fixture header.
    drop(unsafe { Box::from_raw(pointer) });
}
fn reset_reentrant_release() {
    let owner = exporter(false);
    crate::python::attach(|py| {
        let managed = Box::new(DLManagedTensor {
            dl_tensor: DLTensor {
                data: std::ptr::null_mut(),
                device: DLDevice {
                    device_type: 1,
                    device_id: 0,
                },
                ndim: 0,
                dtype: DLDataType {
                    code: 2,
                    bits: 64,
                    lanes: 1,
                },
                shape: std::ptr::null_mut(),
                strides: std::ptr::null_mut(),
                byte_offset: 0,
            },
            manager_ctx: std::ptr::null_mut(),
            deleter: Some(reentrant_deleter),
        });
        let tensor = ManagedTensor::Legacy(Box::into_raw(managed));
        let pin = clone_handle(py, &owner).expect("pin");
        let tracked = TrackedDlpackTensor {
            tensor,
            released: false,
            counted: false,
            _owner: pin.clone_ref(py),
            _capsule: pin,
        };
        // Metadata is copied separately; this test does not dereference data.
        let metadata = PythonDlpackTensorMetadata {
            handle: 0,
            token: 0,
            dtype_code: 2,
            dtype_bits: 64,
            dtype_lanes: 1,
            dtype: "float64".into(),
            device_type: 1,
            device_id: 0,
            dimensions: 0,
            shape: vec![],
            strides: vec![],
            byte_offset: 0,
            has_deleter: true,
            stream_sync_required: false,
        };
        store_tensor(tracked, metadata).expect("store");
    })
    .expect("attached");
    let before = REENTRANT_RELEASES.load(Ordering::SeqCst);
    std::thread::spawn(reset_dlpack_store_for_tests)
        .join()
        .expect("detached reset");
    assert_eq!(REENTRANT_RELEASES.load(Ordering::SeqCst), before + 1);
    assert!(RELEASE_ATTACHED.load(Ordering::SeqCst));
    assert!(STORE_UNLOCKED.load(Ordering::SeqCst));
    close_object(owner).expect("close");
}
