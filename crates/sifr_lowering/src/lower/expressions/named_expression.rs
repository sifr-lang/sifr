use super::{
    consume_affine_value_name, lower_expr, reject_invalid_expression_target, Expr, ExprNamed,
    HirExpr, LowerCtx, Ranged,
};
use crate::lower::python_interop::reject_python_context_borrow_storage;
use sifr_diagnostics::DiagnosticCode;

pub(in crate::lower) fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = if let Expr::Name(name) = named.target.as_ref() {
        name.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "walrus operator target must be a simple name",
            named.target.range(),
        );
        return None;
    };

    let value = lower_expr(&named.value, ctx)?;
    reject_python_context_borrow_storage(&value, named.value.range(), ctx);
    let ty = value.ty().clone();
    if ty.contains_affine_resource() {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "walrus target '{name}' cannot bind a value containing an affine Python buffer because the assignment expression result would create a second owner"
            ),
            named.range(),
        );
        return None;
    }
    consume_affine_value_name(&value, named.value.range(), ctx);
    ctx.scope.define(name.clone(), ty.clone());

    Some(HirExpr::WalrusExpr {
        name,
        value: Box::new(value),
        ty,
    })
}
