use crate::CargoLockMode;
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static CAPTURE_SERIALIZER: Mutex<()> = Mutex::new(());
static CAPTURED: Mutex<Vec<CargoInvocation>> = Mutex::new(Vec::new());
static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoInvocation {
    pub phase: &'static str,
    pub lock_mode: CargoLockMode,
    pub args: Vec<String>,
}

/// Captures exact Cargo argument vectors emitted by one synchronous operation.
///
/// This is public only so cross-crate certification tests can prove that
/// package metadata, generated resolution, Rust probes, and final builds
/// preserve the selected lock mode.
#[doc(hidden)]
pub fn capture_cargo_invocations<T>(operation: impl FnOnce() -> T) -> (T, Vec<CargoInvocation>) {
    let _capture_guard = CAPTURE_SERIALIZER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    CAPTURED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    CAPTURE_ACTIVE.store(true, Ordering::Release);
    let active = ActiveCapture;
    let result = operation();
    drop(active);
    let invocations = std::mem::take(
        &mut *CAPTURED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    (result, invocations)
}

#[doc(hidden)]
pub fn record_cargo_invocation(phase: &'static str, lock_mode: CargoLockMode, command: &Command) {
    if !CAPTURE_ACTIVE.load(Ordering::Acquire) {
        return;
    }
    CAPTURED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(CargoInvocation {
            phase,
            lock_mode,
            args: command
                .get_args()
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect(),
        });
}

struct ActiveCapture;

impl Drop for ActiveCapture {
    fn drop(&mut self) {
        CAPTURE_ACTIVE.store(false, Ordering::Release);
    }
}
