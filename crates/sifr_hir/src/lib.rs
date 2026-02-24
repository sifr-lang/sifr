//! Sifr HIR (High-level Intermediate Representation)
//!
//! Lowers the untyped Python AST into a typed IR with:
//! - Name resolution (every name reference resolved to a definition)
//! - Type checking (every expression carries its resolved type)
//! - Ownership tracking (move vs copy semantics)

#![allow(
    clippy::single_match,
    clippy::single_match_else,
    clippy::redundant_else,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::implicit_clone,
    clippy::needless_pass_by_value,
    clippy::unnecessary_unwrap,
    clippy::bool_to_int_with_if,
    clippy::needless_bool,
    clippy::redundant_closure_for_method_calls,
    clippy::or_fun_call,
    clippy::useless_format,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::map_entry,
    clippy::unnecessary_to_owned,
    clippy::iter_on_single_items,
    clippy::match_single_binding,
    clippy::let_and_return,
    clippy::option_map_or_none,
    clippy::manual_is_variant_and,
    clippy::match_like_matches_macro,
    clippy::explicit_iter_loop,
    clippy::clone_on_copy,
    clippy::if_not_else,
    clippy::unwrap_or_default,
    clippy::uninlined_format_args,
    clippy::replace_box,
    clippy::used_underscore_binding,
    clippy::match_wildcard_for_single_variants,
    clippy::unnecessary_wraps,
    clippy::unnecessary_sort_by,
    clippy::manual_let_else,
    clippy::needless_range_loop,
    clippy::inefficient_to_string,
    clippy::assigning_clones
)]

pub mod cfg;
mod hir_nodes;
mod lower;
mod scope;
pub mod stdlib;

pub use hir_nodes::*;
pub use lower::{
    lower_module, lower_module_stdlib, lower_module_stdlib_with_externals,
    lower_module_with_externals, ExternalDefs, LoweringError, LoweringResult,
};
pub use scope::{NarrowingSnapshot, Scope};
