use super::builtin_calls::{DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use super::{LowerCtx, infer_type_var_bindings, substitute_type_vars};
use super::{str, typing_and_functions};
use sifr_python_ast::{CmpOp, Expr, ExprCall, Operator};
use sifr_type_system::{FunctionType, Type, type_check_binary_op};
use std::collections::HashMap;

mod call_effects;
mod state_collection;
pub(in crate::lower) use state_collection::*;
mod expression_inference;
use expression_inference::{
    analyze_assign, infer_expr_type, lookup_name_type, merge_env_types,
    refine_name_with_binary_context, unify_name_binding,
};
mod defaultdict_inference;
use defaultdict_inference::{
    DefaultdictMethodCall, defaultdict_shape_expr_is_lowering_exact, infer_defaultdict_call_type,
    is_unresolved_defaultdict_inference_type, refine_defaultdict_method_call,
    refine_defaultdict_subscript, unify_matching_defaultdict_aliases,
};
mod type_unification;
use type_unification::{
    has_conflicting_inference, replace_inference_holes_with_any, type_contains_unknown_or_any,
    unify_types,
};
mod generic_call_inference;
use generic_call_inference::infer_registered_call;
mod call_argument_inference;
use call_argument_inference::unify_inferred_call_arguments;
mod return_inference;
use return_inference::unify_function_return;
mod compound_statement_inference;
use compound_statement_inference::{
    analyze_match_stmt, analyze_try_stmt, analyze_with_stmt, collect_compound_local_bindings,
    collect_compound_nonlocals, function_has_value_return, inference_stmt_always_exits,
};
mod capture_collection;
pub(in crate::lower) use capture_collection::collect_referenced_names_in_expr;
mod mutation_collection;
pub(in crate::lower) use mutation_collection::collect_nested_function_mutated_nonlocals;
