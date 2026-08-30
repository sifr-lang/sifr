//! Cooperative cancellation handoff between generated tasks and foreign runtimes.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use crate::async_cleanup::AsyncCleanupEvidence;

pub type CancellationHook = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone)]
pub struct CancellationCarrier {
    state: Arc<Mutex<CancellationState>>,
}

#[derive(Default)]
struct CancellationState {
    fallback: Option<CancellationHook>,
    exact: Option<ExactCancellation>,
    requested: bool,
    fallback_resumed: bool,
    next_generation: u64,
    async_cleanup_evidence: Vec<AsyncCleanupEvidence>,
}

struct ExactCancellation {
    generation: u64,
    hook: CancellationHook,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationBind {
    Bound,
    AlreadyBound,
    InvokedPendingCancellation,
    StateUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationClaimError {
    CancelledBeforeClaim,
    AlreadyClaimed,
    StateUnavailable,
}

#[must_use = "dropping the lease releases the exact cancellation hook"]
pub struct CancellationClaimLease {
    carrier: CancellationCarrier,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationRequest {
    Claimed,
    Fallback,
    FallbackPending,
    AlreadyRequested,
    StateUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationResume {
    Invoked,
    AlreadyResumed,
    NotRequested,
    ExactClaimActive,
    FallbackUnavailable,
    StateUnavailable,
}

/// Sticky notification used by generated cancellation races. Once notified,
/// every current and future waiter observes readiness.
#[derive(Clone, Default)]
pub struct StickyCancellation {
    state: Arc<Mutex<StickyCancellationState>>,
}

#[derive(Default)]
struct StickyCancellationState {
    notified: bool,
    waker: Option<Waker>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationScopeError {
    CancelledBeforeClaim,
    AlreadyClaimed,
    StateUnavailable,
    ChildFallbackUnavailable,
}

/// Parent/child cancellation claim used by generated async cleanup scopes.
#[must_use = "dropping the scope lease releases its parent cancellation claim"]
pub struct CancellationScopeLease {
    parent: CancellationCarrier,
    child: CancellationCarrier,
    notification: StickyCancellation,
    parent_claim: Option<CancellationClaimLease>,
}

impl CancellationCarrier {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CancellationState::default())),
        }
    }

    pub fn bind_fallback(&self, hook: CancellationHook) -> CancellationBind {
        let (outcome, invoke) = match self.state.lock() {
            Ok(mut state) => {
                if state.fallback.is_some() {
                    return CancellationBind::AlreadyBound;
                }
                state.fallback = Some(Arc::clone(&hook));
                if state.requested && state.exact.is_none() && !state.fallback_resumed {
                    state.fallback_resumed = true;
                    (CancellationBind::InvokedPendingCancellation, Some(hook))
                } else {
                    (CancellationBind::Bound, None)
                }
            }
            Err(_) => return CancellationBind::StateUnavailable,
        };
        if let Some(hook) = invoke {
            hook();
        }
        outcome
    }

    pub fn claim(
        &self,
        hook: CancellationHook,
    ) -> Result<CancellationClaimLease, CancellationClaimError> {
        let Ok(mut state) = self.state.lock() else {
            return Err(CancellationClaimError::StateUnavailable);
        };
        if state.requested {
            return Err(CancellationClaimError::CancelledBeforeClaim);
        }
        if state.exact.is_some() {
            return Err(CancellationClaimError::AlreadyClaimed);
        }
        state.next_generation = state.next_generation.wrapping_add(1).max(1);
        let generation = state.next_generation;
        state.exact = Some(ExactCancellation { generation, hook });
        Ok(CancellationClaimLease {
            carrier: self.clone(),
            generation,
        })
    }

    pub fn request_cancel(&self) -> CancellationRequest {
        let (outcome, hook) = match self.state.lock() {
            Ok(mut state) => {
                if state.requested {
                    return CancellationRequest::AlreadyRequested;
                }
                state.requested = true;
                if let Some(exact) = state.exact.as_ref() {
                    (CancellationRequest::Claimed, Some(Arc::clone(&exact.hook)))
                } else {
                    let fallback = state.fallback.as_ref().map(Arc::clone);
                    match fallback {
                        Some(hook) => {
                            state.fallback_resumed = true;
                            (CancellationRequest::Fallback, Some(hook))
                        }
                        None => (CancellationRequest::FallbackPending, None),
                    }
                }
            }
            Err(_) => return CancellationRequest::StateUnavailable,
        };
        if let Some(hook) = hook {
            hook();
        }
        outcome
    }

    pub fn resume_fallback_after_claim(&self) -> CancellationResume {
        let hook = match self.state.lock() {
            Ok(mut state) => {
                if !state.requested {
                    return CancellationResume::NotRequested;
                }
                if state.exact.is_some() {
                    return CancellationResume::ExactClaimActive;
                }
                if state.fallback_resumed {
                    return CancellationResume::AlreadyResumed;
                }
                let Some(hook) = state.fallback.as_ref().map(Arc::clone) else {
                    return CancellationResume::FallbackUnavailable;
                };
                state.fallback_resumed = true;
                hook
            }
            Err(_) => return CancellationResume::StateUnavailable,
        };
        hook();
        CancellationResume::Invoked
    }

    pub fn record_async_cleanup_failed(
        &self,
        error: String,
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    ) {
        self.record_async_cleanup_evidence(AsyncCleanupEvidence::cleanup_failed(
            error, location, resource, operation, budget,
        ));
    }

    pub fn record_async_cleanup_timed_out(
        &self,
        location: String,
        resource: String,
        operation: String,
        budget: Duration,
    ) {
        self.record_async_cleanup_evidence(AsyncCleanupEvidence::cleanup_timed_out(
            location, resource, operation, budget,
        ));
    }

    pub fn record_async_cleanup_evidence(&self, evidence: AsyncCleanupEvidence) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.async_cleanup_evidence.push(evidence);
    }

    #[must_use]
    pub fn take_async_cleanup_evidence(&self) -> Vec<AsyncCleanupEvidence> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::take(&mut state.async_cleanup_evidence)
    }
}

impl StickyCancellation {
    #[must_use]
    pub fn is_notified(&self) -> bool {
        self.state.lock().map_or(true, |state| state.notified)
    }

    pub fn notify(&self) {
        let waker = match self.state.lock() {
            Ok(mut state) => {
                if state.notified {
                    return;
                }
                state.notified = true;
                state.waker.take()
            }
            Err(_) => return,
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl Future for StickyCancellation {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let Ok(mut state) = self.state.lock() else {
            return Poll::Ready(());
        };
        if state.notified {
            return Poll::Ready(());
        }
        if state
            .waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }
}

impl CancellationScopeLease {
    pub fn claim(parent: &CancellationCarrier) -> Result<Self, CancellationScopeError> {
        let child = CancellationCarrier::new();
        let notification = StickyCancellation::default();
        let notify = notification.clone();
        match child.bind_fallback(Arc::new(move || notify.notify())) {
            CancellationBind::Bound => {}
            CancellationBind::AlreadyBound | CancellationBind::InvokedPendingCancellation => {
                return Err(CancellationScopeError::ChildFallbackUnavailable);
            }
            CancellationBind::StateUnavailable => {
                return Err(CancellationScopeError::StateUnavailable);
            }
        }
        let requested_child = child.clone();
        let parent_claim = parent
            .claim(Arc::new(move || {
                let _outcome = requested_child.request_cancel();
            }))
            .map_err(|error| match error {
                CancellationClaimError::CancelledBeforeClaim => {
                    CancellationScopeError::CancelledBeforeClaim
                }
                CancellationClaimError::AlreadyClaimed => CancellationScopeError::AlreadyClaimed,
                CancellationClaimError::StateUnavailable => {
                    CancellationScopeError::StateUnavailable
                }
            })?;
        Ok(Self {
            parent: parent.clone(),
            child,
            notification,
            parent_claim: Some(parent_claim),
        })
    }

    #[must_use]
    pub fn child(&self) -> &CancellationCarrier {
        &self.child
    }

    #[must_use]
    pub fn notification(&self) -> StickyCancellation {
        self.notification.clone()
    }

    /// Release the exact parent claim before resuming its native fallback.
    pub fn release_and_resume_parent(mut self) -> CancellationResume {
        drop(self.parent_claim.take());
        self.parent.resume_fallback_after_claim()
    }
}

impl Drop for CancellationClaimLease {
    fn drop(&mut self) {
        let mut state = self
            .carrier
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .exact
            .as_ref()
            .is_some_and(|exact| exact.generation == self.generation)
        {
            state.exact = None;
        }
    }
}

impl Default for CancellationCarrier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    fn counting_hook(counter: &Arc<AtomicUsize>) -> CancellationHook {
        let counter = Arc::clone(counter);
        Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })
    }

    #[test]
    fn unclaimed_request_uses_fallback_once() {
        let carrier = CancellationCarrier::new();
        let fallback_calls = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            carrier.bind_fallback(counting_hook(&fallback_calls)),
            CancellationBind::Bound
        );

        assert_eq!(carrier.request_cancel(), CancellationRequest::Fallback);
        assert_eq!(
            carrier.request_cancel(),
            CancellationRequest::AlreadyRequested
        );
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn request_before_binding_invokes_late_fallback() {
        let carrier = CancellationCarrier::new();
        let fallback_calls = Arc::new(AtomicUsize::new(0));

        assert_eq!(
            carrier.request_cancel(),
            CancellationRequest::FallbackPending
        );
        assert_eq!(
            carrier.bind_fallback(counting_hook(&fallback_calls)),
            CancellationBind::InvokedPendingCancellation
        );
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn claimed_request_uses_exact_hook_without_fallback() {
        let carrier = CancellationCarrier::new();
        let fallback_calls = Arc::new(AtomicUsize::new(0));
        let exact_calls = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            carrier.bind_fallback(counting_hook(&fallback_calls)),
            CancellationBind::Bound
        );
        let _claim = carrier
            .claim(counting_hook(&exact_calls))
            .expect("fresh carrier should be claimable");

        assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
        assert_eq!(exact_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn claimed_terminal_resume_invokes_fallback_once_after_lease_release() {
        let carrier = CancellationCarrier::new();
        let fallback_calls = Arc::new(AtomicUsize::new(0));
        let exact_calls = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            carrier.bind_fallback(counting_hook(&fallback_calls)),
            CancellationBind::Bound
        );
        let claim = carrier
            .claim(counting_hook(&exact_calls))
            .expect("fresh carrier should be claimable");

        assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
        assert_eq!(
            carrier.resume_fallback_after_claim(),
            CancellationResume::ExactClaimActive
        );
        drop(claim);
        assert_eq!(
            carrier.resume_fallback_after_claim(),
            CancellationResume::Invoked
        );
        assert_eq!(
            carrier.resume_fallback_after_claim(),
            CancellationResume::AlreadyResumed
        );
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
        assert_eq!(exact_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn resume_requires_request_and_bound_fallback() {
        let carrier = CancellationCarrier::new();
        assert_eq!(
            carrier.resume_fallback_after_claim(),
            CancellationResume::NotRequested
        );
        assert_eq!(
            carrier.request_cancel(),
            CancellationRequest::FallbackPending
        );
        assert_eq!(
            carrier.resume_fallback_after_claim(),
            CancellationResume::FallbackUnavailable
        );
    }

    #[test]
    fn cancellation_scope_routes_parent_request_through_child_sticky_fallback() {
        let parent = CancellationCarrier::new();
        let parent_fallback_calls = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            parent.bind_fallback(counting_hook(&parent_fallback_calls)),
            CancellationBind::Bound
        );
        let scope = CancellationScopeLease::claim(&parent).expect("scope should claim parent");
        let notification = scope.notification();

        assert_eq!(parent.request_cancel(), CancellationRequest::Claimed);
        assert!(notification.is_notified());
        assert_eq!(parent_fallback_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            scope.release_and_resume_parent(),
            CancellationResume::Invoked
        );
        assert_eq!(parent_fallback_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancellation_scope_release_observes_request_after_earlier_notification_check() {
        let parent = CancellationCarrier::new();
        let parent_fallback_calls = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            parent.bind_fallback(counting_hook(&parent_fallback_calls)),
            CancellationBind::Bound
        );
        let scope = CancellationScopeLease::claim(&parent).expect("scope should claim parent");

        assert!(!scope.notification().is_notified());
        assert_eq!(parent.request_cancel(), CancellationRequest::Claimed);
        assert_eq!(
            scope.release_and_resume_parent(),
            CancellationResume::Invoked
        );
        assert_eq!(parent_fallback_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancellation_scope_waits_for_child_exact_claim_before_notifying() {
        let parent = CancellationCarrier::new();
        let _bound = parent.bind_fallback(Arc::new(|| {}));
        let scope = CancellationScopeLease::claim(&parent).expect("scope should claim parent");
        let exact_calls = Arc::new(AtomicUsize::new(0));
        let child_claim = scope
            .child()
            .claim(counting_hook(&exact_calls))
            .expect("Python await should claim child");

        assert_eq!(parent.request_cancel(), CancellationRequest::Claimed);
        assert_eq!(exact_calls.load(Ordering::SeqCst), 1);
        assert!(!scope.notification().is_notified());
        drop(child_claim);
        assert_eq!(
            scope.child().resume_fallback_after_claim(),
            CancellationResume::Invoked
        );
        assert!(scope.notification().is_notified());
    }

    struct CountingWake(AtomicUsize);

    impl Wake for CountingWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn sticky_notification_wakes_latest_waiter_and_remains_ready() {
        let notification = StickyCancellation::default();
        let wake = Arc::new(CountingWake(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake));
        let mut context = Context::from_waker(&waker);
        let mut first = notification.clone();
        assert!(Pin::new(&mut first).poll(&mut context).is_pending());

        notification.notify();
        assert_eq!(wake.0.load(Ordering::SeqCst), 1);
        assert!(Pin::new(&mut first).poll(&mut context).is_ready());
        let mut late = notification.clone();
        assert!(Pin::new(&mut late).poll(&mut context).is_ready());
    }

    #[test]
    fn cancellation_scope_rejects_cancelled_parent_before_acquisition() {
        let parent = CancellationCarrier::new();
        assert_eq!(
            parent.request_cancel(),
            CancellationRequest::FallbackPending
        );
        assert!(matches!(
            CancellationScopeLease::claim(&parent),
            Err(CancellationScopeError::CancelledBeforeClaim)
        ));
    }

    #[test]
    fn nested_cancellation_scopes_unwind_in_lexical_lifo_order() {
        let root = CancellationCarrier::new();
        let root_fallbacks = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            root.bind_fallback(counting_hook(&root_fallbacks)),
            CancellationBind::Bound
        );
        let outer = CancellationScopeLease::claim(&root).expect("outer scope");
        let inner = CancellationScopeLease::claim(outer.child()).expect("inner scope");

        assert_eq!(root.request_cancel(), CancellationRequest::Claimed);
        assert!(inner.notification().is_notified());
        assert!(!outer.notification().is_notified());
        assert_eq!(root_fallbacks.load(Ordering::SeqCst), 0);

        assert_eq!(
            inner.release_and_resume_parent(),
            CancellationResume::Invoked
        );
        assert!(outer.notification().is_notified());
        assert_eq!(root_fallbacks.load(Ordering::SeqCst), 0);
        assert_eq!(
            outer.release_and_resume_parent(),
            CancellationResume::Invoked
        );
        assert_eq!(root_fallbacks.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancellation_before_claim_rejects_submission() {
        let carrier = CancellationCarrier::new();
        let exact_calls = Arc::new(AtomicUsize::new(0));

        assert_eq!(
            carrier.request_cancel(),
            CancellationRequest::FallbackPending
        );
        assert!(matches!(
            carrier.claim(counting_hook(&exact_calls)),
            Err(CancellationClaimError::CancelledBeforeClaim)
        ));
        assert_eq!(exact_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn contended_claim_and_request_choose_exactly_one_path() {
        for _ in 0..128 {
            let carrier = CancellationCarrier::new();
            let fallback_calls = Arc::new(AtomicUsize::new(0));
            let exact_calls = Arc::new(AtomicUsize::new(0));
            assert_eq!(
                carrier.bind_fallback(counting_hook(&fallback_calls)),
                CancellationBind::Bound
            );
            let barrier = Arc::new(Barrier::new(3));
            let claim_worker = {
                let carrier = carrier.clone();
                let barrier = Arc::clone(&barrier);
                let exact_calls = Arc::clone(&exact_calls);
                std::thread::spawn(move || {
                    barrier.wait();
                    carrier.claim(counting_hook(&exact_calls))
                })
            };
            let request_worker = {
                let carrier = carrier.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    carrier.request_cancel()
                })
            };
            barrier.wait();
            let claim = claim_worker.join().expect("claim worker should not panic");
            let request = request_worker
                .join()
                .expect("request worker should not panic");

            assert!(matches!(
                (claim, request),
                (Ok(_), CancellationRequest::Claimed)
                    | (
                        Err(CancellationClaimError::CancelledBeforeClaim),
                        CancellationRequest::Fallback
                    )
            ));
            assert_eq!(
                fallback_calls.load(Ordering::SeqCst) + exact_calls.load(Ordering::SeqCst),
                1
            );
        }
    }

    #[test]
    fn terminal_lease_release_allows_a_sequential_claim() {
        let carrier = CancellationCarrier::new();
        let first_calls = Arc::new(AtomicUsize::new(0));
        let second_calls = Arc::new(AtomicUsize::new(0));

        let first = carrier
            .claim(counting_hook(&first_calls))
            .expect("first claim should succeed");
        assert!(matches!(
            carrier.claim(counting_hook(&second_calls)),
            Err(CancellationClaimError::AlreadyClaimed)
        ));
        drop(first);

        let _second = carrier
            .claim(counting_hook(&second_calls))
            .expect("released carrier should accept the next await");
    }

    #[test]
    fn async_cleanup_evidence_is_typed_ordered_and_drained_once() {
        let carrier = CancellationCarrier::new();
        carrier.record_async_cleanup_failed(
            "close failed".to_string(),
            "fixture:1".to_string(),
            "Outer".to_string(),
            "__aexit__".to_string(),
            Duration::from_secs(5),
        );
        carrier.record_async_cleanup_timed_out(
            "fixture:2".to_string(),
            "Inner".to_string(),
            "aclose".to_string(),
            Duration::from_secs(3),
        );

        assert_eq!(
            carrier.take_async_cleanup_evidence(),
            vec![
                AsyncCleanupEvidence::cleanup_failed(
                    "close failed".to_string(),
                    "fixture:1".to_string(),
                    "Outer".to_string(),
                    "__aexit__".to_string(),
                    Duration::from_secs(5),
                ),
                AsyncCleanupEvidence::cleanup_timed_out(
                    "fixture:2".to_string(),
                    "Inner".to_string(),
                    "aclose".to_string(),
                    Duration::from_secs(3),
                ),
            ]
        );
        assert!(carrier.take_async_cleanup_evidence().is_empty());
    }

    #[test]
    fn lease_drop_clears_matching_generation_after_mutex_poisoning() {
        let carrier = CancellationCarrier::new();
        let lease = carrier
            .claim(Arc::new(|| {}))
            .expect("fresh carrier should be claimable");
        let state = Arc::clone(&carrier.state);
        let poisoner = std::thread::spawn(move || {
            let _guard = state.lock().expect("state should lock before poisoning");
            panic!("poison cancellation state");
        });
        assert!(poisoner.join().is_err());

        drop(lease);

        let state = carrier
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(state.exact.is_none());
    }
}
