use super::PythonRuntimeError;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
static FORCE_CALLBACK_SHUTDOWN_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FORCE_ASYNC_CLEANUP_FAILURE: AtomicBool = AtomicBool::new(false);

pub(super) fn shutdown_registered_callbacks() -> Result<(), PythonRuntimeError> {
    #[cfg(test)]
    if FORCE_CALLBACK_SHUTDOWN_FAILURE.swap(false, Ordering::SeqCst) {
        return Err(PythonRuntimeError::AsyncRuntimeFailed(
            "forced callback shutdown failure".to_string(),
        ));
    }
    super::callbacks::shutdown_registered_callback_owners()
}

pub(super) fn run_registered_async_cleanup() -> Result<(), PythonRuntimeError> {
    #[cfg(test)]
    if FORCE_ASYNC_CLEANUP_FAILURE.swap(false, Ordering::SeqCst) {
        return Err(PythonRuntimeError::AsyncRuntimeFailed(
            "forced async cleanup failure".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn force_callback_shutdown_failure() {
    FORCE_CALLBACK_SHUTDOWN_FAILURE.store(true, Ordering::SeqCst);
}

#[cfg(test)]
pub(super) fn force_async_cleanup_failure() {
    FORCE_ASYNC_CLEANUP_FAILURE.store(true, Ordering::SeqCst);
}

#[cfg(test)]
pub(super) fn reset_for_tests() {
    FORCE_CALLBACK_SHUTDOWN_FAILURE.store(false, Ordering::SeqCst);
    FORCE_ASYNC_CLEANUP_FAILURE.store(false, Ordering::SeqCst);
}
