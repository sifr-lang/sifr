use super::{
    Expr, HirExpr, LowerCtx, Ranged, Type, callable_builtin_element_type, lower_expr,
    lower_iterator_protocol_entry, reject_invalid_expression_iteration,
    reject_invalid_expression_target, reject_unsupported_expression_form,
};
use crate::lower::{ExprGenerator, ownership_diagnostics, statement_diagnostics};

pub(in crate::lower) fn lower_generator_expr(
    generator: &ExprGenerator,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if generator
        .generators
        .iter()
        .any(|generator| generator.is_async)
    {
        super::async_comprehension_diagnostics::reject_async_generator_expression(
            ctx,
            generator.range(),
        );
        return None;
    }

    if generator.generators.len() != 1 {
        reject_unsupported_expression_form(
            ctx,
            "only single-generator generator expressions are supported",
            generator.range(),
        );
        return None;
    }

    let comp = &generator.generators[0];
    let var_name = if let Expr::Name(name) = &comp.target {
        name.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "generator target must be a simple name",
            comp.target.range(),
        );
        return None;
    };
    let iter_source_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
        reject_invalid_expression_iteration(ctx, &iter_ty, comp.iter.range());
        return None;
    };
    if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, comp.iter.range()) {
        return None;
    }

    let moved_before_loop = ctx.scope.save_moved_state();
    let lowered = ctx.with_pushed_scope(|ctx| {
        ctx.scope.define_ephemeral(
            var_name.clone(),
            elem_ty.clone(),
            crate::scope::EphemeralOrigin::Comprehension,
        );
        let expr = lower_expr(&generator.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        if statement_diagnostics::reject_affine_comprehension_value(
            ctx,
            &expr_ty,
            generator.elt.range(),
        ) {
            return None;
        }
        let filter = if comp.ifs.is_empty() {
            None
        } else {
            let first = lower_expr(&comp.ifs[0], ctx)?;
            if comp.ifs.len() == 1 {
                Some(Box::new(first))
            } else {
                let mut combined = first;
                for condition in &comp.ifs[1..] {
                    let next = lower_expr(condition, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(Box::new(combined))
            }
        };
        Some((expr, expr_ty, filter))
    });
    ownership_diagnostics::report_moved_across_loop(ctx, &moved_before_loop, generator.range());
    let (expr, expr_ty, filter) = lowered?;
    let result_ty = Type::Iterator(Box::new(expr_ty));
    let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}
