use super::{HirStmt, LowerCtx, Type, checked_place_subscript_failure, statement_diagnostics};
use crate::lower::expressions::lower_expr;
use crate::lower::type_bounds::reject_unavailable_dict_hash_key;
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, StmtDelete};

pub(super) fn lower_delete(del_stmt: &StmtDelete, ctx: &mut LowerCtx) -> Option<HirStmt> {
    if del_stmt.targets.len() != 1 {
        statement_diagnostics::unsupported_form(ctx, "del with multiple targets", del_stmt.range());
        return None;
    }
    let Expr::Subscript(subscript) = &del_stmt.targets[0] else {
        statement_diagnostics::unsupported_form(
            ctx,
            "del is only supported for collection items (del d[key], del a[i])",
            del_stmt.targets[0].range(),
        );
        return None;
    };
    let object = lower_expr(&subscript.value, ctx)?;
    let index = lower_expr(&subscript.slice, ctx)?;
    if reject_unavailable_dict_hash_key(
        object.ty(),
        index.ty(),
        "dict item deletion",
        subscript.range(),
        ctx,
    ) {
        return None;
    }
    let container_ty = object.ty().clone();
    if !matches!(
        container_ty.resolve_alias(),
        Type::List(_) | Type::Dict(_, _)
    ) {
        statement_diagnostics::unsupported_form(
            ctx,
            "del requires a list or dict item",
            subscript.range(),
        );
        return None;
    }
    let failure = checked_place_subscript_failure(
        ctx,
        &container_ty,
        true,
        "collection item deletion",
        subscript,
    );
    Some(HirStmt::Delete {
        object,
        index,
        failure,
    })
}
