//! IR-backed helpers for generating codegen preamble items.

use crate::{RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Visibility};

mod types_and_errors;
pub use types_and_errors::*;
mod task_runtime;
pub use task_runtime::*;
mod cpu_offload_runtime;
pub use cpu_offload_runtime::*;
mod parallel_runtime;
pub(crate) use parallel_runtime::*;
mod io_logging_random;
pub use io_logging_random::*;
mod io_bytes_methods;
pub(crate) use io_bytes_methods::*;
