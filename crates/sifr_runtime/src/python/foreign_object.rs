use super::{update_object_count, PythonRuntimeError};
use pyo3::{ffi, prelude::*, Py, PyAny};
use std::sync::{Arc, LazyLock, Mutex};

static PENDING_RELEASES: LazyLock<Mutex<Vec<Py<PyAny>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Compiler-owned Python identity payload.
///
/// Public Sifr values never expose this payload or a structural token. Generated
/// glue carries it inside the sealed interop handle representation.
#[derive(Clone, Debug)]
pub struct ForeignObject {
    inner: Arc<ForeignObjectInner>,
}

#[derive(Debug)]
struct ForeignObjectInner {
    object: Option<Py<PyAny>>,
}

impl ForeignObject {
    pub(super) fn new(object: Py<PyAny>) -> Result<Self, PythonRuntimeError> {
        update_object_count(1)?;
        Ok(Self {
            inner: Arc::new(ForeignObjectInner {
                object: Some(object),
            }),
        })
    }

    pub(super) fn clone_ref(&self, py: Python<'_>) -> Result<Py<PyAny>, PythonRuntimeError> {
        self.inner
            .object
            .as_ref()
            .map(|object| object.clone_ref(py))
            .ok_or(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is closed".to_string(),
            ))
    }

    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Drop for ForeignObjectInner {
    fn drop(&mut self) {
        let Some(object) = self.object.take() else {
            return;
        };
        if unsafe { ffi::PyGILState_Check() } != 0 {
            drop(object);
            let _ignored = update_object_count(-1);
            return;
        }
        pending_releases().push(object);
    }
}

pub(super) fn drain_pending_releases(_py: Python<'_>) {
    let pending = {
        let mut queue = pending_releases();
        std::mem::take(&mut *queue)
    };
    let released = pending.len();
    drop(pending);
    if released > 0 {
        let released = isize::try_from(released).unwrap_or(isize::MAX);
        let _ignored = update_object_count(-released);
    }
}

#[cfg(test)]
pub(super) fn pending_release_count() -> usize {
    pending_releases().len()
}

#[cfg(test)]
pub(super) fn reset_pending_releases_for_tests() {
    let pending = {
        let mut queue = pending_releases();
        std::mem::take(&mut *queue)
    };
    std::mem::forget(pending);
}

fn pending_releases() -> std::sync::MutexGuard<'static, Vec<Py<PyAny>>> {
    match PENDING_RELEASES.lock() {
        Ok(queue) => queue,
        Err(poisoned) => poisoned.into_inner(),
    }
}
