use super::RustItem;

pub fn build_task_cancellation_items() -> Vec<RustItem> {
    vec![RustItem::Attr(
        r#"tokio::task_local! {
    static __SIFR_TASK_CANCELLATION: sifr_runtime::cancellation::CancellationCarrier;
}

#[derive(Clone)]
struct __SifrCancellationCarrier {
    inner: sifr_runtime::cancellation::CancellationCarrier,
}

impl __SifrCancellationCarrier {
    fn new(
        inner: sifr_runtime::cancellation::CancellationCarrier,
        fallback_abort: tokio::task::AbortHandle,
    ) -> Self {
        let fallback = fallback_abort.clone();
        let _ = inner.bind_fallback(std::sync::Arc::new(move || fallback.abort()));
        Self { inner }
    }

    fn request_cancel(&self) -> sifr_runtime::cancellation::CancellationRequest {
        self.inner.request_cancel()
    }
}

#[allow(dead_code)]
fn __sifr_current_task_cancellation(
) -> Option<sifr_runtime::cancellation::CancellationCarrier> {
    __SIFR_TASK_CANCELLATION
        .try_with(Clone::clone)
        .ok()
}"#
        .to_string(),
    )]
}
