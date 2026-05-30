use super::{HirExceptHandler, HirExpr, HirIteratorOp, HirStmt, RustExpr, RustStmt, Type};
pub(crate) fn io_error_kind_for_handler(error_type: &str) -> Option<&'static str> {
    match error_type {
        "FileNotFoundError" => Some("FileNotFound"),
        "PermissionError" => Some("PermissionDenied"),
        "FileExistsError" => Some("FileExists"),
        "IsADirectoryError" => Some("IsADirectory"),
        "NotADirectoryError" => Some("NotADirectory"),
        "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
        _ => None,
    }
}

pub(crate) fn select_try_error_type(handlers: &[HirExceptHandler]) -> String {
    if handlers.iter().any(|handler| {
        let Some(error_type) = handler.error_type.as_deref() else {
            return false;
        };
        error_type == "IOError" || io_error_kind_for_handler(error_type).is_some()
    }) {
        return "IOError".to_string();
    }

    handlers
        .first()
        .and_then(|handler| handler.error_resolved_type.as_ref())
        .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
        .unwrap_or_else(|| "Error".to_string())
}

pub(crate) fn first_try_error_type_in_stmts(stmts: &[HirStmt]) -> Option<String> {
    for stmt in stmts {
        if let Some(error_type) = first_try_error_type_in_stmt(stmt) {
            return Some(error_type);
        }
    }
    None
}

pub(crate) fn first_try_error_type_in_stmt(stmt: &HirStmt) -> Option<String> {
    match stmt {
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => body_error_types.first().cloned().or_else(|| {
            first_try_error_type_in_stmts(body).or_else(|| {
                handlers
                    .iter()
                    .find_map(|handler| first_try_error_type_in_stmts(&handler.body))
            })
        }),
        HirStmt::TryFinally { body, finalbody } => {
            first_try_error_type_in_stmts(body).or_else(|| first_try_error_type_in_stmts(finalbody))
        }
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => first_try_error_type_in_stmts(then_body)
            .or_else(|| {
                elif_clauses
                    .iter()
                    .find_map(|(_, body)| first_try_error_type_in_stmts(body))
            })
            .or_else(|| else_body.as_deref().and_then(first_try_error_type_in_stmts)),
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        } => first_try_error_type_in_stmts(body)
            .or_else(|| else_body.as_deref().and_then(first_try_error_type_in_stmts)),
        HirStmt::With { body, .. }
        | HirStmt::AsyncWith { body, .. }
        | HirStmt::AsyncFor { body, .. } => first_try_error_type_in_stmts(body),
        HirStmt::NestedFunction { func } => first_try_error_type_in_stmts(&func.body),
        HirStmt::Match { arms, .. } => arms
            .iter()
            .find_map(|arm| first_try_error_type_in_stmts(&arm.body)),
        _ => None,
    }
}

pub(crate) fn can_construct_error_from_message_for_ir(ty_name: &str) -> bool {
    matches!(
        ty_name,
        "Error"
            | "ValueError"
            | "TypeError"
            | "NameError"
            | "ParseError"
            | "OverflowError"
            | "ZeroDivisionError"
            | "LookupError"
            | "IndexError"
            | "KeyError"
            | "RuntimeError"
            | "AssertionError"
            | "ImportError"
            | "IOError"
            | "RegexError"
            | "JsonIntegerRangeError"
            | "JsonLimitError"
            | "HashlibError"
            | "DecimalConversionError"
            | "TimeoutError"
            | "ScopeFailure"
            | "TaskCancelled"
            | "SecondaryError"
    )
}

pub(crate) enum HandlerMatchCondition {
    Unsupported,
    Always,
    Expr(RustExpr),
}

pub(crate) fn canonical_constructor_class_name(class_name: &str) -> &str {
    class_name
        .strip_prefix("__compat_sifr_collections_")
        .unwrap_or(class_name)
}

pub(crate) fn canonical_plain_call_name_for_ir(func: &str) -> &str {
    func.strip_prefix("__compat_sifr_math_")
        .or_else(|| func.strip_prefix("__compat_sifr_heapq_"))
        .unwrap_or(func)
}

pub(crate) fn supports_nonempty_pop_narrowing_type_for_ir(object_ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(object_ty) {
        Type::List(_) => true,
        Type::Class { name, .. } => is_deque_class_name_for_ir(name),
        _ => false,
    }
}

pub(crate) fn is_deque_class_name_for_ir(name: &str) -> bool {
    name == "deque"
        || name
            .rsplit_once('.')
            .is_some_and(|(_, tail)| tail == "deque")
}

pub(crate) fn is_narrowable_pop_call_for_ir(method: &str, args: &[HirExpr]) -> bool {
    match method {
        "pop" => matches!(args, [] | [HirExpr::IntLiteral(0)]),
        "popleft" => args.is_empty(),
        _ => false,
    }
}

pub(crate) fn unwrap_compiler_verified_nonempty_pop_result_for_ir(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    method_return_ty: &Type,
    lowered_expr: RustExpr,
) -> RustExpr {
    if !supports_nonempty_pop_narrowing_type_for_ir(object_ty) {
        return lowered_expr;
    }
    if !is_narrowable_pop_call_for_ir(method, args) {
        return lowered_expr;
    }
    if crate::helpers::is_option_type(method_return_ty) {
        return lowered_expr;
    }
    RustExpr::Block {
        stmts: vec![RustStmt::LetElse {
            pattern: "Some(__sifr_nonempty_pop_value)".to_string(),
            value: lowered_expr,
            else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                name: "unreachable".to_string(),
                args: vec![RustExpr::Literal(crate::RustLiteral::Str(
                    "compiler-verified non-empty pop should return Some".to_string(),
                ))],
            })],
        }],
        expr: Some(Box::new(RustExpr::Ident(
            "__sifr_nonempty_pop_value".to_string(),
        ))),
    }
}

pub(crate) fn iterator_call_func_name(op: &HirIteratorOp) -> &'static str {
    match op {
        HirIteratorOp::Iter => "iter",
        HirIteratorOp::Next => "next",
        HirIteratorOp::Reversed => "reversed",
        HirIteratorOp::Map => "map",
        HirIteratorOp::Filter => "filter",
        HirIteratorOp::Zip => "zip",
        HirIteratorOp::Enumerate => "enumerate",
    }
}

pub(crate) fn call_expr_parts(expr: &HirExpr) -> Option<(&str, &[HirExpr])> {
    match expr {
        HirExpr::Call { func, args, .. } => Some((func.as_str(), args.as_slice())),
        HirExpr::IteratorCall { op, args, .. } => {
            Some((iterator_call_func_name(op), args.as_slice()))
        }
        _ => None,
    }
}

pub(crate) fn should_omit_local_type_annotation(ty: &Type, value: &HirExpr) -> bool {
    match (ty, value) {
        (resolved_ty, HirExpr::Call { func, args, .. })
            if matches!(
                crate::resolve_alias_type_for_plain_call(resolved_ty),
                Type::Set(_)
            ) && func == "set"
                && args.is_empty() =>
        {
            true
        }
        (
            Type::Alias {
                name: alias_name,
                body,
                ..
            },
            HirExpr::Call { func, args, .. },
        ) if func == alias_name
            && args.is_empty()
            && alias_name.starts_with("__compat_defaultdict_") =>
        {
            let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
                return false;
            };
            matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
        }
        _ => false,
    }
}

pub(crate) fn should_force_mutable_binding(ty: &Type) -> bool {
    fn class_has_next_protocol(ty: &Type) -> bool {
        let Type::Class { methods, .. } = ty.resolve_alias() else {
            return false;
        };
        methods.iter().any(|(name, ft)| {
            name == "__next__"
                && ft.params.is_empty()
                && matches!(ft.return_type.as_ref().resolve_alias(), Type::Union(members) if {
                    let has_none = members
                        .iter()
                        .any(|member| matches!(member.resolve_alias(), Type::None));
                    let non_none = members
                        .iter()
                        .filter(|member| !matches!(member.resolve_alias(), Type::None))
                        .count();
                    has_none && non_none == 1
                })
        })
    }

    fn class_has_recursive_option_field(ty: &Type) -> bool {
        let Type::Class { name, fields, .. } = ty.resolve_alias() else {
            return false;
        };
        fields.iter().any(|(_, field_ty)| {
            let Type::Union(members) = field_ty.resolve_alias() else {
                return false;
            };
            members.iter().any(|member| {
                matches!(member.resolve_alias(), Type::Class { name: field_name, .. } if field_name == name)
            })
        })
    }

    matches!(
        ty,
        Type::Alias { name: alias_name, .. } if alias_name.starts_with("__compat_defaultdict_")
    ) || matches!(ty.resolve_alias(), Type::Iterator(_))
        || class_has_next_protocol(ty)
        || class_has_recursive_option_field(ty)
}

pub(crate) fn type_contains_any_or_unknown(ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(ty) {
        Type::Any | Type::Unknown => true,
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Alias { body: inner, .. } => type_contains_any_or_unknown(inner),
        Type::Dict(key, value) | Type::Result(key, value) => {
            type_contains_any_or_unknown(key) || type_contains_any_or_unknown(value)
        }
        Type::Tuple(elements) | Type::Union(elements) | Type::Intersection(elements) => {
            elements.iter().any(type_contains_any_or_unknown)
        }
        Type::Callable(params, _, ret) => {
            params.iter().any(type_contains_any_or_unknown) || type_contains_any_or_unknown(ret)
        }
        Type::Function(ft) => {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_any_or_unknown(param_ty))
                || type_contains_any_or_unknown(&ft.return_type)
        }
        _ => false,
    }
}

macro_rules! stmt_expr_await_and_registry {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::Await { value, .. } = $expr {
            if let Some(duration) = $emitter.active_timeout_durations.last().cloned() {
                let Some(future) = $emitter.lower_timeout_aware_await_future_for_ir(value)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::TimeoutAwait {
                    duration: Box::new(duration),
                    future: Box::new(future),
                }));
            }
            if let HirExpr::Call { func, args, .. } = value.as_ref() {
                if func == "__sifr_task_sleep" {
                    let [duration] = args.as_slice() else {
                        return Ok(None);
                    };
                    let Some(duration_expr) =
                        crate::try_lower_task_duration_expr(duration, "__sifr_task_sleep_seconds")
                    else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::Await(Box::new(
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "tokio".to_string(),
                                "time".to_string(),
                                "sleep".to_string(),
                            ])),
                            args: vec![duration_expr],
                        },
                    ))));
                }
            }
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            let awaited_value = match crate::resolve_alias_type_for_plain_call(value.ty()) {
                Type::Task(_, _) | Type::BlockingTask(_, _) => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_value),
                    method: "join".to_string(),
                    args: vec![],
                },
                _ => lowered_value,
            };
            return Ok(Some(crate::RustExpr::Await(Box::new(awaited_value))));
        }

        let skip_leaf_registry_lowering = matches!(
            $expr,
            HirExpr::Call { .. }
                | HirExpr::IteratorCall { .. }
                | HirExpr::ConstructorCall { .. }
                | HirExpr::MethodCall { .. }
                | HirExpr::BinOp { .. }
                | HirExpr::UnaryOp { .. }
                | HirExpr::Compare { .. }
                | HirExpr::BoolOp { .. }
                | HirExpr::Slice { .. }
        );
        if !skip_leaf_registry_lowering {
            if let Some(lowered) = $emitter.try_lower_registry_expr_result($expr)? {
                return Ok(Some(lowered));
            }
        }
        if let HirExpr::Call { func, args, .. } = $expr {
            if func == "print" {
                return $emitter.lower_print_call_expr_for_ir(args);
            }
        }
        if let HirExpr::FieldAccess { object, field, ty } = $expr {
            if let Some(lowered) =
                $emitter.try_lower_structured_field_access_expr(object, field, ty)?
            {
                return Ok(Some(lowered));
            }
        }
    }};
}

macro_rules! stmt_expr_constructor {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::ConstructorCall {
            class_name, args, ..
        } = $expr
        {
            let emitted_class_name = canonical_constructor_class_name(class_name).to_string();
            let ctor_key = format!("{emitted_class_name}::new");
            let ctor_params = $emitter
                .func_signatures
                .get(&ctor_key)
                .map(|(params, _)| params.clone());
            if let Some(mut lowered_ctor) =
                $emitter.try_lower_registry_plain_call_with_signature(&ctor_key, args)
            {
                if let Some(params) = ctor_params.as_ref() {
                    if let crate::RustExpr::FnCall {
                        args: lowered_args, ..
                    } = &mut lowered_ctor
                    {
                        for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                            let Some((param_ty, _)) = params.get(idx) else {
                                continue;
                            };
                            let is_recursive_ctor_field = $emitter
                                .class_field_order
                                .get(class_name)
                                .and_then(|fields| fields.get(idx))
                                .is_some_and(|field_name| {
                                    $emitter.recursive_fields
                                        .contains(&(class_name.clone(), field_name.clone()))
                                });
                            let is_recursive_container_param = matches!(
                                crate::resolve_alias_type_for_plain_call(param_ty),
                                Type::List(elem)
                                    if matches!(
                                        crate::resolve_alias_type_for_plain_call(elem.as_ref()),
                                        Type::Class { name, .. } if name == class_name
                                    )
                            ) || matches!(
                                crate::resolve_alias_type_for_plain_call(param_ty),
                                Type::Dict(_, value_ty)
                                    if matches!(
                                        crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                                        Type::Class { name, .. } if name == class_name
                                    )
                            );
                            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                            if !crate::helpers::is_option_type(resolved_param) {
                                if (is_recursive_ctor_field || is_recursive_container_param)
                                    && !Self::is_box_new_call_expr_for_ir(lowered_arg)
                                {
                                    *lowered_arg = crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "Box".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![lowered_arg.clone()],
                                    };
                                }
                                continue;
                            }
                            let needs_box_inner = param_ty.rust_type().starts_with("Option<Box<")
                                || is_recursive_ctor_field;
                            if !needs_box_inner || matches!(args[idx], HirExpr::NoneLiteral) {
                                continue;
                            }
                            let arg_is_option = crate::helpers::is_option_type(args[idx].ty());
                            if arg_is_option {
                                *lowered_arg =
                                    Self::ensure_option_box_inner_for_ir(lowered_arg.clone());
                            } else {
                                *lowered_arg =
                                    Self::ensure_some_box_inner_for_ir(lowered_arg.clone());
                            }
                        }
                    }
                }
                return Ok(Some(lowered_ctor));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                let adapted_arg = if let HirExpr::Name { name, ty } = arg {
                    if ($emitter.borrowed_params.contains(name)
                        || $emitter.mut_borrowed_params.contains(name))
                        && ty.ownership() != sifr_type_system::OwnershipKind::Copy
                    {
                        crate::RustExpr::Clone(Box::new(lowered_arg))
                    } else {
                        lowered_arg
                    }
                } else {
                    lowered_arg
                };
                lowered_args.push(adapted_arg);
            }
            for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                let is_recursive_ctor_field = $emitter
                    .class_field_order
                    .get(class_name)
                    .and_then(|fields| fields.get(idx))
                    .is_some_and(|field_name| {
                        $emitter.recursive_fields
                            .contains(&(class_name.clone(), field_name.clone()))
                    });
                let is_recursive_container_arg = matches!(
                    crate::resolve_alias_type_for_plain_call(args[idx].ty()),
                    Type::List(elem)
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(elem.as_ref()),
                            Type::Class { name, .. } if name == class_name
                        )
                ) || matches!(
                    crate::resolve_alias_type_for_plain_call(args[idx].ty()),
                    Type::Dict(_, value_ty)
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                            Type::Class { name, .. } if name == class_name
                        )
                );
                if (!is_recursive_ctor_field && !is_recursive_container_arg)
                    || matches!(args[idx], HirExpr::NoneLiteral)
                {
                    continue;
                }
                let resolved_arg_ty = crate::resolve_alias_type_for_plain_call(args[idx].ty());
                if crate::helpers::is_option_type(resolved_arg_ty) {
                    *lowered_arg = Self::ensure_option_box_inner_for_ir(lowered_arg.clone());
                } else if !Self::is_box_new_call_expr_for_ir(lowered_arg) {
                    *lowered_arg = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Box".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![lowered_arg.clone()],
                    };
                }
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    emitted_class_name,
                    "new".to_string(),
                ])),
                args: lowered_args,
            }));
        }
    }};
}

macro_rules! stmt_expr_literals_and_calls {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::FString { parts, .. } = $expr {
            let mut format_str = String::new();
            let mut args = Vec::new();
            for part in parts {
                match part {
                    HirFStringPart::Literal(text) => {
                        format_str.push_str(&text.replace('{', "{{").replace('}', "}}"));
                    }
                    HirFStringPart::Expr(inner) => {
                        let Some(lowered_inner) = $emitter.lower_stmt_expr_for_ir(inner)? else {
                            return Ok(None);
                        };
                        format_str.push_str("{}");
                        args.push(lowered_inner);
                    }
                }
            }
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str,
                args,
            }));
        }
        if let HirExpr::ListLiteral { elements, ty } = $expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            let list_ty = crate::resolve_alias_type_for_plain_call(ty);
            for element in elements {
                let Some(mut lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                lowered_element = Self::clone_non_copy_name_expr_for_ir(element, lowered_element);
                if matches!(list_ty, Type::Bytes) {
                    lowered_element = crate::RustExpr::Cast {
                        expr: Box::new(lowered_element),
                        ty: crate::RustType::Named("u8".to_string()),
                    };
                }
                lowered_elements.push(lowered_element);
            }
            return Ok(Some(crate::RustExpr::Vec(lowered_elements)));
        }
        if let HirExpr::TupleLiteral { elements, .. } = $expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let Some(lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                lowered_elements.push(Self::clone_non_copy_name_expr_for_ir(
                    element,
                    lowered_element,
                ));
            }
            return Ok(Some(crate::RustExpr::Tuple(lowered_elements)));
        }
        if let HirExpr::DictLiteral { keys, values, .. } = $expr {
            if keys.len() != values.len() {
                return Ok(None);
            }
            let mut stmts = Vec::with_capacity(keys.len() + 1);
            stmts.push(crate::RustStmt::Let {
                mutable: true,
                name: "__dict".to_string(),
                ty: None,
                value: crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "HashMap".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            });
            for (key, value) in keys.iter().zip(values.iter()) {
                let Some(lowered_key) = $emitter.lower_stmt_expr_for_ir(key)? else {
                    return Ok(None);
                };
                let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__dict".to_string())),
                    method: "insert".to_string(),
                    args: vec![lowered_key, lowered_value],
                }));
            }
            return Ok(Some(crate::RustExpr::Block {
                stmts,
                expr: Some(Box::new(crate::RustExpr::Ident("__dict".to_string()))),
            }));
        }
        if let HirExpr::SetLiteral { elements, .. } = $expr {
            let mut stmts = Vec::with_capacity(elements.len() + 1);
            stmts.push(crate::RustStmt::Let {
                mutable: true,
                name: "__set".to_string(),
                ty: None,
                value: crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "HashSet".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            });
            for element in elements {
                let Some(lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__set".to_string())),
                    method: "insert".to_string(),
                    args: vec![lowered_element],
                }));
            }
            return Ok(Some(crate::RustExpr::Block {
                stmts,
                expr: Some(Box::new(crate::RustExpr::Ident("__set".to_string()))),
            }));
        }
        if let Some(lowered_comprehension) = $emitter.try_lower_comprehension_expr_for_ir($expr)? {
            return Ok(Some(lowered_comprehension));
        }
        if let HirExpr::GeneratorExpr {
            expr: value_expr,
            var,
            iter,
            filter,
            ty,
        } = $expr
        {
            if let Some(lowered_generator) = $emitter.try_lower_generator_expr_for_ir(
                value_expr,
                var,
                iter,
                filter.as_deref(),
                ty,
            )? {
                return Ok(Some(lowered_generator));
            }
        }
        if let Some((func, args)) = call_expr_parts($expr) {
            if let Some(lowered_intrinsic) =
                $emitter.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
            }
            if let Some(lowered_builtin) =
                $emitter.try_lower_registry_builtin_call_expr(func, args, Some($expr.ty()))
            {
                return Ok(Some(lowered_builtin));
            }
            if let Some(lowered_plain) =
                $emitter.try_lower_registry_plain_call_with_signature(func, args)
            {
                return Ok(Some(lowered_plain));
            }
            if func == "iter" && args.len() == 1 {
                return $emitter.lower_iter_source_expr_for_ir(&args[0]);
            }
            if func == "next" && args.len() == 1 {
                let Some(lowered_iterator) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "next".to_string(),
                    args: vec![],
                }));
            }
            if func == "anext" && args.len() == 1 {
                let Some(lowered_iterator) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "anext".to_string(),
                    args: vec![],
                }));
            }
            if func == "str" && args.is_empty() {
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }));
            }
            if func == "str" && args.len() == 1 {
                let arg = &args[0];
                let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                    let format_str = if Self::uses_debug_display_format_for_ir(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }));
                }
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: if Self::uses_debug_display_format_for_ir(arg.ty()) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    },
                    args: vec![lowered_arg],
                }));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            let canonical_func = canonical_plain_call_name_for_ir(func);
            lowered_args = $emitter.adapt_plain_call_args_with_signature_for_ir(
                canonical_func,
                args,
                lowered_args,
            );
            if let Some(captures) = $emitter.nested_fn_captures.get(func).cloned() {
                for capture in captures {
                    lowered_args.push($emitter.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(
                    canonical_func
                        .split("::")
                        .map(ToString::to_string)
                        .collect(),
                )),
                args: lowered_args,
            }));
        }
    }};
}
