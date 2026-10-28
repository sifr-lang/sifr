//! Secure lease and publication primitives for compiler-owned cache entries.
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub mod windows_storage_security;
#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
mod child_lease;
#[cfg(unix)]
pub use child_lease::{inherit_lease_for_command, inherit_lease_for_private_command};

mod process_lease;
pub use process_lease::ProcessLease;

#[cfg(all(test, unix))]
mod owned_payload_tests;
