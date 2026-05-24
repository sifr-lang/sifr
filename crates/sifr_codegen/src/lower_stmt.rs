//! Statement lowering scaffolds for the IR lowering.

use crate::helpers::{
    codegen_body_always_exits, detect_and_not_none_vars, detect_is_none_var,
    detect_is_not_none_var, detect_or_is_none_vars,
};
use crate::hir_analysis::queries::{
    body_calls_function, collect_locally_defined_vars, collect_mutated_vars,
    collect_referenced_vars_with_types,
};
use crate::{
    try_lower_leaf_expr, try_lower_leaf_expr_result, CodegenError, RustExpr, RustLiteral,
    RustMatchArm, RustParam, RustStmt, RustType, ScopeContext,
};
use sifr_hir::{
    HirExceptHandler, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt, MethodKind,
};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

mod candidate_and_validation;
pub use candidate_and_validation::*;
mod simple_dispatch_and_bindings;
pub(crate) use simple_dispatch_and_bindings::*;
mod try_tuple_flow;
pub(crate) use try_tuple_flow::*;
mod with_yield_and_match;
use with_yield_and_match::{
    try_lower_loop_else_stmts, try_lower_simple_async_with_stmt, try_lower_simple_match_stmt,
    try_lower_simple_with_stmt, try_lower_simple_yield_stmt,
};
mod loop_lowering;
use loop_lowering::{try_lower_simple_for_stmt, try_lower_simple_while_stmt, SimpleForStmtParts};
mod condition_lowering;
use condition_lowering::{
    is_alias_equivalent_type, is_none_type, is_okwrap_none_expr, is_option_like_type,
    resolve_alias_type, try_lower_attribute_dict_insert_key_expr, try_lower_leaf_or_name_expr,
    try_lower_name_ident_expr, try_lower_simple_condition_test_expr, try_lower_simple_if_stmt,
};
mod subscript_return_assignment;
pub(crate) use subscript_return_assignment::*;

#[cfg(test)]
mod augassign_edge_tests;
#[cfg(test)]
mod binding_and_augassign_tests;
#[cfg(test)]
mod core_and_tuple_tests;
#[cfg(test)]
mod loop_and_context_tests;
#[cfg(test)]
mod match_nested_try_tests;
#[cfg(test)]
mod return_assert_if_tests;
#[cfg(test)]
mod return_raise_assert_tests;
#[cfg(test)]
mod subscript_assignment_tests;
#[cfg(test)]
mod subscript_augassign_tests;
