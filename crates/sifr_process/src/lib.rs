//! Owned subprocesses and scoped cancellation, without compiler dependencies.
#[cfg_attr(windows, path = "process_execution_windows.rs")]
pub mod process_execution;
#[cfg_attr(windows, path = "process_signals_windows.rs")]
pub mod process_signals;
pub use process_execution::OwnedChild;

mod compiled_identity;
#[doc(hidden)]
pub use compiled_identity::compiled_input_tokens;

mod worker_exchange;
pub use worker_exchange::exchange_with_deadline;

#[cfg(unix)]
mod socket_readiness;
#[cfg(unix)]
pub use socket_readiness::{SocketReadiness, socket_readiness};
