use super::{infer_type_var_bindings, substitute_type_vars, LowerCtx};
use super::{str, typing_and_functions};
use sifr_python_ast::{CmpOp, Expr, ExprCall, Operator};
use sifr_type_system::{type_check_binary_op, FunctionType, Type};
use std::collections::HashMap;

mod state_collection;
pub(in crate::lower) use state_collection::*;
mod expression_inference;
use expression_inference::{
    analyze_assign, infer_expr_type, merge_env_types, refine_name_with_binary_context,
    type_contains_unknown_or_any, unify_name_binding, unify_types,
};
mod generic_call_inference;
use generic_call_inference::infer_registered_call;
mod return_inference;
use return_inference::unify_function_return;
mod compound_statement_inference;
use compound_statement_inference::{
    analyze_match_stmt, analyze_try_stmt, analyze_with_stmt, collect_compound_local_bindings,
    collect_compound_nonlocals, function_has_value_return, inference_stmt_always_exits,
};
mod capture_collection;
pub(in crate::lower) use capture_collection::collect_referenced_names_in_expr;
