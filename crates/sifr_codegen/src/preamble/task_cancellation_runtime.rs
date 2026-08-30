use super::RustItem;

pub fn build_task_cancellation_items(
    include_current_accessor: bool,
    include_cleanup_evidence: bool,
) -> Vec<RustItem> {
    let mut source = r#"tokio::task_local! {
    static __SIFR_TASK_CANCELLATION: ::sifr_runtime::cancellation::CancellationCarrier;
}

#[derive(Clone)]
struct __SifrCancellationCarrier {
    inner: ::sifr_runtime::cancellation::CancellationCarrier,
}

impl __SifrCancellationCarrier {
    fn new(
        inner: ::sifr_runtime::cancellation::CancellationCarrier,
        fallback_abort: tokio::task::AbortHandle,
    ) -> Self {
        let fallback = fallback_abort.clone();
        let _ = inner.bind_fallback(std::sync::Arc::new(move || fallback.abort()));
        Self { inner }
    }

    fn request_cancel(&self) -> ::sifr_runtime::cancellation::CancellationRequest {
        self.inner.request_cancel()
    }
}
"#
    .to_string();

    if include_current_accessor || include_cleanup_evidence {
        source.push_str(
            r#"
fn __sifr_current_task_cancellation(
) -> Option<::sifr_runtime::cancellation::CancellationCarrier> {
    __SIFR_TASK_CANCELLATION
        .try_with(Clone::clone)
        .ok()
}"#,
        );
    }

    if include_cleanup_evidence {
        source.push_str(
            r#"

fn __sifr_take_current_async_cleanup_secondary() -> Vec<SecondaryError> {
    __sifr_current_task_cancellation().map_or_else(Vec::new, |carrier| {
        carrier
            .take_async_cleanup_evidence()
            .into_iter()
            .map(SecondaryError::from_async_cleanup)
            .collect()
    })
}

impl __SifrCancellationCarrier {
    fn take_async_cleanup_secondary(&self) -> Vec<SecondaryError> {
        self.inner
            .take_async_cleanup_evidence()
            .into_iter()
            .map(SecondaryError::from_async_cleanup)
            .collect()
    }
}"#,
        );
    }

    vec![RustItem::Attr(source)]
}
