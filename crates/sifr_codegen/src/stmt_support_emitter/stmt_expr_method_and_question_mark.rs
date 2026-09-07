use super::{
    HirExpr, HirFStringPart, RustEmitter, Type, call_expr_parts,
    can_construct_error_from_message_for_ir, canonical_constructor_class_name,
    canonical_plain_call_name_for_ir, generic_call_target_for_ir,
    unwrap_compiler_verified_nonempty_pop_result_for_ir,
};

fn is_imported_project_call_for_ir(
    expression: &HirExpr,
    imported_project_functions: &std::collections::HashSet<String>,
) -> bool {
    matches!(
        expression,
        HirExpr::Call { func, .. }
            | HirExpr::GenericCall { func, .. }
            | HirExpr::PythonCall { func, .. }
            if imported_project_functions.contains(canonical_plain_call_name_for_ir(func))
    )
}

macro_rules! stmt_expr_method_call {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            receiver_target,
            mutable_arg_places,
            ..
        } = $expr
        {
            if let Some(lowered) = $emitter.try_lower_python_raw_object_method(
                object,
                method,
                args,
                *receiver_convention,
                receiver_target.as_ref(),
                $expr.ty(),
            ) {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_dict_indexed_list_append_expr($expr) {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_dict_indexed_list_pop_expr($expr) {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_dict_indexed_list_len_expr($expr) {
                return Ok(Some(lowered));
            }
            if method == "len"
                && args.is_empty()
                && matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::Str | Type::LiteralStr(_)
                )
            {
                let Some(lowered_object) = $emitter.lower_method_receiver_place_for_stmt(
                    object,
                    *receiver_convention,
                    receiver_target.as_ref(),
                )?
                else {
                    return Ok(None);
                };
                return Ok(Some($emitter.lower_string_len_with_cache(object, lowered_object)));
            }
            if method == "len" && args.is_empty() {
                if let HirExpr::Call {
                    func,
                    args: set_args,
                    ..
                } = object.as_ref()
                {
                    if func == "set"
                        && set_args.len() == 1
                        && matches!(
                            crate::resolve_alias_type_for_plain_call(set_args[0].ty()),
                            Type::Str | Type::LiteralStr(_)
                        )
                    {
                        let Some(lowered_source) =
                            $emitter.lower_stmt_expr_for_ir(&set_args[0])?
                        else {
                            return Ok(None);
                        };
                        let char_set = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_source),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "collect::<std::collections::HashSet<_>>".to_string(),
                            args: vec![],
                        };
                        return Ok(Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "SifrInt".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(char_set),
                                method: "len".to_string(),
                                args: vec![],
                            }],
                        }));
                    }
                }
            }
            if method == "append"
                && args.len() == 1
                && matches!(
                    receiver_target.as_ref(),
                    Some(sifr_ir::MutableReceiverTarget::SpecializedIndexedStorage(_))
                )
            {
                if let HirExpr::Index {
                    object: index_object,
                    index,
                    ..
                } = object.as_ref()
                {
                    let index_object_ty =
                        crate::resolve_alias_type_for_plain_call(index_object.ty());
                    if let Type::Dict(_, value_ty) = index_object_ty {
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                            Type::List(_)
                        ) {
                            let Some(
                                sifr_ir::MutableReceiverTarget::SpecializedIndexedStorage(
                                    base_place,
                                ),
                            ) = receiver_target.as_ref()
                            else {
                                return Ok(None);
                            };
                            let Some(lowered_object) =
                                $emitter.emit_checked_place(index_object, base_place)
                            else {
                                return Ok(None);
                            };
                            let Some(lowered_index) = $emitter.lower_stmt_expr_for_ir(index)? else {
                                return Ok(None);
                            };
                            let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(&args[0])? else {
                                return Ok(None);
                            };
                            let lowered_index =
                                Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
                            let lowered_arg = $emitter
                                .materialize_reusable_value_for_ir(&args[0], lowered_arg);
                            let key_arg = Self::build_dict_lookup_key_arg_for_ir(lowered_index);
                            return Ok(Some(crate::RustExpr::Block {
                                stmts: vec![crate::RustStmt::IfLet {
                                    pattern: "Some(__elem)".to_string(),
                                    expr: crate::RustExpr::MethodCall {
                                        receiver: Box::new(lowered_object),
                                        method: "get_mut".to_string(),
                                        args: vec![key_arg],
                                    },
                                    then_body: vec![crate::RustStmt::Expr(
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__elem".to_string(),
                                            )),
                                            method: "push".to_string(),
                                            args: vec![lowered_arg],
                                        },
                                    )],
                                    else_body: None,
                                }],
                                expr: None,
                            }));
                        }
                    }
                }
            }
            if method == "__sifr_timeout"
                && matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::Task(_, _)
                )
            {
                let [duration] = args.as_slice() else {
                    return Ok(None);
                };
                let Some(lowered_object) = $emitter.lower_method_receiver_place_for_stmt(
                    object,
                    *receiver_convention,
                    receiver_target.as_ref(),
                )? else {
                    return Ok(None);
                };
                let Some(lowered_duration) = $emitter.lower_stmt_expr_for_ir(duration)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: method.clone(),
                    args: vec![crate::task_duration_expr_from_seconds(
                        lowered_duration,
                        "__sifr_task_timeout_seconds",
                    )],
                }));
            }
            let lowered_registry = $emitter.try_lower_registry_method_call_expr(
                object,
                method,
                args,
                crate::place_emitter::MethodCallPlaces::new(
                    *receiver_convention,
                    receiver_target.as_ref(),
                    mutable_arg_places,
                ),
                $expr.ty(),
            )?;
            if let Some(lowered_registry) = lowered_registry {
                return Ok(Some(lowered_registry));
            }

            let Some(lowered_object) = $emitter.lower_method_receiver_place_for_stmt(
                object,
                *receiver_convention,
                receiver_target.as_ref(),
            )? else {
                return Ok(None);
            };
            if let Some(materialized) = $emitter.materialize_explicit_borrowed_clone(
                object,
                method,
                args,
                lowered_object.clone(),
            ) {
                return Ok(Some(materialized));
            }
            let effective_object_ty = $emitter.effective_method_object_ty(object);
            let method_params =
                $emitter.resolve_registry_method_params(&effective_object_ty, method);
            let mut lowered_args = Vec::with_capacity(args.len());
            for (idx, arg) in args.iter().enumerate() {
                let convention = method_params
                    .as_ref()
                    .and_then(|params| params.get(idx))
                    .map_or(sifr_type_system::ParamConvention::default(), |(_, convention)| {
                        *convention
                    });
                let Some(lowered_arg) = $emitter.lower_method_argument_place_for_stmt(
                    arg,
                    convention,
                    mutable_arg_places.get(idx).and_then(Option::as_ref),
                )? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                Type::Decimal | Type::BigDecimal
            ) && matches!(method.as_str(), "quantize" | "round")
                && args.len() == 1
            {
                let Some(scale) = crate::integer_literal_decimal(&args[0])
                    .and_then(|value| value.parse::<i64>().ok())
                else {
                    return Ok(None);
                };
                lowered_args[0] = crate::RustExpr::Literal(crate::RustLiteral::Int(scale));
            }
            if effective_object_ty.callable_field_type(method).is_some() {
                if let Some(method_params) = method_params.as_deref() {
                    $emitter.apply_registry_method_arg_conventions(
                        args,
                        method_params,
                        &mut lowered_args,
                    );
                }
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: method.clone(),
                    }))),
                    args: lowered_args,
                }));
            }
            if method == "append"
                && lowered_args.len() == 1
                && matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::List(_)
                )
            {
                lowered_args[0] = $emitter
                    .materialize_reusable_value_for_ir(&args[0], lowered_args[0].clone());
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "push".to_string(),
                    args: lowered_args,
                }));
            }
            if method == "cloned"
                && lowered_args.is_empty()
                && matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::List(_)
                )
            {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "clone".to_string(),
                    args: vec![],
                }));
            }
            if method == "cloned" && lowered_args.is_empty() {
                let collected_vec = match &lowered_object {
                    crate::RustExpr::MethodCall { method, .. } => {
                        method == "collect" || method.starts_with("collect::<")
                    }
                    crate::RustExpr::Paren(inner) => {
                        matches!(
                            inner.as_ref(),
                            crate::RustExpr::MethodCall { method, .. }
                                if method == "collect" || method.starts_with("collect::<")
                        )
                    }
                    _ => false,
                };
                if collected_vec {
                    return Ok(Some(lowered_object));
                }
            }
            if let Some(method_params) = method_params {
                let method_receiver_class =
                    match crate::resolve_alias_type_for_plain_call(&effective_object_ty) {
                        Type::Class { name, .. } => Some(name.clone()),
                        _ => None,
                    };
                for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                    if method_receiver_class.as_ref().is_some_and(|class_name| {
                        $emitter.method_param_lowers_to_sifr_int_result(class_name, method, idx)
                    }) {
                        *lowered_arg = $emitter.coerce_result_int_expr_to_sifr_int_value(
                            $emitter.rewrite_stdlib_constant_idents_in_expr(lowered_arg.clone()),
                        );
                        continue;
                    }
                    if let (Some((param_ty, convention)), Some(arg)) =
                        (method_params.get(idx), args.get(idx))
                    {
                        *lowered_arg = $emitter.apply_registry_method_arg_convention(
                            arg,
                            param_ty,
                            *convention,
                            lowered_arg.clone(),
                        );
                    }
                }
            }
            let lowered_method = crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object.clone()),
                method: method.clone(),
                args: lowered_args,
            };
            let lowered_method = unwrap_compiler_verified_nonempty_pop_result_for_ir(
                &effective_object_ty,
                method,
                args,
                $expr.ty(),
                lowered_object,
                $emitter.is_deque_data_field(object),
                lowered_method,
            );
            if matches!(
                crate::resolve_alias_type_for_plain_call($expr.ty()),
                Type::Int
            ) && matches!(method.as_str(), "len" | "count")
                && !matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::Class { .. }
                )
            {
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "SifrInt".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![lowered_method],
                }));
            }
            return Ok(Some(lowered_method));
        }
    }};
}

macro_rules! stmt_expr_question_mark {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::QuestionMark { expr: inner, .. } = $expr {
            let Some(lowered_inner) = $emitter.lower_stmt_expr_for_ir(inner)? else {
                return Ok(None);
            };
            if let Some(target_err_ty) = $emitter.try_closure_error_type.last().cloned() {
                let resolved_inner_ty = crate::resolve_alias_type_for_plain_call(inner.ty());
                if let Type::Result(_, inner_err_ty) = resolved_inner_ty {
                    let inner_err_ty_name =
                        crate::render_type(&crate::sifr_type_to_rust_type(inner_err_ty));
                    let error_ident = crate::RustExpr::Ident("__e".to_string());
                    let target_error_info = $emitter
                        .try_closure_error_type_info
                        .last()
                        .and_then(Option::as_ref)
                        .cloned();
                    let imported_project_call = is_imported_project_call_for_ir(
                        inner.as_ref(),
                        &$emitter.imported_project_functions,
                    );
                    if imported_project_call
                        && inner_err_ty.is_python_error_contract()
                        && target_error_info
                            .as_ref()
                            .is_some_and(Type::is_python_error_contract)
                    {
                        let target_name = target_error_info
                            .as_ref()
                            .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
                            .unwrap_or_else(|| target_err_ty.clone());
                        let field = |name: &str| crate::RustExpr::Field {
                            expr: Box::new(error_ident.clone()),
                            field: name.to_string(),
                        };
                        let converted = crate::RustExpr::StructInit {
                            name: target_name,
                            fields: vec![
                                ("message".to_string(), field("message")),
                                ("kind".to_string(), field("kind")),
                                ("exception_type".to_string(), field("exception_type")),
                                ("traceback".to_string(), field("traceback")),
                                ("context".to_string(), field("context")),
                                (
                                    "__sifr_python_error".to_string(),
                                    field("__sifr_python_error"),
                                ),
                            ],
                        };
                        return Ok(Some(crate::RustExpr::Try(Box::new(
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_inner))),
                                method: "map_err".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(converted),
                                    is_move: false,
                                }],
                            },
                        ))));
                    }
                    let converted_error = target_error_info.as_ref().map(|target| {
                        $emitter.consuming_value_conversion_for_ir(
                            target,
                            inner_err_ty,
                            error_ident.clone(),
                        )
                    });
                    if converted_error
                        .as_ref()
                        .is_some_and(|converted| converted != &error_ident)
                    {
                        return Ok(Some(crate::RustExpr::Try(Box::new(
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_inner))),
                                method: "map_err".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(converted_error.unwrap_or(error_ident)),
                                    is_move: false,
                                }],
                            },
                        ))));
                    }
                    if inner_err_ty_name != target_err_ty
                        && can_construct_error_from_message_for_ir(&target_err_ty)
                    {
                        let ctor_func = if target_err_ty.contains("::") {
                            let mut path: Vec<String> =
                                target_err_ty.split("::").map(str::to_string).collect();
                            path.push("new".to_string());
                            crate::RustExpr::Path(path)
                        } else {
                            crate::RustExpr::Path(vec![target_err_ty.clone(), "new".to_string()])
                        };
                        return Ok(Some(crate::RustExpr::Try(Box::new(
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_inner))),
                                method: "map_err".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::FnCall {
                                        func: Box::new(ctor_func),
                                        args: vec![crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__e".to_string(),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        }],
                                    }),
                                    is_move: false,
                                }],
                            },
                        ))));
                    }
                }
            }
            return Ok(Some(crate::RustExpr::Try(Box::new(lowered_inner))));
        }
    }};
}

impl RustEmitter {
    pub(crate) fn lower_stmt_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
        {
            if let Some(witness) = self.checked_place_read_witness(object, index, ty) {
                return Ok(Some(witness));
            }
        }
        if let HirExpr::Lambda { params, body, .. } = expr {
            let Some(lowered_body) = self.lower_stmt_expr_for_ir(body)? else {
                return Ok(None);
            };
            let lowered_params = params
                .iter()
                .map(|param| crate::RustParam::Named {
                    name: param.name.clone(),
                    ty: crate::RustType::Named("_".to_string()),
                })
                .collect::<Vec<_>>();
            return Ok(Some(crate::RustExpr::Closure {
                params: lowered_params,
                body: Box::new(lowered_body),
                is_move: false,
            }));
        }
        stmt_expr_await_and_registry!(self, expr);
        stmt_expr_constructor!(self, expr);
        stmt_expr_literals_and_calls!(self, expr);
        stmt_expr_method_call!(self, expr);
        stmt_expr_question_mark!(self, expr);
        stmt_expr_slice!(self, expr);
        stmt_expr_wrappers_range_index!(self, expr);
        stmt_expr_contains_unary_compare_bool!(self, expr);
        stmt_expr_binop!(self, expr);
        if matches!(expr, HirExpr::Name { .. }) {
            return Ok(self
                .try_lower_registry_expr_strict(expr)
                .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_imported_project_calls_use_the_canonical_function_name() {
        let expression = HirExpr::GenericCall {
            func: "load::<i64>".to_string(),
            type_args: vec![Type::Int],
            args: Vec::new(),
            mutable_arg_places: Vec::new(),
            ty: Type::None,
        };
        let imported = std::collections::HashSet::from(["load".to_string()]);

        assert!(is_imported_project_call_for_ir(&expression, &imported));
    }
}
