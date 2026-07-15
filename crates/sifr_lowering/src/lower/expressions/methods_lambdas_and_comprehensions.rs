use super::{
    callable_builtin_element_type, canonicalize_class_surface_type,
    consume_affine_collection_method_arguments, container_literal_diagnostics,
    invalidate_collection_flow_facts_for_method, is_task_handle_type, lower_expr,
    lower_method_call_args, lower_signature_call_args, lower_task_handle_method_call,
    refine_defaultdict_binding_expr, refine_empty_list_binding_expr, refine_empty_set_binding_expr,
    refine_generic_class_binding_expr, refine_nonempty_method_return_type,
    reject_immutable_parameter_method_mutation, resolve_annotation_expr,
    resolve_bigint_method_type, resolve_bytes_method_type, resolve_class_method_type,
    resolve_decimal_method_type, resolve_dict_method_type, resolve_enum_method_type,
    resolve_fixed_width_method_type, resolve_list_method_type, resolve_newtype_method_type,
    resolve_protocol_method_type, resolve_python_buffer_method_type, resolve_set_method_type,
    resolve_str_method_type, resolve_tuple_method_type, str, tsc, ClassMethodSurface,
    DiagnosticCode, Expr, ExprAttribute, ExprCall, ExprDictComp, ExprLambda, ExprListComp,
    ExprSetComp, FunctionType, HirExpr, HirIteratorOp, HirParam, LowerCtx, ParamConvention, Ranged,
    TextRange, Type, DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS,
};
use crate::lower::python_interop::callback_method_arg_ranges;
use crate::lower::{
    nested_function_inference::collect_referenced_names_in_expr, ownership_diagnostics,
    parallel_calls, statement_diagnostics, task_join_set_calls, task_scope_offload_calls,
};
use sifr_ir::CompilerIntrinsicId;
use std::collections::HashSet;
pub(in crate::lower) fn lower_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    // Handle super().__init__() and super().method() calls
    if let Expr::Call(super_call) = attr.value.as_ref() {
        if let Expr::Name(name) = super_call.func.as_ref() {
            if name.id.as_str() == "super" {
                let method_name = attr.attr.to_string();
                if let Some(parent_name) = ctx.current_parent_class.clone() {
                    let mut args = Vec::new();
                    for arg in &call.arguments.args {
                        let expr = lower_expr(arg, ctx)?;
                        args.push(expr);
                    }

                    return Some(HirExpr::SuperCall {
                        parent_class: parent_name,
                        method: if method_name == "__init__" {
                            "new".to_string()
                        } else {
                            method_name
                        },
                        args,
                        ty: Type::None,
                    });
                }
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_INVALID_BASE,
                    "super() used outside of a class with a parent".to_string(),
                    attr.value.range(),
                );
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if ctx.class_types.contains_key(&class_name) {
            let method_name = attr.attr.to_string();
            // Lower arguments
            let mut args = Vec::new();
            for arg in &call.arguments.args {
                let expr = lower_expr(arg, ctx)?;
                args.push(expr);
            }
            // Look up the method's return type from the class type
            if let Some(Type::Class { methods, .. }) = ctx.class_types.get(&class_name) {
                if let Some((_, ft)) = methods.iter().find(|(n, _)| n == &method_name) {
                    let return_ty = *ft.return_type.clone();
                    return Some(HirExpr::Call {
                        func: format!("{class_name}::{method_name}"),
                        args,
                        ty: return_ty,
                    });
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_MEMBER,
                format!("type '{class_name}' has no class/static method '{method_name}'"),
                attr.attr.range(),
            );
            return None;
        }
    }

    let mut object = lower_expr(&attr.value, ctx)?;
    let method_name = attr.attr.to_string();
    if let Some(result) = super::blocking_executor_calls::lower_thread_pool_submit_call(
        &object,
        &method_name,
        call,
        ctx,
    ) {
        return result;
    }
    if let Some(result) =
        parallel_calls::lower_parallel_pool_method_call(&object, &method_name, call, ctx)
    {
        return result;
    }
    if let Some(result) =
        task_join_set_calls::lower_join_set_method_call(object.clone(), &method_name, call, ctx)
    {
        return result;
    }
    if let Some(result) = task_scope_offload_calls::lower_task_scope_offload_method_call(
        object.clone(),
        &method_name,
        call,
        ctx,
    ) {
        return result;
    }
    if tsc::is_task_scope_type(object.ty()) && method_name == "spawn" {
        return tsc::lower_task_scope_spawn_call(object, attr, call, ctx);
    }
    if is_task_handle_type(object.ty()) {
        if let Some(expr) = lower_task_handle_method_call(object.clone(), &method_name, call, ctx) {
            return Some(expr);
        }
    }
    let object_ty_for_args = canonicalize_class_surface_type(object.ty().resolve_alias());
    super::workload_annotations::reject_async_direct_method_call(
        ctx,
        &object_ty_for_args,
        &method_name,
        attr.attr.range(),
    );
    let args = match &object_ty_for_args {
        Type::Class { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                let defaults_key = format!("{name}.{method_name}");
                let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
                lower_signature_call_args(
                    call,
                    &format!("{name}.{method_name}"),
                    &ft,
                    method_defaults.as_deref(),
                    ctx,
                )?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                lower_signature_call_args(call, &format!("{name}.{method_name}"), &ft, None, ctx)?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        _ => lower_method_call_args(object.ty(), &method_name, call, ctx)?,
    };

    if matches!(method_name.as_str(), "append" | "insert" | "extend") {
        object = refine_empty_list_binding_expr(object, &method_name, &args, ctx);
    }
    if matches!(
        method_name.as_str(),
        "add" | "remove" | "discard" | "contains"
    ) {
        if let Some(first_arg_ty) = args.first().map(|arg| arg.ty().clone()) {
            object = refine_empty_set_binding_expr(object, first_arg_ty, ctx);
        }
    }
    if let Some(refined_object) =
        refine_defaultdict_binding_expr(object.clone(), &method_name, &args, ctx)
    {
        object = refined_object;
    }
    object = refine_generic_class_binding_expr(object, &method_name, &args, ctx);
    let object_ty = object.ty().clone();
    if reject_immutable_parameter_method_mutation(
        ctx,
        &object,
        &object_ty,
        &method_name,
        attr.value.range(),
    ) {
        return None;
    }
    let method_arg_ranges =
        callback_method_arg_ranges(&object, &object_ty_for_args, &method_name, call, &args, ctx);
    let resolved_method_type = resolve_method_type(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        attr.attr.range(),
        ctx,
    )?;
    consume_affine_collection_method_arguments(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        ctx,
    );
    let return_ty = refine_nonempty_method_return_type(
        &object_ty,
        &object,
        &method_name,
        &args,
        &resolved_method_type,
        ctx,
    );
    tsc::validate_channel_send_element(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        call,
        ctx,
    );
    invalidate_collection_flow_facts_for_method(ctx, &object, &object_ty, &method_name);
    if matches!(object_ty.resolve_alias(), Type::Str) && method_name == "encode" {
        let mut intrinsic_args = vec![object];
        let intrinsic = if args.is_empty() {
            CompilerIntrinsicId::StringEncode
        } else {
            CompilerIntrinsicId::StringEncodeWithEncoding
        };
        intrinsic_args.extend(args.iter().cloned());
        let mut intrinsic_arg_ranges = vec![attr.value.range()];
        intrinsic_arg_ranges.extend(method_arg_ranges.iter().copied());
        return Some(HirExpr::IntrinsicCall {
            intrinsic,
            args: intrinsic_args,
            ty: return_ty,
            call_range: call.range(),
            arg_ranges: intrinsic_arg_ranges,
        });
    }
    if matches!(object_ty.resolve_alias(), Type::Bytes) && method_name == "decode" {
        let mut intrinsic_args = vec![object];
        let intrinsic = if args.is_empty() {
            CompilerIntrinsicId::BytesDecode
        } else {
            CompilerIntrinsicId::BytesDecodeWithEncoding
        };
        intrinsic_args.extend(args.iter().cloned());
        let mut intrinsic_arg_ranges = vec![attr.value.range()];
        intrinsic_arg_ranges.extend(method_arg_ranges.iter().copied());
        return Some(HirExpr::IntrinsicCall {
            intrinsic,
            args: intrinsic_args,
            ty: return_ty,
            call_range: call.range(),
            arg_ranges: intrinsic_arg_ranges,
        });
    }

    if let Type::Class {
        name: class_name, ..
    } = object_ty.resolve_alias()
    {
        let qualified = format!("{class_name}.{method_name}");
        if ctx.python_context_exit_methods.contains(&qualified) {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::PYCTX_INVALID_DECLARATION,
                "invalid Python context declaration: context exit methods are compiler-invoked and cannot be called directly"
                    .to_string(),
                call.range(),
            );
            return None;
        }
        if ctx.python_consuming_methods.contains(&qualified) {
            if let HirExpr::Name { name, .. } = &object {
                ctx.mark_moved_with_flow(name);
            }
        }
    }

    if matches!(object_ty.resolve_alias(), Type::PythonBuffer(_))
        && method_name == "release"
        && !super::consume_python_buffer_release_receiver(&object, attr.value.range(), ctx)
    {
        return None;
    }

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
pub(in crate::lower) fn resolve_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let canonical_object_ty = canonicalize_class_surface_type(object_ty);
    let object_ty = &canonical_object_ty;
    if let Type::Alias {
        name: alias_name,
        body,
        ..
    } = object_ty
    {
        if matches!(
            alias_name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        ) {
            return resolve_method_type(body, method, args, arg_ranges, method_range, ctx);
        }
    }
    if matches!(object_ty, Type::AsyncGenerator(_, _)) {
        return super::async_generator_methods::resolve_async_generator_method_type(
            object_ty,
            method,
            args,
            arg_ranges,
            method_range,
            ctx,
        );
    }
    match object_ty {
        Type::List(elem_ty) => {
            resolve_list_method_type(elem_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Dict(key_ty, val_ty) => {
            resolve_dict_method_type(key_ty, val_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Set(elem_ty) => {
            resolve_set_method_type(elem_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Str => resolve_str_method_type(method, args, arg_ranges, method_range, ctx),
        Type::Bytes => resolve_bytes_method_type(method, args, arg_ranges, method_range, ctx),
        Type::FixedInt(fixed) => {
            resolve_fixed_width_method_type(*fixed, method, args, arg_ranges, method_range, ctx)
        }
        Type::Tuple(_) => resolve_tuple_method_type(method, args, arg_ranges, method_range, ctx),
        Type::PythonBuffer(element) => {
            resolve_python_buffer_method_type(element, method, args, arg_ranges, method_range, ctx)
        }
        Type::Class {
            name,
            fields,
            methods,
            ..
        } => resolve_class_method_type(
            ClassMethodSurface {
                name,
                fields,
                methods,
            },
            method,
            args,
            arg_ranges,
            method_range,
            ctx,
        ),
        Type::Protocol { name, methods, .. } => {
            resolve_protocol_method_type(name, methods, method, args, arg_ranges, method_range, ctx)
        }
        Type::Newtype { name, inner } => {
            resolve_newtype_method_type(name, inner, method, args, arg_ranges, method_range, ctx)
        }
        Type::Enum { name, .. } => {
            resolve_enum_method_type(name, method, args, arg_ranges, method_range, ctx)
        }
        Type::BigInt => resolve_bigint_method_type(method, args, arg_ranges, method_range, ctx),
        Type::Decimal | Type::BigDecimal => {
            resolve_decimal_method_type(object_ty, method, args, arg_ranges, method_range, ctx)
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type '{}' has no method '{}'",
                    object_ty.display_name(),
                    method
                ),
                method_range,
            );
            None
        }
    }
}
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
            .filter(|info| info.ty.contains_affine_resource())
            .map(|info| (name, info.ty.clone()))
    });
    let Some((name, ty)) = capture else {
        return false;
    };
    ownership_diagnostics::affine_reusable_callable_capture(
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
    let result = (|| {
        // Process each generator: push scope, define var, lower iter
        for gen in &comp.generators {
            let var_name = match &gen.target {
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
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };

            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, gen.iter.range()) {
                return None;
            }

            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }

            let filter = if gen.ifs.is_empty() {
                None
            } else {
                let first = lower_expr(&gen.ifs[0], ctx)?;
                if gen.ifs.len() == 1 {
                    Some(first)
                } else {
                    let mut combined = first;
                    for cond in &gen.ifs[1..] {
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
    let result = (|| {
        for gen in &comp.generators {
            let var_name = if let Expr::Name(n) = &gen.target {
                n.id.to_string()
            } else {
                reject_invalid_expression_target(
                    ctx,
                    "set comprehension target must be a simple name",
                    gen.target.range(),
                );
                return None;
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, gen.iter.range()) {
                return None;
            }
            ctx.scope.push();
            pushed_scopes += 1;
            ctx.scope.define(var_name.clone(), elem_ty.clone());
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
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
    let result = (|| {
        for gen in &comp.generators {
            let var_name = match &gen.target {
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
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "dict comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            if statement_diagnostics::reject_affine_iteration(ctx, &elem_ty, gen.iter.range()) {
                return None;
            }
            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
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
    result
}

pub(super) fn lower_iterator_protocol_entry(iter_source_expr: HirExpr, elem_ty: Type) -> HirExpr {
    HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iter_source_expr],
        ty: Type::Iterator(Box::new(elem_ty)),
    }
}
