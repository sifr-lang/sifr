use super::{
    ast_convention_to_param, collect_declared_nonlocals, collect_yield_types,
    first_await_range_in_stmts, first_yield_range_in_stmts, format_type_name,
    function_body_contains_yield, infer_function_return_type, invalid_type_annotation,
    is_valid_error_type, lower_expr, lower_stmts, make_union, ownership_diagnostics,
    reserved_integer_width_name, resolve_type_annotation, str, substitute_type_vars, unknown_type,
    workload_annotations, DiagnosticCode, Expr, FunctionType, HashMap, HirFunction, HirParam,
    LowerCtx, MethodKind, Number, Operator, OwnershipKind, ParamConvention, Ranged,
    StmtFunctionDef, Type,
};
use crate::lower::rust_interop::{
    classify_rust_interop_stub_body, collect_rust_interop_declarations,
    has_rust_interop_decorator_syntax, RustInteropOwner,
};
use crate::lower::{compiler_intrinsics, diagnostics::has_decorator};
pub(in crate::lower) fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
            // Check type variables first (e.g., T from TypeVar)
            if ctx.type_vars.contains(name.id.as_str()) {
                return Type::TypeVar(name.id.to_string());
            }
            // Check type aliases first
            if let Some(alias_ty) = ctx.scope.lookup_type_alias(&name.id) {
                return alias_ty.clone();
            }
            // Check class types
            if let Some(class_ty) = ctx.class_types.get(name.id.as_str()) {
                return class_ty.clone();
            }
            if matches!(name.id.as_str(), "int128" | "uint128") {
                reserved_integer_width_name(ctx, &name.id, name.range());
                return Type::Any;
            }
            if name.id.as_str() == "bigint" {
                ctx.warn_bigint_transition_alias(name.range());
                return Type::BigInt;
            }
            resolve_type_annotation(&name.id).unwrap_or_else(|| {
                unknown_type(ctx, &name.id, name.range());
                Type::Any
            })
        }
        Expr::NoneLiteral(_) => Type::None,
        // Union type syntax: int | str (parsed as BinOp with BitOr)
        Expr::BinOp(binop) if matches!(binop.op, Operator::BitOr) => {
            let left = resolve_annotation_expr(&binop.left, ctx);
            let right = resolve_annotation_expr(&binop.right, ctx);
            make_union(vec![left, right])
        }
        // Literal string in type position: "GET" | "POST"
        Expr::StringLiteral(s) => Type::LiteralStr(s.value.to_str().to_string()),
        // Literal int in type position: 200 | 404
        Expr::NumberLiteral(num) => {
            if let Number::Int(i) = &num.value {
                if let Some(val) = i.as_i64() {
                    Type::LiteralInt(val)
                } else {
                    invalid_type_annotation(
                        ctx,
                        "integer literal too large for type annotation",
                        num.range(),
                    );
                    Type::Any
                }
            } else {
                invalid_type_annotation(
                    ctx,
                    "only integer literals are supported in type annotations",
                    num.range(),
                );
                Type::Any
            }
        }
        // Literal bool in type position: True | False
        Expr::BooleanLiteral(b) => Type::LiteralBool(b.value),
        Expr::Subscript(sub) => {
            // Handle generic type annotations: list[int], dict[str, int], tuple[int, str]
            let base_name = if let Expr::Name(n) = sub.value.as_ref() {
                n.id.to_string()
            } else {
                invalid_type_annotation(ctx, "unsupported type annotation base", sub.value.range());
                return Type::Any;
            };
            match base_name.as_str() {
                "list" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::List(Box::new(elem_ty))
                }
                "set" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Set(Box::new(elem_ty))
                }
                "dict" => {
                    // dict[K, V] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "dict type annotation requires exactly 2 type parameters",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let key_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                        let val_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                        Type::Dict(Box::new(key_ty), Box::new(val_ty))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "dict type annotation requires [K, V] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                "tuple" => {
                    // tuple[A, B, ...] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        let elem_types: Vec<Type> = tuple
                            .elts
                            .iter()
                            .map(|e| resolve_annotation_expr(e, ctx))
                            .collect();
                        Type::Tuple(elem_types)
                    } else {
                        // Single-element tuple: tuple[int]
                        let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                        Type::Tuple(vec![elem_ty])
                    }
                }
                "Iterable" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Iterable(Box::new(elem_ty))
                }
                "Iterator" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Iterator(Box::new(elem_ty))
                }
                "Awaitable" => {
                    let result_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Awaitable(Box::new(result_ty))
                }
                "TimeoutResult" => {
                    if matches!(sub.slice.as_ref(), Expr::Tuple(_)) {
                        invalid_type_annotation(
                            ctx,
                            "TimeoutResult type annotation requires exactly 1 type parameter",
                            sub.slice.range(),
                        );
                        return Type::Any;
                    }
                    let err_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::TimeoutResult(Box::new(err_ty))
                }
                "Failure" => {
                    if matches!(sub.slice.as_ref(), Expr::Tuple(_)) {
                        invalid_type_annotation(
                            ctx,
                            "Failure type annotation requires exactly 1 type parameter",
                            sub.slice.range(),
                        );
                        return Type::Any;
                    }
                    let err_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Failure(Box::new(err_ty))
                }
                "Coroutine" | "Task" | "TaskResult" | "Select2" | "BlockingTask" | "JoinSet"
                | "Future" | "AsyncIterator" | "AsyncGenerator" => {
                    let Expr::Tuple(tuple) = sub.slice.as_ref() else {
                        invalid_type_annotation(
                            ctx,
                            format!("{base_name} type annotation requires [T, E] syntax"),
                            sub.slice.range(),
                        );
                        return Type::Any;
                    };
                    if tuple.elts.len() != 2 {
                        invalid_type_annotation(
                            ctx,
                            format!(
                                "{base_name} type annotation requires exactly 2 type parameters"
                            ),
                            sub.slice.range(),
                        );
                        return Type::Any;
                    }
                    let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                    let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                    match base_name.as_str() {
                        "Coroutine" => Type::Coroutine(Box::new(ok_ty), Box::new(err_ty)),
                        "Task" => Type::Task(Box::new(ok_ty), Box::new(err_ty)),
                        "TaskResult" => Type::TaskResult(Box::new(ok_ty), Box::new(err_ty)),
                        "Select2" => Type::Select2(Box::new(ok_ty), Box::new(err_ty)),
                        "JoinSet" => Type::JoinSet(Box::new(ok_ty), Box::new(err_ty)),
                        "BlockingTask" | "Future" => {
                            Type::BlockingTask(Box::new(ok_ty), Box::new(err_ty))
                        }
                        "AsyncIterator" => Type::AsyncIterator(Box::new(ok_ty), Box::new(err_ty)),
                        "AsyncGenerator" => Type::AsyncGenerator(Box::new(ok_ty), Box::new(err_ty)),
                        _ => Type::Any,
                    }
                }
                "Reversible" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::reversible(elem_ty)
                }
                "Result" => {
                    // Result[T, E] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "Result type annotation requires exactly 2 type parameters",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                        let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                        // Enforce: E must be a class extending Error
                        if !is_valid_error_type(&err_ty, ctx) {
                            let err_name = format_type_name(&err_ty);
                            ctx.error_with_code_at(
                                DiagnosticCode::RESULT_INVALID_ERROR_TYPE,
                                format!(
                                "`{}` is not a valid error type in Result — use a class extending Error, e.g. `Result[{}, ValueError]`",
                                err_name,
                                format_type_name(&ok_ty),
                                ),
                                tuple.elts[1].range(),
                            );
                            return Type::Any;
                        }
                        Type::Result(Box::new(ok_ty), Box::new(err_ty))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "Result type annotation requires [T, E] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                "Option" => {
                    // Option[T] -> T | None (sugar)
                    let inner_ty = resolve_annotation_expr(&sub.slice, ctx);
                    make_union(vec![inner_ty, Type::None])
                }
                "TypeGuard" => {
                    // TypeGuard[T] -- type predicate return type

                    // Store as the inner type; the function signature handler
                    // will recognize TypeGuard and mark it as a type predicate
                    resolve_annotation_expr(&sub.slice, ctx)
                }
                "Callable" => {
                    // Callable[[param_types], return_type]
                    // The slice is a Tuple of [List[param_types], return_type]
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "Callable type requires exactly 2 type parameters: [[param_types], return_type]",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        // First element should be a list of parameter types
                        let param_types = if let Expr::List(list) = &tuple.elts[0] {
                            list.elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect::<Vec<_>>()
                        } else {
                            invalid_type_annotation(
                                ctx,
                                "Callable parameter types must be a list: Callable[[int, str], bool]",
                                tuple.elts[0].range(),
                            );
                            return Type::Any;
                        };
                        let return_type = resolve_annotation_expr(&tuple.elts[1], ctx);
                        let conventions = param_types
                            .iter()
                            .map(|ty| {
                                if ty.ownership() == OwnershipKind::Copy {
                                    ParamConvention::own()
                                } else {
                                    ParamConvention::borrow()
                                }
                            })
                            .collect();
                        Type::Callable(param_types, conventions, Box::new(return_type))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "Callable type requires [[param_types], return_type] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                _ => {
                    // Check if it's a generic type alias (e.g., Pair[int])
                    if let Some((alias_params, alias_body)) =
                        ctx.scope.lookup_generic_type_alias(&base_name).cloned()
                    {
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup
                                .elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        if alias_params.len() != type_args.len() {
                            invalid_type_annotation(
                                ctx,
                                format!(
                                "generic type alias '{base_name}' expects {} type argument(s), got {}",
                                alias_params.len(),
                                type_args.len()
                                ),
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let mut bindings = HashMap::new();
                        for (i, tp) in alias_params.iter().enumerate() {
                            if let Some(arg) = type_args.get(i) {
                                bindings.insert(tp.clone(), arg.clone());
                            }
                        }
                        return substitute_type_vars(&alias_body, &bindings);
                    }
                    // Check if it's a generic class instantiation (e.g., Stack[int])
                    if let Some(class_ty) = ctx.class_types.get(&base_name).cloned() {
                        // Resolve type arguments and substitute into the class type
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup
                                .elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        // Build substitution map from class type params to concrete args
                        if let Type::Class {
                            ref fields,
                            ref methods,
                            ref parent_class,
                            ..
                        } = class_ty
                        {
                            let class_type_params = ctx
                                .class_declared_type_params
                                .get(&base_name)
                                .cloned()
                                .unwrap_or_default();
                            if !type_args.is_empty() {
                                if class_type_params.is_empty() {
                                    invalid_type_annotation(
                                        ctx,
                                        format!(
                                        "class '{base_name}' does not declare type parameters; use `class {base_name}[T]: ...`"
                                        ),
                                        sub.value.range(),
                                    );
                                    return Type::Any;
                                }
                                if class_type_params.len() != type_args.len() {
                                    invalid_type_annotation(
                                        ctx,
                                        format!(
                                        "generic class '{base_name}' expects {} type argument(s), got {}",
                                        class_type_params.len(),
                                        type_args.len()
                                        ),
                                        sub.slice.range(),
                                    );
                                    return Type::Any;
                                }
                                let mut bindings = HashMap::new();
                                for (tp, arg) in class_type_params.iter().zip(type_args.iter()) {
                                    bindings.insert(tp.clone(), arg.clone());
                                }
                                let subst_fields: Vec<(String, Type)> = fields
                                    .iter()
                                    .map(|(n, t)| (n.clone(), substitute_type_vars(t, &bindings)))
                                    .collect();
                                let subst_methods: Vec<(String, FunctionType)> = methods
                                    .iter()
                                    .map(|(n, ft)| {
                                        let subst_params: Vec<(String, Type, ParamConvention)> = ft
                                            .params
                                            .iter()
                                            .map(|(pn, pt, pc)| {
                                                (
                                                    pn.clone(),
                                                    substitute_type_vars(pt, &bindings),
                                                    *pc,
                                                )
                                            })
                                            .collect();
                                        let subst_ret =
                                            substitute_type_vars(&ft.return_type, &bindings);
                                        (
                                            n.clone(),
                                            FunctionType {
                                                params: subst_params,
                                                return_type: Box::new(subst_ret),
                                            },
                                        )
                                    })
                                    .collect();
                                return Type::Class {
                                    name: base_name.clone(),
                                    fields: subst_fields,
                                    methods: subst_methods,
                                    parent_class: parent_class.clone(),
                                };
                            }
                        }
                        class_ty
                    } else {
                        unknown_type(ctx, &base_name, sub.value.range());
                        Type::Any
                    }
                }
            }
        }
        _ => {
            invalid_type_annotation(ctx, "unsupported type annotation expression", expr.range());
            Type::Any
        }
    }
}

pub(in crate::lower) fn lower_function(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) -> Option<HirFunction> {
    let ft = ctx.functions.get::<str>(func.name.as_ref())?.clone();
    let effective_is_async = func.is_async;

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
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

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
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());
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
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        params.push(HirParam {
            name,
            ty,
            default,
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
    let compiler_intrinsic = ctx.compiler_intrinsics.get(func.name.as_str()).copied();
    let has_compiler_intrinsic_syntax =
        compiler_intrinsics::has_decorator_syntax(&func.decorator_list);
    let stub_body = if has_compiler_intrinsic_syntax {
        if !rust_interop.is_empty() {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
                "@compiler_intrinsic and Rust interop decorators cannot be combined".to_string(),
                func.name.range(),
            );
        }
        compiler_intrinsics::classify_stub_body(func, compiler_intrinsic, ctx)
    } else {
        classify_rust_interop_stub_body(
            &func.body,
            has_rust_interop_decorator_syntax(&func.decorator_list),
            ctx,
        )
    };

    let is_async_generator = !stub_body.skips_normal_body_lowering()
        && effective_is_async
        && function_body_contains_yield(&func.body);
    workload_annotations::reject_async_function_annotation(ctx, func, effective_is_async);
    if !stub_body.skips_normal_body_lowering()
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
    if !stub_body.skips_normal_body_lowering() && effective_is_async {
        if is_async_generator {
            if let Some(yield_range) = first_yield_range_in_stmts(&func.body) {
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
    let previous_join_set_terminal_awaitables =
        std::mem::take(&mut ctx.join_set_terminal_awaitables);
    ctx.current_function_is_async = effective_is_async;
    ctx.current_function_is_async_generator = is_async_generator;
    ctx.current_function_trusts_dynamic_python = has_decorator(func, "trust_python_dynamic");
    let body = if stub_body.skips_normal_body_lowering() {
        Vec::new()
    } else {
        lower_stmts(&func.body, &ft, ctx)
    };
    reject_live_join_sets_at_function_exit(func, ctx);
    ctx.live_join_set_bindings = previous_live_join_sets;
    ctx.join_set_terminal_awaitables = previous_join_set_terminal_awaitables;
    ctx.current_function_is_async = previous_async;
    ctx.current_function_is_async_generator = previous_async_generator;
    ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
    ctx.current_function_return_type = previous_return_type;
    ctx.current_owner = previous_owner;

    ctx.borrowed_params.clear();

    ctx.exit_function_scope();

    let has_yield = !collect_yield_types(&body).is_empty();
    if !stub_body.skips_normal_body_lowering()
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
    let inferred_return_type = if stub_body.skips_normal_body_lowering() {
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

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: inferred_return_type,
        body,
        is_async: effective_is_async,
        method_kind: MethodKind::Regular,
        decorators,
        rust_interop,
        compiler_intrinsic,
        type_params,
    })
}

fn reject_live_join_sets_at_function_exit(func: &StmtFunctionDef, ctx: &mut LowerCtx) {
    let mut live_sets = ctx
        .live_join_set_bindings
        .iter()
        .filter(|name| !ctx.scope.is_moved(name))
        .cloned()
        .collect::<Vec<_>>();
    live_sets.sort();
    for name in live_sets {
        ctx.error_with_code_at(
            DiagnosticCode::OWN_USE_AFTER_MOVE,
            format!(
                "JoinSet binding '{name}' accepted task handles and must be consumed with await {name}.join_all() or await {name}.cancel_all() before function exit"
            ),
            func.name.range(),
        );
    }
    ctx.live_join_set_bindings.clear();
    ctx.join_set_terminal_awaitables.clear();
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
