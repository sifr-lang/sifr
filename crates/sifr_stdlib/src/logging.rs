//! Native backing for `sifr.logging` process-global state.

use std::sync::{LazyLock, Mutex, MutexGuard};

use sifr_runtime::interop::SifrIntBridge;

static GLOBAL_LOG_LEVEL: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(20));

pub fn set_global_level(level: SifrIntBridge) {
    *global_log_level() = level.to_i64_saturating();
}

#[must_use]
pub fn get_global_level() -> SifrIntBridge {
    SifrIntBridge::from(*global_log_level())
}

fn global_log_level() -> MutexGuard<'static, i64> {
    GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
