use super::{
    HirExpr, RustEmitter, RustExpr, Type, methods, registry_box_iterator_expr,
    registry_defaultdict_alias_parts, registry_expr_is_vec_like,
    registry_iterable_to_owned_iter_expr, registry_iterable_to_set_expr,
};
use crate::place_emitter::MethodCallPlaces;
use sifr_ir::MutableReceiverTarget;
impl RustEmitter {
    pub(crate) fn try_lower_registry_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        places: MethodCallPlaces<'_>,
        method_return_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let is_defaultdict_bucket_mutator = match object {
            HirExpr::Index {
                object: base_object,
                ..
            } => registry_defaultdict_alias_parts(base_object.ty()).is_some_and(
                |(_, _, value_ty)| methods::is_in_place_collection_method(value_ty, method),
            ),
            _ => false,
        };
        if is_defaultdict_bucket_mutator {
            return self
                .try_lower_defaultdict_index_method_call_expr(
                    object,
                    method,
                    args,
                    places,
                    method_return_ty,
                )
                .map(Some)
                .ok_or_else(|| {
                    crate::CodegenError::new(format!(
                        "defaultdict bucket mutator `{method}` could not be lowered safely"
                    ))
                });
        }

        Ok(self.try_lower_registry_method_call_expr_unchecked(
            object,
            method,
            args,
            places,
            method_return_ty,
            false,
        ))
    }

    pub(crate) fn try_lower_registry_discarded_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        places: MethodCallPlaces<'_>,
        method_return_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if method != "setdefault" {
            return self.try_lower_registry_method_call_expr(
                object,
                method,
                args,
                places,
                method_return_ty,
            );
        }
        Ok(self.try_lower_registry_method_call_expr_unchecked(
            object,
            method,
            args,
            places,
            method_return_ty,
            true,
        ))
    }

    fn try_lower_registry_method_call_expr_unchecked(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        places: MethodCallPlaces<'_>,
        method_return_ty: &Type,
        discard_result: bool,
    ) -> Option<crate::RustExpr> {
        let effective_object_ty = self.effective_method_object_ty(object);
        let object_ty = crate::resolve_alias_type_for_plain_call(&effective_object_ty);
        if let Some(lowered) = crate::python_buffer_codegen::lower_python_buffer_method(
            self,
            object,
            method,
            args,
            places,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered) = crate::python_arrow_codegen::lower_python_arrow_method(
            self,
            object,
            method,
            args,
            places,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered) = crate::python_dlpack_codegen::lower_python_dlpack_method(
            self,
            object,
            method,
            args,
            places,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered_object) = self.try_lower_registry_expr_strict(object)
            && let Some(materialized) =
                self.materialize_explicit_borrowed_clone(object, method, args, lowered_object)
        {
            return Some(materialized);
        }
        if method == "append" && args.len() == 1 {
            if let HirExpr::Index {
                object: index_object,
                index,
                ..
            } = object
            {
                if matches!(
                    crate::resolve_alias_type_for_plain_call(index_object.ty()),
                    Type::Dict(_, _)
                ) && matches!(object_ty, Type::List(_))
                {
                    let MutableReceiverTarget::SpecializedIndexedStorage(base_place) =
                        places.receiver_target?
                    else {
                        return None;
                    };
                    let lowered_object = self.emit_checked_place(index_object, base_place)?;
                    let lowered_index = self.try_lower_registry_expr_strict(index)?;
                    let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
                    let key_arg = Self::build_dict_lookup_key_arg_for_ir(
                        Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                    );
                    let pushed_arg = self.materialize_reusable_value_for_ir(&args[0], lowered_arg);
                    return Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::IfLet {
                            pattern: "Some(__elem)".to_string(),
                            expr: crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get_mut".to_string(),
                                args: vec![key_arg],
                            },
                            then_body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                                method: "push".to_string(),
                                args: vec![pushed_arg],
                            })],
                            else_body: None,
                        }],
                        expr: None,
                    });
                }
            }
        }
        if method == "len" && args.is_empty() {
            if let HirExpr::Index {
                object: index_object,
                index,
                ..
            } = object
            {
                if let Some(bucket) = self.checked_place_read_borrow_witness(index_object, index) {
                    return Some(self.lower_checked_place_len_with_witness(object, bucket));
                }
                let effective_index_object_ty = self.effective_registry_expr_ty(index_object);
                if let Type::Dict(_, value_ty) =
                    crate::resolve_alias_type_for_plain_call(&effective_index_object_ty)
                {
                    if matches!(
                        crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                        Type::List(_)
                    ) {
                        let lowered_object =
                            self.try_lower_dict_indexed_list_mutation_object(index_object)?;
                        let lowered_index = self.try_lower_registry_expr_strict(index)?;
                        let key_arg = self.list_indexed_dict_lookup_key_arg(index, lowered_index);
                        return Some(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "map_or_else".to_string(),
                            args: vec![
                                RustExpr::Closure {
                                    params: vec![],
                                    body: Box::new(RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "SifrInt".to_string(),
                                            "from_i64".to_string(),
                                        ])),
                                        args: vec![RustExpr::Literal(crate::RustLiteral::Int(0))],
                                    }),
                                    is_move: false,
                                },
                                RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__sifr_bucket".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "SifrInt".to_string(),
                                            "from".to_string(),
                                        ])),
                                        args: vec![RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__sifr_bucket".to_string(),
                                            )),
                                            method: "len".to_string(),
                                            args: vec![],
                                        }],
                                    }),
                                    is_move: false,
                                },
                            ],
                        });
                    }
                }
            }
        }
        let is_deque_data_field = self.is_deque_data_field(object);
        let object_expr = self.lower_method_receiver_place_for_registry(
            object,
            places.receiver_convention,
            places.receiver_target,
        )?;
        if method == "len"
            && args.is_empty()
            && matches!(object_ty, Type::Str | Type::LiteralStr(_))
        {
            return Some(self.lower_string_len_with_cache(object, object_expr));
        }
        let method_params = self.resolve_registry_method_params(&effective_object_ty, method);
        let mut arg_exprs = Vec::with_capacity(args.len());
        for (index, argument) in args.iter().enumerate() {
            let convention = method_params
                .as_ref()
                .and_then(|params| params.get(index))
                .map_or(
                    sifr_type_system::ParamConvention::default(),
                    |(_, convention)| *convention,
                );
            arg_exprs.push(
                self.lower_method_argument_place_for_registry(
                    argument,
                    convention,
                    places
                        .mutable_arg_places
                        .get(index)
                        .and_then(Option::as_ref),
                )?,
            );
        }

        if matches!(object_ty, Type::Decimal | Type::BigDecimal)
            && matches!(method, "quantize" | "round")
            && args.len() == 1
        {
            let scale = crate::integer_literal_decimal(&args[0])
                .and_then(|value| value.parse::<i64>().ok())?;
            arg_exprs[0] = RustExpr::Literal(crate::RustLiteral::Int(scale));
        }

        let collection_element_targets = match (object_ty, method) {
            (Type::List(element_ty), "append" | "appendleft") => {
                vec![(0, element_ty.as_ref())]
            }
            (Type::List(element_ty), "insert") => vec![(1, element_ty.as_ref())],
            (Type::Set(element_ty), "add") => vec![(0, element_ty.as_ref())],
            (Type::Dict(key_ty, value_ty), "setdefault") if discard_result => {
                vec![(0, key_ty.as_ref()), (1, value_ty.as_ref())]
            }
            _ => Vec::new(),
        };
        self.adapt_owned_mapping_default(object_ty, method, args, &mut arg_exprs);
        for (index, target_ty) in collection_element_targets {
            if let (Some(argument), Some(lowered_arg)) = (args.get(index), arg_exprs.get_mut(index))
            {
                *lowered_arg = self.coerce_collection_element_for_registry(
                    target_ty,
                    argument,
                    lowered_arg.clone(),
                );
                *lowered_arg =
                    self.materialize_reusable_value_for_ir(argument, lowered_arg.clone());
            }
        }

        if let Type::Dict(key_ty, value_ty) = object_ty
            && matches!(
                crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                Type::Str | Type::LiteralStr(_)
            )
            && let Some(HirExpr::Name { name, .. }) = args.first()
            && let Some(lowered_key) = arg_exprs.first()
            && (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
        {
            let lookup = |rust_method: &str| RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: rust_method.to_string(),
                args: vec![lowered_key.clone()],
            };
            match method {
                "contains" => return Some(lookup("contains_key")),
                "get" => {
                    let value = RustExpr::MethodCall {
                        receiver: Box::new(lookup("get")),
                        method: "cloned".to_string(),
                        args: Vec::new(),
                    };
                    return match arg_exprs.as_slice() {
                        [_] => Some(crate::helpers::normalize_safe_option_result(
                            value_ty.as_ref(),
                            value,
                        )),
                        [_, default] => Some(RustExpr::MethodCall {
                            receiver: Box::new(value),
                            method: "unwrap_or".to_string(),
                            args: vec![default.clone()],
                        }),
                        _ => None,
                    };
                }
                "pop" | "remove" => {
                    let removed = lookup("remove");
                    return match arg_exprs.as_slice() {
                        [_] => Some(crate::helpers::normalize_safe_option_result(
                            value_ty.as_ref(),
                            removed,
                        )),
                        [_, default] => Some(RustExpr::MethodCall {
                            receiver: Box::new(removed),
                            method: "unwrap_or".to_string(),
                            args: vec![default.clone()],
                        }),
                        _ => None,
                    };
                }
                _ => {}
            }
        }

        if matches!(object_ty, Type::Dict(key_ty, _) if matches!(crate::resolve_alias_type_for_plain_call(key_ty.as_ref()), Type::Str | Type::LiteralStr(_)))
            && matches!(method, "get" | "contains" | "remove" | "pop")
            && !args.is_empty()
        {
            let key_arg_ty = crate::resolve_alias_type_for_plain_call(args[0].ty());
            let key_is_string_like = matches!(key_arg_ty, Type::Str | Type::LiteralStr(_));
            let already_as_str =
                matches!(&arg_exprs[0], RustExpr::MethodCall { method, .. } if method == "as_str");
            if key_is_string_like && !already_as_str {
                arg_exprs[0] = self.string_view_expr(&args[0], arg_exprs[0].clone());
            }
        }

        if object_ty.callable_field_type(method).is_some() {
            if let Some(method_params) = method_params.as_deref() {
                self.apply_registry_method_arg_conventions(args, method_params, &mut arg_exprs);
            }
            return Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Field {
                    expr: Box::new(object_expr),
                    field: method.to_string(),
                }))),
                args: arg_exprs,
            });
        }

        if matches!(object_ty, Type::List(_)) && method == "extend" && args.len() == 1 {
            return Some(crate::RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: "extend".to_string(),
                args: vec![registry_iterable_to_owned_iter_expr(self, &args[0])?],
            });
        }

        if matches!(object_ty, Type::List(_) | Type::Set(_))
            && method == "contains"
            && let ([argument], [lowered_argument]) = (args, arg_exprs.as_slice())
            && matches!(argument, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name))
        {
            return Some(crate::RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: "contains".to_string(),
                args: vec![lowered_argument.clone()],
            });
        }

        if let Type::Set(element_ty) = object_ty
            && matches!(method, "remove" | "discard")
            && let ([argument], [lowered_argument]) = (args, arg_exprs.as_slice())
            && argument
                .ty()
                .optional_member_type()
                .is_some_and(|inner| inner.is_assignable_to(element_ty.as_ref()))
        {
            return Some(crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::IfLet {
                    pattern: "Some(__sifr_set_value)".to_string(),
                    expr: lowered_argument.clone(),
                    then_body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr.clone()),
                        method: "remove".to_string(),
                        args: vec![crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__sifr_set_value".to_string())),
                        }],
                    })],
                    else_body: None,
                }],
                expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
            });
        }

        if matches!(object_ty, Type::Set(_)) {
            if let Some(lowered) =
                self.try_lower_registry_set_method_call_expr(&object_expr, method, args)
            {
                return Some(lowered);
            }
        }

        let lowered = methods::lower_method_with_discard_context(
            object_ty,
            method,
            &object_expr,
            &arg_exprs,
            is_deque_data_field,
            discard_result,
        )?;
        let lowered_expr = Self::unwrap_compiler_verified_nonempty_pop_result(
            object_ty,
            method,
            args,
            method_return_ty,
            object_expr,
            is_deque_data_field,
            lowered.expr,
        );
        if matches!(
            crate::resolve_alias_type_for_plain_call(method_return_ty),
            Type::Iterator(_)
        ) && registry_expr_is_vec_like(&lowered_expr)
        {
            return Some(registry_box_iterator_expr(RustExpr::MethodCall {
                receiver: Box::new(lowered_expr),
                method: "into_iter".to_string(),
                args: vec![],
            }));
        }
        Some(lowered_expr)
    }

    pub(crate) fn unwrap_compiler_verified_nonempty_pop_result(
        object_ty: &Type,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
        object_expr: crate::RustExpr,
        is_deque_data_field: bool,
        lowered_expr: crate::RustExpr,
    ) -> crate::RustExpr {
        crate::stmt_support_emitter::unwrap_compiler_verified_nonempty_pop_result_for_ir(
            object_ty,
            method,
            args,
            method_return_ty,
            object_expr,
            is_deque_data_field,
            lowered_expr,
        )
    }

    pub(crate) fn try_lower_registry_set_method_call_expr(
        &mut self,
        object_expr: &crate::RustExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        match method {
            "update" => {
                let mut stmts = Vec::with_capacity(args.len());
                for arg in args {
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr.clone()),
                        method: "extend".to_string(),
                        args: vec![crate::intrinsic_method_emitters::registry_iterable_to_owned_into_iter_arg_expr(self, arg)?],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                });
            }
            "union" => {
                let mut stmts = vec![crate::RustStmt::Let {
                    mutable: true,
                    name: "__result".to_string(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr.clone()),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                }];
                for arg in args {
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__result".to_string())),
                        method: "extend".to_string(),
                        args: vec![crate::intrinsic_method_emitters::registry_iterable_to_owned_into_iter_arg_expr(self, arg)?],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(crate::RustExpr::Ident("__result".to_string()))),
                });
            }
            "intersection" | "difference" | "intersection_update" | "difference_update" => {
                let result_name = if method.ends_with("_update") {
                    None
                } else {
                    Some("__result".to_string())
                };
                let mut stmts = Vec::new();
                if let Some(result_name) = result_name.as_ref() {
                    stmts.push(crate::RustStmt::Let {
                        mutable: true,
                        name: result_name.clone(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(object_expr.clone()),
                            method: "clone".to_string(),
                            args: vec![],
                        },
                    });
                }
                let target = result_name.as_ref().map_or_else(
                    || object_expr.clone(),
                    |name| crate::RustExpr::Ident(name.clone()),
                );
                let keep_on_match = method.starts_with("intersection");
                for (index, arg) in args.iter().enumerate() {
                    let temp_name = format!("__set_arg_{index}");
                    stmts.push(crate::RustStmt::Let {
                        mutable: false,
                        name: temp_name.clone(),
                        ty: None,
                        value: registry_iterable_to_set_expr(self, arg)?,
                    });
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(target.clone()),
                        method: "retain".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__item".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(if keep_on_match {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(temp_name)),
                                    method: "contains".to_string(),
                                    args: vec![crate::RustExpr::Ident("__item".to_string())],
                                }
                            } else {
                                crate::RustExpr::UnaryOp {
                                    op: "!".to_string(),
                                    operand: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(temp_name)),
                                        method: "contains".to_string(),
                                        args: vec![crate::RustExpr::Ident("__item".to_string())],
                                    }),
                                }
                            }),
                            is_move: false,
                        }],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(result_name.map_or_else(
                        || crate::RustExpr::Literal(crate::RustLiteral::Unit),
                        crate::RustExpr::Ident,
                    ))),
                });
            }
            "symmetric_difference" | "symmetric_difference_update" => {
                if args.len() != 1 {
                    return None;
                }
                let other = registry_iterable_to_set_expr(self, &args[0])?;
                let diff_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(if method.ends_with("_update") {
                            object_expr.clone()
                        } else {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(object_expr.clone()),
                                method: "clone".to_string(),
                                args: vec![],
                            }
                        }),
                        method: "symmetric_difference".to_string(),
                        args: vec![crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__other".to_string())),
                        }],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                };
                let new_set_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(diff_expr),
                    method: "collect::<std::collections::HashSet<_>>".to_string(),
                    args: vec![],
                };
                let mut stmts = vec![crate::RustStmt::Let {
                    mutable: false,
                    name: "__other".to_string(),
                    ty: None,
                    value: other,
                }];
                if method.ends_with("_update") {
                    stmts.push(crate::RustStmt::Assign {
                        target: object_expr.clone(),
                        value: new_set_expr,
                    });
                    return Some(crate::RustExpr::Block {
                        stmts,
                        expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                    });
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(new_set_expr)),
                });
            }
            _ => {}
        }
        None
    }

    pub(crate) fn try_lower_registry_exprs_strict(
        &mut self,
        exprs: &[HirExpr],
    ) -> Option<Vec<crate::RustExpr>> {
        let mut lowered = Vec::with_capacity(exprs.len());
        for expr in exprs {
            lowered.push(self.try_lower_registry_expr_strict(expr)?);
        }
        Some(lowered)
    }

    pub(crate) fn try_lower_registry_expr_strict(
        &mut self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        if let HirExpr::Index { object, index, ty } = expr
            && !crate::helpers::is_option_type(ty)
            && let Some(lowered) = self
                .lower_non_option_index_expr_for_ir(object, index)
                .ok()?
        {
            return Some(lowered);
        }
        if matches!(
            expr,
            HirExpr::ListLiteral { .. }
                | HirExpr::TupleLiteral { .. }
                | HirExpr::DictLiteral { .. }
                | HirExpr::SetLiteral { .. }
        ) {
            return self.try_lower_registry_expr_recursive(expr);
        }
        match self.try_lower_registry_expr_result(expr) {
            Ok(Some(lowered_expr)) => Some(lowered_expr),
            Ok(None) => self.try_lower_registry_expr_recursive(expr),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                None
            }
        }
    }
}
