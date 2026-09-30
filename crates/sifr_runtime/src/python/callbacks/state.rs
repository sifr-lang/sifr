use super::super::PythonError;
use super::{errors, registry};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use tokio::sync::Notify;

type UnregisterAction = Arc<dyn Fn() -> Result<(), PythonError> + Send + Sync + 'static>;
type CaptureReleaseAction = Box<dyn FnOnce() + Send + 'static>;
type AsyncCancellationAction = Arc<dyn Fn() + Send + Sync + 'static>;

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
    async_changed: Notify,
    unregister: Option<UnregisterAction>,
    releases: Mutex<Vec<CaptureReleaseAction>>,
    retained: bool,
}

struct OwnerData {
    status: CallbackOwnerStatus,
    active_calls: usize,
    synchronous_entries: BTreeSet<u64>,
    next_sequence: u64,
    next_callback_id: u64,
    first_failure: Option<CallbackFailureEvidence>,
    first_failure_observed: bool,
    captures_released: bool,
    closer_active: bool,
    unregister_status: CallbackUnregisterStatus,
    async_entries: BTreeMap<u64, AsyncEntry>,
}

struct AsyncEntry {
    callback_id: u64,
    cancel: AsyncCancellationAction,
    cancellation_requested: bool,
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

pub struct CallbackInvocationPollGuard {
    owner_id: u64,
    callback_id: u64,
    active: bool,
}

pub(super) struct CallbackAsyncEntryLease {
    owner: CallbackOwnerState,
    entry_sequence: u64,
    active: bool,
}

pub struct CallbackOwnerUnregisterGuard {
    owner: CallbackOwnerState,
    active: bool,
}

// Admission stays closed when a cancelled closer relinquishes election. Another
// closer can acquire authority and finish the same drain; release actions run
// only under one authority, and contain no suspension point.
struct CloseAuthority(CallbackOwnerState);

impl Drop for CloseAuthority {
    fn drop(&mut self) {
        lock_state(&self.0.inner).closer_active = false;
        self.0.inner.changed.notify_all();
        self.0.inner.async_changed.notify_waiters();
    }
}

impl CallbackOwnerState {
    pub(super) fn is_call_scoped(&self) -> bool {
        !self.inner.retained
    }

    pub(super) fn request_callback_entry_cancellation(&self, callback_id: u64) {
        let cancellations =
            pending_async_cancellations_for_callback(&mut lock_state(&self.inner), callback_id);
        invoke_cancellations(cancellations);
    }

    pub(super) fn require_call_scope(&self) -> Result<(), PythonError> {
        if self.inner.retained {
            return Err(errors::unavailable(
                "borrowed callback requires a call-scoped owner",
            ));
        }
        Ok(())
    }

    pub(super) fn reject_close_reentrancy(&self) -> Result<(), PythonError> {
        if owner_is_active(self.inner.id) {
            return Err(errors::close_from_invocation(self.inner.id));
        }
        Ok(())
    }

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
                synchronous_entries: BTreeSet::new(),
                next_sequence: 0,
                next_callback_id: 0,
                first_failure: None,
                first_failure_observed: false,
                captures_released: false,
                closer_active: false,
                unregister_status: CallbackUnregisterStatus::NotStarted,
                async_entries: BTreeMap::new(),
            }),
            changed: Condvar::new(),
            async_changed: Notify::new(),
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
            let sequence = state.next_sequence;
            state.synchronous_entries.insert(sequence);
            sequence
        };
        Ok(CallbackInvocationLease {
            owner: self.clone(),
            callback_id,
            entry_sequence,
            active: true,
        })
    }

    pub(super) fn allocate_callback_id(&self) -> Result<u64, PythonError> {
        let mut state = lock_state(&self.inner);
        state.next_callback_id = state
            .next_callback_id
            .checked_add(1)
            .ok_or_else(|| errors::unavailable("callback identity space"))?;
        Ok(state.next_callback_id)
    }

    pub fn reject_serial_reentrancy(&self, callback_id: u64) -> Result<(), PythonError> {
        if callback_is_active(self.inner.id, callback_id) {
            return Err(errors::reentrant(self.inner.id, callback_id));
        }
        Ok(())
    }

    pub(super) fn register_async_entry(
        &self,
        callback_id: u64,
        entry_sequence: u64,
        cancel: AsyncCancellationAction,
    ) -> Result<CallbackAsyncEntryLease, PythonError> {
        let cancel_now = {
            let mut state = lock_state(&self.inner);
            if state.async_entries.contains_key(&entry_sequence) {
                return Err(errors::unavailable("async callback entry identity"));
            }
            let cancel_now = state.status != CallbackOwnerStatus::Open;
            // Setup holds synchronous admission until the async entry owns its
            // future. A synchronous wrapper drain may then leave owned executor
            // bookkeeping to the async wrapper without waiting on that executor.
            state.synchronous_entries.remove(&entry_sequence);
            state.async_entries.insert(
                entry_sequence,
                AsyncEntry {
                    callback_id,
                    cancel: Arc::clone(&cancel),
                    cancellation_requested: cancel_now,
                },
            );
            self.inner.changed.notify_all();
            self.inner.async_changed.notify_waiters();
            cancel_now
        };
        if cancel_now {
            cancel();
        }
        Ok(CallbackAsyncEntryLease {
            owner: self.clone(),
            entry_sequence,
            active: true,
        })
    }

    pub(super) async fn cancel_callback_entries(&self, callback_id: u64) {
        loop {
            let notified = self.inner.async_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let (finished, cancellations) = {
                let mut state = lock_state(&self.inner);
                let cancellations =
                    pending_async_cancellations_for_callback(&mut state, callback_id);
                let finished = !state
                    .async_entries
                    .values()
                    .any(|entry| entry.callback_id == callback_id);
                (finished, cancellations)
            };
            invoke_cancellations(cancellations);
            if finished {
                return;
            }
            notified.await;
        }
    }

    pub fn close_call_scope(&self) -> Result<(), PythonError> {
        if self.inner.retained {
            return Err(errors::unavailable(
                "retained owner close without unregister authority",
            ));
        }
        self.close_after_unregister(false, true)
    }

    pub async fn close_call_scope_async(&self) -> Result<(), PythonError> {
        if self.inner.retained {
            return Err(errors::unavailable(
                "retained owner close without unregister authority",
            ));
        }
        self.close_after_unregister_async(false, true).await
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

    pub async fn close_after_owner_unregister_async(&self) -> Result<(), PythonError> {
        if self.inner.unregister.is_some() {
            let state = lock_state(&self.inner);
            if state.unregister_status == CallbackUnregisterStatus::NotStarted {
                return Err(errors::unavailable("owner unregister authority"));
            }
        }
        self.close_after_unregister_async(false, true).await
    }

    pub async fn close_after_owner_unregister_with_typed_observer_async(
        &self,
    ) -> Result<(), PythonError> {
        if self.inner.unregister.is_some() {
            let state = lock_state(&self.inner);
            if state.unregister_status == CallbackUnregisterStatus::NotStarted {
                return Err(errors::unavailable("owner unregister authority"));
            }
        }
        self.close_after_unregister_async(false, false).await
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
        loop {
            if state.status == CallbackOwnerStatus::Closed {
                drop(state);
                return self.retained_failure_result(surface_retained_failure);
            }
            if !state.closer_active {
                state.status = CallbackOwnerStatus::Closing;
                state.closer_active = true;
                break;
            }
            state = self
                .inner
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        let _authority = CloseAuthority(self.clone());
        loop {
            let cancellations = pending_async_cancellations(&mut state);
            if state.active_calls == 0 && state.async_entries.is_empty() {
                break;
            }
            if cancellations.is_empty() {
                state = self
                    .inner
                    .changed
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            } else {
                drop(state);
                invoke_cancellations(cancellations);
                state = lock_state(&self.inner);
            }
        }
        drop(state);
        self.finish_close();
        self.retained_failure_result(surface_retained_failure)
    }

    async fn close_after_unregister_async(
        &self,
        runtime_shutdown: bool,
        surface_retained_failure: bool,
    ) -> Result<(), PythonError> {
        if !runtime_shutdown && owner_is_active(self.inner.id) {
            return Err(errors::close_from_invocation(self.inner.id));
        }
        loop {
            let notified = self.inner.async_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let running =
                lock_state(&self.inner).unregister_status == CallbackUnregisterStatus::Running;
            if !running {
                break;
            }
            notified.await;
        }
        loop {
            let notified = self.inner.async_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            {
                let mut state = lock_state(&self.inner);
                if state.status == CallbackOwnerStatus::Closed {
                    drop(state);
                    return self.retained_failure_result(surface_retained_failure);
                }
                if !state.closer_active {
                    state.status = CallbackOwnerStatus::Closing;
                    state.closer_active = true;
                    break;
                }
            }
            notified.await;
        }
        let _authority = CloseAuthority(self.clone());
        loop {
            let notified = self.inner.async_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let (finished, cancellations) = {
                let mut state = lock_state(&self.inner);
                let cancellations = pending_async_cancellations(&mut state);
                (
                    state.active_calls == 0 && state.async_entries.is_empty(),
                    cancellations,
                )
            };
            invoke_cancellations(cancellations);
            if finished {
                break;
            }
            notified.await;
        }
        self.finish_close();
        self.retained_failure_result(surface_retained_failure)
    }

    // Borrowed synchronous wrappers need every admitted decoding/handler/encoding
    // use to finish before freeing their target. They must not block the executor
    // that owns a sibling asyncio entry's supervisor. Closing admission and
    // draining synchronous setup/use establishes that local lifetime boundary;
    // the last remaining owned lease completes the shared owner close.
    pub(super) fn close_synchronous_call_scope(&self) -> Result<(), PythonError> {
        self.require_call_scope()?;
        self.reject_close_reentrancy()?;
        let drain = || {
            let mut state = lock_state(&self.inner);
            if state.status == CallbackOwnerStatus::Open {
                state.status = CallbackOwnerStatus::Closing;
            }
            let cancellations = pending_async_cancellations(&mut state);
            drop(state);
            invoke_cancellations(cancellations);
            let mut state = lock_state(&self.inner);
            while !state.synchronous_entries.is_empty() {
                state = self
                    .inner
                    .changed
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
            drop(state);
            self.finish_call_scope_if_drained();
        };
        // A foreign invocation must reacquire Python entry to encode its output.
        // Release any caller GIL while waiting for that synchronous invocation.
        if pyo3::Python::try_attach(|py| py.detach(drain)).is_none() {
            drain();
        }
        Ok(())
    }

    fn finish_call_scope_if_drained(&self) {
        let mut state = lock_state(&self.inner);
        if self.inner.retained
            || state.status != CallbackOwnerStatus::Closing
            || state.closer_active
            || state.active_calls != 0
            || !state.async_entries.is_empty()
        {
            return;
        }
        state.closer_active = true;
        drop(state);
        let _authority = CloseAuthority(self.clone());
        self.finish_close();
    }

    fn finish_close(&self) {
        let releases = std::mem::take(
            &mut *self
                .inner
                .releases
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        // Capture destruction is a callback ownership operation too. Reentrant
        // close must return an error instead of waiting on this closer itself.
        ACTIVE_CALLBACKS.with(|active| active.borrow_mut().push((self.inner.id, 0)));
        let _release_guard = CallbackInvocationPollGuard {
            owner_id: self.inner.id,
            callback_id: 0,
            active: true,
        };
        for release in releases {
            release();
        }
        drop(_release_guard);
        let mut state = lock_state(&self.inner);
        state.captures_released = true;
        state.status = CallbackOwnerStatus::Closed;
        self.inner.changed.notify_all();
        self.inner.async_changed.notify_waiters();
        drop(state);
        if self.inner.retained {
            registry::unregister(self.inner.id);
        }
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

    #[must_use]
    pub fn enter_poll(&self) -> CallbackInvocationPollGuard {
        ACTIVE_CALLBACKS.with(|active| {
            active
                .borrow_mut()
                .push((self.owner.inner.id, self.callback_id));
        });
        CallbackInvocationPollGuard {
            owner_id: self.owner.inner.id,
            callback_id: self.callback_id,
            active: true,
        }
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
        state.synchronous_entries.remove(&self.entry_sequence);
        self.owner.inner.changed.notify_all();
        if state.status == CallbackOwnerStatus::Closing && state.active_calls == 0 {
            self.owner.inner.changed.notify_all();
        }
        self.owner.inner.async_changed.notify_waiters();
        drop(state);
        self.owner.finish_call_scope_if_drained();
    }
}

impl Drop for CallbackAsyncEntryLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut state = lock_state(&self.owner.inner);
        state.async_entries.remove(&self.entry_sequence);
        self.owner.inner.changed.notify_all();
        self.owner.inner.async_changed.notify_waiters();
        drop(state);
        self.owner.finish_call_scope_if_drained();
    }
}

impl Drop for CallbackInvocationGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        remove_active_callback(self.owner_id, self.callback_id);
        drop(self.lease.take());
    }
}

impl Drop for CallbackInvocationPollGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        remove_active_callback(self.owner_id, self.callback_id);
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
            self.owner.inner.async_changed.notify_waiters();
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

#[must_use]
pub fn current_callback_origin() -> Option<(u64, u64)> {
    ACTIVE_CALLBACKS.with(|active| active.borrow().last().copied())
}

fn remove_active_callback(owner_id: u64, callback_id: u64) {
    ACTIVE_CALLBACKS.with(|active| {
        let mut active = active.borrow_mut();
        if active.last() == Some(&(owner_id, callback_id)) {
            active.pop();
        } else if let Some(index) = active
            .iter()
            .rposition(|entry| *entry == (owner_id, callback_id))
        {
            active.remove(index);
        }
    });
}

fn pending_async_cancellations(state: &mut OwnerData) -> Vec<AsyncCancellationAction> {
    state
        .async_entries
        .values_mut()
        .filter_map(|entry| {
            if entry.cancellation_requested {
                return None;
            }
            entry.cancellation_requested = true;
            Some(Arc::clone(&entry.cancel))
        })
        .collect()
}

fn pending_async_cancellations_for_callback(
    state: &mut OwnerData,
    callback_id: u64,
) -> Vec<AsyncCancellationAction> {
    state
        .async_entries
        .values_mut()
        .filter(|entry| entry.callback_id == callback_id && !entry.cancellation_requested)
        .map(|entry| {
            entry.cancellation_requested = true;
            Arc::clone(&entry.cancel)
        })
        .collect()
}

fn invoke_cancellations(cancellations: Vec<AsyncCancellationAction>) {
    for cancel in cancellations {
        cancel();
    }
}

fn lock_state(owner: &CallbackOwnerInner) -> std::sync::MutexGuard<'_, OwnerData> {
    owner
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
