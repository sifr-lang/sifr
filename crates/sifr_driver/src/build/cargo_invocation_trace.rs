pub use sifr_package::CargoInvocation;
use sifr_package::CargoLockMode;
use std::process::Command;

/// Captures the exact Cargo argument vectors emitted by one synchronous driver
/// operation. This is public only so cross-crate certification tests can prove
/// that resolution, probes, and final builds preserve the requested lock mode.
#[doc(hidden)]
pub fn capture_cargo_invocations<T>(operation: impl FnOnce() -> T) -> (T, Vec<CargoInvocation>) {
    sifr_package::capture_cargo_invocations(operation)
}

pub(super) fn record_cargo_invocation(
    phase: &'static str,
    lock_mode: CargoLockMode,
    command: &Command,
) {
    sifr_package::record_cargo_invocation(phase, lock_mode, command);
}
