use super::*;
use crate::python::{
    close_object, initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard,
};
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use std::ffi::{c_char, c_int, c_void};
use std::mem::size_of;
use std::ptr;

#[pyclass]
struct TypedAccessExporter {
    storage: Vec<u8>,
    format: [u8; 2],
    item_size: isize,
    shape: isize,
    stride: isize,
}

#[pymethods]
impl TypedAccessExporter {
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
        let format_pointer = exporter.format.as_ptr().cast::<c_char>().cast_mut();
        let shape_pointer = ptr::from_ref(&exporter.shape).cast_mut();
        let stride_pointer = ptr::from_ref(&exporter.stride).cast_mut();
        let length = isize::try_from(exporter.storage.len())
            .map_err(|_| PyBufferError::new_err("buffer length exceeds Py_ssize_t"))?;
        let item_size = exporter.item_size;
        drop(exporter);

        unsafe {
            (*view).obj = slf.into_any().into_ptr();
            (*view).buf = data_pointer;
            (*view).len = length;
            (*view).readonly = 0;
            (*view).itemsize = item_size;
            (*view).format = if flags & ffi::PyBUF_FORMAT == ffi::PyBUF_FORMAT {
                format_pointer
            } else {
                ptr::null_mut()
            };
            (*view).ndim = 1;
            (*view).shape = if flags & ffi::PyBUF_ND == ffi::PyBUF_ND {
                shape_pointer
            } else {
                ptr::null_mut()
            };
            (*view).strides = if flags & ffi::PyBUF_STRIDES == ffi::PyBUF_STRIDES {
                stride_pointer
            } else {
                ptr::null_mut()
            };
            (*view).suboffsets = ptr::null_mut();
            (*view).internal = ptr::null_mut();
        }
        Ok(())
    }

    #[expect(
        clippy::unused_self,
        reason = "PyO3 buffer protocol requires an exporter receiver"
    )]
    unsafe fn __releasebuffer__(&self, _view: *mut ffi::Py_buffer) {}
}

fn typed_exporter(format: u8, item_size: usize) -> ObjectHandle {
    super::super::attach(|py| {
        let storage_length = item_size * 2;
        let item_size = isize::try_from(item_size)
            .map_err(|_| PyBufferError::new_err("item size exceeds Py_ssize_t"))?;
        let exporter = Bound::new(
            py,
            TypedAccessExporter {
                storage: vec![0; storage_length],
                format: [format, 0],
                item_size,
                shape: 2,
                stride: item_size,
            },
        )?;
        super::super::object_ops::store_object(exporter.into_any().unbind())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("attach should succeed")
    .expect("typed exporter should be stored")
}

macro_rules! assert_typed_round_trip {
    ($format:literal, $element:ident, $ty:ty, $read:ident, $write:ident, $copy:ident, $value:expr) => {{
        let object = typed_exporter($format, size_of::<$ty>());
        let view = acquire_buffer(
            &object,
            PythonBufferRequest {
                element: PythonBufferElement::$element,
                access: PythonBufferAccess::Write,
                layout: PythonBufferLayout::CContiguous,
            },
        )
        .expect("typed buffer should acquire");
        let key = (view.handle, view.token);
        $write(key, 1, $value).expect("typed write should succeed");
        assert_eq!($read(key, 1).expect("typed read should succeed"), $value);
        assert_eq!(
            $copy(key, 0, 2).expect("typed copy should succeed"),
            vec![0 as $ty, $value]
        );
        release_buffer(key).expect("typed buffer should release");
        close_object(object).expect("typed exporter should close");
    }};
}

fn assert_float_round_trip() {
    let object = typed_exporter(b'd', size_of::<f64>());
    let view = acquire_buffer(
        &object,
        PythonBufferRequest {
            element: PythonBufferElement::F64,
            access: PythonBufferAccess::Write,
            layout: PythonBufferLayout::CContiguous,
        },
    )
    .expect("float buffer should acquire");
    let key = (view.handle, view.token);
    let value = 1.5_f64;
    buffer_write_f64(key, 1, value).expect("float write should succeed");
    assert_eq!(
        buffer_read_f64(key, 1)
            .expect("float read should succeed")
            .to_bits(),
        value.to_bits()
    );
    let copied = copy_buffer_slice_f64(key, 0, 2).expect("float copy should succeed");
    assert_eq!(
        copied.iter().map(|item| item.to_bits()).collect::<Vec<_>>(),
        vec![0_f64.to_bits(), value.to_bits()]
    );
    release_buffer(key).expect("float buffer should release");
    close_object(object).expect("float exporter should close");
}

#[test]
fn every_supported_primitive_buffer_accessor_round_trips() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("buffer-all-typed-accessors")).expect("init should succeed");

    assert_typed_round_trip!(
        b'b',
        I8,
        i8,
        buffer_read_i8,
        buffer_write_i8,
        copy_buffer_slice_i8,
        -7
    );
    assert_typed_round_trip!(
        b'h',
        I16,
        i16,
        buffer_read_i16,
        buffer_write_i16,
        copy_buffer_slice_i16,
        -17
    );
    assert_typed_round_trip!(
        b'i',
        I32,
        i32,
        buffer_read_i32,
        buffer_write_i32,
        copy_buffer_slice_i32,
        -27
    );
    assert_typed_round_trip!(
        b'q',
        I64,
        i64,
        buffer_read_i64,
        buffer_write_i64,
        copy_buffer_slice_i64,
        -37
    );
    assert_typed_round_trip!(
        b'n',
        ISize,
        isize,
        buffer_read_isize,
        buffer_write_isize,
        copy_buffer_slice_isize,
        -47
    );
    assert_typed_round_trip!(
        b'B',
        U8,
        u8,
        buffer_read_u8,
        buffer_write_u8,
        copy_buffer_slice_u8,
        7
    );
    assert_typed_round_trip!(
        b'H',
        U16,
        u16,
        buffer_read_u16,
        buffer_write_u16,
        copy_buffer_slice_u16,
        17
    );
    assert_typed_round_trip!(
        b'I',
        U32,
        u32,
        buffer_read_u32,
        buffer_write_u32,
        copy_buffer_slice_u32,
        27
    );
    assert_typed_round_trip!(
        b'Q',
        U64,
        u64,
        buffer_read_u64,
        buffer_write_u64,
        copy_buffer_slice_u64,
        37
    );
    assert_typed_round_trip!(
        b'N',
        USize,
        usize,
        buffer_read_usize,
        buffer_write_usize,
        copy_buffer_slice_usize,
        47
    );
    assert_float_round_trip();
}
