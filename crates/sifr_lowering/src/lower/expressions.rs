use super::async_await::coroutine_result_type;
use super::async_generator_advances::lower_anext_call;
use super::builtin_calls::{
    callable_builtin_element_type, lower_bytes_constructor_call, lower_chr_call,
    lower_dict_constructor_call, lower_isinstance_call, lower_len_call,
    lower_list_constructor_call, lower_ord_call, lower_range_call, lower_reveal_type_call,
    lower_set_constructor_call, lower_tuple_constructor_call, DEFAULTDICT_INT_ALIAS,
    DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS,
};
use super::bytes_methods::{resolve_bytes_method_type, resolve_str_encode_method_type};
use super::call_argument_ranges::{call_argument_ranges_by_param, type_param_argument_range};
use super::call_iterable_validation::{
    validate_dict_update_arg, validate_list_extend_arg, validate_set_iterable_arg,
};
use super::classes::is_hashable_type;
use super::decimal_methods::{
    decimal_conversion_error_type, lower_bigdecimal_constructor_call,
    lower_decimal_constructor_call, resolve_decimal_method_type,
};
use super::defaultdict_refinement::refine_defaultdict_binding_expr;
use super::diagnostics::list_append_argument_type_mismatch;
use super::empty_collection_refinement::{
    refine_empty_list_binding_expr, refine_empty_set_binding_expr,
};
use super::expression_abs::lower_abs_call;
use super::expression_diagnostics;
use super::expression_functional_builtins::{
    lower_any_all_call, lower_filter_call, lower_map_call, lower_zip_call,
};
use super::expression_iter_builtins::{lower_enumerate_call, lower_reversed_call};
use super::expression_sum_sorted::{lower_sorted_call, lower_sum_call};
use super::fixed_width_arithmetic_methods::resolve_fixed_width_method_type;
use super::generic_constructor_specialization::refine_constructor_return_type_from_args;
use super::generic_receiver_specialization::refine_generic_class_binding_expr;
use super::method_call_args::{
    lower_function_call_args, lower_method_call_args, lower_python_function_call_args,
    lower_signature_call_args,
};
use super::method_diagnostics::{
    method_count_range, reject_exact_method_arg_count, reject_max_method_arg_count,
    reject_method_arg_count, reject_no_method_args,
};
use super::min_max_validation::validate_variadic_min_max_operands;
use super::mutating_methods::{
    invalidate_collection_flow_facts_for_method, reject_immutable_parameter_method_mutation,
};
use super::name_diagnostics;
use super::nonempty_method_narrowing::refine_nonempty_method_return_type;
use super::numeric_sentinels::{
    float_sentinel_expr, float_sentinel_kind_from_call, normalize_min_max_numeric_sentinels,
};
use super::ownership_diagnostics;
use super::protocol_diagnostics;
use super::task_handle_calls::{is_task_handle_type, lower_task_handle_method_call};
use super::task_scope_calls as tsc;
use super::type_bounds::{type_satisfies_bound, type_satisfies_constraint};
use super::typevar_shape_compat::is_compatible_with_unresolved_typevars;
use super::typing_and_functions::resolve_annotation_expr;
use super::{
    async_await, async_comprehension_diagnostics, async_comprehensions, async_generator_methods,
    attribute_access, blocking_executor_calls, builtin_calls, collect_type_vars,
    container_literal_diagnostics, decode_typevar_constraint, expression_operators,
    fstring_support, if_expression, infer_type_var_bindings, integer_literals,
    sequence_guard_detection, str, subscript_type, substitute_type_vars, task_calls, tuple_unpack,
    workload_annotations, DiagnosticCode, Expr, ExprAttribute, ExprCall, ExprDictComp, ExprLambda,
    ExprListComp, ExprNamed, ExprSetComp, FunctionType, HashMap, HirExpr, HirIteratorOp, HirParam,
    LowerCtx, ParamConvention, Ranged, TextRange, Type,
};

mod core_and_calls;
pub(in crate::lower) use core_and_calls::*;
mod affine_resources;
use affine_resources::consume_affine_collection_method_arguments;
mod generator_expression;
pub(in crate::lower) use generator_expression::lower_generator_expr;
mod call_builtins;
use call_builtins::{lower_unshadowed_builtin_call, CallLowering};
mod call_shadowable_builtins;
use call_shadowable_builtins::lower_shadowable_builtin_call;
mod regular_calls;
use regular_calls::lower_regular_call;
mod methods_lambdas_and_comprehensions;
pub(in crate::lower) use methods_lambdas_and_comprehensions::*;
mod named_expression;
pub(in crate::lower) use named_expression::lower_named_expr;
mod method_type_collections;
use method_type_collections::{
    resolve_dict_method_type, resolve_list_method_type, resolve_set_method_type,
};
mod method_type_objects;
use method_type_objects::{
    resolve_bigint_method_type, resolve_class_method_type, resolve_enum_method_type,
    resolve_newtype_method_type, resolve_protocol_method_type, resolve_str_method_type,
    resolve_tuple_method_type, ClassMethodSurface,
};
mod python_buffer_methods;
use python_buffer_methods::{
    consume_python_buffer_release_receiver, resolve_python_buffer_method_type,
};
