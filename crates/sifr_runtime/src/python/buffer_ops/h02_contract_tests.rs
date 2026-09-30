use super::*;
use crate::python::{
    PythonResourceIdentity, close_object, initialize_runtime, reset_runtime_state_for_tests,
    test_config, test_guard,
};
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use std::ffi::c_int;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy)]
enum Layout {
    Direct,
    Fortran,
    Negative,
    Indirect,
    NullRow,
    OverflowStride,
    OverflowAddress,
    MisalignedShape,
    MisalignedSlot,
    UnalignedI16,
    BadLength,
    FailedAcquisition,
}

#[derive(Default)]
struct Metrics {
    acquisitions: AtomicUsize,
    releases: AtomicUsize,
    drops: AtomicUsize,
}

#[pyclass]
struct Exporter {
    data: Vec<u8>,
    rows: Vec<usize>,
    shape: Vec<isize>,
    strides: Vec<isize>,
    suboffsets: Vec<isize>,
    layout: Layout,
    metrics: Arc<Metrics>,
}

impl Drop for Exporter {
    fn drop(&mut self) {
        self.metrics.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[pymethods]
impl Exporter {
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        _flags: c_int,
    ) -> PyResult<()> {
        if view.is_null() {
            return Err(PyBufferError::new_err("null view"));
        }
        let exporter = slf.borrow();
        exporter.metrics.acquisitions.fetch_add(1, Ordering::SeqCst);
        if matches!(exporter.layout, Layout::FailedAcquisition) {
            return Err(PyBufferError::new_err(
                "acquisition rejected before ownership transfer",
            ));
        }
        let mut data = exporter.data.as_ptr().cast_mut().cast();
        let mut shape = exporter.shape.as_ptr().cast_mut();
        let mut len = 4;
        let mut width = 1;
        let mut format = c"B".as_ptr().cast_mut();
        match exporter.layout {
            Layout::Negative => data = exporter.data.as_ptr().cast_mut().wrapping_add(3).cast(),
            Layout::Indirect | Layout::NullRow => data = exporter.rows.as_ptr().cast_mut().cast(),
            Layout::OverflowAddress => data = ptr::with_exposed_provenance_mut(usize::MAX - 1),
            Layout::MisalignedShape => shape = shape.wrapping_byte_add(1),
            Layout::MisalignedSlot => {
                data = exporter
                    .rows
                    .as_ptr()
                    .cast_mut()
                    .wrapping_byte_add(1)
                    .cast()
            }
            Layout::UnalignedI16 => {
                data = exporter.data.as_ptr().cast_mut().wrapping_add(1).cast();
                width = 2;
                format = c"h".as_ptr().cast_mut();
            }
            Layout::BadLength => len = 3,
            _ => {}
        }
        let dimensions = i32::try_from(exporter.shape.len()).expect("bounded fixture dimensions");
        let strides = exporter.strides.as_ptr().cast_mut();
        let suboffsets = if exporter.suboffsets.is_empty() {
            ptr::null_mut()
        } else {
            exporter.suboffsets.as_ptr().cast_mut()
        };
        drop(exporter);
        // SAFETY: PyO3 supplies the output view under the GIL. The strong owner
        // pins all fixture allocations. Malformed modes exercise checks that
        // must reject their layouts before reading data or invalid pointer slots.
        unsafe {
            *view = ffi::Py_buffer {
                obj: slf.into_any().into_ptr(),
                buf: data,
                len,
                itemsize: width,
                readonly: 0,
                ndim: dimensions,
                format,
                shape,
                strides,
                suboffsets,
                internal: ptr::null_mut(),
            };
        }
        Ok(())
    }

    unsafe fn __releasebuffer__(&self, _view: *mut ffi::Py_buffer) {
        self.metrics.releases.fetch_add(1, Ordering::SeqCst);
    }
}

fn fixture(layout: Layout) -> (ObjectHandle, Arc<Metrics>) {
    let metrics = Arc::new(Metrics::default());
    let object = super::super::attach(|py| {
        let data = vec![1, 2, 3, 4, 0];
        let mut rows = vec![
            data.as_ptr() as usize,
            data.as_ptr().wrapping_add(2) as usize,
        ];
        let (mut shape, mut strides, mut suboffsets) = (vec![4], vec![1], vec![]);
        match layout {
            Layout::Negative => strides = vec![-1],
            Layout::Fortran => {
                shape = vec![2, 2];
                strides = vec![1, 2];
            }
            Layout::Indirect | Layout::NullRow | Layout::MisalignedSlot => {
                shape = vec![2, 2];
                strides = vec![
                    isize::try_from(std::mem::size_of::<usize>()).expect("width"),
                    1,
                ];
                suboffsets = vec![0, -1];
                if matches!(layout, Layout::NullRow) {
                    rows[1] = 0;
                }
            }
            Layout::OverflowStride => strides = vec![isize::MAX],
            Layout::UnalignedI16 => {
                shape = vec![2];
                strides = vec![2];
            }
            _ => {}
        }
        let exporter = Bound::new(
            py,
            Exporter {
                data,
                rows,
                shape,
                strides,
                suboffsets,
                layout,
                metrics: Arc::clone(&metrics),
            },
        )?;
        super::super::object_ops::store_object(exporter.into_any().unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach")
    .expect("fixture");
    (object, metrics)
}

fn key(view: &PythonBufferMetadata) -> BufferHandle {
    (view.handle, view.token)
}
fn value(counter: &AtomicUsize) -> usize {
    counter.load(Ordering::SeqCst)
}

#[test]
fn layout_bounds_and_alias_admission() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("h02e-layout-bounds-alias")).expect("init");
    // Invalid layouts carry real owners and must release exactly once without
    // following null, misaligned or arithmetic-overflowing addresses.
    for layout in [
        Layout::NullRow,
        Layout::OverflowStride,
        Layout::OverflowAddress,
        Layout::MisalignedShape,
        Layout::MisalignedSlot,
        Layout::BadLength,
    ] {
        let (object, metrics) = fixture(layout);
        buffer_u8(&object, true).expect_err("malformed layout must reject before access");
        assert_eq!(value(&metrics.releases), 1);
        close_object(object).expect("close");
        assert_eq!(value(&metrics.drops), 1);
    }
    for (layout, expected) in [
        (Layout::Negative, vec![4, 3, 2, 1]),
        (Layout::Indirect, vec![1, 2, 3, 4]),
    ] {
        let (object, metrics) = fixture(layout);
        let reader = buffer_u8(&object, false).expect("reader");
        let second = buffer_u8(&object, false).expect("shared reader");
        assert_eq!(copy_buffer_u8(key(&reader)).expect("copy"), expected);
        buffer_u8(&object, true).expect_err("writer conflicts with shared storage");
        buffer_read_u8(key(&reader), -1).expect_err("negative index");
        buffer_read_u8(key(&reader), 4).expect_err("past last index");
        copy_buffer_slice_u8(key(&reader), 3, 2).expect_err("slice bounds");
        buffer_write_u8(key(&reader), 0, 9).expect_err("read admission cannot write");
        release_buffer(key(&reader)).expect("release");
        buffer_u8(&object, true).expect_err("remaining reader still conflicts");
        release_buffer(key(&second)).expect("release");
        let writer = buffer_u8(&object, true).expect("writer after readers release");
        buffer_write_u8(key(&writer), 1, 9).expect("write");
        assert_eq!(buffer_read_u8(key(&writer), 1).expect("read"), 9);
        buffer_u8(&object, false).expect_err("writer excludes readers");
        if matches!(layout, Layout::Indirect) {
            super::super::attach(|py| {
                let alias = clone_handle(py, &object).expect("fixture alias");
                let mut exporter = alias
                    .bind(py)
                    .cast::<Exporter>()
                    .expect("fixture type")
                    .borrow_mut();
                exporter.rows.fill(0);
                exporter.strides.fill(isize::MAX);
            })
            .expect("attach");
            // Access uses the allocation addresses pinned at admission, rather
            // than re-following changed foreign metadata outside that footprint.
            assert_eq!(
                buffer_read_u8(key(&writer), 0).expect("admitted pointer remains valid"),
                1
            );
            buffer_write_u8(key(&writer), 3, 7).expect("write admitted snapshot");
            assert_eq!(
                buffer_read_u8(key(&writer), 3).expect("read admitted snapshot"),
                7
            );
        }
        release_buffer(key(&writer)).expect("release");
        assert_eq!(value(&metrics.acquisitions), value(&metrics.releases));
        close_object(object).expect("close");
    }
    let (object, _) = fixture(Layout::Fortran);
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::FContiguous,
        },
    )
    .expect("Fortran dense layout");
    assert!(view.f_contiguous);
    assert!(!view.c_contiguous);
    assert_eq!(
        copy_buffer_u8(key(&view)).expect("logical C order"),
        vec![1, 3, 2, 4]
    );
    buffer_write_u8(key(&view), 1, 8).expect("Fortran logical write");
    assert_eq!(
        buffer_read_u8(key(&view), 1).expect("Fortran logical read"),
        8
    );
    release_buffer(key(&view)).expect("release");
    close_object(object).expect("close");
    // Element alignment is intentionally unrestricted: access uses unaligned
    // primitive operations. Metadata/pointer-slot alignment is required above.
    let (object, _) = fixture(Layout::UnalignedI16);
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I16,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect("unaligned elements");
    buffer_write_i16(key(&view), 1, -1234).expect("unaligned write");
    assert_eq!(
        buffer_read_i16(key(&view), 1).expect("unaligned read"),
        -1234
    );
    release_buffer(key(&view)).expect("release");
    close_object(object).expect("close");
}

#[test]
fn all_release_paths_are_exact_once() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("h02e-exact-release")).expect("init");
    let (object, metrics) = fixture(Layout::FailedAcquisition);
    buffer_u8(&object, false).expect_err("failed acquisition");
    assert_eq!(value(&metrics.acquisitions), 1);
    assert_eq!(
        value(&metrics.releases),
        0,
        "failed acquisition never transferred an export"
    );
    close_object(object).expect("close");
    assert_eq!(value(&metrics.drops), 1);

    for automatic in [false, true] {
        let (object, metrics) = fixture(Layout::Direct);
        let view = buffer_u8(&object, true).expect("acquire");
        let snapshot = buffer_snapshot(key(&view)).expect("snapshot");
        close_object(object).expect("export pins lifetime after foreign handle closes");
        assert_eq!(value(&metrics.drops), 0);
        if automatic {
            std::thread::spawn(move || drop(PythonResourceIdentity::buffer(key(&view))))
                .join()
                .expect("foreign-thread resource drop");
        } else {
            release_buffer(key(&view)).expect("explicit release");
            release_buffer(key(&view)).expect_err("duplicate release");
            buffer_read_u8(key(&view), 0).expect_err("closed access");
        }
        assert_eq!(value(&metrics.releases), 1);
        assert_eq!(value(&metrics.drops), 1);
        drop(snapshot);
        assert_eq!(
            value(&metrics.releases),
            1,
            "snapshot drop cannot release twice"
        );
    }
    let (object, metrics) = fixture(Layout::Direct);
    acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::I16,
            access: PythonBufferAccess::Read,
            layout: PythonBufferLayout::Any,
        },
    )
    .expect_err("format failure");
    assert_eq!(value(&metrics.releases), 1);
    let reader = buffer_u8(&object, false).expect("reader");
    buffer_u8(&object, true).expect_err("admission failure");
    assert_eq!(value(&metrics.releases), 2);
    release_buffer(key(&reader)).expect("reader release");
    let previous = {
        let mut store = buffer_store().expect("store");
        std::mem::replace(&mut store.next_handle, i64::MAX)
    };
    buffer_u8(&object, false).expect_err("storage failure");
    buffer_store().expect("store").next_handle = previous;
    assert_eq!(value(&metrics.acquisitions), 4);
    assert_eq!(value(&metrics.releases), 4);
    close_object(object).expect("close");
    assert_eq!(value(&metrics.drops), 1);
}
