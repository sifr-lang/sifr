//! Expression lowering scaffolds for the IR lowering.

use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_hir::{HirExpr, HirFStringPart, HirParam};
use sifr_type_system::{ParamConvention, Type};

mod leaves_and_plain_calls;
pub use leaves_and_plain_calls::*;
mod iterators_and_callables;
use iterators_and_callables::{
    try_lower_dict_get_key_expr, try_lower_simple_constructor_call_expr,
    try_lower_simple_defaultdict_index_expr, try_lower_simple_divmod_call_expr,
    try_lower_simple_filter_call_expr, try_lower_simple_iter_source_expr,
    try_lower_simple_map_call_expr, try_lower_simple_method_call_expr,
};
mod collections_and_comprehensions;
use collections_and_comprehensions::{
    detect_is_some_guard_name, is_bool_like_simple, is_enum_like_simple, is_int_like_simple,
    is_mixed_simple_float_binop, is_mixed_simple_float_floor_division_binop, is_numeric_simple,
    is_option_like_simple, is_promoted_fixed_width_integer_binop, is_reserved_builtin_call_func,
    is_safe_simple_binop, is_safe_simple_compare, is_simple_int_true_division_binop,
    is_string_like_simple, normalize_binop_op, normalize_compare_op, resolve_alias_type,
    try_lower_guarded_option_compare_expr, try_lower_mixed_float_operand_expr,
    try_lower_none_identity_compare_expr, try_lower_option_none_compare_expr,
    try_lower_promoted_integer_operand_expr, try_lower_simple_binop_operand_expr,
    try_lower_simple_compare_operand_expr, try_lower_simple_dict_comp_expr,
    try_lower_simple_dict_literal_expr, try_lower_simple_fstring_expr,
    try_lower_simple_generator_expr, try_lower_simple_index_expr, try_lower_simple_lambda_expr,
    try_lower_simple_list_comp_expr, try_lower_simple_range_operand_expr,
    try_lower_simple_set_comp_expr, try_lower_simple_set_literal_expr, try_lower_simple_slice_expr,
};

#[cfg(test)]
mod comprehension_and_misc_tests;
#[cfg(test)]
mod leaves_and_compound_tests;
#[cfg(test)]
mod option_compare_tests;
