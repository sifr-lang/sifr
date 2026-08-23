use super::raw::OwnedPyBuffer;
use super::{BufferAccessError, BufferHandle, PythonBufferElement, PythonError, with_live_buffer};
use std::ptr;

macro_rules! typed_buffer_accessors {
    ($read:ident, $write:ident, $copy:ident, $element:ident, $ty:ty) => {
        pub fn $read(buffer: BufferHandle, index: i64) -> Result<$ty, PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, false, |value| {
                read_typed(value, index)
            })
        }

        pub fn $write(buffer: BufferHandle, index: i64, value: $ty) -> Result<(), PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, true, |target| {
                write_typed(target, index, value)
            })
        }

        pub fn $copy(
            buffer: BufferHandle,
            start: i64,
            length: i64,
        ) -> Result<Vec<$ty>, PythonError> {
            with_live_buffer(buffer, PythonBufferElement::$element, false, |value| {
                copy_typed_slice(value, start, length)
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
    with_live_buffer(buffer, PythonBufferElement::U8, false, |value| {
        copy_typed_range(value, 0, value.item_count())
    })
}

fn read_typed<T: Copy>(buffer: &OwnedPyBuffer, index: i64) -> Result<T, BufferAccessError> {
    let index = checked_index(index, buffer.item_count())?;
    let pointer = buffer
        .item_ptr(index)
        .ok_or(BufferAccessError::Index("buffer index is out of bounds"))?;
    // SAFETY: acquisition validated the element format and width. The pointer
    // addresses one logical element and unaligned access handles arbitrary
    // valid PEP 3118 strides.
    Ok(unsafe { ptr::read_unaligned(pointer.cast::<T>()) })
}

fn write_typed<T: Copy>(
    buffer: &OwnedPyBuffer,
    index: i64,
    value: T,
) -> Result<(), BufferAccessError> {
    let index = checked_index(index, buffer.item_count())?;
    let pointer = buffer
        .item_ptr(index)
        .ok_or(BufferAccessError::Index("buffer index is out of bounds"))?;
    // SAFETY: write admission is checked against both the declaration and the
    // exporter request. Format/width validation and unaligned writes make this
    // valid for arbitrary accepted strides.
    unsafe { ptr::write_unaligned(pointer.cast::<T>(), value) };
    Ok(())
}

fn copy_typed_slice<T: Copy>(
    buffer: &OwnedPyBuffer,
    start: i64,
    length: i64,
) -> Result<Vec<T>, BufferAccessError> {
    let start = checked_index_inclusive(start, buffer.item_count())?;
    let length = usize::try_from(length)
        .map_err(|_| BufferAccessError::Index("buffer slice length must be non-negative"))?;
    let end = start
        .checked_add(length)
        .filter(|end| *end <= buffer.item_count())
        .ok_or(BufferAccessError::Index("buffer slice is out of bounds"))?;
    copy_typed_range(buffer, start, end)
}

fn copy_typed_range<T: Copy>(
    buffer: &OwnedPyBuffer,
    start: usize,
    end: usize,
) -> Result<Vec<T>, BufferAccessError> {
    let mut values = Vec::with_capacity(end.saturating_sub(start));
    for index in start..end {
        let pointer = buffer
            .item_ptr(index)
            .ok_or(BufferAccessError::Index("buffer slice is out of bounds"))?;
        // SAFETY: identical to `read_typed`, repeated in logical C order.
        values.push(unsafe { ptr::read_unaligned(pointer.cast::<T>()) });
    }
    Ok(values)
}

fn checked_index(index: i64, length: usize) -> Result<usize, BufferAccessError> {
    let index = usize::try_from(index)
        .map_err(|_| BufferAccessError::Index("buffer index must be non-negative"))?;
    if index >= length {
        return Err(BufferAccessError::Index("buffer index is out of bounds"));
    }
    Ok(index)
}

fn checked_index_inclusive(index: i64, length: usize) -> Result<usize, BufferAccessError> {
    let index = usize::try_from(index)
        .map_err(|_| BufferAccessError::Index("buffer slice start must be non-negative"))?;
    if index > length {
        return Err(BufferAccessError::Index(
            "buffer slice start is out of bounds",
        ));
    }
    Ok(index)
}
