//! IR-backed helpers for generating codegen preamble items.

use crate::{RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Visibility};

mod types_and_errors;
pub use types_and_errors::*;
mod task_context_runtime;
pub use task_context_runtime::*;
mod task_runtime;
pub use task_runtime::*;
mod task_scope_offload_runtime;
pub use task_scope_offload_runtime::*;
mod cpu_offload_runtime;
pub use cpu_offload_runtime::*;
mod join_set_runtime;
pub use join_set_runtime::*;
mod parallel_runtime;
pub(crate) use parallel_runtime::*;
mod process_async_child_runtime;
mod process_async_runtime;
mod process_child_pipes;
mod process_runtime;
pub(crate) use process_async_runtime::*;
pub(crate) use process_runtime::*;
mod io_logging_random;
pub use io_logging_random::*;
mod io_bytes_methods;
pub(crate) use io_bytes_methods::*;
