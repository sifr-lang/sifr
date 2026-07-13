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
    exact: Option<ExactCancellation>,
    requested: bool,
    fallback_resumed: bool,
    next_generation: u64,
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
                } else if let Some(hook) = state.fallback.as_ref().map(Arc::clone) {
                    state.fallback_resumed = true;
                    (CancellationRequest::Fallback, Some(hook))
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
