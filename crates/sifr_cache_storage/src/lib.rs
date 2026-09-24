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
