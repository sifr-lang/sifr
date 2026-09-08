macro_rules! stmt_expr_constructor {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::ConstructorCall {
            class_name, args, ty,
        } = $expr
        {
            let emitted_class_name = canonical_constructor_class_name(class_name, ty);
            let ctor_key = format!("{emitted_class_name}::new");
            let source_ctor_key = format!("{class_name}::new");
            let registry_ctor_key = if $emitter.func_signatures.contains_key(&ctor_key) {
                &ctor_key
            } else {
                &source_ctor_key
            };
            let ctor_params = $emitter
                .func_signatures
                .get(&ctor_key)
                .or_else(|| $emitter.func_signatures.get(&source_ctor_key))
                .map(|(params, _)| params.clone());
            if emitted_class_name == "Counter"
                && args.len() == 2
                && matches!(args[0], HirExpr::NoneLiteral)
            {
                if let HirExpr::Call {
                    func,
                    args: list_args,
                    ..
                } = &args[1]
                {
                    if func == "list"
                        && list_args.len() == 1
                        && matches!(
                            crate::resolve_alias_type_for_plain_call(list_args[0].ty()),
                            Type::Str
                        )
                    {
                        let Some(source) = $emitter.lower_stmt_expr_for_ir(&list_args[0])? else {
                            return Ok(None);
                        };
                        return Ok(Some(crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_counter_chars".to_string(),
                                    ty: Some(crate::RustType::HashMap(
                                        Box::new(crate::RustType::Named("char".to_string())),
                                        Box::new(crate::RustType::Named("usize".to_string())),
                                    )),
                                    value: crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "HashMap".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                },
                                crate::RustStmt::For {
                                    var: "__sifr_counter_char".to_string(),
                                    iter: crate::RustExpr::MethodCall {
                                        receiver: Box::new(source),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    },
                                    body: vec![crate::RustStmt::AugAssign {
                                        target: crate::RustExpr::Deref(Box::new(
                                            crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "__sifr_counter_chars".to_string(),
                                                    )),
                                                    method: "entry".to_string(),
                                                    args: vec![crate::RustExpr::Ident(
                                                        "__sifr_counter_char".to_string(),
                                                    )],
                                                }),
                                                method: "or_insert".to_string(),
                                                args: vec![crate::RustExpr::Literal(
                                                    crate::RustLiteral::Int(0),
                                                )],
                                            },
                                        )),
                                        op: "+=".to_string(),
                                        value: crate::RustExpr::Literal(crate::RustLiteral::Int(1)),
                                    }],
                                },
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_counter_counts".to_string(),
                                    ty: Some(crate::RustType::HashMap(
                                        Box::new(crate::RustType::String_),
                                        Box::new(crate::RustType::Named(
                                            "SifrInt".to_string(),
                                        )),
                                    )),
                                    value: crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "HashMap".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                },
                                crate::RustStmt::For {
                                    var: "(__sifr_counter_char, __sifr_counter_count)"
                                        .to_string(),
                                    iter: crate::RustExpr::Ident(
                                        "__sifr_counter_chars".to_string(),
                                    ),
                                    body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "__sifr_counter_counts".to_string(),
                                        )),
                                        method: "insert".to_string(),
                                        args: vec![
                                            crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::Ident(
                                                    "__sifr_counter_char".to_string(),
                                                )),
                                                method: "to_string".to_string(),
                                                args: vec![],
                                            },
                                            crate::RustExpr::FnCall {
                                                func: Box::new(crate::RustExpr::Path(vec![
                                                    "SifrInt".to_string(),
                                                    "from".to_string(),
                                                ])),
                                                args: vec![crate::RustExpr::Ident(
                                                    "__sifr_counter_count".to_string(),
                                                )],
                                            },
                                        ],
                                    })],
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::StructInit {
                                name: "Counter".to_string(),
                                fields: vec![(
                                    "counts".to_string(),
                                    crate::RustExpr::Ident("__sifr_counter_counts".to_string()),
                                )],
                            })),
                        }));
                    }
                }
            }
            if let Some(mut lowered_ctor) = $emitter
                .try_lower_registry_plain_call_with_signature(registry_ctor_key, args)
            {
                if let crate::RustExpr::FnCall { func, .. } = &mut lowered_ctor {
                    *func = Box::new(crate::RustExpr::Path(vec![
                        emitted_class_name.clone(),
                        "new".to_string(),
                    ]));
                }
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
                            if crate::helpers::is_option_type(resolved_param) {
                                continue;
                            }
                            if is_recursive_ctor_field || is_recursive_container_param {
                                *lowered_arg =
                                    Self::box_recursive_value_for_ir(lowered_arg.clone());
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
                let adapted_arg = if let HirExpr::Name { name, ty, .. } = arg {
                    if ($emitter.borrowed_params.contains(name)
                        || $emitter.mut_borrowed_params.contains(name))
                        && !crate::helpers::is_copy_type_for_codegen(ty)
                    {
                        crate::ownership_plan::materialize_owned_value(ty, lowered_arg)
                    } else {
                        lowered_arg
                    }
                } else {
                    lowered_arg
                };
                lowered_args.push(adapted_arg);
            }
            for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                if let Type::StructuralRecord(record) = ty.resolve_alias()
                    && let Some(field) = record.source_fields().get(idx)
                {
                    *lowered_arg = $emitter.coerce_local_value_for_target_type_for_ir(
                        field.ty(),
                        &args[idx],
                        lowered_arg.clone(),
                    )?;
                    if matches!(
                        field.ty().resolve_alias(),
                        Type::Callable(..) | Type::AsyncCallable(..)
                    ) {
                        *lowered_arg = Self::arc_constructor_callable_value(
                            field.ty(),
                            lowered_arg.clone(),
                        );
                    }
                }
                let Some((param_ty, convention)) =
                    ctor_params.as_ref().and_then(|params| params.get(idx))
                else {
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
                if crate::helpers::is_option_type(
                    crate::resolve_alias_type_for_plain_call(param_ty),
                ) {
                    if let Some(adapted) =
                        $emitter.try_adapt_recursive_option_constructor_arg_for_ir(
                            &crate::stmt_support_emitter::RecursiveOptionConstructorArgContext {
                                ctor_class_name: Some(class_name.as_str()),
                                index: idx,
                                param_ty,
                                arg: &args[idx],
                                effective_arg_ty: args[idx].ty(),
                                convention: *convention,
                                borrowed_name_arg: false,
                                borrowed_name_materialized: false,
                            },
                            lowered_arg.clone(),
                        )
                    {
                        *lowered_arg = adapted;
                    }
                    continue;
                }
                if (!is_recursive_ctor_field && !is_recursive_container_arg)
                    || matches!(args[idx], HirExpr::NoneLiteral)
                {
                    continue;
                }
                let resolved_arg_ty = crate::resolve_alias_type_for_plain_call(args[idx].ty());
                if !crate::helpers::is_option_type(resolved_arg_ty) {
                    *lowered_arg = Self::box_recursive_value_for_ir(lowered_arg.clone());
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
                        let Some((placeholder, lowered_inner)) =
                            $emitter.lower_formatted_value_for_ir(inner)?
                        else {
                            return Ok(None);
                        };
                        format_str.push_str(&placeholder);
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
        if let HirExpr::TemplateString(template) = $expr {
            return Ok($emitter.try_lower_template_string_expr_for_ir(template));
        }
        if let HirExpr::ListLiteral { elements, ty } = $expr {
            if elements.is_empty()
                && let Some(lowered) = crate::lower_expr::typed_empty_list_expr(ty)
            {
                return Ok(Some(lowered));
            }
            let mut lowered_elements = Vec::with_capacity(elements.len());
            let list_ty = crate::resolve_alias_type_for_plain_call(ty);
            for element in elements {
                let Some(mut lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                if let Type::List(element_ty) = list_ty {
                    lowered_element = crate::helpers::adapt_collection_value_for_target(
                        element_ty.as_ref(),
                        element,
                        lowered_element,
                    );
                }
                if matches!(list_ty, Type::Bytes) {
                    let Some(lowered_element) =
                        crate::helpers::adapt_bytes_element_for_storage(element, lowered_element)
                    else {
                        return Ok(None);
                    };
                    lowered_elements.push(lowered_element);
                    continue;
                }
                lowered_element = $emitter
                    .materialize_reusable_value_for_ir(element, lowered_element);
                lowered_elements.push(lowered_element);
            }
            return Ok(Some(crate::RustExpr::Vec(lowered_elements)));
        }
        if let HirExpr::Call { func, args, .. } = $expr {
            if func == "set"
                && args.len() == 1
                && matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Str | Type::LiteralStr(_)
                )
            {
                let Some(source) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: true,
                            name: "__sifr_set_chars".to_string(),
                            ty: Some(crate::RustType::HashSet(Box::new(crate::RustType::Named(
                                "char".to_string(),
                            )))),
                            value: crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "HashSet".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        crate::RustStmt::For {
                            var: "__sifr_set_char".to_string(),
                            iter: crate::RustExpr::MethodCall {
                                receiver: Box::new(source),
                                method: "chars".to_string(),
                                args: vec![],
                            },
                            body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__sifr_set_chars".to_string(),
                                )),
                                method: "insert".to_string(),
                                args: vec![crate::RustExpr::Ident("__sifr_set_char".to_string())],
                            })],
                        },
                        crate::RustStmt::Let {
                            mutable: true,
                            name: "__sifr_set_strings".to_string(),
                            ty: Some(crate::RustType::HashSet(Box::new(crate::RustType::String_))),
                            value: crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "HashSet".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        crate::RustStmt::For {
                            var: "__sifr_set_char".to_string(),
                            iter: crate::RustExpr::Ident("__sifr_set_chars".to_string()),
                            body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__sifr_set_strings".to_string(),
                                )),
                                method: "insert".to_string(),
                                args: vec![crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(
                                        "__sifr_set_char".to_string(),
                                    )),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                }],
                            })],
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::Ident(
                        "__sifr_set_strings".to_string(),
                    ))),
                }));
            }
        }
        if let HirExpr::TupleLiteral { elements, ty } = $expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let Some(lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                lowered_elements.push(
                    $emitter.materialize_reusable_value_for_ir(element, lowered_element),
                );
            }
            if crate::homogeneous_large_tuple_backing_array(ty).is_some() {
                return Ok(Some(crate::RustExpr::Array(lowered_elements)));
            }
            return Ok(Some(crate::RustExpr::Tuple(lowered_elements)));
        }
        if let HirExpr::DictLiteral { keys, values, ty } = $expr {
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
                let Some(mut lowered_key) = $emitter.lower_stmt_expr_for_ir(key)? else {
                    return Ok(None);
                };
                let Some(mut lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                if let Type::Dict(key_ty, value_ty) = crate::resolve_alias_type_for_plain_call(ty) {
                    lowered_key = crate::helpers::adapt_collection_value_for_target(
                        key_ty.as_ref(),
                        key,
                        lowered_key,
                    );
                    lowered_value = crate::helpers::adapt_collection_value_for_target(
                        value_ty.as_ref(),
                        value,
                        lowered_value,
                    );
                }
                lowered_key =
                    $emitter.materialize_reusable_value_for_ir(key, lowered_key);
                lowered_value =
                    $emitter.materialize_reusable_value_for_ir(value, lowered_value);
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
        if let HirExpr::SetLiteral { elements, ty } = $expr {
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
                let Some(mut lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                if let Type::Set(element_ty) = crate::resolve_alias_type_for_plain_call(ty) {
                    lowered_element = crate::helpers::adapt_collection_value_for_target(
                        element_ty.as_ref(),
                        element,
                        lowered_element,
                    );
                }
                lowered_element =
                    $emitter.materialize_reusable_value_for_ir(element, lowered_element);
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
        if let HirExpr::IntrinsicCall {
            intrinsic, args, ..
        } = $expr
        {
            return Ok($emitter.try_lower_registry_intrinsic_call_expr(
                *intrinsic,
                args,
                $expr.ty(),
            ));
        }
        if let Some((func, args)) = call_expr_parts($expr) {
            let mutable_arg_places = match $expr {
                HirExpr::Call {
                    mutable_arg_places, ..
                }
                | HirExpr::GenericCall {
                    mutable_arg_places, ..
                }
                | HirExpr::IteratorCall {
                    mutable_arg_places, ..
                } => mutable_arg_places.as_slice(),
                _ => &[],
            };
            if let Some(lowered_builtin) = $emitter.try_lower_registry_builtin_call_expr(
                func,
                args,
                Some($expr.ty()),
                mutable_arg_places,
            ) {
                return Ok(Some(lowered_builtin));
            }
            if func == "len"
                && let [HirExpr::Index { object, index, .. }] = args
                && let Some(value) =
                    $emitter.checked_place_read_borrow_witness(object, index)
            {
                return Ok(Some(
                    $emitter.lower_checked_place_len_with_witness(&args[0], value),
                ));
            }
            if let Some(mut lowered_plain) =
                $emitter.try_lower_registry_plain_call_with_places(func, args, mutable_arg_places)
            {
                if let HirExpr::GenericCall {
                    func, type_args, ..
                } = $expr
                {
                    if let crate::RustExpr::FnCall { func: target, .. } = &mut lowered_plain {
                        **target = generic_call_target_for_ir(func, type_args);
                    }
                }
                return Ok(Some(lowered_plain));
            }
            if func == "iter" && args.len() == 1 {
                return $emitter.lower_iter_source_expr_for_ir(&args[0]);
            }
            if func == "next" && args.len() == 1 {
                let Some(lowered_iterator) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                let next_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "next".to_string(),
                    args: vec![],
                };
                let Some(payload) = args[0].ty().iterator_element_type() else {
                    return Ok(None);
                };
                return Ok(Some(crate::helpers::normalize_safe_option_result(
                    &payload, next_expr,
                )));
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
            if func == "float"
                && args.len() == 1
                && matches!(
                    crate::resolve_alias_type_for_plain_call($expr.ty()),
                    Type::Float
                )
            {
                let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(
                    match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                        Type::Int | Type::LiteralInt(_) => crate::stmt_support_emitter::checked_integer_codegen::exact_integer_float_literal(&args[0])?,
                        Type::FixedInt(_) => crate::RustExpr::Cast {
                            expr: Box::new(lowered_arg),
                            ty: crate::RustType::F64,
                        },
                        _ => lowered_arg,
                    },
                ));
            }
            if func == "filter" && args.len() == 2 {
                return $emitter.try_lower_filter_call_for_ir(args);
            }
            if func == "list" && args.len() == 1 {
                let Some(iterable) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(iterable),
                    method: "collect::<Vec<_>>".to_string(),
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
                    let format_str = if Self::uses_debug_display_format_for_ir(&inner) {
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
            for (index, arg) in args.iter().enumerate() {
                let Some(mut lowered_arg) = $emitter.lower_call_argument_for_stmt(
                    arg,
                    mutable_arg_places.get(index).and_then(Option::as_ref),
                )?
                else {
                    return Ok(None);
                };
                if matches!(func, "py_local_callback" | "py_threadsafe_callback")
                    && matches!(arg, HirExpr::Lambda { .. })
                {
                    if let crate::RustExpr::Closure { is_move, .. } = &mut lowered_arg {
                        *is_move = true;
                    }
                }
                lowered_args.push(lowered_arg);
            }
            let canonical_func = canonical_plain_call_name_for_ir(func);
            lowered_args = $emitter.adapt_plain_call_args_with_signature_for_ir(
                canonical_func,
                args,
                lowered_args,
            );
            if canonical_func == "_call_object_callback" {
                if let Some(first_arg) = args.first() {
                    if matches!(first_arg.ty().resolve_alias(), Type::Callable(..))
                        && !matches!(lowered_args.first(), Some(crate::RustExpr::Ref { .. }))
                    {
                        if let Some(lowered_first) = lowered_args.first_mut() {
                            *lowered_first = crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_first.clone()),
                            };
                        }
                    }
                }
            }
            if let Some(captures) = $emitter.nested_fn_captures.get(canonical_func).cloned() {
                for capture in captures {
                    lowered_args.push($emitter.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            let call_target = match $expr {
                HirExpr::GenericCall {
                    func, type_args, ..
                } => generic_call_target_for_ir(func, type_args),
                _ => crate::RustExpr::Path(
                    canonical_func
                        .split("::")
                        .map(ToString::to_string)
                        .collect(),
                ),
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(call_target),
                args: lowered_args,
            }));
        }
    }};
}
