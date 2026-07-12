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
    boundary_path: String,
}

#[derive(Debug)]
struct ForeignObjectInner {
    state: Mutex<ForeignObjectState>,
}

#[derive(Debug)]
enum ForeignObjectState {
    Open(Py<PyAny>),
    Poisoned(Py<PyAny>),
    Closed,
}

impl ForeignObject {
    pub(super) fn new(object: Py<PyAny>) -> Result<Self, PythonRuntimeError> {
        update_object_count(1)?;
        Ok(Self {
            inner: Arc::new(ForeignObjectInner {
                state: Mutex::new(ForeignObjectState::Open(object)),
            }),
            boundary_path: String::new(),
        })
    }

    pub(super) fn clone_ref(&self, py: Python<'_>) -> Result<Py<PyAny>, PythonRuntimeError> {
        match &*lock_state(&self.inner.state) {
            ForeignObjectState::Open(object) => Ok(object.clone_ref(py)),
            ForeignObjectState::Poisoned(_) => Err(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is poisoned after failed semantic cleanup".to_string(),
            )),
            ForeignObjectState::Closed => Err(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is closed".to_string(),
            )),
        }
    }

    pub(super) fn poison(&self) {
        let mut state = lock_state(&self.inner.state);
        if let ForeignObjectState::Open(object) =
            std::mem::replace(&mut *state, ForeignObjectState::Closed)
        {
            *state = ForeignObjectState::Poisoned(object);
        }
    }

    pub(super) fn close(&self) {
        let object = {
            let mut state = lock_state(&self.inner.state);
            match std::mem::replace(&mut *state, ForeignObjectState::Closed) {
                ForeignObjectState::Open(object) | ForeignObjectState::Poisoned(object) => {
                    Some(object)
                }
                ForeignObjectState::Closed => None,
            }
        };
        if let Some(object) = object {
            release_object(object);
        }
    }

    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub(super) fn with_child_path(mut self, child: impl AsRef<str>) -> Self {
        self.boundary_path.push_str(child.as_ref());
        self
    }

    pub(super) fn boundary_path(&self) -> &str {
        &self.boundary_path
    }
}

impl Drop for ForeignObjectInner {
    fn drop(&mut self) {
        let state = match self.state.get_mut() {
            Ok(state) => std::mem::replace(state, ForeignObjectState::Closed),
            Err(poisoned) => std::mem::replace(poisoned.into_inner(), ForeignObjectState::Closed),
        };
        if let ForeignObjectState::Open(object) | ForeignObjectState::Poisoned(object) = state {
            release_object(object);
        }
    }
}

fn release_object(object: Py<PyAny>) {
    if unsafe { ffi::PyGILState_Check() } != 0 {
        drop(object);
        let _ignored = update_object_count(-1);
    } else {
        pending_releases().push(object);
    }
}

fn lock_state(state: &Mutex<ForeignObjectState>) -> std::sync::MutexGuard<'_, ForeignObjectState> {
    match state.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
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
