use super::RustItem;

pub fn build_task_cancellation_items() -> Vec<RustItem> {
    vec![RustItem::Attr(
        r#"tokio::task_local! {
    static __SIFR_TASK_CANCELLATION: sifr_runtime::cancellation::CancellationCarrier;
}

const __SIFR_COOPERATIVE_SUPERVISORS_READY: bool = false;

#[derive(Clone)]
struct __SifrCancellationCarrier {
    inner: sifr_runtime::cancellation::CancellationCarrier,
    fallback_abort: tokio::task::AbortHandle,
}

impl __SifrCancellationCarrier {
    fn new(
        inner: sifr_runtime::cancellation::CancellationCarrier,
        fallback_abort: tokio::task::AbortHandle,
    ) -> Self {
        let fallback = fallback_abort.clone();
        let _ = inner.bind_fallback(std::sync::Arc::new(move || fallback.abort()));
        Self {
            inner,
            fallback_abort,
        }
    }

    fn request_cancel(&self) -> sifr_runtime::cancellation::CancellationRequest {
        self.inner.request_cancel()
    }

    fn abort(&self) {
        debug_assert!(
            !self.inner.is_claimed().unwrap_or(false),
            "fallback-only task supervisor observed a claimed cancellation carrier",
        );
        if self.inner.is_claimed().unwrap_or(false) {
            let _ = self.inner.request_cancel();
        } else {
            self.fallback_abort.abort();
        }
    }

    fn abort_handle(&self) -> tokio::task::AbortHandle {
        self.fallback_abort.clone()
    }
}

#[allow(dead_code)]
fn __sifr_claim_current_task_cancellation(
    hook: sifr_runtime::cancellation::CancellationHook,
) -> Option<sifr_runtime::cancellation::CancellationClaim> {
    if !__SIFR_COOPERATIVE_SUPERVISORS_READY {
        return None;
    }
    __SIFR_TASK_CANCELLATION
        .try_with(|carrier| carrier.claim(hook))
        .ok()
}"#
        .to_string(),
    )]
}
