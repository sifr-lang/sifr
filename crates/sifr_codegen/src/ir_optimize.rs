use crate::{RustExpr, RustLiteral, RustType};

mod mutability_and_clone_rewrites;
pub(crate) use mutability_and_clone_rewrites::*;
mod clone_chain_rewrite;
mod compiler_generated_mutating_methods;
mod optimization_helpers;
pub(crate) use optimization_helpers::*;
mod string_key_loop_rewrite;
