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

/// Runtime-private pin for an identity used by an in-flight async declaration.
///
/// The pin is deliberately not exposed through the Sifr type model. It keeps the
/// exact identity resolvable on the owned asyncio thread even if runtime cleanup
/// concurrently marks the public handle closed.
#[derive(Debug)]
pub(super) struct ForeignObjectLease {
    inner: Arc<ForeignObjectInner>,
    boundary_path: String,
}

#[derive(Debug)]
struct ForeignObjectInner {
    state: Mutex<ForeignObjectState>,
}

#[derive(Debug)]
struct ForeignObjectState {
    status: ForeignObjectStatus,
    object: Option<Py<PyAny>>,
    active_leases: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForeignObjectStatus {
    Open,
    Closing,
    Poisoned,
    Closed,
}

impl ForeignObject {
    pub(super) fn new(object: Py<PyAny>) -> Result<Self, PythonRuntimeError> {
        update_object_count(1)?;
        Ok(Self {
            inner: Arc::new(ForeignObjectInner {
                state: Mutex::new(ForeignObjectState {
                    status: ForeignObjectStatus::Open,
                    object: Some(object),
                    active_leases: 0,
                }),
            }),
            boundary_path: String::new(),
        })
    }

    pub(super) fn clone_ref(&self, py: Python<'_>) -> Result<Py<PyAny>, PythonRuntimeError> {
        let state = lock_state(&self.inner.state);
        match state.status {
            ForeignObjectStatus::Open => state.object.as_ref().map_or_else(
                || {
                    Err(PythonRuntimeError::PythonOperationFailed(
                        "Python object identity is unavailable".to_string(),
                    ))
                },
                |object| Ok(object.clone_ref(py)),
            ),
            ForeignObjectStatus::Closing => Err(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is closing".to_string(),
            )),
            ForeignObjectStatus::Poisoned => Err(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is poisoned after failed semantic cleanup".to_string(),
            )),
            ForeignObjectStatus::Closed => Err(PythonRuntimeError::PythonOperationFailed(
                "Python object identity is closed".to_string(),
            )),
        }
    }

    pub(super) fn lease(&self) -> Result<ForeignObjectLease, PythonRuntimeError> {
        let mut state = lock_state(&self.inner.state);
        if state.status != ForeignObjectStatus::Open || state.object.is_none() {
            return Err(unavailable_for_status(state.status));
        }
        state.active_leases = state.active_leases.saturating_add(1);
        Ok(ForeignObjectLease {
            inner: Arc::clone(&self.inner),
            boundary_path: self.boundary_path.clone(),
        })
    }

    pub(super) fn begin_semantic_close(&self) -> Result<ForeignObjectLease, PythonRuntimeError> {
        let mut state = lock_state(&self.inner.state);
        if state.status != ForeignObjectStatus::Open || state.object.is_none() {
            return Err(unavailable_for_status(state.status));
        }
        state.status = ForeignObjectStatus::Closing;
        state.active_leases = state.active_leases.saturating_add(1);
        Ok(ForeignObjectLease {
            inner: Arc::clone(&self.inner),
            boundary_path: self.boundary_path.clone(),
        })
    }

    pub(super) fn finish_semantic_close(&self, succeeded: bool) {
        let object = {
            let mut state = lock_state(&self.inner.state);
            if state.status != ForeignObjectStatus::Closing {
                return;
            }
            state.status = if succeeded {
                ForeignObjectStatus::Closed
            } else {
                ForeignObjectStatus::Poisoned
            };
            if succeeded && state.active_leases == 0 {
                state.object.take()
            } else {
                None
            }
        };
        if let Some(object) = object {
            release_object(object);
        }
    }

    pub(super) fn poison(&self) {
        let mut state = lock_state(&self.inner.state);
        if state.status == ForeignObjectStatus::Open {
            state.status = ForeignObjectStatus::Poisoned;
        }
    }

    pub(super) fn close(&self) {
        let object = {
            let mut state = lock_state(&self.inner.state);
            state.status = ForeignObjectStatus::Closed;
            if state.active_leases == 0 {
                state.object.take()
            } else {
                None
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
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(object) = state.object.take() {
            release_object(object);
        }
    }
}

impl ForeignObjectLease {
    pub(super) fn clone_ref(&self, py: Python<'_>) -> Result<Py<PyAny>, PythonRuntimeError> {
        let state = lock_state(&self.inner.state);
        state.object.as_ref().map_or_else(
            || {
                Err(PythonRuntimeError::PythonOperationFailed(
                    "leased Python object identity is unavailable".to_string(),
                ))
            },
            |object| Ok(object.clone_ref(py)),
        )
    }

    pub(super) fn boundary_path(&self) -> &str {
        &self.boundary_path
    }
}

impl Drop for ForeignObjectLease {
    fn drop(&mut self) {
        let object = {
            let mut state = lock_state(&self.inner.state);
            state.active_leases = state.active_leases.saturating_sub(1);
            if state.active_leases == 0 && state.status == ForeignObjectStatus::Closed {
                state.object.take()
            } else {
                None
            }
        };
        if let Some(object) = object {
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

fn unavailable_for_status(status: ForeignObjectStatus) -> PythonRuntimeError {
    let state = match status {
        ForeignObjectStatus::Open => "unavailable",
        ForeignObjectStatus::Closing => "closing",
        ForeignObjectStatus::Poisoned => "poisoned after failed semantic cleanup",
        ForeignObjectStatus::Closed => "closed",
    };
    PythonRuntimeError::PythonOperationFailed(format!("Python object identity is {state}"))
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
