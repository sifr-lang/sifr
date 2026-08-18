use super::annotation_union_validation::has_conflicting_class_specializations;
use super::{
    format_type_name, invalid_type_annotation, is_valid_error_type, make_union,
    reserved_integer_width_name, resolve_python_dlpack_tensor_annotation,
    resolve_python_resource_attribute_annotation, resolve_type_annotation, substitute_type_vars,
    unknown_type, DiagnosticCode, Expr, FunctionType, HashMap, LowerCtx, Number, Operator,
    OwnershipKind, ParamConvention, Ranged, Type,
};

pub(in crate::lower) fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
            if ctx.adapter_marker_bindings.contains_key(name.id.as_str()) {
                ctx.error_with_code_at(
                    DiagnosticCode::META_MALFORMED_DECLARATION,
                    "class-adapter markers are erased base-only declarations and cannot be used as annotations".to_string(),
                    name.range(),
                );
                return Type::Any;
            }
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
            resolve_type_annotation(&name.id).unwrap_or_else(|| {
                unknown_type(ctx, &name.id, name.range());
                Type::Any
            })
        }
        Expr::NoneLiteral(_) => Type::None,
        Expr::Attribute(attribute) if matches!(attribute.value.as_ref(), Expr::Name(root) if root.id.as_str() == "python") => {
            resolve_python_resource_attribute_annotation(
                attribute.attr.as_str(),
                attribute.range(),
                ctx,
            )
        }
        // Union type syntax: int | str (parsed as BinOp with BitOr)
        Expr::BinOp(binop) if matches!(binop.op, Operator::BitOr) => {
            let left = resolve_annotation_expr(&binop.left, ctx);
            let right = resolve_annotation_expr(&binop.right, ctx);
            let union = make_union(vec![left, right]);
            if has_conflicting_class_specializations(&union) {
                invalid_type_annotation(
                    ctx,
                    "a union cannot contain multiple specializations of the same generic class",
                    binop.range(),
                );
                Type::Any
            } else {
                union
            }
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
            let base_name = match sub.value.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                Expr::Attribute(attribute)
                    if matches!(attribute.value.as_ref(), Expr::Name(root) if root.id.as_str() == "python")
                        && attribute.attr.as_str() == "Buffer" =>
                {
                    return super::resolve_python_buffer_annotation(&sub.slice, ctx);
                }
                Expr::Attribute(attribute)
                    if matches!(attribute.value.as_ref(), Expr::Name(root) if root.id.as_str() == "python")
                        && attribute.attr.as_str() == "DlpackTensor" =>
                {
                    return resolve_python_dlpack_tensor_annotation(&sub.slice, ctx);
                }
                _ => {
                    invalid_type_annotation(
                        ctx,
                        "unsupported type annotation base",
                        sub.value.range(),
                    );
                    return Type::Any;
                }
            };
            match base_name.as_str() {
                "Annotated" => {
                    let Expr::Tuple(tuple) = sub.slice.as_ref() else {
                        invalid_type_annotation(
                            ctx,
                            "Annotated requires a type and at least one descriptor",
                            sub.slice.range(),
                        );
                        return Type::Any;
                    };
                    if tuple.elts.len() < 2 {
                        invalid_type_annotation(
                            ctx,
                            "Annotated requires a type and at least one descriptor",
                            sub.slice.range(),
                        );
                        return Type::Any;
                    }
                    resolve_annotation_expr(&tuple.elts[0], ctx)
                }
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
                "Callable" | "AsyncCallable" => {
                    // Callable[[param_types], return_type] and
                    // AsyncCallable[[param_types], return_type].
                    let is_async = base_name == "AsyncCallable";
                    let label = if is_async {
                        "AsyncCallable"
                    } else {
                        "Callable"
                    };
                    // The slice is a Tuple of [List[param_types], return_type]
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                format!("{label} type requires exactly 2 type parameters: [[param_types], return_type]"),
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
                                format!("{label} parameter types must be a list: {label}[[int, str], bool]"),
                                tuple.elts[0].range(),
                            );
                            return Type::Any;
                        };
                        let return_type = resolve_annotation_expr(&tuple.elts[1], ctx);
                        let conventions = param_types
                            .iter()
                            .map(|ty| {
                                if is_async || ty.ownership() == OwnershipKind::Copy {
                                    // An async callable future must own its boundary
                                    // values; a borrowed argument cannot outlive the
                                    // closure call that constructs the future.
                                    ParamConvention::own()
                                } else {
                                    ParamConvention::borrow()
                                }
                            })
                            .collect();
                        if is_async {
                            Type::AsyncCallable(param_types, conventions, Box::new(return_type))
                        } else {
                            Type::Callable(param_types, conventions, Box::new(return_type))
                        }
                    } else {
                        invalid_type_annotation(
                            ctx,
                            format!("{label} type requires [[param_types], return_type] syntax"),
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
                            ref identity,
                            ref name,
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
                                                receiver: ft.receiver,
                                                params: subst_params,
                                                return_type: Box::new(subst_ret),
                                            },
                                        )
                                    })
                                    .collect();
                                return Type::Class {
                                    // Keep the identity selected by import resolution. Merged
                                    // stdlib classes stay canonical, while project-module aliases
                                    // use their collision-safe local emitted spelling.
                                    identity: identity.clone(),
                                    type_args: type_args.clone(),
                                    name: name.clone(),
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
