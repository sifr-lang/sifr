use crate::{RustExpr, RustLiteral, RustType};

mod mutability_and_clone_rewrites;
pub(crate) use mutability_and_clone_rewrites::*;
mod canonical_control_flow;
mod clone_chain_rewrite;
mod compiler_generated_mutating_methods;
pub(crate) use canonical_control_flow::simplify_control_flow_in_items;
mod dead_bindings;
pub(crate) use dead_bindings::remove_unread_pure_bindings_in_items;
mod optimization_helpers;
pub(crate) use optimization_helpers::*;
mod string_key_loop_rewrite;
