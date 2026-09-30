//! Revocable borrowed invocation storage. The wrapper owns the drain registry;
//! workers and queued Python completions own only slots in that registry.
use super::asyncio::{AsyncioOutput, BoxCallbackFuture};
use super::{CallbackExecutionError, CallbackInvocationLease};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll};

#[derive(Default)]
pub(super) struct InvocationDrain(Mutex<Vec<Weak<InvocationSlot>>>);

pub(super) struct InvocationSlot(Mutex<Option<InvocationState>>);

enum InvocationState {
    Pending {
        // Drop borrowed state before releasing owner admission.
        future: BoxCallbackFuture<'static>,
        invocation: CallbackInvocationLease,
    },
    Ready(InvocationCompletion),
}

pub(super) struct InvocationCompletion {
    // Field order is part of the drain proof when a completion is discarded.
    pub(super) outcome: Result<Box<dyn AsyncioOutput + 'static>, CallbackExecutionError>,
    pub(super) invocation: CallbackInvocationLease,
}

pub(super) struct InvocationFuture(pub(super) Arc<InvocationSlot>);

impl InvocationDrain {
    pub(super) fn register(
        &self,
        invocation: CallbackInvocationLease,
        future: BoxCallbackFuture<'static>,
    ) -> InvocationFuture {
        let slot = Arc::new(InvocationSlot(Mutex::new(Some(InvocationState::Pending {
            invocation,
            future,
        }))));
        let mut slots = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slots.retain(|slot| slot.strong_count() != 0);
        slots.push(Arc::downgrade(&slot));
        InvocationFuture(slot)
    }

    /// Call only after closing and draining setup admission. Mutex exclusion
    /// waits for any ongoing handler poll/encoding, then drops every pending
    /// borrowed future or queued borrowed output before captures can expire.
    pub(super) fn revoke(&self) {
        let slots = std::mem::take(
            &mut *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for slot in slots.into_iter().filter_map(|slot| slot.upgrade()) {
            slot.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        }
    }
}

impl Future for InvocationFuture {
    type Output = Arc<InvocationSlot>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self
            .0
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(InvocationState::Pending { invocation, future }) = state.as_mut() else {
            return Poll::Ready(Arc::clone(&self.0));
        };
        let _active = invocation.enter_poll();
        let outcome = match future.as_mut().poll(context) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(outcome) => outcome,
        };
        let Some(InvocationState::Pending { invocation, .. }) = state.take() else {
            unreachable!("invocation state is protected by this mutex")
        };
        *state = Some(InvocationState::Ready(InvocationCompletion {
            invocation,
            outcome,
        }));
        Poll::Ready(Arc::clone(&self.0))
    }
}

impl InvocationSlot {
    pub(super) fn complete<T>(
        &self,
        complete: impl FnOnce(InvocationCompletion) -> T,
    ) -> Option<T> {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(InvocationState::Ready(completion)) = state.take() else {
            return None;
        };
        // Keep the mutex through encoding: output may still borrow captures.
        Some(complete(completion))
    }

    pub(super) fn discard(&self) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

#[cfg(test)]
pub(super) fn prove_queued_output_revocation() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct BorrowedOutput<'a>(
        &'a AtomicUsize,
        &'a AtomicUsize,
        &'a super::CallbackOwnerState,
    );
    impl AsyncioOutput for BorrowedOutput<'_> {
        fn encode(
            self: Box<Self>,
        ) -> Result<crate::python::ObjectHandle, crate::python::PythonError> {
            self.1.fetch_add(1, Ordering::SeqCst);
            crate::python::from_none()
        }
    }
    impl Drop for BorrowedOutput<'_> {
        fn drop(&mut self) {
            assert_eq!(
                self.2.active_calls(),
                1,
                "lease outlives borrowed output destruction"
            );
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let releases = AtomicUsize::new(0);
    let encodes = AtomicUsize::new(0);
    let owner = super::CallbackOwnerState::new_call_scoped().expect("owner");
    let drain = InvocationDrain::default();
    let output = BorrowedOutput(&releases, &encodes, &owner);
    let future: BoxCallbackFuture<'_> =
        Box::pin(async move { Ok(Box::new(output) as Box<dyn AsyncioOutput + '_>) });
    // SAFETY: the drain registry revokes the erased output before either local
    // counter ends, even when a worker handle is forgotten and its slot escapes.
    let future =
        unsafe { std::mem::transmute::<BoxCallbackFuture<'_>, BoxCallbackFuture<'static>>(future) };
    let mut worker = Box::pin(drain.register(owner.accept(1, false).expect("admit"), future));
    let mut context = Context::from_waker(std::task::Waker::noop());
    let Poll::Ready(shell) = worker.as_mut().poll(&mut context) else {
        panic!("ready fixture future must complete")
    };
    std::mem::forget(worker);
    assert_eq!(owner.active_calls(), 1);
    assert_eq!(releases.load(Ordering::SeqCst), 0);
    drain.revoke();
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert_eq!(owner.active_calls(), 0);
    assert!(shell.complete(|completion| completion.outcome).is_none());
    assert_eq!(encodes.load(Ordering::SeqCst), 0);
    drain.revoke();
    assert_eq!(releases.load(Ordering::SeqCst), 1);
}
