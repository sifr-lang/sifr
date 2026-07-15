use super::{
    is_direct_type, parameter_metadata, parse_method_target_path, receiver_is_owned, LowerCtx,
};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirExpr, HirFStringPart, HirFunction, HirParam, HirWithItemKind, PythonCleanupPolicy,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
};
use sifr_python_ast::{Expr, ExprCall, Parameters};
use sifr_type_system::Type;

mod async_with;
mod borrows;
mod declarations;

pub(in crate::lower) use async_with::try_lower_python_async_with;
pub(in crate::lower) use borrows::{
    lower_python_context_owned_expr, python_context_borrow_in_owned_expr,
    python_context_borrow_reference, reject_python_context_borrow_created_value,
    reject_python_context_borrow_discard, reject_python_context_borrow_storage,
};
pub(in crate::lower) use declarations::{parse_context_method, validate_context_method_signature};
pub(in crate::lower) use declarations::{python_context_item_kind, validate_context_class_methods};

fn invalid_context(ctx: &mut LowerCtx, reason: &str, span: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCTX_INVALID_DECLARATION,
        format!("invalid Python context declaration: {reason}"),
        span,
    );
}
