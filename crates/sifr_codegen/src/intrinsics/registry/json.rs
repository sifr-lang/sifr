//! JSON intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustMatchArm, RustParam, RustStmt, RustType};

mod decode_and_value_intrinsics;
pub(super) use decode_and_value_intrinsics::*;
mod encode_profile_intrinsics;
pub(super) use encode_profile_intrinsics::*;
