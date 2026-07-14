use super::raw::OwnedPyBuffer;
use super::{index_error, with_live_buffer, BufferHandle, PythonBufferElement, PythonError};
use pyo3::Python;
use std::ptr;

macro_rules! typed_buffer_accessors {
    ($read:ident, $write:ident, $copy:ident, $element:ident, $ty:ty) => {
        pub fn $read(buffer: BufferHandle, index: i64) -> Result<$ty, PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, false, |py, value| {
                read_typed(py, value, index)
            })
        }

        pub fn $write(buffer: BufferHandle, index: i64, value: $ty) -> Result<(), PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, true, |py, target| {
                write_typed(py, target, index, value)
            })
        }

        pub fn $copy(
            buffer: BufferHandle,
            start: i64,
            length: i64,
        ) -> Result<Vec<$ty>, PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, false, |py, value| {
                copy_typed_slice(py, value, start, length)
            })
        }
    };
}

typed_buffer_accessors!(
    buffer_read_i8,
    buffer_write_i8,
    copy_buffer_slice_i8,
    I8,
    i8
);
typed_buffer_accessors!(
    buffer_read_i16,
    buffer_write_i16,
    copy_buffer_slice_i16,
    I16,
    i16
);
typed_buffer_accessors!(
    buffer_read_i32,
    buffer_write_i32,
    copy_buffer_slice_i32,
    I32,
    i32
);
typed_buffer_accessors!(
    buffer_read_i64,
    buffer_write_i64,
    copy_buffer_slice_i64,
    I64,
    i64
);
typed_buffer_accessors!(
    buffer_read_isize,
    buffer_write_isize,
    copy_buffer_slice_isize,
    ISize,
    isize
);
typed_buffer_accessors!(
    buffer_read_u8,
    buffer_write_u8,
    copy_buffer_slice_u8,
    U8,
    u8
);
typed_buffer_accessors!(
    buffer_read_u16,
    buffer_write_u16,
    copy_buffer_slice_u16,
    U16,
    u16
);
typed_buffer_accessors!(
    buffer_read_u32,
    buffer_write_u32,
    copy_buffer_slice_u32,
    U32,
    u32
);
typed_buffer_accessors!(
    buffer_read_u64,
    buffer_write_u64,
    copy_buffer_slice_u64,
    U64,
    u64
);
typed_buffer_accessors!(
    buffer_read_usize,
    buffer_write_usize,
    copy_buffer_slice_usize,
    USize,
    usize
);
typed_buffer_accessors!(
    buffer_read_f64,
    buffer_write_f64,
    copy_buffer_slice_f64,
    F64,
    f64
);

pub fn copy_buffer_u8(buffer: BufferHandle) -> Result<Vec<u8>, PythonError> {
    with_live_buffer(buffer, PythonBufferElement::U8, false, |py, value| {
        copy_typed_range(py, value, 0, value.item_count())
    })
}

fn read_typed<T: Copy>(
    py: Python<'_>,
    buffer: &OwnedPyBuffer,
    index: i64,
) -> Result<T, PythonError> {
    let index = checked_index(py, index, buffer.item_count())?;
    let pointer = buffer
        .item_ptr(index)
        .ok_or_else(|| index_error(py, "buffer index is out of bounds"))?;
    // SAFETY: acquisition validated the element format and width. The pointer
    // addresses one logical element and unaligned access handles arbitrary
    // valid PEP 3118 strides.
    Ok(unsafe { ptr::read_unaligned(pointer.cast::<T>()) })
}

fn write_typed<T: Copy>(
    py: Python<'_>,
    buffer: &OwnedPyBuffer,
    index: i64,
    value: T,
) -> Result<(), PythonError> {
    let index = checked_index(py, index, buffer.item_count())?;
    let pointer = buffer
        .item_ptr(index)
        .ok_or_else(|| index_error(py, "buffer index is out of bounds"))?;
    // SAFETY: write admission is checked against both the declaration and the
    // exporter request. Format/width validation and unaligned writes make this
    // valid for arbitrary accepted strides.
    unsafe { ptr::write_unaligned(pointer.cast::<T>(), value) };
    Ok(())
}

fn copy_typed_slice<T: Copy>(
    py: Python<'_>,
    buffer: &OwnedPyBuffer,
    start: i64,
    length: i64,
) -> Result<Vec<T>, PythonError> {
    let start = checked_index_inclusive(py, start, buffer.item_count())?;
    let length = usize::try_from(length)
        .map_err(|_| index_error(py, "buffer slice length must be non-negative"))?;
    let end = start
        .checked_add(length)
        .filter(|end| *end <= buffer.item_count())
        .ok_or_else(|| index_error(py, "buffer slice is out of bounds"))?;
    copy_typed_range(py, buffer, start, end)
}

fn copy_typed_range<T: Copy>(
    py: Python<'_>,
    buffer: &OwnedPyBuffer,
    start: usize,
    end: usize,
) -> Result<Vec<T>, PythonError> {
    let mut values = Vec::with_capacity(end.saturating_sub(start));
    for index in start..end {
        let pointer = buffer
            .item_ptr(index)
            .ok_or_else(|| index_error(py, "buffer slice is out of bounds"))?;
        // SAFETY: identical to `read_typed`, repeated in logical C order.
        values.push(unsafe { ptr::read_unaligned(pointer.cast::<T>()) });
    }
    Ok(values)
}

fn checked_index(py: Python<'_>, index: i64, length: usize) -> Result<usize, PythonError> {
    let index =
        usize::try_from(index).map_err(|_| index_error(py, "buffer index must be non-negative"))?;
    if index >= length {
        return Err(index_error(py, "buffer index is out of bounds"));
    }
    Ok(index)
}

fn checked_index_inclusive(
    py: Python<'_>,
    index: i64,
    length: usize,
) -> Result<usize, PythonError> {
    let index = usize::try_from(index)
        .map_err(|_| index_error(py, "buffer slice start must be non-negative"))?;
    if index > length {
        return Err(index_error(py, "buffer slice start is out of bounds"));
    }
    Ok(index)
}
