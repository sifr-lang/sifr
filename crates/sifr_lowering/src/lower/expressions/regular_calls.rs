use super::{
    call_argument_ranges_by_param, collect_type_vars, coroutine_result_type,
    decode_typevar_constraint, expression_diagnostics, infer_type_var_bindings,
    is_compatible_with_unresolved_typevars, lower_expr, lower_function_call_args, lower_name,
    lower_python_function_call_args, lower_signature_call_args, name_diagnostics,
    ownership_diagnostics, protocol_diagnostics, refine_constructor_return_type_from_args,
    substitute_type_vars, tsc, type_param_argument_range, type_satisfies_bound,
    type_satisfies_constraint, DiagnosticCode, Expr, ExprCall, HashMap, HirExpr, LowerCtx,
    OwnershipKind, ParamConvention, Ranged, Type,
};
use crate::lower::{ipc_payload_calls, parallel_calls};
pub(super) fn lower_regular_call(
    func_name: String,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if ctx.python_import_module_bindings.contains(&func_name) {
        if let Some(arg) = call.arguments.args.first() {
            match arg {
                Expr::StringLiteral(literal) => {
                    let import_name = literal.value.to_str();
                    let root = import_name.split('.').next().unwrap_or_default();
                    let required_roots = ctx
                        .python_trust_policy
                        .as_ref()
                        .map_or(&[][..], |policy| policy.required_import_roots.as_slice());
                    let trusted_roots = ctx
                        .python_trust_policy
                        .as_ref()
                        .map_or(&[][..], |policy| policy.trusted_import_roots.as_slice());
                    if !python_root_allowed(required_roots, root) {
                        ctx.error_with_code_at(
                            DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED,
                            format!(
                                "Python import root '{root}' is not in the canonical requirement set"
                            ),
                            arg.range(),
                        );
                        return None;
                    }
                    if !python_root_allowed(trusted_roots, root) {
                        ctx.error_with_code_at(
                            DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED,
                            format!("required Python import root '{root}' is not authorized by root [trust].python"),
                            arg.range(),
                        );
                        return None;
                    }
                }
                _ if !ctx.current_function_trusts_dynamic_python => {
                    ctx.error_with_code_at(
                        DiagnosticCode::PYTRUST_DYNAMIC_IMPORT_REQUIRES_TRUST,
                        format!(
                            "dynamic Python import through '{func_name}' requires @trust_python_dynamic"
                        ),
                        call.func.range(),
                    );
                    return None;
                }
                _ => {}
            }
        }
    }
    if let Some(result) = parallel_calls::lower_parallel_imported_call(&func_name, call, ctx) {
        return result;
    }

    // Check if this is a Callable-typed variable being called
    let callable_info = ctx
        .scope
        .lookup(&func_name)
        .and_then(|info| match &info.ty {
            Type::Callable(param_types, conventions, ret_type) => Some((
                param_types.clone(),
                conventions.clone(),
                *ret_type.clone(),
                false,
            )),
            Type::AsyncCallable(param_types, conventions, ret_type) => Some((
                param_types.clone(),
                conventions.clone(),
                *ret_type.clone(),
                true,
            )),
            _ => None,
        });
    if let Some((param_types, conventions, ret_type, is_async_callable)) = callable_info {
        // Lower arguments
        let mut args = Vec::new();
        for arg in &call.arguments.args {
            let expr = lower_expr(arg, ctx)?;
            args.push(expr);
        }
        if args.len() != param_types.len() {
            let range = if args.len() > param_types.len() {
                call.arguments.args[param_types.len()].range()
            } else {
                call.func.range()
            };
            expression_diagnostics::call_not_callable_or_arity(
                ctx,
                format!(
                    "callable '{}' expects {} argument(s), got {}",
                    func_name,
                    param_types.len(),
                    args.len()
                ),
                range,
            );
            return None;
        }
        // Type check arguments and apply convention-aware move tracking
        for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "argument {} of callable '{}': expected '{}', got '{}'",
                        i + 1,
                        func_name,
                        param_ty.display_name(),
                        arg.ty().display_name()
                    ),
                    call.arguments.args[i].range(),
                );
            }
            // Apply move tracking based on convention
            let convention = conventions
                .get(i)
                .copied()
                .unwrap_or(ParamConvention::borrow());
            if convention.is_owned() {
                // Own convention: transfer ownership, mark variable as moved
                if let HirExpr::Name { name, ty } = arg {
                    if ty.ownership() == OwnershipKind::Move {
                        ctx.mark_moved_with_flow(name);
                    }
                }
            }
            // Borrow/MutBorrow: no move, variable remains usable
        }
        return Some(HirExpr::Call {
            func: func_name,
            args,
            ty: if is_async_callable {
                coroutine_result_type(&ret_type)
            } else {
                ret_type
            },
        });
    }

    let callable_object_ft =
        ctx.scope
            .lookup(&func_name)
            .and_then(|info| match info.effective_type().resolve_alias() {
                Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods
                    .iter()
                    .find(|(name, _)| name == "__call__")
                    .map(|(_, ft)| ft.clone()),
                _ => None,
            });
    if let Some(call_ft) = callable_object_ft {
        let Expr::Name(name_expr) = call.func.as_ref() else {
            expression_diagnostics::call_not_callable_or_arity(
                ctx,
                "only simple function calls are supported".to_string(),
                call.func.range(),
            );
            return None;
        };
        let object = lower_name(name_expr, ctx)?;
        let args =
            lower_signature_call_args(call, &format!("{func_name}.__call__"), &call_ft, None, ctx)?;
        return Some(HirExpr::MethodCall {
            object: Box::new(object),
            method: "__call__".to_string(),
            args,
            ty: *call_ft.return_type.clone(),
        });
    }

    let ft = ctx.functions.get(&func_name).cloned().or_else(|| {
        name_diagnostics::undefined_function(ctx, &func_name, call.func.range());
        None
    })?;
    let is_async_function = ctx.async_functions.contains(&func_name);
    let is_async_generator_function = ctx.async_generator_functions.contains(&func_name);
    if is_async_function && !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "async function '{func_name}' cannot be called from sync code; call it from an async function and await the returned coroutine"
            ),
            call.func.range(),
        );
        return None;
    }
    super::workload_annotations::reject_async_direct_call(ctx, &func_name, call.func.range());
    let call_defaults = ctx.function_defaults.get(&func_name).cloned();
    let call_vararg = ctx.vararg_functions.get(&func_name).copied();

    // Resolve keyword arguments to positional order
    let python_call_shapes = ctx.python_call_shapes.get(&func_name).cloned();
    let mut python_record_expansions = Vec::new();
    let mut args = if let Some(shapes) = python_call_shapes.as_deref() {
        let lowered = lower_python_function_call_args(
            call,
            &func_name,
            &ft,
            call_defaults.as_deref(),
            shapes,
            ctx,
        )?;
        python_record_expansions = lowered.record_expansions;
        lowered.args
    } else if func_name == "print" {
        let mut args = Vec::with_capacity(call.arguments.args.len());
        for arg in &call.arguments.args {
            args.push(lower_expr(arg, ctx)?);
        }
        args
    } else if func_name.ends_with("_Counter")
        && ft.params.len() == 2
        && call.arguments.args.len() == 1
        && call.arguments.keywords.is_empty()
    {
        let lowered_arg = lower_expr(&call.arguments.args[0], ctx)?;
        let source_ty = &ft.params[0].1;
        let iterable_ty = &ft.params[1].1;
        if lowered_arg.ty().is_assignable_to(source_ty)
            || is_compatible_with_unresolved_typevars(lowered_arg.ty(), source_ty)
        {
            vec![lowered_arg, HirExpr::NoneLiteral]
        } else if lowered_arg.ty().is_assignable_to(iterable_ty)
            || is_compatible_with_unresolved_typevars(lowered_arg.ty(), iterable_ty)
        {
            vec![HirExpr::NoneLiteral, lowered_arg]
        } else if matches!(lowered_arg.ty().resolve_alias(), Type::Str) {
            let iterable_arg = HirExpr::Call {
                func: "list".to_string(),
                args: vec![lowered_arg],
                ty: Type::List(Box::new(Type::Str)),
            };
            if iterable_arg.ty().is_assignable_to(iterable_ty)
                || is_compatible_with_unresolved_typevars(iterable_arg.ty(), iterable_ty)
            {
                vec![HirExpr::NoneLiteral, iterable_arg]
            } else {
                lower_function_call_args(
                    call,
                    &func_name,
                    &ft,
                    call_defaults.as_deref(),
                    call_vararg,
                    ctx,
                )?
            }
        } else {
            lower_function_call_args(
                call,
                &func_name,
                &ft,
                call_defaults.as_deref(),
                call_vararg,
                ctx,
            )?
        }
    } else {
        lower_function_call_args(
            call,
            &func_name,
            &ft,
            call_defaults.as_deref(),
            call_vararg,
            ctx,
        )?
    };

    let mut provided_arguments = vec![true; args.len()];
    if let Some(defaults) = &call_defaults {
        for (index, default) in defaults {
            let HirExpr::Call { func, .. } = default else {
                continue;
            };
            if func != "__sifr_python_omitted_argument" {
                continue;
            }
            let Some(argument) = args.get_mut(*index) else {
                continue;
            };
            if matches!(argument, HirExpr::Call { func, .. } if func == "__sifr_python_omitted_argument")
            {
                provided_arguments[*index] = false;
                continue;
            }
            let ty = argument.ty().clone();
            let value = std::mem::replace(argument, HirExpr::NoneLiteral);
            *argument = HirExpr::Call {
                func: "__sifr_python_present_argument".to_string(),
                args: vec![value],
                ty,
            };
        }
    }

    let arg_ranges = call_argument_ranges_by_param(call, &ft);
    ipc_payload_calls::validate_require_serializable_call(
        &func_name,
        &args,
        &arg_ranges,
        call,
        ctx,
    );
    crate::lower::python_interop::validate_callback_call_captures(
        &func_name,
        &args,
        &arg_ranges,
        None,
        call.range(),
        ctx,
    );
    if func_name == "require_serializable" {
        // This marker is checked entirely during lowering. Emit a concrete
        // expression so statement-position calls do not generate an ambiguous
        // Rust `None` literal before the generated schema extractor exists.
        return Some(HirExpr::BoolLiteral(true));
    }

    // Check argument types (skip for print)
    if func_name != "print" {
        let is_generic_function = ctx.generic_functions.contains_key(&func_name);
        for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
            if is_generic_function {
                let mut type_vars = Vec::new();
                collect_type_vars(param_ty, &mut type_vars);
                if !type_vars.is_empty() {
                    // Generic params are validated after binding/substitution.
                    continue;
                }
            }
            if !arg.ty().is_assignable_to(param_ty) {
                let primary_range = arg_ranges
                    .get(i)
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| call.range());
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                        i + 1,
                        param_name,
                        func_name,
                        param_ty.display_name(),
                        arg.ty().display_name()
                    ),
                    primary_range,
                );
            }
        }
    }

    // Exclusivity check: enforce that the same variable is not passed as mut twice,
    // or as both mut and immutable borrow in the same call.
    {
        let mut mut_borrowed: Vec<String> = Vec::new();
        let mut immut_borrowed: Vec<String> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if let HirExpr::Name { name, ty } = arg {
                if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    let primary_range = arg_ranges
                        .get(i)
                        .copied()
                        .flatten()
                        .unwrap_or_else(|| call.range());
                    let convention = ft
                        .params
                        .get(i)
                        .map(|(_, _, c)| *c)
                        .unwrap_or(ParamConvention::borrow());
                    if convention.is_mut_borrow() {
                        ctx.record_flow_effect(sifr_ir::FlowEffect::Borrow {
                            binding: name.clone(),
                            mutable: true,
                        });
                        if mut_borrowed.contains(name) {
                            ownership_diagnostics::double_mutable_borrow(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        } else if immut_borrowed.contains(name) {
                            ownership_diagnostics::mutable_borrow_after_immutable(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        }
                        mut_borrowed.push(name.clone());
                    } else if convention.is_shared_borrow() {
                        ctx.record_flow_effect(sifr_ir::FlowEffect::Borrow {
                            binding: name.clone(),
                            mutable: false,
                        });
                        if mut_borrowed.contains(name) {
                            ownership_diagnostics::immutable_borrow_after_mutable(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        }
                        immut_borrowed.push(name.clone());
                    } else {
                        // Ownership transfer, including `own mut`, does not create a borrow conflict.
                    }
                }
            }
        }
    }

    // Track ownership: only mark arguments as moved when the parameter convention is Own
    // and the argument type is Move. Borrow and MutBorrow do not consume the value.
    for (i, arg) in args.iter().enumerate() {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                let convention = ft
                    .params
                    .get(i)
                    .map(|(_, _, c)| *c)
                    .unwrap_or(ParamConvention::borrow());
                if convention.is_owned() {
                    ctx.mark_moved_with_flow(name);
                }
            }
        }
    }

    // If this is a generic function, infer type variable bindings and substitute
    let return_type = if ctx.generic_functions.contains_key(&func_name) {
        let mut bindings = HashMap::new();
        for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) {
            infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
        }
        // Re-check argument types after TypeVar substitution so repeated type
        // parameters (e.g. assert_eq[T](a: T, b: T)) enforce consistent types.
        if func_name != "print" {
            for (i, (arg, (param_name, param_ty, _))) in
                args.iter().zip(ft.params.iter()).enumerate()
            {
                let concrete_param_ty = substitute_type_vars(param_ty, &bindings);
                let mut unresolved_type_vars = Vec::new();
                collect_type_vars(&concrete_param_ty, &mut unresolved_type_vars);
                if !unresolved_type_vars.is_empty() {
                    if !is_compatible_with_unresolved_typevars(arg.ty(), &concrete_param_ty) {
                        let primary_range = arg_ranges
                            .get(i)
                            .copied()
                            .flatten()
                            .unwrap_or_else(|| call.range());
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            format!(
                                "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                                i + 1,
                                param_name,
                                func_name,
                                concrete_param_ty.display_name(),
                                arg.ty().display_name()
                            ),
                            primary_range,
                        );
                    }
                    continue;
                }
                if !arg.ty().is_assignable_to(&concrete_param_ty) {
                    let primary_range = arg_ranges
                        .get(i)
                        .copied()
                        .flatten()
                        .unwrap_or_else(|| call.range());
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_MISMATCH,
                        format!(
                            "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            func_name,
                            concrete_param_ty.display_name(),
                            arg.ty().display_name()
                        ),
                        primary_range,
                    );
                }
            }
        }
        // Check protocol bounds on type parameters (scoped to this function)
        if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) {
            let owner_bounds = owner_bounds.clone();
            for (tv_name, concrete_ty) in &bindings {
                if let Some(specs) = owner_bounds.get(tv_name) {
                    let mut required_bounds = Vec::new();
                    let mut constraints = Vec::new();
                    for spec in specs {
                        if let Some(constraint_name) = decode_typevar_constraint(spec) {
                            constraints.push(constraint_name.to_string());
                        } else {
                            required_bounds.push(spec.clone());
                        }
                    }

                    for bound in required_bounds {
                        if !type_satisfies_bound(concrete_ty, &bound, ctx) {
                            protocol_diagnostics::bound_not_satisfied(
                                ctx,
                                &concrete_ty.display_name(),
                                &bound,
                                tv_name,
                                call.range(),
                            );
                        }
                    }

                    if !constraints.is_empty()
                        && !constraints.iter().any(|constraint| {
                            type_satisfies_constraint(concrete_ty, constraint, ctx)
                        })
                    {
                        let primary_range = type_param_argument_range(call, &ft, tv_name)
                            .unwrap_or_else(|| call.range());
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
                            format!(
                                "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'",
                                actual = concrete_ty.display_name(),
                                constraints = constraints.join(", "),
                                type_param = tv_name
                            ),
                            primary_range,
                        );
                    }
                }
            }
        }
        if bindings.is_empty() {
            ft.return_type.as_ref().clone()
        } else {
            substitute_type_vars(&ft.return_type, &bindings)
        }
    } else {
        ft.return_type.as_ref().clone()
    };

    let return_type = refine_constructor_return_type_from_args(&ft, &args, &return_type);
    tsc::validate_shared_constructor(&func_name, &args, &arg_ranges, call, ctx);
    let call_type = if is_async_function && !is_async_generator_function {
        coroutine_result_type(&return_type)
    } else {
        return_type
    };

    if let Some(intrinsic) = ctx.compiler_intrinsics.get(&func_name).copied() {
        let intrinsic_arg_ranges = arg_ranges
            .iter()
            .map(|range| range.unwrap_or_else(|| call.range()))
            .collect();
        Some(HirExpr::IntrinsicCall {
            intrinsic,
            args,
            ty: call_type,
            call_range: call.range(),
            arg_ranges: intrinsic_arg_ranges,
        })
    // If this is a class constructor call, emit ConstructorCall
    } else if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: call_type,
        })
    } else if python_call_shapes.is_some() {
        Some(HirExpr::PythonCall {
            func: func_name,
            args,
            provided_arguments,
            record_expansions: python_record_expansions,
            ty: call_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: call_type,
        })
    }
}

fn python_root_allowed(roots: &[String], root: &str) -> bool {
    roots
        .iter()
        .any(|candidate| candidate == root || candidate == "*")
}
