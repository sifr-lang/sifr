//! Cooperative cancellation handoff between generated tasks and foreign runtimes.

use std::sync::{Arc, Mutex};

pub type CancellationHook = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone)]
pub struct CancellationCarrier {
    state: Arc<Mutex<CancellationState>>,
}

#[derive(Default)]
struct CancellationState {
    fallback: Option<CancellationHook>,
    exact: Option<CancellationHook>,
    requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationBind {
    Bound,
    AlreadyBound,
    InvokedPendingCancellation,
    StateUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationClaim {
    Claimed,
    CancelledBeforeClaim,
    AlreadyClaimed,
    StateUnavailable,
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
pub struct CancellationStateError;

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
                if state.requested && state.exact.is_none() {
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

    pub fn claim(&self, hook: CancellationHook) -> CancellationClaim {
        let Ok(mut state) = self.state.lock() else {
            return CancellationClaim::StateUnavailable;
        };
        if state.requested {
            return CancellationClaim::CancelledBeforeClaim;
        }
        if state.exact.is_some() {
            return CancellationClaim::AlreadyClaimed;
        }
        state.exact = Some(hook);
        CancellationClaim::Claimed
    }

    pub fn request_cancel(&self) -> CancellationRequest {
        let (outcome, hook) = match self.state.lock() {
            Ok(mut state) => {
                if state.requested {
                    return CancellationRequest::AlreadyRequested;
                }
                state.requested = true;
                if let Some(hook) = state.exact.as_ref() {
                    (CancellationRequest::Claimed, Some(Arc::clone(hook)))
                } else if let Some(hook) = state.fallback.as_ref() {
                    (CancellationRequest::Fallback, Some(Arc::clone(hook)))
                } else {
                    (CancellationRequest::FallbackPending, None)
                }
            }
            Err(_) => return CancellationRequest::StateUnavailable,
        };
        if let Some(hook) = hook {
            hook();
        }
        outcome
    }

    pub fn fallback_hook(&self) -> Result<Option<CancellationHook>, CancellationStateError> {
        self.state
            .lock()
            .map(|state| state.fallback.as_ref().map(Arc::clone))
            .map_err(|_| CancellationStateError)
    }

    pub fn is_claimed(&self) -> Result<bool, CancellationStateError> {
        self.state
            .lock()
            .map(|state| state.exact.is_some())
            .map_err(|_| CancellationStateError)
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Barrier;

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
        assert_eq!(
            carrier.claim(counting_hook(&exact_calls)),
            CancellationClaim::Claimed
        );

        assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
        assert_eq!(exact_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancellation_before_claim_rejects_submission() {
        let carrier = CancellationCarrier::new();
        let exact_calls = Arc::new(AtomicUsize::new(0));

        assert_eq!(
            carrier.request_cancel(),
            CancellationRequest::FallbackPending
        );
        assert_eq!(
            carrier.claim(counting_hook(&exact_calls)),
            CancellationClaim::CancelledBeforeClaim
        );
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
                (CancellationClaim::Claimed, CancellationRequest::Claimed)
                    | (
                        CancellationClaim::CancelledBeforeClaim,
                        CancellationRequest::Fallback
                    )
            ));
            assert_eq!(
                fallback_calls.load(Ordering::SeqCst) + exact_calls.load(Ordering::SeqCst),
                1
            );
        }
    }
}
