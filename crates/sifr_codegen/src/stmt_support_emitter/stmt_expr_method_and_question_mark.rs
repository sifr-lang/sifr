use super::{
    call_expr_parts, can_construct_error_from_message_for_ir, canonical_constructor_class_name,
    canonical_plain_call_name_for_ir, is_result_int_division_error_type,
    unwrap_compiler_verified_nonempty_pop_result_for_ir, HirExpr, HirFStringPart, RustEmitter,
    Type,
};
macro_rules! stmt_expr_method_call {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = $expr
        {
            if method == "append" && args.len() == 1 {
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
                            let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(index_object)?
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
                            let lowered_arg =
                                Self::clone_non_copy_name_expr_for_ir(&args[0], lowered_arg);
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
            let needs_field_clone_suppression =
                $emitter.method_call_needs_field_clone_suppression(object, method);
            let suppression_prev = $emitter.pending_self_field_clone_suppression;
            if needs_field_clone_suppression {
                $emitter.pending_self_field_clone_suppression += 1;
            }
            let lowered_registry =
                $emitter.try_lower_registry_method_call_expr(object, method, args, $expr.ty());
            if needs_field_clone_suppression
                && $emitter.pending_self_field_clone_suppression > suppression_prev
            {
                $emitter.pending_self_field_clone_suppression -= 1;
            }
            if let Some(lowered_registry) = lowered_registry {
                return Ok(Some(lowered_registry));
            }

            let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = $emitter.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            let effective_object_ty = $emitter.effective_method_object_ty(object);
            if method == "append"
                && lowered_args.len() == 1
                && matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::List(_)
                )
            {
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
            if let Some(method_params) =
                $emitter.resolve_registry_method_params(&effective_object_ty, method)
            {
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
                receiver: Box::new(lowered_object),
                method: method.clone(),
                args: lowered_args,
            };
            let lowered_method = unwrap_compiler_verified_nonempty_pop_result_for_ir(
                &effective_object_ty,
                method,
                args,
                $expr.ty(),
                lowered_method,
            );
            if matches!(
                crate::resolve_alias_type_for_plain_call($expr.ty()),
                Type::Int
            ) && matches!(method.as_str(), "len" | "count")
            {
                return Ok(Some(crate::RustExpr::Cast {
                    expr: Box::new(lowered_method),
                    ty: crate::RustType::I64,
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
