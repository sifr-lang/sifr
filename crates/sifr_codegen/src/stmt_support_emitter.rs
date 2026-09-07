use crate::hir_analysis::queries;
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_ir::{HirExceptHandler, HirExpr, HirFStringPart, HirIteratorOp, HirStmt};
use sifr_type_system::Type;

#[macro_use]
mod stmt_expr_await_and_registry;
#[macro_use]
mod expr_call_and_literal_helpers;
mod expr_call_metadata;
pub(crate) use expr_call_metadata::{
    call_expr_parts, canonical_constructor_class_name, canonical_plain_call_name_for_ir,
    compiler_verified_pop_lowers_as_option_for_ir, generic_call_target_for_ir,
    plain_call_target_for_ir, should_force_mutable_binding, should_omit_local_type_annotation,
    type_contains_any_or_unknown, unwrap_compiler_verified_nonempty_pop_result_for_ir,
};
#[macro_use]
mod stmt_expr_stepped_slice;
#[macro_use]
mod stmt_expr_unit_slice;
#[macro_use]
mod stmt_expr_slice;
#[macro_use]
mod stmt_expr_wrappers_and_compare;
mod boolop_operand;
#[macro_use]
mod stmt_expr_binop;
mod stmt_expr_binop_option;
mod stmt_expr_method_and_question_mark;

mod assert_and_augassign;
mod assignment_validation;
mod async_cleanup;
mod async_with_and_for;
mod await_and_async_comprehension;
mod borrowed_operand_lowering;
mod call_args_and_returns;
pub(crate) mod checked_decimal_codegen;
pub(crate) mod checked_integer_codegen;
mod class_upcasts;
mod comprehension_exprs;
mod condition_lowering;
mod field_assignment;
mod if_condition_lowering;
mod iterator_lowering;
mod loops_try_finally;
#[cfg(test)]
mod loops_try_finally_tests;
mod native_async_context;
mod native_async_for;
mod nested_subscript_assignment_helpers;
pub(crate) mod performance_lowering_gate;
mod print_calls;
mod python_context;
mod recursive_constructor_args;
pub(crate) use recursive_constructor_args::RecursiveOptionConstructorArgContext;
mod none_comparison;
mod result_type_helpers;
mod singleton_repeat;
mod statement_output;
mod stmt_block;
mod stmt_block_helpers;
mod string_assignment;
mod structured_return_if_while;
mod subscript_augassign_delete;
mod try_error_helpers;
mod try_handlers;
mod tuple_unpack_block;
pub(crate) use result_type_helpers::integer_float_conversion_error_union;
use result_type_helpers::{
    is_none_like_result_value, is_result_int_division_error_type, result_int_to_sifr_int_rust_type,
};
pub(crate) use stmt_expr_binop_option::binop_with_optional_operands;
use try_error_helpers::{
    HandlerMatchCondition, can_construct_error_from_message_for_ir, first_try_error_type_in_stmts,
    io_error_kind_for_handler, select_try_error_type,
};
pub(crate) use try_error_helpers::{declaration_only_try_bindings, successful_try_bindings};
