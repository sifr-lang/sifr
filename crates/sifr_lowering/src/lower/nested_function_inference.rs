use super::LowerCtx;
use super::{str, typing_and_functions};
use sifr_python_ast::{CmpOp, Expr, ExprCall, Operator};
use sifr_type_system::{type_check_binary_op, Type};
use std::collections::HashMap;

mod state_collection;
pub(in crate::lower) use state_collection::*;
mod expression_inference;
use expression_inference::{
    analyze_assign, infer_expr_type, merge_env_types, refine_name_with_binary_context,
    type_contains_unknown_or_any, unify_function_return, unify_name_binding, unify_types,
};
mod capture_collection;
pub(in crate::lower) use capture_collection::collect_referenced_names_in_expr;
