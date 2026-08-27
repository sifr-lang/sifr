//! Renderer for structured Rust IR nodes.

use crate::{
    RustExpr, RustFile, RustItem, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType,
    Visibility,
};
use std::fmt::Write as _;

mod render_core;
pub(crate) use render_core::*;
mod render_expr_and_blocks;
mod render_helpers;
pub use render_helpers::*;
mod render_paths;
