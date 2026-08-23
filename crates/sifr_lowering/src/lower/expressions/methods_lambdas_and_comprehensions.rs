use super::{
    DiagnosticCode, Expr, ExprDictComp, ExprLambda, ExprListComp, ExprSetComp, FunctionType,
    HirExpr, HirParam, LowerCtx, ParamConvention, Ranged, TextRange, Type,
    callable_builtin_element_type, container_literal_diagnostics, lower_expr,
    lower_iterator_protocol_entry, resolve_annotation_expr, str,
};
use crate::lower::{
    nested_function_inference::collect_referenced_names_in_expr, ownership_diagnostics,
    statement_diagnostics,
};
use std::collections::HashSet;
pub(in crate::lower) fn lower_lambda_with_context(
    expr: &Expr,
    context_types: &[Type],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if let Expr::Lambda(lambda) = expr {
        if reject_affine_lambda_captures(lambda, ctx) {
            return None;
        }
        let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
            let mut params = Vec::new();
            if let Some(ref parameters) = lambda.parameters {
                for (i, param) in parameters.args.iter().enumerate() {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else if i < context_types.len() {
                        // Use contextual type
                        context_types[i].clone()
                    } else {
                        Type::Any
                    };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }
            }

            let body = lower_expr(&lambda.body, ctx)?;
            let body_ty = body.ty().clone();
            Some((params, body, body_ty))
        })?;

        let param_types: Vec<(String, Type)> = params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

        Some(HirExpr::Lambda {
            params,
            body: Box::new(body),
            ty: fn_ty,
        })
    } else {
        // Not a lambda, lower normally
        lower_expr(expr, ctx)
    }
}

pub(in crate::lower) fn lower_lambda(lambda: &ExprLambda, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if reject_affine_lambda_captures(lambda, ctx) {
        return None;
    }
    let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
        let mut params = Vec::new();
        if let Some(ref parameters) = lambda.parameters {
            for param in &parameters.args {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    // Unannotated lambda params start as Any and may be refined
                    // by contextual typing at call sites.
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }
        }

        let body = lower_expr(&lambda.body, ctx)?;
        let body_ty = body.ty().clone();
        Some((params, body, body_ty))
    })?;

    // Build the function type for the lambda
    let param_types: Vec<(String, Type)> = params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

    Some(HirExpr::Lambda {
        params,
        body: Box::new(body),
        ty: fn_ty,
    })
}

fn reject_affine_lambda_captures(lambda: &ExprLambda, ctx: &mut LowerCtx) -> bool {
    let mut referenced = HashSet::new();
    collect_referenced_names_in_expr(&lambda.body, &mut referenced);
    if let Some(parameters) = &lambda.parameters {
        for parameter in &parameters.args {
            referenced.remove(parameter.parameter.name.as_str());
        }
    }
    let capture = referenced.into_iter().find_map(|name| {
        ctx.scope
            .lookup(&name)
            .filter(|info| {
                info.ty.contains_affine_resource()
                    || ctx.must_use_obligation_for_type(&info.ty).is_some()
            })
            .map(|info| (name, info.ty.clone()))
    });
    let Some((name, ty)) = capture else {
        return false;
    };
    ownership_diagnostics::must_use_reusable_callable_capture(
        ctx,
        "lambda",
        &name,
        &ty,
        lambda.range(),
    );
    true
}

pub(super) fn reject_invalid_expression_target(
    ctx: &mut LowerCtx,
    message: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET,
        message.to_string(),
        range,
    );
}

pub(super) fn reject_invalid_expression_iteration(
    ctx: &mut LowerCtx,
    iter_ty: &Type,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ITERATION,
        format!("cannot iterate over type '{}'", iter_ty.display_name()),
        range,
    );
}

pub(super) fn reject_unsupported_expression_form(
    ctx: &mut LowerCtx,
    message: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        message.to_string(),
        range,
    );
}

pub(in crate::lower) fn lower_list_comp(
    comp: &ExprListComp,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        reject_unsupported_expression_form(
            ctx,
            "list comprehension must have at least one generator",
            comp.range(),
        );
        return None;
    }

    if let Some(result) = super::async_comprehensions::lower_list_comp(comp, ctx) {
        return result;
    }

    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "list",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let mut moved_before_loop = None;
    let result = (|| {
        // Process each generator: push scope, define var, lower iter
        for generator in &comp.generators {
            let var_name = match &generator.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "comprehension tuple target must contain only simple names",
                            generator.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "comprehension target must be a simple name or tuple",
                        generator.target.range(),
                    );
                    return None;
                }
            };

            let iter_source_expr = lower_expr(&generator.iter, ctx)?;
            moved_before_loop.get_or_insert_with(|| ctx.scope.save_moved_state());
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, generator.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, generator.iter.range())
            {
                return None;
            }

            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define_ephemeral(
                            (*name).to_string(),
                            ty,
                            crate::scope::EphemeralOrigin::Comprehension,
                        );
                    }
                } else {
                    for name in &names {
                        ctx.scope.define_ephemeral(
                            (*name).to_string(),
                            Type::Any,
                            crate::scope::EphemeralOrigin::Comprehension,
                        );
                    }
                }
            } else {
                ctx.scope.define_ephemeral(
                    var_name.clone(),
                    elem_ty.clone(),
                    crate::scope::EphemeralOrigin::Comprehension,
                );
            }

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

            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }

        // Lower the expression (all generator vars are in scope)
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        if statement_diagnostics::reject_affine_comprehension_value(ctx, &expr_ty, comp.elt.range())
        {
            return None;
        }
        let result_ty = Type::List(Box::new(expr_ty));

        Some(HirExpr::ListComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    if let Some(snapshot) = &moved_before_loop {
        ownership_diagnostics::report_moved_across_loop(ctx, snapshot, comp.range());
    }
    result
}

pub(in crate::lower) fn lower_set_comp(comp: &ExprSetComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "set",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let mut moved_before_loop = None;
    let result = (|| {
        for generator in &comp.generators {
            let var_name = if let Expr::Name(n) = &generator.target {
                n.id.to_string()
            } else {
                reject_invalid_expression_target(
                    ctx,
                    "set comprehension target must be a simple name",
                    generator.target.range(),
                );
                return None;
            };
            let iter_source_expr = lower_expr(&generator.iter, ctx)?;
            moved_before_loop.get_or_insert_with(|| ctx.scope.save_moved_state());
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, generator.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, generator.iter.range())
            {
                return None;
            }
            ctx.scope.push();
            pushed_scopes += 1;
            ctx.scope.define_ephemeral(
                var_name.clone(),
                elem_ty.clone(),
                crate::scope::EphemeralOrigin::Comprehension,
            );
            let filter = if generator.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&generator.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        if statement_diagnostics::reject_affine_comprehension_value(ctx, &expr_ty, comp.elt.range())
            || container_literal_diagnostics::reject_unhashable_container_type(
                ctx,
                "set comprehension element",
                &expr_ty,
                comp.elt.range(),
            )
        {
            return None;
        }
        let result_ty = Type::Set(Box::new(expr_ty));
        Some(HirExpr::SetComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    if let Some(snapshot) = &moved_before_loop {
        ownership_diagnostics::report_moved_across_loop(ctx, snapshot, comp.range());
    }
    result
}

pub(in crate::lower) fn lower_dict_comp(
    comp: &ExprDictComp,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "dict",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let mut moved_before_loop = None;
    let result = (|| {
        for generator in &comp.generators {
            let var_name = match &generator.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "dict comprehension tuple target must contain only simple names",
                            generator.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "dict comprehension target must be a simple name or tuple",
                        generator.target.range(),
                    );
                    return None;
                }
            };
            let iter_source_expr = lower_expr(&generator.iter, ctx)?;
            moved_before_loop.get_or_insert_with(|| ctx.scope.save_moved_state());
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, generator.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, generator.iter.range())
            {
                return None;
            }
            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define_ephemeral(
                            (*name).to_string(),
                            ty,
                            crate::scope::EphemeralOrigin::Comprehension,
                        );
                    }
                } else {
                    for name in &names {
                        ctx.scope.define_ephemeral(
                            (*name).to_string(),
                            Type::Any,
                            crate::scope::EphemeralOrigin::Comprehension,
                        );
                    }
                }
            } else {
                ctx.scope.define_ephemeral(
                    var_name.clone(),
                    elem_ty.clone(),
                    crate::scope::EphemeralOrigin::Comprehension,
                );
            }
            let filter = if generator.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&generator.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let key_expr = lower_expr(&comp.key, ctx)?;
        let val_expr = lower_expr(&comp.value, ctx)?;
        let key_ty = key_expr.ty().clone();
        let val_ty = val_expr.ty().clone();
        if container_literal_diagnostics::reject_unhashable_container_type(
            ctx,
            "dict comprehension key",
            &key_ty,
            comp.key.range(),
        ) || statement_diagnostics::reject_affine_comprehension_value(
            ctx,
            &val_ty,
            comp.value.range(),
        ) {
            return None;
        }
        let result_ty = Type::Dict(Box::new(key_ty), Box::new(val_ty));
        Some(HirExpr::DictComp {
            key_expr: Box::new(key_expr),
            val_expr: Box::new(val_expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    if let Some(snapshot) = &moved_before_loop {
        ownership_diagnostics::report_moved_across_loop(ctx, snapshot, comp.range());
    }
    result
}
