use super::object_ops::clone_handle;
use super::{ObjectHandle, PythonError, PythonRuntimeError};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyBufferError;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::{LazyLock, Mutex, MutexGuard};

static BUFFER_STORE: LazyLock<Mutex<BufferStore>> =
    LazyLock::new(|| Mutex::new(BufferStore::default()));
static TOKEN_HASHER: LazyLock<std::collections::hash_map::RandomState> =
    LazyLock::new(std::collections::hash_map::RandomState::new);

pub type BufferHandle = (i64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonBufferMetadata {
    pub handle: i64,
    pub token: i64,
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
    buffer: TrackedBuffer,
}

struct TrackedBuffer {
    buffer: Option<PyBuffer<u8>>,
}

impl Drop for TrackedBuffer {
    fn drop(&mut self) {
        let Some(buffer) = self.buffer.take() else {
            return;
        };
        drop(buffer);
        let _ignored = super::update_object_count(-1);
    }
}

pub fn buffer_u8(
    object: ObjectHandle,
    require_writable: bool,
) -> Result<PythonBufferMetadata, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let buffer = PyBuffer::<u8>::get(object.bind(py))
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "Py_buffer<u8>"))?;
        if require_writable && buffer.readonly() {
            return Err(PythonError::from_pyerr(
                py,
                PyBufferError::new_err("requested writable buffer from readonly exporter"),
                "zero-copy",
                "writable Py_buffer<u8>",
            ));
        }
        let metadata = metadata_for_buffer(&buffer)?;
        store_buffer(buffer, metadata)
    })
    .map_err(PythonError::runtime)?
}

pub fn copy_buffer_u8(buffer: BufferHandle) -> Result<Vec<u8>, PythonError> {
    super::attach(|py| {
        let store = buffer_store()?;
        let entry = lookup_buffer(&store, buffer)?;
        entry
            .buffer
            .buffer
            .as_ref()
            .ok_or_else(|| closed_error(buffer.0))?
            .to_vec(py)
            .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "copy Py_buffer<u8>"))
    })
    .map_err(PythonError::runtime)?
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
    drop(entry);
    Ok(())
}

fn metadata_for_buffer(buffer: &PyBuffer<u8>) -> Result<PythonBufferMetadata, PythonError> {
    Ok(PythonBufferMetadata {
        handle: -1,
        token: 0,
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
    buffer: PyBuffer<u8>,
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
            buffer: TrackedBuffer {
                buffer: Some(buffer),
            },
        },
    );
    Ok(metadata)
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

fn closed_error(handle: i64) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonClosedBuffer".to_string(),
        message: format!("Python buffer handle {handle} is closed"),
        traceback: String::new(),
        context: "buffer handle lookup".to_string(),
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        close_object, from_bytes, from_int, initialize_runtime, reset_runtime_state_for_tests,
        resource_diagnostics, test_config, test_guard, PythonResourceDiagnostics,
    };

    #[test]
    fn buffer_view_tracks_metadata_copy_and_release() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("buffer-view")).expect("init should succeed");

        let object = from_bytes(&[1, 2, 3]).expect("bytes should be stored");
        let view = buffer_u8(object, false).expect("bytes should expose u8 buffer");

        assert_eq!(view.len_bytes, 3);
        assert_eq!(view.item_size, 1);
        assert!(view.readonly);
        assert!(view.c_contiguous);
        assert_eq!(
            copy_buffer_u8((view.handle, view.token)).expect("buffer should copy"),
            vec![1, 2, 3]
        );
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 2,
                leaked_objects: 0,
            }
        );
        release_buffer((view.handle, view.token)).expect("buffer should release");
        close_object(object).expect("object should close");
        assert_eq!(
            resource_diagnostics().expect("diagnostics should be available"),
            PythonResourceDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn buffer_double_release_is_deterministic_resource_error() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("buffer-double-release")).expect("init should succeed");

        let object = from_bytes(&[1]).expect("bytes should be stored");
        let view = buffer_u8(object, false).expect("bytes should expose u8 buffer");
        release_buffer((view.handle, view.token)).expect("buffer should release");
        let error = release_buffer((view.handle, view.token)).expect_err("second release fails");
        let copy_error =
            copy_buffer_u8((view.handle, view.token)).expect_err("copy after release fails");

        assert_eq!(error.kind, "resource");
        assert_eq!(error.exception_type, "SifrPythonClosedBuffer");
        assert_eq!(copy_error.kind, "resource");
        assert_eq!(copy_error.exception_type, "SifrPythonClosedBuffer");
        close_object(object).expect("object should close");
    }

    #[test]
    fn buffer_rejects_wrong_dtype_and_readonly_writable_request() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("buffer-rejects")).expect("init should succeed");

        let object = from_bytes(&[1]).expect("bytes should be stored");
        let writable = buffer_u8(object, true).expect_err("bytes are readonly");
        assert_eq!(writable.kind, "zero-copy");

        let integer = from_int(1).expect("int should be stored");
        let unsupported = buffer_u8(integer, false).expect_err("int has no u8 buffer");
        assert_eq!(unsupported.kind, "zero-copy");

        close_object(object).expect("object should close");
        close_object(integer).expect("integer should close");
    }
}
