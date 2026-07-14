use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::buffer::{Element, PyBuffer};
use pyo3::exceptions::{PyBufferError, PyIndexError, PyTypeError};
use pyo3::Python;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

static BUFFER_STORE: LazyLock<Mutex<BufferStore>> =
    LazyLock::new(|| Mutex::new(BufferStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type BufferHandle = (i64, i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonBufferElement {
    I8,
    I16,
    I32,
    I64,
    ISize,
    U8,
    U16,
    U32,
    U64,
    USize,
    F64,
}

impl PythonBufferElement {
    #[must_use]
    pub const fn source_name(self) -> &'static str {
        match self {
            Self::I8 => "int8",
            Self::I16 => "int16",
            Self::I32 => "int32",
            Self::I64 => "int64",
            Self::ISize => "isize",
            Self::U8 => "uint8",
            Self::U16 => "uint16",
            Self::U32 => "uint32",
            Self::U64 => "uint64",
            Self::USize => "usize",
            Self::F64 => "float",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonBufferAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonBufferLayout {
    Any,
    CContiguous,
    FContiguous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonBufferRequest {
    pub element: PythonBufferElement,
    pub access: PythonBufferAccess,
    pub layout: PythonBufferLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonBufferMetadata {
    pub handle: i64,
    pub token: i64,
    pub element: PythonBufferElement,
    pub len_bytes: i64,
    pub item_size: i64,
    pub readonly: bool,
    pub dimensions: i64,
    pub shape: Vec<i64>,
    pub strides: Vec<i64>,
    pub suboffsets: Vec<i64>,
    pub c_contiguous: bool,
    pub f_contiguous: bool,
    pub format: String,
}

#[derive(Default)]
struct BufferStore {
    next_handle: i64,
    next_nonce: u64,
    buffers: HashMap<i64, BufferEntry>,
}

struct BufferEntry {
    token: i64,
    buffer: Arc<TrackedBuffer>,
    metadata: PythonBufferMetadata,
}

enum TrackedBuffer {
    I8(PyBuffer<i8>),
    I16(PyBuffer<i16>),
    I32(PyBuffer<i32>),
    I64(PyBuffer<i64>),
    ISize(PyBuffer<isize>),
    U8(PyBuffer<u8>),
    U16(PyBuffer<u16>),
    U32(PyBuffer<u32>),
    U64(PyBuffer<u64>),
    USize(PyBuffer<usize>),
    F64(PyBuffer<f64>),
}

impl Drop for TrackedBuffer {
    fn drop(&mut self) {
        let _ignored = super::update_object_count(-1);
    }
}

pub fn acquire_buffer(
    object: &ObjectHandle,
    request: PythonBufferRequest,
) -> Result<PythonBufferMetadata, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let object = object.bind(py);
        match request.element {
            PythonBufferElement::I8 => acquire_typed(object, request, TrackedBuffer::I8),
            PythonBufferElement::I16 => acquire_typed(object, request, TrackedBuffer::I16),
            PythonBufferElement::I32 => acquire_typed(object, request, TrackedBuffer::I32),
            PythonBufferElement::I64 => acquire_typed(object, request, TrackedBuffer::I64),
            PythonBufferElement::ISize => acquire_typed(object, request, TrackedBuffer::ISize),
            PythonBufferElement::U8 => acquire_typed(object, request, TrackedBuffer::U8),
            PythonBufferElement::U16 => acquire_typed(object, request, TrackedBuffer::U16),
            PythonBufferElement::U32 => acquire_typed(object, request, TrackedBuffer::U32),
            PythonBufferElement::U64 => acquire_typed(object, request, TrackedBuffer::U64),
            PythonBufferElement::USize => acquire_typed(object, request, TrackedBuffer::USize),
            PythonBufferElement::F64 => acquire_typed(object, request, TrackedBuffer::F64),
        }
    })
    .map_err(PythonError::runtime)?
}

pub fn buffer_u8(
    object: &ObjectHandle,
    require_writable: bool,
) -> Result<PythonBufferMetadata, PythonError> {
    acquire_buffer(
        object,
        PythonBufferRequest {
            element: PythonBufferElement::U8,
            access: if require_writable {
                PythonBufferAccess::Write
            } else {
                PythonBufferAccess::Read
            },
            layout: PythonBufferLayout::Any,
        },
    )
}

fn acquire_typed<T: Element>(
    object: &pyo3::Bound<'_, pyo3::types::PyAny>,
    request: PythonBufferRequest,
    wrap: fn(PyBuffer<T>) -> TrackedBuffer,
) -> Result<PythonBufferMetadata, PythonError> {
    let py = object.py();
    let buffer = PyBuffer::<T>::get(object).map_err(|error| {
        PythonError::from_pyerr(
            py,
            error,
            "zero-copy",
            format!("Py_buffer<{}>", request.element.source_name()),
        )
    })?;
    validate_request(py, &buffer, request)?;
    let metadata = metadata_for_buffer(&buffer, request.element)?;
    store_buffer(wrap(buffer), metadata)
}

fn validate_request<T: Element>(
    py: Python<'_>,
    buffer: &PyBuffer<T>,
    request: PythonBufferRequest,
) -> Result<(), PythonError> {
    if request.access == PythonBufferAccess::Write && buffer.readonly() {
        return Err(buffer_error(
            py,
            "requested writable buffer from readonly exporter",
            "writable buffer acquisition",
        ));
    }
    let layout_valid = match request.layout {
        PythonBufferLayout::Any => true,
        PythonBufferLayout::CContiguous => buffer.is_c_contiguous(),
        PythonBufferLayout::FContiguous => buffer.is_fortran_contiguous(),
    };
    if !layout_valid {
        return Err(buffer_error(
            py,
            "exporter does not satisfy the declared buffer layout",
            "buffer layout validation",
        ));
    }
    let dimensions = buffer.dimensions();
    if buffer.shape().len() != dimensions || buffer.strides().len() != dimensions {
        return Err(buffer_error(
            py,
            "exporter returned inconsistent shape or stride metadata",
            "buffer metadata validation",
        ));
    }
    if buffer
        .suboffsets()
        .is_some_and(|suboffsets| suboffsets.len() != dimensions)
    {
        return Err(buffer_error(
            py,
            "exporter returned inconsistent suboffset metadata",
            "buffer metadata validation",
        ));
    }
    let logical_items = if dimensions == 0 {
        1
    } else {
        buffer
            .shape()
            .iter()
            .try_fold(1_usize, |count, dimension| {
                count.checked_mul(*dimension).ok_or_else(|| {
                    buffer_error(
                        py,
                        "buffer shape exceeds the supported address space",
                        "buffer metadata validation",
                    )
                })
            })?
    };
    if logical_items != buffer.item_count() {
        return Err(buffer_error(
            py,
            "exporter length does not match its shape and item size",
            "buffer metadata validation",
        ));
    }
    Ok(())
}

fn metadata_for_buffer<T: Element>(
    buffer: &PyBuffer<T>,
    element: PythonBufferElement,
) -> Result<PythonBufferMetadata, PythonError> {
    Ok(PythonBufferMetadata {
        handle: -1,
        token: 0,
        element,
        len_bytes: checked_i64(buffer.len_bytes(), "buffer length")?,
        item_size: checked_i64(buffer.item_size(), "buffer item size")?,
        readonly: buffer.readonly(),
        dimensions: checked_i64(buffer.dimensions(), "buffer dimensions")?,
        shape: checked_i64_vec(buffer.shape(), "buffer shape")?,
        strides: checked_i64_vec(buffer.strides(), "buffer strides")?,
        suboffsets: checked_i64_vec(buffer.suboffsets().unwrap_or(&[]), "buffer suboffsets")?,
        c_contiguous: buffer.is_c_contiguous(),
        f_contiguous: buffer.is_fortran_contiguous(),
        format: buffer.format().to_string_lossy().into_owned(),
    })
}

fn store_buffer(
    buffer: TrackedBuffer,
    mut metadata: PythonBufferMetadata,
) -> Result<PythonBufferMetadata, PythonError> {
    let mut store = buffer_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    super::update_object_count(1).map_err(PythonError::runtime)?;
    metadata.handle = handle;
    metadata.token = token;
    store.buffers.insert(
        handle,
        BufferEntry {
            token,
            buffer: Arc::new(buffer),
            metadata: metadata.clone(),
        },
    );
    Ok(metadata)
}

pub fn copy_buffer_u8(buffer: BufferHandle) -> Result<Vec<u8>, PythonError> {
    let (tracked, _) = buffer_snapshot(buffer)?;
    super::attach(|py| match tracked.as_ref() {
        TrackedBuffer::U8(value) => value.to_vec(py).map_err(|error| {
            PythonError::from_pyerr(py, error, "zero-copy", "copy Py_buffer<uint8>")
        }),
        _ => Err(type_error(py, "buffer element type is not uint8")),
    })
    .map_err(PythonError::runtime)?
}

pub fn buffer_shape(buffer: BufferHandle) -> Result<Vec<i64>, PythonError> {
    buffer_metadata(buffer).map(|metadata| metadata.shape)
}

pub fn buffer_strides(buffer: BufferHandle) -> Result<Vec<i64>, PythonError> {
    buffer_metadata(buffer).map(|metadata| metadata.strides)
}

pub fn buffer_suboffsets(buffer: BufferHandle) -> Result<Vec<i64>, PythonError> {
    buffer_metadata(buffer).map(|metadata| metadata.suboffsets)
}

macro_rules! typed_buffer_accessors {
    ($read:ident, $write:ident, $copy:ident, $variant:ident, $ty:ty) => {
        pub fn $read(buffer: BufferHandle, index: i64) -> Result<$ty, PythonError> {
            let (tracked, _) = buffer_snapshot(buffer)?;
            super::attach(|py| match tracked.as_ref() {
                TrackedBuffer::$variant(value) => read_typed(py, value, index),
                _ => Err(type_error(
                    py,
                    "buffer element type does not match accessor",
                )),
            })
            .map_err(PythonError::runtime)?
        }

        pub fn $write(buffer: BufferHandle, index: i64, value: $ty) -> Result<(), PythonError> {
            let (tracked, _) = buffer_snapshot(buffer)?;
            super::attach(|py| match tracked.as_ref() {
                TrackedBuffer::$variant(target) => write_typed(py, target, index, value),
                _ => Err(type_error(
                    py,
                    "buffer element type does not match accessor",
                )),
            })
            .map_err(PythonError::runtime)?
        }

        pub fn $copy(
            buffer: BufferHandle,
            start: i64,
            length: i64,
        ) -> Result<Vec<$ty>, PythonError> {
            let (tracked, _) = buffer_snapshot(buffer)?;
            super::attach(|py| match tracked.as_ref() {
                TrackedBuffer::$variant(value) => copy_typed_slice(py, value, start, length),
                _ => Err(type_error(
                    py,
                    "buffer element type does not match accessor",
                )),
            })
            .map_err(PythonError::runtime)?
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

fn read_typed<T: Element>(
    py: Python<'_>,
    buffer: &PyBuffer<T>,
    index: i64,
) -> Result<T, PythonError> {
    let index = checked_index(py, index, buffer.item_count())?;
    readable_slice(py, buffer)?
        .get(index)
        .map(pyo3::buffer::ReadOnlyCell::get)
        .ok_or_else(|| index_error(py, "buffer index is out of bounds"))
}

fn write_typed<T: Element>(
    py: Python<'_>,
    buffer: &PyBuffer<T>,
    index: i64,
    value: T,
) -> Result<(), PythonError> {
    let index = checked_index(py, index, buffer.item_count())?;
    let slice = buffer
        .as_mut_slice(py)
        .or_else(|| buffer.as_fortran_mut_slice(py))
        .ok_or_else(|| {
            buffer_error(
                py,
                "writable typed access requires a writable contiguous buffer",
                "buffer element write",
            )
        })?;
    slice
        .get(index)
        .ok_or_else(|| index_error(py, "buffer index is out of bounds"))?
        .set(value);
    Ok(())
}

fn copy_typed_slice<T: Element>(
    py: Python<'_>,
    buffer: &PyBuffer<T>,
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
    Ok(readable_slice(py, buffer)?[start..end]
        .iter()
        .map(pyo3::buffer::ReadOnlyCell::get)
        .collect())
}

fn readable_slice<'a, T: Element>(
    py: Python<'a>,
    buffer: &'a PyBuffer<T>,
) -> Result<&'a [pyo3::buffer::ReadOnlyCell<T>], PythonError> {
    buffer
        .as_slice(py)
        .or_else(|| buffer.as_fortran_slice(py))
        .ok_or_else(|| {
            buffer_error(
                py,
                "typed access requires a contiguous buffer",
                "buffer element access",
            )
        })
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

pub fn release_buffer((handle, token): BufferHandle) -> Result<(), PythonError> {
    let entry = {
        let mut store = buffer_store()?;
        if store
            .buffers
            .get(&handle)
            .is_some_and(|entry| entry.token == token)
        {
            store.buffers.remove(&handle)
        } else {
            return Err(closed_error(handle));
        }
    };
    super::attach(|_py| drop(entry)).map_err(PythonError::runtime)?;
    Ok(())
}

fn buffer_snapshot(
    buffer: BufferHandle,
) -> Result<(Arc<TrackedBuffer>, PythonBufferMetadata), PythonError> {
    let store = buffer_store()?;
    let entry = lookup_buffer(&store, buffer)?;
    Ok((Arc::clone(&entry.buffer), entry.metadata.clone()))
}

fn buffer_metadata(buffer: BufferHandle) -> Result<PythonBufferMetadata, PythonError> {
    buffer_snapshot(buffer).map(|(_, metadata)| metadata)
}

fn lookup_buffer(
    store: &BufferStore,
    (handle, token): BufferHandle,
) -> Result<&BufferEntry, PythonError> {
    store
        .buffers
        .get(&handle)
        .filter(|entry| entry.token == token)
        .ok_or_else(|| closed_error(handle))
}

fn reserve_handle(store: &mut BufferStore) -> Result<BufferHandle, PythonError> {
    store.next_handle = store.next_handle.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python buffer handle space exhausted".to_string(),
        ))
    })?;
    store.next_nonce = store.next_nonce.checked_add(1).ok_or_else(|| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(
            "Python buffer token space exhausted".to_string(),
        ))
    })?;
    Ok((
        store.next_handle,
        token_for(store.next_handle, store.next_nonce),
    ))
}

fn checked_i64(value: usize, context: &'static str) -> Result<i64, PythonError> {
    i64::try_from(value).map_err(|_| {
        PythonError::runtime(PythonRuntimeError::PythonOperationFailed(format!(
            "{context} exceeds Sifr int range"
        )))
    })
}

fn checked_i64_vec<T>(values: &[T], context: &'static str) -> Result<Vec<i64>, PythonError>
where
    T: Copy + TryInto<i64>,
{
    values
        .iter()
        .copied()
        .map(|value| {
            value.try_into().map_err(|_| {
                PythonError::runtime(PythonRuntimeError::PythonOperationFailed(format!(
                    "{context} entry exceeds Sifr int range"
                )))
            })
        })
        .collect()
}

fn buffer_error(py: Python<'_>, message: &str, context: &str) -> PythonError {
    PythonError::from_pyerr(
        py,
        PyBufferError::new_err(message.to_string()),
        "zero-copy",
        context,
    )
}

fn type_error(py: Python<'_>, message: &str) -> PythonError {
    PythonError::from_pyerr(
        py,
        PyTypeError::new_err(message.to_string()),
        "zero-copy",
        "buffer typed access",
    )
}

fn index_error(py: Python<'_>, message: &str) -> PythonError {
    PythonError::from_pyerr(
        py,
        PyIndexError::new_err(message.to_string()),
        "zero-copy",
        "buffer bounds validation",
    )
}

fn closed_error(handle: i64) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonClosedBuffer".to_string(),
        message: format!("Python buffer handle {handle} is closed"),
        traceback: String::new(),
        context: "buffer handle lookup".to_string(),
        replay: None,
    }
}

fn token_for(handle: i64, nonce: u64) -> i64 {
    let hash = TOKEN_HASHER.hash_one((handle, nonce));
    i64::from_ne_bytes(hash.to_ne_bytes())
}

fn buffer_store() -> Result<MutexGuard<'static, BufferStore>, PythonError> {
    BUFFER_STORE.lock().map_err(|_| PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python buffer store is unavailable".to_string(),
        traceback: String::new(),
        context: "buffer store".to_string(),
        replay: None,
    })
}

#[cfg(test)]
mod tests;
