use super::expressions::lower_expr;
use super::{ownership_diagnostics, LowerCtx};
use crate::hir_nodes::HirExpr;
use crate::scope::MovedSnapshot;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprDictComp, ExprListComp, ExprSetComp};
use sifr_type_system::Type;

pub(in crate::lower) fn lower_list_comp(
    comp: &ExprListComp,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !comp.generators.iter().any(|generator| generator.is_async) {
        return None;
    }

    if super::async_comprehension_diagnostics::reject_unsupported_basic_async_comprehension_shape(
        ctx,
        &comp.generators,
        comp.range(),
    ) {
        return Some(None);
    }
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async list comprehensions are only valid inside async functions".to_string(),
            comp.range(),
        );
        return Some(None);
    }

    let generator = &comp.generators[0];
    let var_name = if let Expr::Name(name) = &generator.target {
        name.id.to_string()
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::FLOW_INVALID_ITERATION,
            "async list comprehension target must be a simple name".to_string(),
            generator.target.range(),
        );
        return Some(None);
    };
    let iter_source_expr = lower_expr(&generator.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some((elem_ty, iter_error_ty)) = super::async_for::async_iterator_parts(&iter_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "async list comprehension requires AsyncIterator[T, E] with anext() -> Result[Option[T], E], got '{}'",
                iter_ty.display_name()
            ),
            generator.iter.range(),
        );
        return Some(None);
    };
    if super::statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, generator.iter.range())
    {
        return Some(None);
    }
    let return_type = ctx
        .current_function_return_type
        .clone()
        .unwrap_or(Type::None);
    if !super::async_for::return_type_accepts_error(&return_type, &iter_error_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "fallible async list comprehension requires the enclosing function to return a compatible Result error type".to_string(),
            generator.iter.range(),
        );
        return Some(None);
    }

    let moved_before_loop = ctx.scope.save_moved_state();
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty);
    let result = (|| {
        let filter = if generator.ifs.is_empty() {
            None
        } else {
            let first = lower_expr(&generator.ifs[0], ctx)?;
            if generator.ifs.len() == 1 {
                Some(first)
            } else {
                let mut combined = first;
                for cond in &generator.ifs[1..] {
                    let next = lower_expr(cond, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(combined)
            }
        };
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        if super::statement_diagnostics::reject_affine_comprehension_value(
            ctx,
            &expr_ty,
            comp.elt.range(),
        ) {
            return None;
        }
        Some(HirExpr::ListComp {
            expr: Box::new(expr),
            generators: vec![(var_name, iter_source_expr, filter)],
            ty: Type::List(Box::new(expr_ty)),
        })
    })();
    ctx.scope.pop();
    ownership_diagnostics::report_moved_across_loop(ctx, &moved_before_loop, comp.range());
    Some(result)
}

pub(in crate::lower) fn lower_set_comp(
    comp: &ExprSetComp,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !comp.generators.iter().any(|generator| generator.is_async) {
        return None;
    }
    let Some((var_name, iter_source_expr, filter, moved_before_loop)) =
        lower_single_async_generator(&comp.generators, comp.range(), ctx)
    else {
        return Some(None);
    };

    let elem_ty = super::async_for::async_iterator_parts(iter_source_expr.ty())
        .map(|(elem_ty, _)| elem_ty)
        .unwrap_or(Type::Any);
    if super::statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, comp.range()) {
        return Some(None);
    }
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty);
    let result = (|| {
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        if super::statement_diagnostics::reject_affine_comprehension_value(
            ctx,
            &expr_ty,
            comp.elt.range(),
        ) || super::container_literal_diagnostics::reject_unhashable_container_type(
            ctx,
            "set comprehension element",
            &expr_ty,
            comp.elt.range(),
        ) {
            return None;
        }
        Some(HirExpr::SetComp {
            expr: Box::new(expr),
            generators: vec![(var_name, iter_source_expr, filter)],
            ty: Type::Set(Box::new(expr_ty)),
        })
    })();
    ctx.scope.pop();
    ownership_diagnostics::report_moved_across_loop(ctx, &moved_before_loop, comp.range());
    Some(result)
}

pub(in crate::lower) fn lower_dict_comp(
    comp: &ExprDictComp,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    if !comp.generators.iter().any(|generator| generator.is_async) {
        return None;
    }
    let Some((var_name, iter_source_expr, filter, moved_before_loop)) =
        lower_single_async_generator(&comp.generators, comp.range(), ctx)
    else {
        return Some(None);
    };

    let elem_ty = super::async_for::async_iterator_parts(iter_source_expr.ty())
        .map(|(elem_ty, _)| elem_ty)
        .unwrap_or(Type::Any);
    if super::statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, comp.range()) {
        return Some(None);
    }
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty);
    let result = (|| {
        let key_expr = lower_expr(&comp.key, ctx)?;
        let val_expr = lower_expr(&comp.value, ctx)?;
        let key_ty = key_expr.ty().clone();
        let val_ty = val_expr.ty().clone();
        if super::container_literal_diagnostics::reject_unhashable_container_type(
            ctx,
            "dict comprehension key",
            &key_ty,
            comp.key.range(),
        ) || super::statement_diagnostics::reject_affine_comprehension_value(
            ctx,
            &val_ty,
            comp.value.range(),
        ) {
            return None;
        }
        Some(HirExpr::DictComp {
            key_expr: Box::new(key_expr),
            val_expr: Box::new(val_expr),
            generators: vec![(var_name, iter_source_expr, filter)],
            ty: Type::Dict(Box::new(key_ty), Box::new(val_ty)),
        })
    })();
    ctx.scope.pop();
    ownership_diagnostics::report_moved_across_loop(ctx, &moved_before_loop, comp.range());
    Some(result)
}

fn lower_single_async_generator(
    generators: &[sifr_python_ast::Comprehension],
    fallback_range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) -> Option<(String, HirExpr, Option<HirExpr>, MovedSnapshot)> {
    if super::async_comprehension_diagnostics::reject_unsupported_basic_async_comprehension_shape(
        ctx,
        generators,
        fallback_range,
    ) {
        return None;
    }
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async comprehensions are only valid inside async functions".to_string(),
            fallback_range,
        );
        return None;
    }

    let generator = &generators[0];
    let var_name = if let Expr::Name(name) = &generator.target {
        name.id.to_string()
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::FLOW_INVALID_ITERATION,
            "async comprehension target must be a simple name".to_string(),
            generator.target.range(),
        );
        return None;
    };
    let iter_source_expr = lower_expr(&generator.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some((elem_ty, iter_error_ty)) = super::async_for::async_iterator_parts(&iter_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "async comprehension requires AsyncIterator[T, E] with anext() -> Result[Option[T], E], got '{}'",
                iter_ty.display_name()
            ),
            generator.iter.range(),
        );
        return None;
    };
    let return_type = ctx
        .current_function_return_type
        .clone()
        .unwrap_or(Type::None);
    if !super::async_for::return_type_accepts_error(&return_type, &iter_error_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "fallible async comprehension requires the enclosing function to return a compatible Result error type".to_string(),
            generator.iter.range(),
        );
        return None;
    }

    let moved_before_loop = ctx.scope.save_moved_state();
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty);
    let filter = if generator.ifs.is_empty() {
        None
    } else {
        let first = lower_expr(&generator.ifs[0], ctx)?;
        if generator.ifs.len() == 1 {
            Some(first)
        } else {
            let mut combined = first;
            for cond in &generator.ifs[1..] {
                let next = lower_expr(cond, ctx)?;
                combined = HirExpr::BoolOp {
                    op: "and".to_string(),
                    values: vec![combined, next],
                    ty: Type::Bool,
                };
            }
            Some(combined)
        }
    };
    ctx.scope.pop();

    Some((var_name, iter_source_expr, filter, moved_before_loop))
}
