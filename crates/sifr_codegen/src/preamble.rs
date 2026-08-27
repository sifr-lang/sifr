//! IR-backed helpers for generating codegen preamble items.

use crate::{RustExpr, RustItem, RustMatchArm, RustParam, RustStmt, RustType, Visibility};

mod types_and_errors;
pub use types_and_errors::*;
mod task_context_runtime;
mod type_validation;
pub use task_context_runtime::*;
pub(crate) use type_validation::validate_codegen_module_types;
mod task_cancellation_runtime;
pub use task_cancellation_runtime::*;
mod task_runtime;
pub use task_runtime::*;
mod task_scope_offload_runtime;
pub use task_scope_offload_runtime::*;
mod task_supervisor_runtime;
pub use task_supervisor_runtime::*;
mod cpu_offload_runtime;
pub use cpu_offload_runtime::*;
mod join_set_runtime;
pub use join_set_runtime::*;
mod parallel_runtime;
pub(crate) use parallel_runtime::*;
mod io_file_handles;
pub use io_file_handles::*;
mod io_bytes_methods;
pub(crate) use io_bytes_methods::*;
