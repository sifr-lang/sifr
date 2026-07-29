use std::sync::atomic::{AtomicU64, Ordering};

static PROBE_NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn unique_probe_nonce() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = PROBE_NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp}_{counter}")
}
