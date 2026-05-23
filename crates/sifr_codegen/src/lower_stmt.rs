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

include!("lower_stmt/candidate_and_validation.rs");
include!("lower_stmt/simple_dispatch_and_bindings.rs");
include!("lower_stmt/try_tuple_flow.rs");
include!("lower_stmt/with_yield_and_match.rs");
include!("lower_stmt/loop_lowering.rs");
include!("lower_stmt/condition_lowering.rs");
include!("lower_stmt/subscript_return_assignment.rs");

#[cfg(test)]
mod tests {
    include!("lower_stmt/core_and_tuple_tests.rs");
    include!("lower_stmt/subscript_assignment_tests.rs");
    include!("lower_stmt/subscript_augassign_tests.rs");
    include!("lower_stmt/binding_and_augassign_tests.rs");
    include!("lower_stmt/augassign_edge_tests.rs");
    include!("lower_stmt/return_raise_assert_tests.rs");
    include!("lower_stmt/return_assert_if_tests.rs");
    include!("lower_stmt/loop_and_context_tests.rs");
    include!("lower_stmt/match_nested_try_tests.rs");
}
