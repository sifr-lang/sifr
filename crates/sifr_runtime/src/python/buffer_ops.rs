use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::exceptions::{PyBufferError, PyIndexError, PyTypeError};
use pyo3::Python;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

mod access;
mod raw;
pub use access::*;
use raw::{BufferFootprint, OwnedPyBuffer, ValidatedBuffer};

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
    admissions: HashMap<i64, BufferAdmission>,
}

struct BufferEntry {
    token: i64,
    buffer: Arc<TrackedBuffer>,
    metadata: PythonBufferMetadata,
}

struct BufferAdmission {
    footprint: BufferFootprint,
    access: PythonBufferAccess,
}

struct TrackedBuffer {
    element: PythonBufferElement,
    access: PythonBufferAccess,
    buffer: Mutex<Option<OwnedPyBuffer>>,
}

impl TrackedBuffer {
    fn new(buffer: OwnedPyBuffer, request: PythonBufferRequest) -> Self {
        Self {
            element: request.element,
            access: request.access,
            buffer: Mutex::new(Some(buffer)),
        }
    }

    fn take_for_release(&self) -> Result<OwnedPyBuffer, PythonError> {
        self.buffer
            .lock()
            .map_err(|_| buffer_state_error())?
            .take()
            .ok_or_else(|| closed_error(-1))
    }
}

impl Drop for TrackedBuffer {
    fn drop(&mut self) {
        let state = match self.buffer.get_mut() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(buffer) = state.take() {
            drop(buffer);
            let _ignored = super::update_object_count(-1);
        }
    }
}

pub fn acquire_buffer(
    object: &ObjectHandle,
    request: PythonBufferRequest,
) -> Result<PythonBufferMetadata, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let object = object.bind(py);
        let buffer =
            OwnedPyBuffer::acquire(py, object, request.access == PythonBufferAccess::Write)
                .map_err(|error| {
                    PythonError::from_pyerr(
                        py,
                        error,
                        "zero-copy",
                        format!("Py_buffer<{}>", request.element.source_name()),
                    )
                })?;
        let validated = buffer
            .validate(request.element)
            .map_err(|message| buffer_error(py, &message, "buffer metadata validation"))?;
        validate_request(py, &validated, request)?;
        let footprint = buffer
            .footprint(&validated)
            .map_err(|message| buffer_error(py, &message, "buffer access admission"))?;
        let metadata = metadata_for_buffer(&validated, request.element)?;
        store_buffer(buffer, request, metadata, footprint)
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

fn validate_request(
    py: Python<'_>,
    buffer: &ValidatedBuffer,
    request: PythonBufferRequest,
) -> Result<(), PythonError> {
    if request.access == PythonBufferAccess::Write && buffer.readonly {
        return Err(buffer_error(
            py,
            "requested writable buffer from readonly exporter",
            "writable buffer acquisition",
        ));
    }
    let layout_valid = match request.layout {
        PythonBufferLayout::Any => true,
        PythonBufferLayout::CContiguous => buffer.c_contiguous,
        PythonBufferLayout::FContiguous => buffer.f_contiguous,
    };
    if !layout_valid {
        return Err(buffer_error(
            py,
            "exporter does not satisfy the declared buffer layout",
            "buffer layout validation",
        ));
    }
    Ok(())
}

fn metadata_for_buffer(
    buffer: &ValidatedBuffer,
    element: PythonBufferElement,
) -> Result<PythonBufferMetadata, PythonError> {
    Ok(PythonBufferMetadata {
        handle: -1,
        token: 0,
        element,
        len_bytes: checked_i64(buffer.len_bytes, "buffer length")?,
        item_size: checked_i64(buffer.item_size, "buffer item size")?,
        readonly: buffer.readonly,
        dimensions: checked_i64(buffer.dimensions, "buffer dimensions")?,
        shape: checked_i64_vec(&buffer.shape, "buffer shape")?,
        strides: checked_i64_vec(&buffer.strides, "buffer strides")?,
        suboffsets: checked_i64_vec(&buffer.suboffsets, "buffer suboffsets")?,
        c_contiguous: buffer.c_contiguous,
        f_contiguous: buffer.f_contiguous,
        format: buffer.format.clone(),
    })
}

fn store_buffer(
    buffer: OwnedPyBuffer,
    request: PythonBufferRequest,
    mut metadata: PythonBufferMetadata,
    footprint: BufferFootprint,
) -> Result<PythonBufferMetadata, PythonError> {
    let mut store = buffer_store()?;
    let (handle, token) = reserve_handle(&mut store)?;
    admit_buffer(&mut store, handle, footprint, request.access)?;
    if let Err(error) = super::update_object_count(1) {
        let _ignored = release_buffer_admission(&mut store, handle);
        return Err(PythonError::runtime(error));
    }
    metadata.handle = handle;
    metadata.token = token;
    store.buffers.insert(
        handle,
        BufferEntry {
            token,
            buffer: Arc::new(TrackedBuffer::new(buffer, request)),
            metadata: metadata.clone(),
        },
    );
    Ok(metadata)
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
    }
    .ok_or_else(|| closed_error(handle))?;
    let release_result = match entry.buffer.take_for_release() {
        Ok(buffer) => {
            let python_result =
                super::attach(|py| buffer.release(py)).map_err(PythonError::runtime);
            let count_result = super::update_object_count(-1).map_err(PythonError::runtime);
            python_result.and(count_result)
        }
        Err(error) => Err(error),
    };
    let admission_result = {
        let mut store = buffer_store()?;
        release_buffer_admission(&mut store, handle)
    };
    release_result.and(admission_result)
}

fn admit_buffer(
    store: &mut BufferStore,
    handle: i64,
    footprint: BufferFootprint,
    access: PythonBufferAccess,
) -> Result<(), PythonError> {
    let conflicts = store.admissions.values().any(|admission| {
        access != PythonBufferAccess::Read || admission.access != PythonBufferAccess::Read
    } && footprints_overlap(&footprint, &admission.footprint));
    if conflicts {
        return Err(exporter_admission_error());
    }
    if store
        .admissions
        .insert(handle, BufferAdmission { footprint, access })
        .is_some()
    {
        return Err(buffer_state_error());
    }
    Ok(())
}

fn release_buffer_admission(store: &mut BufferStore, handle: i64) -> Result<(), PythonError> {
    store
        .admissions
        .remove(&handle)
        .map(|_| ())
        .ok_or_else(buffer_state_error)
}

fn footprints_overlap(left: &BufferFootprint, right: &BufferFootprint) -> bool {
    match (left, right) {
        (BufferFootprint::Empty, _) | (_, BufferFootprint::Empty) => false,
        (
            BufferFootprint::Direct {
                ranges: left_ranges,
            },
            BufferFootprint::Direct {
                ranges: right_ranges,
            },
        ) => sorted_ranges_overlap(left_ranges, right_ranges),
    }
}

fn sorted_ranges_overlap(
    left: &[std::ops::Range<usize>],
    right: &[std::ops::Range<usize>],
) -> bool {
    let (mut left_index, mut right_index) = (0, 0);
    while let (Some(left_range), Some(right_range)) = (left.get(left_index), right.get(right_index))
    {
        if left_range.start < right_range.end && right_range.start < left_range.end {
            return true;
        }
        if left_range.end <= right_range.start {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    false
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BufferAccessError {
    State,
    Closed(i64),
    Type(&'static str),
    Buffer {
        message: &'static str,
        context: &'static str,
    },
    Index(&'static str),
}

impl BufferAccessError {
    fn into_python(self, py: Python<'_>) -> PythonError {
        match self {
            Self::State => buffer_state_error(),
            Self::Closed(handle) => closed_error(handle),
            Self::Type(message) => type_error(py, message),
            Self::Buffer { message, context } => buffer_error(py, message, context),
            Self::Index(message) => index_error(py, message),
        }
    }
}

fn with_live_buffer<R>(
    buffer: BufferHandle,
    expected: PythonBufferElement,
    require_write: bool,
    operation: impl FnOnce(&OwnedPyBuffer) -> Result<R, BufferAccessError>,
) -> Result<R, PythonError> {
    with_live_buffer_and_converter(buffer, expected, require_write, operation, |error| {
        super::attach(|py| error.into_python(py))
    })
}

fn with_live_buffer_and_converter<R>(
    buffer: BufferHandle,
    expected: PythonBufferElement,
    require_write: bool,
    operation: impl FnOnce(&OwnedPyBuffer) -> Result<R, BufferAccessError>,
    convert_error: impl FnOnce(BufferAccessError) -> Result<PythonError, PythonRuntimeError>,
) -> Result<R, PythonError> {
    let (tracked, _) = buffer_snapshot(buffer)?;
    let outcome = super::attach(|_py| {
        let state = tracked
            .buffer
            .lock()
            .map_err(|_| BufferAccessError::State)?;
        let value = state.as_ref().ok_or(BufferAccessError::Closed(buffer.0))?;
        if tracked.element != expected {
            return Err(BufferAccessError::Type(
                "buffer element type does not match accessor",
            ));
        }
        if require_write && tracked.access != PythonBufferAccess::Write {
            return Err(BufferAccessError::Buffer {
                message: "buffer was acquired with read-only declaration access",
                context: "buffer element write",
            });
        }
        operation(value)
    })
    .map_err(PythonError::runtime)?;
    match outcome {
        Ok(value) => Ok(value),
        Err(error) => {
            let error = convert_error(error).map_err(PythonError::runtime)?;
            Err(error)
        }
    }
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

fn buffer_state_error() -> PythonError {
    PythonError {
        kind: "runtime".to_string(),
        exception_type: "SifrPythonRuntimeError".to_string(),
        message: "Python buffer state is unavailable".to_string(),
        traceback: String::new(),
        context: "buffer state".to_string(),
        replay: None,
    }
}

fn exporter_admission_error() -> PythonError {
    PythonError {
        kind: "zero-copy".to_string(),
        exception_type: "BufferError".to_string(),
        message: "buffer exporter already has an active conflicting view".to_string(),
        traceback: String::new(),
        context: "buffer exporter access admission".to_string(),
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
mod release_evidence_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typed_access_evidence_tests;
