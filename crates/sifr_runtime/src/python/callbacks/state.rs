use super::super::PythonError;
use super::{errors, registry};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

type UnregisterAction = Arc<dyn Fn() -> Result<(), PythonError> + Send + Sync + 'static>;
type CaptureReleaseAction = Box<dyn FnOnce() + Send + 'static>;

static NEXT_OWNER_ID: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static ACTIVE_CALLBACKS: RefCell<Vec<(u64, u64)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackOwnerStatus {
    Open,
    Closing,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallbackFailureEvidence {
    pub entry_sequence: u64,
    pub exception_type: String,
    pub message: String,
}

#[derive(Clone)]
pub struct CallbackOwnerState {
    pub(super) inner: Arc<CallbackOwnerInner>,
}

pub(super) struct CallbackOwnerInner {
    id: u64,
    state: Mutex<OwnerData>,
    changed: Condvar,
    unregister: Option<UnregisterAction>,
    releases: Mutex<Vec<CaptureReleaseAction>>,
    retained: bool,
}

struct OwnerData {
    status: CallbackOwnerStatus,
    active_calls: usize,
    next_sequence: u64,
    first_failure: Option<CallbackFailureEvidence>,
    first_failure_observed: bool,
    captures_released: bool,
    unregister_status: CallbackUnregisterStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallbackUnregisterStatus {
    NotStarted,
    Running,
    Finished,
}

pub struct CallbackInvocationLease {
    owner: CallbackOwnerState,
    callback_id: u64,
    entry_sequence: u64,
    active: bool,
}

pub struct CallbackInvocationGuard {
    owner_id: u64,
    callback_id: u64,
    lease: Option<CallbackInvocationLease>,
    active: bool,
}

pub struct CallbackOwnerUnregisterGuard {
    owner: CallbackOwnerState,
    active: bool,
}

impl CallbackOwnerState {
    pub fn new_call_scoped() -> Result<Self, PythonError> {
        Self::new(false, None, None)
    }

    pub fn new_call_scoped_with_release(
        release: impl FnOnce() + Send + 'static,
    ) -> Result<Self, PythonError> {
        Self::new(false, None, Some(Box::new(release)))
    }

    pub fn new_retained(
        unregister: impl Fn() -> Result<(), PythonError> + Send + Sync + 'static,
    ) -> Result<Self, PythonError> {
        Self::new(true, Some(Arc::new(unregister)), None)
    }

    pub fn new_retained_with_release(
        unregister: impl Fn() -> Result<(), PythonError> + Send + Sync + 'static,
        release: impl FnOnce() + Send + 'static,
    ) -> Result<Self, PythonError> {
        Self::new(true, Some(Arc::new(unregister)), Some(Box::new(release)))
    }

    fn new(
        retained: bool,
        unregister: Option<UnregisterAction>,
        release: Option<CaptureReleaseAction>,
    ) -> Result<Self, PythonError> {
        let id = NEXT_OWNER_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| errors::unavailable("owner identity space"))?
            .checked_add(1)
            .ok_or_else(|| errors::unavailable("owner identity space"))?;
        let inner = Arc::new(CallbackOwnerInner {
            id,
            state: Mutex::new(OwnerData {
                status: CallbackOwnerStatus::Open,
                active_calls: 0,
                next_sequence: 0,
                first_failure: None,
                first_failure_observed: false,
                captures_released: false,
                unregister_status: CallbackUnregisterStatus::NotStarted,
            }),
            changed: Condvar::new(),
            unregister,
            releases: Mutex::new(release.into_iter().collect()),
            retained,
        });
        if retained {
            registry::register(id, &inner);
        }
        Ok(Self { inner })
    }

    pub(super) fn from_inner(inner: Arc<CallbackOwnerInner>) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn owner_id(&self) -> u64 {
        self.inner.id
    }

    #[must_use]
    pub fn status(&self) -> CallbackOwnerStatus {
        lock_state(&self.inner).status
    }

    #[must_use]
    pub fn active_calls(&self) -> usize {
        lock_state(&self.inner).active_calls
    }

    #[must_use]
    pub fn captures_released(&self) -> bool {
        lock_state(&self.inner).captures_released
    }

    pub fn accept(
        &self,
        callback_id: u64,
        serial: bool,
    ) -> Result<CallbackInvocationLease, PythonError> {
        if serial && callback_is_active(self.inner.id, callback_id) {
            return Err(errors::reentrant(self.inner.id, callback_id));
        }
        let entry_sequence = {
            let mut state = lock_state(&self.inner);
            if state.status != CallbackOwnerStatus::Open {
                return Err(errors::closed(self.inner.id));
            }
            state.next_sequence = state
                .next_sequence
                .checked_add(1)
                .ok_or_else(|| errors::unavailable("entry sequence"))?;
            state.active_calls = state
                .active_calls
                .checked_add(1)
                .ok_or_else(|| errors::unavailable("active-call count"))?;
            state.next_sequence
        };
        Ok(CallbackInvocationLease {
            owner: self.clone(),
            callback_id,
            entry_sequence,
            active: true,
        })
    }

    pub fn reject_serial_reentrancy(&self, callback_id: u64) -> Result<(), PythonError> {
        if callback_is_active(self.inner.id, callback_id) {
            return Err(errors::reentrant(self.inner.id, callback_id));
        }
        Ok(())
    }

    pub fn close_call_scope(&self) -> Result<(), PythonError> {
        if self.inner.retained {
            return Err(errors::unavailable(
                "retained owner close without unregister authority",
            ));
        }
        self.close_after_unregister(false, true)
    }

    pub fn close_after_owner_unregister(&self) -> Result<(), PythonError> {
        if self.inner.unregister.is_some() {
            let state = lock_state(&self.inner);
            if state.unregister_status == CallbackUnregisterStatus::NotStarted {
                return Err(errors::unavailable("owner unregister authority"));
            }
        }
        self.close_after_unregister(false, true)
    }

    pub fn close_after_owner_unregister_with_typed_observer(&self) -> Result<(), PythonError> {
        if self.inner.unregister.is_some() {
            let state = lock_state(&self.inner);
            if state.unregister_status == CallbackUnregisterStatus::NotStarted {
                return Err(errors::unavailable("owner unregister authority"));
            }
        }
        self.close_after_unregister(false, false)
    }

    pub fn begin_owner_unregister(
        &self,
    ) -> Result<Option<CallbackOwnerUnregisterGuard>, PythonError> {
        if owner_is_active(self.inner.id) {
            return Err(errors::close_from_invocation(self.inner.id));
        }
        let mut state = lock_state(&self.inner);
        if self.inner.unregister.is_none()
            || state.status == CallbackOwnerStatus::Closed
            || state.unregister_status != CallbackUnregisterStatus::NotStarted
        {
            return Ok(None);
        }
        state.unregister_status = CallbackUnregisterStatus::Running;
        Ok(Some(CallbackOwnerUnregisterGuard {
            owner: self.clone(),
            active: true,
        }))
    }

    pub fn record_failure(
        &self,
        entry_sequence: u64,
        exception_type: impl Into<String>,
        message: impl Into<String>,
    ) {
        let mut state = lock_state(&self.inner);
        if state
            .first_failure
            .as_ref()
            .is_none_or(|current| entry_sequence < current.entry_sequence)
        {
            state.first_failure = Some(CallbackFailureEvidence {
                entry_sequence,
                exception_type: exception_type.into(),
                message: message.into(),
            });
            state.first_failure_observed = false;
        }
    }

    pub fn observe_failure(&self, entry_sequence: u64) {
        let mut state = lock_state(&self.inner);
        if state
            .first_failure
            .as_ref()
            .map(|failure| failure.entry_sequence)
            == Some(entry_sequence)
        {
            state.first_failure_observed = true;
        }
    }

    pub fn retain_capture(
        &self,
        release: impl FnOnce() + Send + 'static,
    ) -> Result<(), PythonError> {
        let state = lock_state(&self.inner);
        if state.status != CallbackOwnerStatus::Open {
            return Err(errors::closed(self.inner.id));
        }
        self.inner
            .releases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(Box::new(release));
        Ok(())
    }

    #[must_use]
    pub fn first_failure(&self) -> Option<CallbackFailureEvidence> {
        lock_state(&self.inner).first_failure.clone()
    }

    pub(super) fn shutdown_from_runtime(&self) -> Result<(), PythonError> {
        let mut unregister_error = None;
        if let Some(unregister) = &self.inner.unregister {
            if let Some(unregister_guard) = self.begin_owner_unregister()? {
                if let Err(error) = unregister() {
                    unregister_error = Some(error);
                }
                drop(unregister_guard);
            }
        }
        let close_error = self.close_after_unregister(true, true).err();
        unregister_error.or(close_error).map_or(Ok(()), Err)
    }

    fn close_after_unregister(
        &self,
        runtime_shutdown: bool,
        surface_retained_failure: bool,
    ) -> Result<(), PythonError> {
        if !runtime_shutdown && owner_is_active(self.inner.id) {
            return Err(errors::close_from_invocation(self.inner.id));
        }
        let mut state = lock_state(&self.inner);
        while state.unregister_status == CallbackUnregisterStatus::Running {
            state = self
                .inner
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        match state.status {
            CallbackOwnerStatus::Closed => {
                drop(state);
                return self.retained_failure_result(surface_retained_failure);
            }
            CallbackOwnerStatus::Closing => {
                while state.status != CallbackOwnerStatus::Closed {
                    state = self
                        .inner
                        .changed
                        .wait(state)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                }
                drop(state);
                return self.retained_failure_result(surface_retained_failure);
            }
            CallbackOwnerStatus::Open => {
                state.status = CallbackOwnerStatus::Closing;
            }
        }
        while state.active_calls > 0 {
            state = self
                .inner
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        drop(state);
        let releases = std::mem::take(
            &mut *self
                .inner
                .releases
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for release in releases {
            release();
        }
        let mut state = lock_state(&self.inner);
        state.captures_released = true;
        state.status = CallbackOwnerStatus::Closed;
        self.inner.changed.notify_all();
        drop(state);
        if self.inner.retained {
            registry::unregister(self.inner.id);
        }
        self.retained_failure_result(surface_retained_failure)
    }

    fn retained_failure_result(&self, surface: bool) -> Result<(), PythonError> {
        if self.inner.retained && surface {
            let state = lock_state(&self.inner);
            if !state.first_failure_observed {
                if let Some(evidence) = &state.first_failure {
                    return Err(errors::recorded_handler_failure(self.inner.id, evidence));
                }
            }
        }
        Ok(())
    }
}

impl CallbackInvocationLease {
    #[must_use]
    pub fn entry_sequence(&self) -> u64 {
        self.entry_sequence
    }

    pub fn enter(self) -> Result<CallbackInvocationGuard, PythonError> {
        ACTIVE_CALLBACKS.with(|active| {
            active
                .borrow_mut()
                .push((self.owner.inner.id, self.callback_id));
        });
        Ok(CallbackInvocationGuard {
            owner_id: self.owner.inner.id,
            callback_id: self.callback_id,
            lease: Some(self),
            active: true,
        })
    }
}

impl Drop for CallbackInvocationLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut state = lock_state(&self.owner.inner);
        state.active_calls = state.active_calls.saturating_sub(1);
        if state.status == CallbackOwnerStatus::Closing && state.active_calls == 0 {
            self.owner.inner.changed.notify_all();
        }
    }
}

impl Drop for CallbackInvocationGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        ACTIVE_CALLBACKS.with(|active| {
            let mut active = active.borrow_mut();
            if active.last() == Some(&(self.owner_id, self.callback_id)) {
                active.pop();
            } else if let Some(index) = active
                .iter()
                .rposition(|entry| *entry == (self.owner_id, self.callback_id))
            {
                active.remove(index);
            }
        });
        drop(self.lease.take());
    }
}

impl Drop for CallbackOwnerUnregisterGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut state = lock_state(&self.owner.inner);
        if state.unregister_status == CallbackUnregisterStatus::Running {
            state.unregister_status = CallbackUnregisterStatus::Finished;
            self.owner.inner.changed.notify_all();
        }
    }
}

fn callback_is_active(owner_id: u64, callback_id: u64) -> bool {
    ACTIVE_CALLBACKS.with(|active| active.borrow().contains(&(owner_id, callback_id)))
}

fn owner_is_active(owner_id: u64) -> bool {
    ACTIVE_CALLBACKS.with(|active| {
        active
            .borrow()
            .iter()
            .any(|(active_owner, _)| *active_owner == owner_id)
    })
}

fn lock_state(owner: &CallbackOwnerInner) -> std::sync::MutexGuard<'_, OwnerData> {
    owner
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
