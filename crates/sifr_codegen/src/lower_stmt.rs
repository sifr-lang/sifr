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
use sifr_ir::{
    HirExceptHandler, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt, MethodKind,
};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

mod candidate_and_validation;
#[cfg(test)]
use candidate_and_validation::try_lower_simple_stmt_with_scope_result;
pub(crate) use candidate_and_validation::{
    is_simple_stmt_candidate, try_lower_simple_stmt_with_scope_result_and_bindings,
};
pub use candidate_and_validation::{
    try_lower_expr_stmt, try_lower_simple_stmt, SimpleStmtLoweringCtx,
};
use candidate_and_validation::{try_lower_expr_stmt_with_bindings, SimpleStmtBindings};
mod simple_dispatch_and_bindings;
pub(crate) use simple_dispatch_and_bindings::{
    try_lower_simple_stmt_with_ctx, try_lower_simple_stmt_with_ctx_and_bindings,
};
mod try_tuple_flow;
pub(crate) use try_tuple_flow::{lower_tuple_unpack_targets, tuple_unpack_source_is_borrowed};
use try_tuple_flow::{
    try_lower_simple_star_unpack_stmt, try_lower_simple_stmt_block,
    try_lower_simple_try_except_stmt, try_lower_simple_tuple_unpack_stmt,
};
mod with_yield_and_match;
use with_yield_and_match::{
    try_lower_loop_else_stmts, try_lower_simple_async_with_stmt, try_lower_simple_match_stmt,
    try_lower_simple_with_stmt, try_lower_simple_yield_stmt,
};
mod loop_lowering;
use loop_lowering::{try_lower_simple_for_stmt, try_lower_simple_while_stmt, SimpleForStmtParts};
mod condition_lowering;
use condition_lowering::{try_lower_simple_condition_test_expr, try_lower_simple_if_stmt};
mod condition_type_and_expr_helpers;
use condition_type_and_expr_helpers::{
    detect_option_truthiness_alias, is_alias_equivalent_type, is_none_type, is_okwrap_none_expr,
    is_option_like_type, lower_if_not_none_chain, option_binding_pattern, resolve_alias_type,
    try_lower_attribute_dict_insert_key_expr, try_lower_leaf_or_name_expr,
    try_lower_name_ident_expr,
};
mod return_and_assignment_values;
use return_and_assignment_values::{
    try_lower_simple_assign_value, try_lower_simple_augassign_stmt,
    try_lower_simple_field_assign_stmt, try_lower_simple_let_value, try_lower_simple_return_stmt,
};
mod subscript_assignment;
pub(crate) use subscript_assignment::{
    build_dict_subscript_assign_stmt, build_list_subscript_assign_stmt,
    build_normalized_list_index_i64_expr,
};
use subscript_assignment::{
    try_lower_simple_attribute_nested_subscript_assign_stmt,
    try_lower_simple_attribute_subscript_assign_stmt, try_lower_simple_delete_stmt,
    try_lower_simple_nested_subscript_assign_stmt, try_lower_simple_subscript_assign_stmt,
    try_lower_simple_subscript_augassign_stmt,
};

#[cfg(test)]
mod augassign_edge_tests;
#[cfg(test)]
mod augassign_tests;
#[cfg(test)]
mod binding_tests;
#[cfg(test)]
mod core_and_tuple_tests;
#[cfg(test)]
mod for_loop_tests;
#[cfg(test)]
mod match_nested_try_tests;
#[cfg(test)]
mod raise_assert_tests;
#[cfg(test)]
mod return_assert_if_tests;
#[cfg(test)]
mod return_tests;
#[cfg(test)]
mod subscript_assignment_tests;
#[cfg(test)]
mod subscript_augassign_assignment_tests;
#[cfg(test)]
mod subscript_augassign_tests;
#[cfg(test)]
mod while_loop_tests;
#[cfg(test)]
mod yield_unpack_with_tests;
