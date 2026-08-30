use lsp_server::RequestId;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub(crate) struct CancellationRegistry {
    state: Arc<Mutex<CancellationState>>,
}

#[derive(Default)]
struct CancellationState {
    active: Option<(RequestId, Arc<AtomicBool>)>,
    pending: Vec<RequestId>,
}

impl CancellationRegistry {
    pub(crate) fn activate(&self, token: &CancellationToken) {
        if let Ok(mut state) = self.state.lock() {
            let cancelled_before_activation = state
                .pending
                .iter()
                .position(|request_id| request_id == &token.request_id)
                .map(|index| state.pending.remove(index))
                .is_some();
            let flag = token.flag();
            if cancelled_before_activation {
                flag.store(true, Ordering::Release);
            }
            state.active = Some((token.request_id.clone(), flag));
        }
    }

    pub(crate) fn finish(&self, request_id: &RequestId) {
        if let Ok(mut state) = self.state.lock() {
            if state
                .active
                .as_ref()
                .is_some_and(|(active_id, _)| active_id == request_id)
            {
                state.active = None;
            }
            state.pending.retain(|pending| pending != request_id);
        }
    }

    pub(crate) fn cancel(&self, request_id: &RequestId) {
        if let Ok(mut state) = self.state.lock() {
            if let Some((_, flag)) = state
                .active
                .as_ref()
                .filter(|(active_id, _)| active_id == request_id)
            {
                flag.store(true, Ordering::Release);
            } else if !state.pending.contains(request_id) {
                state.pending.push(request_id.clone());
            }
        }
    }

    pub(crate) fn clear(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = None;
            state.pending.clear();
        }
    }

    pub(crate) fn discard_pending(&self, request_id: &RequestId) {
        if let Ok(mut state) = self.state.lock() {
            state.pending.retain(|pending| pending != request_id);
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CancellationToken {
    request_id: RequestId,
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub(crate) fn new(request_id: &RequestId) -> Self {
        Self {
            request_id: request_id.clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    pub(crate) fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationRegistry, CancellationToken};
    use lsp_server::RequestId;
    use std::sync::atomic::Ordering;

    #[test]
    fn token_preserves_request_identity() {
        let id = RequestId::from("cancellation-token".to_string());
        let token = CancellationToken::new(&id);

        assert_eq!(token.request_id(), &id);
        assert!(!token.flag().load(Ordering::Acquire));
        token.cancel();
        assert!(token.flag().load(Ordering::Acquire));
    }

    #[test]
    fn registry_signals_only_the_active_request() {
        let registry = CancellationRegistry::default();
        let active_id = RequestId::from(1);
        let other_id = RequestId::from(2);
        let token = CancellationToken::new(&active_id);
        registry.activate(&token);

        registry.cancel(&other_id);
        assert!(!token.flag().load(Ordering::Acquire));
        registry.cancel(&active_id);
        assert!(token.flag().load(Ordering::Acquire));
        registry.finish(&active_id);
    }

    #[test]
    fn registry_preserves_cancellation_that_arrives_before_activation() {
        let registry = CancellationRegistry::default();
        let id = RequestId::from(3);
        registry.cancel(&id);
        let token = CancellationToken::new(&id);

        registry.activate(&token);

        assert!(token.is_cancelled());
        registry.finish(&id);
    }
}
