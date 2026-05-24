//! Collections intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustStmt, RustType};

mod set_and_list_intrinsics;
pub(super) use set_and_list_intrinsics::*;
mod counter_defaultdict_intrinsics;
pub(super) use counter_defaultdict_intrinsics::*;
