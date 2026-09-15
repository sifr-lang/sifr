use super::{
    HirExpr, HirIteratorOp, RustEmitter, RustExpr, RustStmt, Type, plain_call_target_for_ir,
};
impl RustEmitter {
    pub(crate) fn lower_timeout_aware_await_future_for_ir(
        &mut self,
        value: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Call { func, args, .. } = value {
            let params = self.resolve_plain_call_param_info(func, args.len());
            let needs_async_callable_adapter = params.as_ref().is_some_and(|params| {
                params.iter().any(|(param, _)| {
                    matches!(
                        crate::resolve_alias_type_for_plain_call(param),
                        Type::AsyncCallable(..)
                    )
                })
            }) || args.iter().any(|arg| {
                matches!(
                    crate::resolve_alias_type_for_plain_call(arg.ty()),
                    Type::Function(_) | Type::AsyncFunction(_) | Type::AsyncCallable(..)
                )
            });
            if needs_async_callable_adapter {
                let Some(params) = params else {
                    return Ok(None);
                };
                let mut lowered = Vec::with_capacity(args.len());
                for (arg, (param, _)) in args.iter().zip(params.iter()) {
                    let Some(mut value) = self.lower_stmt_expr_for_ir(arg)? else {
                        return Ok(None);
                    };
                    if let Type::AsyncCallable(callback_args, _, _) =
                        crate::resolve_alias_type_for_plain_call(param)
                    {
                        value = Self::send_async_callable_adapter(value, callback_args.len());
                    }
                    lowered.push(value);
                }
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(plain_call_target_for_ir(func)),
                    args: lowered,
                }));
            }
            if func == "__sifr_task_sleep" {
                let [duration] = args.as_slice() else {
                    return Ok(None);
                };
                let Some(duration_expr) = crate::try_lower_task_duration_expr(
                    duration,
                    "__sifr_task_timeout_sleep_seconds",
                ) else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "tokio".to_string(),
                        "time".to_string(),
                        "sleep".to_string(),
                    ])),
                    args: vec![duration_expr],
                }));
            }
        }

        if matches!(
            crate::resolve_alias_type_for_plain_call(value.ty()),
            Type::Task(_, _) | Type::BlockingTask(_, _)
        ) {
            let Some(receiver) = crate::try_lower_leaf_or_name_expr_result(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "join".to_string(),
                args: vec![],
            }));
        }

        self.lower_stmt_expr_for_ir(value)
    }

    pub(crate) fn wrap_option_local_value_for_ir(
        target_ty: &Type,
        value: &HirExpr,
        value_ty: &Type,
        lowered_value: crate::RustExpr,
    ) -> crate::RustExpr {
        if !crate::helpers::is_option_type(target_ty) {
            return lowered_value;
        }
        if matches!(value, HirExpr::NoneLiteral) || matches!(value_ty, Type::None) {
            return crate::RustExpr::Literal(crate::RustLiteral::None);
        }
        if crate::helpers::is_option_type(value_ty) {
            return lowered_value;
        }
        crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_value],
        }
    }

    pub(crate) fn coerce_local_value_for_target_type_for_ir(
        &mut self,
        target_ty: &Type,
        value: &HirExpr,
        lowered_value: crate::RustExpr,
    ) -> Result<crate::RustExpr, crate::CodegenError> {
        if matches!(
            crate::resolve_alias_type_for_plain_call(target_ty),
            Type::Iterable(_)
        ) {
            if let Some(coerced) =
                crate::intrinsic_method_emitters::registry_iterable_to_vec_expr(self, value)
            {
                return Ok(coerced);
            }
            return Err(crate::CodegenError::new(
                "failed to coerce iterable local binding value",
            ));
        }
        let value_ty = match value {
            HirExpr::QuestionMark { expr, .. } => {
                if let Type::Result(ok_ty, _) = crate::resolve_alias_type_for_plain_call(expr.ty())
                {
                    ok_ty.as_ref()
                } else {
                    value.ty()
                }
            }
            HirExpr::Name { name, ty, .. }
                if self.none_widened_local_bindings.contains(name)
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(ty),
                        Type::Any | Type::Unknown
                    ) =>
            {
                self.local_binding_types.get(name).unwrap_or(ty)
            }
            HirExpr::Name { ty, .. } => ty,
            _ => value.ty(),
        };
        if let Some(coerced) = crate::fixed_width_literal_expr_for_target(target_ty, value) {
            return Ok(coerced);
        }
        if matches!(
            crate::resolve_alias_type_for_plain_call(target_ty),
            Type::Int | Type::LiteralInt(_)
        ) {
            let owned_value = self.materialize_reusable_value_for_ir(value, lowered_value);
            return Ok(self.coerce_typed_expr_to_sifr_int_value(owned_value, value_ty));
        }
        if matches!(
            crate::resolve_alias_type_for_plain_call(target_ty),
            Type::Awaitable(_)
        ) && matches!(
            crate::resolve_alias_type_for_plain_call(value_ty),
            Type::Awaitable(_)
        ) && matches!(
            value,
            HirExpr::MethodCall { method, .. }
                if method == "__sifr_join_all" || method == "__sifr_cancel_all"
        ) {
            return Ok(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "Box".to_string(),
                    "pin".to_string(),
                ])),
                args: vec![lowered_value],
            });
        }
        let upcasted_value =
            self.consuming_value_conversion_for_ir(target_ty, value_ty, lowered_value.clone());
        let value_was_upcasted = upcasted_value != lowered_value;
        let lowered_value = upcasted_value;
        let wrapped_member = if matches!(value, HirExpr::NoneLiteral) {
            &Type::None
        } else {
            value_ty
        };
        if !value_was_upcasted {
            if let Some(wrapped) = crate::helpers::wrap_union_member_expr(
                target_ty,
                wrapped_member,
                lowered_value.clone(),
            ) {
                return Ok(wrapped);
            }
        }
        let lowered_value =
            crate::helpers::adapt_collection_storage_for_target(target_ty, value_ty, lowered_value);
        let lowered_value =
            Self::wrap_option_local_value_for_ir(target_ty, value, value_ty, lowered_value);
        if crate::helpers::is_option_type(target_ty) {
            return Ok(lowered_value);
        }
        Ok(lowered_value)
    }

    pub(crate) fn uses_debug_display_format_for_ir(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Int
            | Type::FixedInt(_)
            | Type::Float
            | Type::Bool
            | Type::Str
            | Type::None
            | Type::Range
            | Type::Union(_)
            | Type::LiteralInt(_)
            | Type::LiteralStr(_)
            | Type::LiteralBool(_)
            | Type::Class { .. }
            | Type::Newtype { .. }
            | Type::TypeVar(_)
            | Type::Enum { .. }
            | Type::Decimal
            | Type::BigDecimal => false,
            Type::List(_)
            | Type::Bytes
            | Type::Dict(_, _)
            | Type::Set(_)
            | Type::Tuple(_)
            | Type::Iterable(_)
            | Type::Iterator(_)
            | Type::Function(_)
            | Type::AsyncFunction(_)
            | Type::Coroutine(_, _)
            | Type::Task(_, _)
            | Type::TaskResult(_, _)
            | Type::Failure(_)
            | Type::TimeoutResult(_)
            | Type::Select2(_, _)
            | Type::BlockingTask(_, _)
            | Type::JoinSet(_, _)
            | Type::Awaitable(_)
            | Type::AsyncIterator(_, _)
            | Type::AsyncGenerator(_, _)
            | Type::PythonBuffer(_)
            | Type::PythonArrow(_)
            | Type::PythonDlpackTensor(_)
            | Type::PythonDlpackStream
            | Type::Callable(..)
            | Type::AsyncCallable(..)
            | Type::Result(_, _)
            | Type::Protocol { .. }
            | Type::Any
            | Type::Unknown
            | Type::Intersection(_)
            | Type::Never
            | Type::Template(_) => true,
            Type::Alias { body, .. } => Self::uses_debug_display_format_for_ir(body),
            Type::StructuralRecord(_) => !ty.supports_display_formatting(),
        }
    }

    pub(crate) fn option_inner_type_for_ir(ty: &Type) -> Option<Type> {
        ty.optional_member_type()
    }

    pub(crate) fn collect_stmt_string_concat_parts_for_ir<'a>(
        expr: &'a HirExpr,
        parts: &mut Vec<&'a HirExpr>,
    ) {
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            if op == "+" && matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                Self::collect_stmt_string_concat_parts_for_ir(left, parts);
                Self::collect_stmt_string_concat_parts_for_ir(right, parts);
                return;
            }
        }
        parts.push(expr);
    }

    pub(crate) fn try_lower_stmt_string_concat_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        else {
            return Ok(None);
        };
        if op != "+" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            return Ok(None);
        }

        let mut parts = Vec::new();
        Self::collect_stmt_string_concat_parts_for_ir(left, &mut parts);
        Self::collect_stmt_string_concat_parts_for_ir(right, &mut parts);

        if parts
            .iter()
            .all(|part| matches!(part, HirExpr::StringLiteral(_)))
        {
            let mut combined = String::new();
            for part in parts {
                if let HirExpr::StringLiteral(value) = part {
                    combined.push_str(value);
                }
            }
            return Ok(Some(crate::RustExpr::Literal(crate::RustLiteral::Str(
                combined,
            ))));
        }

        let capacity = Self::string_concat_capacity_expr_for_ir(&parts);
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: "__sifr_concat".to_string(),
            ty: Some(crate::RustType::String_),
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "String".to_string(),
                    "with_capacity".to_string(),
                ])),
                args: vec![capacity],
            },
        }];
        for part in parts {
            let (push_method, push_arg) = self.lower_string_push_method_and_arg_for_ir(part)?;
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident("__sifr_concat".to_string())),
                method: push_method,
                args: vec![push_arg],
            }));
        }
        Ok(Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(
                "__sifr_concat".to_string(),
            ))),
        }))
    }

    fn string_concat_capacity_expr_for_ir(parts: &[&HirExpr]) -> crate::RustExpr {
        let mut capacity_parts = Vec::with_capacity(parts.len());
        for part in parts {
            let len_expr = if let HirExpr::StringLiteral(value) = part {
                crate::RustExpr::Verbatim(format!("{}usize", value.len()))
            } else if let HirExpr::Name { name, .. } = part {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                    method: "len".to_string(),
                    args: vec![],
                }
            } else {
                crate::RustExpr::Verbatim("0usize".to_string())
            };
            capacity_parts.push(len_expr);
        }
        let mut iter = capacity_parts.into_iter();
        let Some(mut capacity) = iter.next() else {
            return crate::RustExpr::Verbatim("0usize".to_string());
        };
        for part in iter {
            capacity = crate::RustExpr::BinOp {
                left: Box::new(capacity),
                op: "+".to_string(),
                right: Box::new(part),
            };
        }
        capacity
    }

    pub(crate) fn resolve_alias_type_for_loop_iter(ty: &Type) -> &Type {
        match ty {
            Type::Alias { body, .. } => Self::resolve_alias_type_for_loop_iter(body),
            _ => ty,
        }
    }

    pub(crate) fn int_sifr_literal_expr(value: i64) -> RustExpr {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrInt".to_string(),
                "from_i64".to_string(),
            ])),
            args: vec![RustExpr::Literal(crate::RustLiteral::Int(value))],
        }
    }

    pub(crate) fn try_lower_range_iter_expr_for_ir(
        &mut self,
        start: &HirExpr,
        end: &HirExpr,
        step: Option<&HirExpr>,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(lowered_start) = self.lower_stmt_expr_for_ir(start)? else {
            return Ok(None);
        };
        let lowered_start = Self::clone_non_copy_name_expr_for_ir(start, lowered_start);
        let Some(lowered_end) = self.lower_stmt_expr_for_ir(end)? else {
            return Ok(None);
        };
        let lowered_end = Self::clone_non_copy_name_expr_for_ir(end, lowered_end);
        let lowered_step = if let Some(step) = step {
            let Some(lowered) = self.lower_stmt_expr_for_ir(step)? else {
                return Ok(None);
            };
            Self::clone_non_copy_name_expr_for_ir(step, lowered)
        } else {
            Self::int_sifr_literal_expr(1)
        };
        Ok(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrRange".to_string(),
                "new_known_nonzero".to_string(),
            ])),
            args: vec![lowered_start, lowered_end, lowered_step],
        }))
    }

    pub(crate) fn lower_comprehension_iter_for_ir(
        &mut self,
        iter_expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if let HirExpr::IteratorCall { op, args, .. } = iter_expr {
            if *op == HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], None);
            }
        }
        self.lower_structural_iter_source_expr_for_ir(iter_expr, None)
    }

    pub(crate) fn async_iterator_error_type_for_ir(iter_expr: &HirExpr) -> Option<Type> {
        match Self::resolve_alias_type_for_loop_iter(iter_expr.ty()) {
            Type::AsyncIterator(_, err_ty) | Type::AsyncGenerator(_, err_ty) => {
                Some(err_ty.as_ref().clone())
            }
            _ => None,
        }
    }

    pub(crate) fn try_lower_async_list_comp_for_ir(
        &mut self,
        value_expr: &HirExpr,
        generators: &[(String, HirExpr, Option<HirExpr>)],
        result_ty: &Type,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some((var, iter_expr, maybe_filter)) = generators.first() else {
            return Ok(None);
        };
        if generators.len() != 1 || var.contains(',') {
            return Ok(None);
        }
        let Some(iter_error_ty) = Self::async_iterator_error_type_for_ir(iter_expr) else {
            return Ok(None);
        };
        let Some(lowered_iter) = self.lower_stmt_expr_for_ir(iter_expr)? else {
            return Ok(None);
        };
        let Some(mut lowered_value) = self.lower_stmt_expr_for_ir(value_expr)? else {
            return Ok(None);
        };
        lowered_value = crate::ownership_plan::materialize_comprehension_value(
            value_expr,
            lowered_value,
            generators,
        );
        if let Type::List(element_ty) = Self::resolve_alias_type_for_loop_iter(result_ty) {
            lowered_value = crate::helpers::adapt_collection_value_for_target(
                element_ty.as_ref(),
                value_expr,
                lowered_value,
            );
        }

        let result_ident = "__sifr_async_list_comp".to_string();
        let iter_ident = "__sifr_async_list_iter".to_string();
        let next_ident = "__sifr_async_list_next".to_string();

        let push_stmt = RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(result_ident.clone())),
            method: "push".to_string(),
            args: vec![lowered_value],
        });
        let value_body = if let Some(filter) = maybe_filter {
            let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                return Ok(None);
            };
            vec![RustStmt::If {
                cond: lowered_filter,
                then_body: vec![push_stmt],
                else_body: None,
            }]
        } else {
            vec![push_stmt]
        };

        let next_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(iter_ident.clone())),
            method: "anext".to_string(),
            args: vec![],
        }));
        let next_value = if matches!(iter_error_ty.resolve_alias(), Type::Never) {
            next_call
        } else {
            RustExpr::Try(Box::new(next_call))
        };

        Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: true,
                    name: result_ident.clone(),
                    ty: None,
                    value: RustExpr::Vec(vec![]),
                },
                RustStmt::Let {
                    mutable: true,
                    name: iter_ident,
                    ty: None,
                    value: lowered_iter,
                },
                RustStmt::Loop {
                    body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: next_ident.clone(),
                            ty: None,
                            value: next_value,
                        },
                        RustStmt::Match {
                            expr: RustExpr::Ident(next_ident),
                            arms: vec![
                                crate::RustMatchArm {
                                    pattern: format!("Some({var})"),
                                    bindings: vec![var.clone()],
                                    guard: None,
                                    body: value_body,
                                },
                                crate::RustMatchArm {
                                    pattern: "None".to_string(),
                                    bindings: vec![],
                                    guard: None,
                                    body: vec![RustStmt::Break],
                                },
                            ],
                        },
                    ],
                },
            ],
            expr: Some(Box::new(RustExpr::Ident(result_ident))),
        }))
    }

    pub(crate) fn try_lower_async_set_comp_for_ir(
        &mut self,
        value_expr: &HirExpr,
        generators: &[(String, HirExpr, Option<HirExpr>)],
        result_ty: &Type,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some((var, iter_expr, maybe_filter)) = generators.first() else {
            return Ok(None);
        };
        if generators.len() != 1 || var.contains(',') {
            return Ok(None);
        }
        let Some(iter_error_ty) = Self::async_iterator_error_type_for_ir(iter_expr) else {
            return Ok(None);
        };
        let Some(lowered_iter) = self.lower_stmt_expr_for_ir(iter_expr)? else {
            return Ok(None);
        };
        let Some(mut lowered_value) = self.lower_stmt_expr_for_ir(value_expr)? else {
            return Ok(None);
        };
        if let Type::Set(element_ty) = Self::resolve_alias_type_for_loop_iter(result_ty) {
            lowered_value = crate::helpers::adapt_collection_value_for_target(
                element_ty.as_ref(),
                value_expr,
                lowered_value,
            );
        }

        let result_ident = "__sifr_async_set_comp".to_string();
        let iter_ident = "__sifr_async_set_iter".to_string();
        let next_ident = "__sifr_async_set_next".to_string();

        let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(result_ident.clone())),
            method: "insert".to_string(),
            args: vec![lowered_value],
        });
        let value_body = if let Some(filter) = maybe_filter {
            let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                return Ok(None);
            };
            vec![RustStmt::If {
                cond: lowered_filter,
                then_body: vec![insert_stmt],
                else_body: None,
            }]
        } else {
            vec![insert_stmt]
        };

        let next_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(iter_ident.clone())),
            method: "anext".to_string(),
            args: vec![],
        }));
        let next_value = if matches!(iter_error_ty.resolve_alias(), Type::Never) {
            next_call
        } else {
            RustExpr::Try(Box::new(next_call))
        };

        Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: true,
                    name: result_ident.clone(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "HashSet".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    },
                },
                RustStmt::Let {
                    mutable: true,
                    name: iter_ident,
                    ty: None,
                    value: lowered_iter,
                },
                RustStmt::Loop {
                    body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: next_ident.clone(),
                            ty: None,
                            value: next_value,
                        },
                        RustStmt::Match {
                            expr: RustExpr::Ident(next_ident),
                            arms: vec![
                                crate::RustMatchArm {
                                    pattern: format!("Some({var})"),
                                    bindings: vec![var.clone()],
                                    guard: None,
                                    body: value_body,
                                },
                                crate::RustMatchArm {
                                    pattern: "None".to_string(),
                                    bindings: vec![],
                                    guard: None,
                                    body: vec![RustStmt::Break],
                                },
                            ],
                        },
                    ],
                },
            ],
            expr: Some(Box::new(RustExpr::Ident(result_ident))),
        }))
    }

    pub(crate) fn try_lower_async_dict_comp_for_ir(
        &mut self,
        key_expr: &HirExpr,
        val_expr: &HirExpr,
        generators: &[(String, HirExpr, Option<HirExpr>)],
        result_ty: &Type,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some((var, iter_expr, maybe_filter)) = generators.first() else {
            return Ok(None);
        };
        if generators.len() != 1 || var.contains(',') {
            return Ok(None);
        }
        let Some(iter_error_ty) = Self::async_iterator_error_type_for_ir(iter_expr) else {
            return Ok(None);
        };
        let Some(lowered_iter) = self.lower_stmt_expr_for_ir(iter_expr)? else {
            return Ok(None);
        };
        let Some(mut lowered_key) = self.lower_stmt_expr_for_ir(key_expr)? else {
            return Ok(None);
        };
        lowered_key =
            crate::helpers::clone_dict_key_for_reused_value(key_expr, val_expr, lowered_key);
        let Some(mut lowered_value) = self.lower_stmt_expr_for_ir(val_expr)? else {
            return Ok(None);
        };
        if let Type::Dict(key_ty, value_ty) = Self::resolve_alias_type_for_loop_iter(result_ty) {
            lowered_key = crate::helpers::adapt_collection_value_for_target(
                key_ty.as_ref(),
                key_expr,
                lowered_key,
            );
            lowered_value = crate::helpers::adapt_collection_value_for_target(
                value_ty.as_ref(),
                val_expr,
                lowered_value,
            );
        }

        let result_ident = "__sifr_async_dict_comp".to_string();
        let iter_ident = "__sifr_async_dict_iter".to_string();
        let next_ident = "__sifr_async_dict_next".to_string();

        let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(result_ident.clone())),
            method: "insert".to_string(),
            args: vec![lowered_key, lowered_value],
        });
        let value_body = if let Some(filter) = maybe_filter {
            let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                return Ok(None);
            };
            vec![RustStmt::If {
                cond: lowered_filter,
                then_body: vec![insert_stmt],
                else_body: None,
            }]
        } else {
            vec![insert_stmt]
        };

        let next_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(iter_ident.clone())),
            method: "anext".to_string(),
            args: vec![],
        }));
        let next_value = if matches!(iter_error_ty.resolve_alias(), Type::Never) {
            next_call
        } else {
            RustExpr::Try(Box::new(next_call))
        };

        Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: true,
                    name: result_ident.clone(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "HashMap".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    },
                },
                RustStmt::Let {
                    mutable: true,
                    name: iter_ident,
                    ty: None,
                    value: lowered_iter,
                },
                RustStmt::Loop {
                    body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: next_ident.clone(),
                            ty: None,
                            value: next_value,
                        },
                        RustStmt::Match {
                            expr: RustExpr::Ident(next_ident),
                            arms: vec![
                                crate::RustMatchArm {
                                    pattern: format!("Some({var})"),
                                    bindings: vec![var.clone()],
                                    guard: None,
                                    body: value_body,
                                },
                                crate::RustMatchArm {
                                    pattern: "None".to_string(),
                                    bindings: vec![],
                                    guard: None,
                                    body: vec![RustStmt::Break],
                                },
                            ],
                        },
                    ],
                },
            ],
            expr: Some(Box::new(RustExpr::Ident(result_ident))),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fallible_buffer_value(target_ty: Type) -> HirExpr {
        HirExpr::QuestionMark {
            expr: Box::new(HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "make_buffer".to_string(),
                args: Vec::new(),
                ty: Type::Result(
                    Box::new(Type::PythonBuffer(Box::new(Type::FixedInt(
                        sifr_type_system::FixedIntType::U8,
                    )))),
                    Box::new(Type::Str),
                ),
            }),
            ty: target_ty,
        }
    }

    #[test]
    fn fallible_local_binding_coercion_uses_unwrapped_source_type() {
        let buffer_ty =
            Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
        let option_ty = Type::Union(vec![buffer_ty.clone(), Type::None]);
        let mut emitter = RustEmitter::new();
        let option_value = fallible_buffer_value(option_ty.clone());
        let lowered = emitter
            .coerce_local_value_for_target_type_for_ir(
                &option_ty,
                &option_value,
                RustExpr::Ident("value".to_string()),
            )
            .expect("option coercion");
        assert!(matches!(lowered, RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &["Some".to_string()])));

        let buffer_variant = buffer_ty.union_variant_name();
        let union_ty = Type::Union(vec![buffer_ty, Type::Int, Type::None]);
        let union_value = fallible_buffer_value(union_ty.clone());
        let lowered = emitter
            .coerce_local_value_for_target_type_for_ir(
                &union_ty,
                &union_value,
                RustExpr::Ident("value".to_string()),
            )
            .expect("union coercion");
        assert!(matches!(lowered, RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path.last() == Some(&buffer_variant))));
    }
}
