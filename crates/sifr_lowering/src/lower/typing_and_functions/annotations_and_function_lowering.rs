use super::{
    ast_convention_to_param, collect_declared_nonlocals, collect_yield_types,
    first_await_range_in_stmts, first_yield_range_in_stmts, function_body_contains_yield,
    infer_function_return_type, lower_expr, lower_function_stmts, ownership_diagnostics,
    reject_borrowed_affine_generator_params, reject_declared_async_generator_boundary,
    reject_live_join_sets_at_function_exit, reject_live_must_use_bindings_at_function_exit, str,
    workload_annotations, DiagnosticCode, Expr, HirFunction, HirParam, LowerCtx, MethodKind,
    OwnershipKind, Ranged, StmtFunctionDef, Type,
};
use crate::lower::python_interop::{
    classify_python_interop_stub_body, collect_python_interop_declarations,
    has_python_interop_decorator_syntax, is_python_omit, validate_python_interop_signature,
};
use crate::lower::rust_interop::{
    classify_rust_interop_stub_body, collect_rust_interop_declarations,
    has_rust_interop_decorator_syntax, RustInteropOwner,
};
use crate::lower::rust_interop_structural::{
    validate_structural_function_contract, StructuralFunctionContract,
};
use crate::lower::{compiler_intrinsics, diagnostics::has_decorator};
pub(in crate::lower) fn lower_function(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) -> Option<HirFunction> {
    let ft = ctx.functions.get::<str>(func.name.as_ref())?.clone();
    let effective_is_async = func.is_async;
    let has_python_interop = has_python_interop_decorator_syntax(&func.decorator_list);

    ctx.enter_function_scope(collect_declared_nonlocals(&func.body));

    // Define parameters in scope, handling defaults
    let mut params = Vec::new();

    // Regular args
    for (i, param_def) in func.parameters.args.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft
            .params
            .get(i)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention);

        let default = param_def.default.as_ref().and_then(|d| {
            (!has_python_interop || !is_python_omit(d))
                .then(|| lower_expr(d, ctx))
                .flatten()
        });

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: false,
            convention,
        });
    }

    // Vararg parameter (*args) -- becomes Vec<T>
    if let Some(ref vararg) = func.parameters.vararg {
        let name = vararg.name.to_string();
        let regular_count = func.parameters.args.len();
        let ty = ft
            .params
            .get(regular_count)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(vararg.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention);
        params.push(HirParam {
            name,
            ty,
            default: None,
            keyword_only: false,
            convention,
        });
    }

    // Keyword-only args (after * separator)
    let regular_count = func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
    for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft
            .params
            .get(regular_count + i)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention);

        let default = param_def.default.as_ref().and_then(|d| {
            (!has_python_interop || !is_python_omit(d))
                .then(|| lower_expr(d, ctx))
                .flatten()
        });

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: true,
            convention,
        });
    }

    if let Some(ref kwarg) = func.parameters.kwarg {
        let name = kwarg.name.to_string();
        let index = regular_count + func.parameters.kwonlyargs.len();
        let ty = ft
            .params
            .get(index)
            .map(|(_, ty, _)| ty.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(kwarg.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention);
        params.push(HirParam {
            name,
            ty,
            default: None,
            keyword_only: true,
            convention,
        });
    }

    // Populate borrowed_params for escape analysis in lower_return / lower_let.
    // Any borrowed move-type parameter, shared or mutable, is escape-unsafe.
    // Exclude TypeVar parameters: generics are monomorphized by Rust and ownership is handled
    // by the Rust compiler, not by Sifr's escape analysis.
    ctx.borrowed_params.clear();
    for param in &params {
        if param.convention.is_borrowed()
            && param.ty.ownership() == OwnershipKind::Move
            && !matches!(param.ty, Type::TypeVar(_))
        {
            ctx.borrowed_params.insert(param.name.clone());
        }
    }
    let rust_interop = collect_rust_interop_declarations(
        &func.decorator_list,
        RustInteropOwner::Function,
        ctx,
        has_decorator(func, "blocking_io"),
        has_decorator(func, "cpu_heavy"),
        effective_is_async,
    );
    let mut python_interop = collect_python_interop_declarations(
        &func.decorator_list,
        &func.parameters,
        effective_is_async,
        ctx,
    );
    let compiler_intrinsic = ctx.compiler_intrinsics.get(func.name.as_str()).copied();
    let has_compiler_intrinsic_syntax =
        compiler_intrinsics::has_decorator_syntax(&func.decorator_list);
    let skips_normal_body_lowering = if has_compiler_intrinsic_syntax {
        if !rust_interop.is_empty() || !python_interop.is_empty() {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
                "@compiler_intrinsic and interop decorators cannot be combined".to_string(),
                func.name.range(),
            );
        }
        compiler_intrinsics::classify_stub_body(func, compiler_intrinsic, ctx)
            .skips_normal_body_lowering()
    } else if has_python_interop {
        classify_python_interop_stub_body(&func.body, true, ctx).skips_normal_body_lowering()
    } else {
        classify_rust_interop_stub_body(
            &func.body,
            has_rust_interop_decorator_syntax(&func.decorator_list),
            ctx,
        )
        .skips_normal_body_lowering()
    };

    let has_generator_body =
        !skips_normal_body_lowering && function_body_contains_yield(&func.body);
    let is_async_generator = has_generator_body && effective_is_async;
    if has_generator_body {
        if let Some(yield_range) = first_yield_range_in_stmts(&func.body) {
            reject_borrowed_affine_generator_params(func.name.as_str(), &params, yield_range, ctx);
        }
    }
    workload_annotations::reject_async_function_annotation(ctx, func, effective_is_async);
    if !skips_normal_body_lowering
        && effective_is_async
        && matches!(
            ctx.async_suspension_summaries.get(func.name.as_str()),
            Some(super::async_effects::AsyncSuspensionSummary::NoSuspend)
        )
    {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_NO_SUSPEND,
            format!(
                "async function '{}' has no real suspension effect; use 'def' unless an explicit async protocol escape hatch is required",
                func.name
            ),
            func.name.range(),
        );
    }
    if !skips_normal_body_lowering && effective_is_async {
        if is_async_generator {
            if let Some(yield_range) = first_yield_range_in_stmts(&func.body) {
                reject_declared_async_generator_boundary(
                    func.name.as_str(),
                    &params,
                    &ft.return_type,
                    yield_range,
                    ctx,
                );
                for param in &params {
                    if param.convention.is_mut_borrow()
                        && param.ty.ownership() == OwnershipKind::Move
                        && !matches!(param.ty, Type::TypeVar(_))
                    {
                        ownership_diagnostics::mutable_borrow_across_yield(
                            ctx,
                            &param.name,
                            yield_range,
                        );
                    }
                }
            }
            if let Some(await_range) = first_await_range_in_stmts(&func.body) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "await inside async generator bodies requires async generator state-machine lowering and is not supported yet".to_string(),
                    await_range,
                );
            }
        }
        if let Some(await_range) = first_await_range_in_stmts(&func.body) {
            for param in &params {
                if param.convention.is_mut_borrow()
                    && param.ty.ownership() == OwnershipKind::Move
                    && !matches!(param.ty, Type::TypeVar(_))
                {
                    ownership_diagnostics::mutable_borrow_across_await(
                        ctx,
                        &param.name,
                        await_range,
                    );
                }
            }
        }
    }

    // Lower body
    let previous_owner = ctx.current_owner.replace(func.name.to_string());
    let previous_async = ctx.current_function_is_async;
    let previous_async_generator = ctx.current_function_is_async_generator;
    let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
    let previous_return_type = ctx
        .current_function_return_type
        .replace(ft.return_type.as_ref().clone());
    let previous_live_join_sets = std::mem::take(&mut ctx.live_join_set_bindings);
    let previous_must_use_bindings = std::mem::take(&mut ctx.live_must_use_bindings);
    let previous_join_set_terminal_awaitables =
        std::mem::take(&mut ctx.join_set_terminal_awaitables);
    ctx.current_function_is_async = effective_is_async;
    ctx.current_function_is_async_generator = is_async_generator;
    ctx.current_function_trusts_dynamic_python = has_decorator(func, "trust_python_dynamic");
    if !skips_normal_body_lowering {
        for param in &params {
            if param.convention.is_owned() {
                ctx.record_must_use_binding(&param.name, &param.ty);
            }
        }
    }
    let body = if skips_normal_body_lowering {
        Vec::new()
    } else {
        lower_function_stmts(&func.body, &ft, ctx)
    };
    reject_live_join_sets_at_function_exit(func, ctx);
    reject_live_must_use_bindings_at_function_exit(func, ctx);
    ctx.live_join_set_bindings = previous_live_join_sets;
    ctx.live_must_use_bindings = previous_must_use_bindings;
    ctx.join_set_terminal_awaitables = previous_join_set_terminal_awaitables;
    ctx.current_function_is_async = previous_async;
    ctx.current_function_is_async_generator = previous_async_generator;
    ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
    ctx.current_function_return_type = previous_return_type;
    ctx.current_owner = previous_owner;

    ctx.borrowed_params.clear();

    ctx.exit_function_scope();

    let has_yield = !collect_yield_types(&body).is_empty();
    if !skips_normal_body_lowering
        && !has_yield
        && requires_exhaustive_return_annotation(func, ft.return_type.as_ref())
    {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::cfg::flow_facts(&body).always_exits()
        })) {
            Ok(false) => {
                let return_type = ft.return_type.display_name();
                super::flow_diagnostics::missing_return_value(
                    ctx,
                    func.name.as_str(),
                    return_type.as_str(),
                    func.name.range(),
                );
            }
            Ok(true) => {}
            Err(_) => {
                // Fail closed: skipping return-completeness validation after an
                // invalid CFG would let an unsound function compile.
                ctx.error_with_code_at(
                    DiagnosticCode::INTERNAL_COMPILER_PANIC,
                    format!(
                        "internal compiler error: invalid control-flow graph while validating exhaustive return for '{}'",
                        func.name
                    ),
                    func.name.range(),
                );
            }
        }
    }

    let return_annotation_range = func
        .returns
        .as_ref()
        .map_or_else(|| func.name.range(), |returns| returns.range());
    let mut inferred_return_type = if skips_normal_body_lowering {
        ft.return_type.as_ref().clone()
    } else {
        infer_function_return_type(
            func.name.as_ref(),
            effective_is_async,
            ft.return_type.as_ref(),
            func.returns.is_some(),
            &body,
            |message| {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    message,
                    return_annotation_range,
                );
            },
        )
    };
    if func.returns.is_none() && inferred_return_type.has_conflicting_class_specializations() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "function '{}' cannot infer a union containing multiple specializations of the same generic class",
                func.name
            ),
            func.name.range(),
        );
        inferred_return_type = Type::Any;
    }
    validate_python_interop_signature(&mut python_interop, &params, &inferred_return_type, ctx);

    // Collect user-defined decorators (excluding classmethod/staticmethod)
    let decorators: Vec<String> = func
        .decorator_list
        .iter()
        .filter_map(|d| {
            if let Expr::Name(n) = &d.expression {
                let name = n.id.to_string();
                if name != "classmethod" && name != "staticmethod" {
                    Some(name)
                } else {
                    None
                }
            } else if matches!(
                &d.expression,
                Expr::Call(call)
                    if matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "attached_api")
            ) {
                Some("attached_api".to_string())
            } else {
                None
            }
        })
        .collect();
    // Collect type parameters for generic functions
    let type_params = ctx
        .generic_functions
        .get::<str>(func.name.as_ref())
        .cloned()
        .unwrap_or_default();
    validate_structural_function_contract(
        StructuralFunctionContract {
            function_name: func.name.as_str(),
            params: &params,
            return_type: &inferred_return_type,
            type_params: &type_params,
            declarations: &rust_interop,
            is_async: effective_is_async,
            span: func.name.range(),
        },
        ctx,
    );

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: inferred_return_type,
        body,
        is_async: effective_is_async,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators,
        rust_interop,
        python_interop,
        compiler_intrinsic,
        type_params,
    })
}

pub(super) fn requires_exhaustive_return_annotation(
    func: &StmtFunctionDef,
    return_type: &Type,
) -> bool {
    if func.returns.is_none() {
        return false;
    }
    match return_type.resolve_alias() {
        Type::None => false,
        Type::Union(members) => !members
            .iter()
            .any(|member| matches!(member.resolve_alias(), Type::None)),
        _ => true,
    }
}
