use crate::hir_analysis::queries;
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_hir::{HirExceptHandler, HirExpr, HirFStringPart, HirIteratorOp, HirStmt};
use sifr_type_system::Type;

#[macro_use]
mod expr_call_and_literal_helpers;
pub(crate) use expr_call_and_literal_helpers::*;
#[macro_use]
mod stmt_expr_slice;
#[macro_use]
mod stmt_expr_wrappers_and_compare;
#[macro_use]
mod stmt_expr_binop;
mod stmt_expr_method_and_question_mark;

mod await_and_async_comprehension;
mod comprehension_and_nested_subscript;
mod condition_lowering;
mod iterator_lowering;
mod loops_try_finally;
mod print_calls_and_returns;
mod stmt_block;
mod structured_return_if_while;
mod subscript_augassign_delete;
mod try_handlers_and_cleanup;
mod with_async_and_if;
use try_handlers_and_cleanup::{
    inject_async_for_early_exit_cleanup, inject_async_with_return_cleanup,
    is_result_int_division_error_type, result_int_to_sifr_int_rust_type,
};
