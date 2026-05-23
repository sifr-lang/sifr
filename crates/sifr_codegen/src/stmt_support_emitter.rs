use crate::hir_analysis::queries;
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_hir::{HirExceptHandler, HirExpr, HirFStringPart, HirIteratorOp, HirStmt};
use sifr_type_system::Type;

include!("stmt_support_emitter/expr_call_and_literal_helpers.rs");
include!("stmt_support_emitter/stmt_expr_slice.rs");
include!("stmt_support_emitter/stmt_expr_wrappers_and_compare.rs");
include!("stmt_support_emitter/stmt_expr_binop.rs");
include!("stmt_support_emitter/stmt_expr_method_and_question_mark.rs");

include!("stmt_support_emitter/await_and_async_comprehension.rs");
include!("stmt_support_emitter/comprehension_and_nested_subscript.rs");
include!("stmt_support_emitter/subscript_augassign_delete.rs");
include!("stmt_support_emitter/print_calls_and_returns.rs");
include!("stmt_support_emitter/stmt_block.rs");
include!("stmt_support_emitter/condition_lowering.rs");
include!("stmt_support_emitter/iterator_lowering.rs");
include!("stmt_support_emitter/with_async_and_if.rs");
include!("stmt_support_emitter/structured_return_if_while.rs");
include!("stmt_support_emitter/loops_try_finally.rs");
include!("stmt_support_emitter/try_handlers_and_cleanup.rs");
