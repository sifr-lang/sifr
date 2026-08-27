use super::{HirExpr, HirIteratorOp, RustEmitter, RustStmt, Type};
impl RustEmitter {
    pub(crate) fn try_lower_for_iter_expr_for_ir(
        &mut self,
        iter: &HirExpr,
        target_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::IteratorCall { op, args, .. } = iter {
            if *op == HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], Some(target_ty));
            }
            if *op == HirIteratorOp::Enumerate && args.len() == 1 {
                return self.lower_enumerate_for_iter_expr_for_ir(&args[0], Some(target_ty));
            }
        }
        if let HirExpr::Call { func, args, .. } = iter {
            if func == "iter" && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], Some(target_ty));
            }
            if func == "enumerate" && args.len() == 1 {
                return self.lower_enumerate_for_iter_expr_for_ir(&args[0], Some(target_ty));
            }
        }
        self.lower_structural_iter_source_expr_for_ir(iter, Some(target_ty))
    }

    pub(crate) fn lower_enumerate_iter_chain_for_ir(
        iter_source: crate::RustExpr,
    ) -> crate::RustExpr {
        crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(iter_source),
                method: "enumerate".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![crate::RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "(i, v)".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(crate::RustExpr::Tuple(vec![
                    crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("i".to_string())),
                        ty: crate::RustType::I64,
                    },
                    crate::RustExpr::Ident("v".to_string()),
                ])),
                is_move: false,
            }],
        }
    }

    pub(crate) fn lower_enumerate_for_iter_expr_for_ir(
        &mut self,
        source: &HirExpr,
        element_type_hint: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let Some(iter_source) =
            self.lower_structural_iter_source_expr_for_ir(source, element_type_hint)?
        else {
            return Ok(None);
        };
        Ok(Some(Self::lower_enumerate_iter_chain_for_ir(iter_source)))
    }

    pub(crate) fn lower_structural_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
        element_type_hint: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        self.lower_iter_source_expr_for_ir_with_mode(source, false, element_type_hint, None)
    }

    pub(crate) fn lower_string_chars_for_iter_expr_for_ir(
        &mut self,
        source: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::IteratorCall { op, args, .. } = source {
            if *op == HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_string_chars_for_iter_expr_for_ir(&args[0]);
            }
        }
        if let HirExpr::Call { func, args, .. } = source {
            if func == "iter" && args.len() == 1 {
                return self.lower_string_chars_for_iter_expr_for_ir(&args[0]);
            }
        }
        let Some(lowered_source) = self.lower_rendered_expr_for_ir(source)? else {
            return Ok(None);
        };
        Ok(Some(crate::RustExpr::MethodCall {
            receiver: Box::new(Self::normalize_for_loop_iter_expr(lowered_source)),
            method: "chars".to_string(),
            args: vec![],
        }))
    }

    pub(crate) fn lower_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        self.lower_iter_source_expr_for_ir_with_mode(source, true, None, None)
    }

    pub(crate) fn lower_escaping_iter_return_expr_for_ir(
        &mut self,
        value: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let source = match value {
            HirExpr::IteratorCall { op, args, .. }
                if *op == HirIteratorOp::Iter && args.len() == 1 =>
            {
                &args[0]
            }
            HirExpr::Call { func, args, .. } if func == "iter" && args.len() == 1 => &args[0],
            _ => return Ok(None),
        };

        let can_consume_owned_source = match source {
            HirExpr::Name { name, .. } => {
                !self.borrowed_params.contains(name) && !self.mut_borrowed_params.contains(name)
            }
            HirExpr::FieldAccess { .. } | HirExpr::MethodCall { .. } => true,
            _ => matches!(
                crate::helpers::classify_value_category(source),
                crate::helpers::ValueCategory::Temporary
            ),
        };

        if !can_consume_owned_source {
            return Ok(None);
        }

        self.lower_iter_source_expr_for_ir_with_mode(
            source,
            true,
            None,
            Some(crate::helpers::SourceAccessMode::Consume),
        )
    }

    pub(crate) fn class_method_signature_for_iter_for_ir<'a>(
        methods: &'a [(String, sifr_type_system::FunctionType)],
        method_name: &str,
    ) -> Option<&'a sifr_type_system::FunctionType> {
        methods.iter().find_map(
            |(name, ft)| {
                if name == method_name { Some(ft) } else { None }
            },
        )
    }

    pub(crate) fn class_has_next_for_iter_for_ir(
        methods: &[(String, sifr_type_system::FunctionType)],
    ) -> bool {
        Self::class_method_signature_for_iter_for_ir(methods, "__next__").is_some_and(|next_ft| {
            next_ft.params.is_empty() && next_ft.return_type.optional_member_type().is_some()
        })
    }

    pub(crate) fn class_next_iter_expr_for_ir(source_expr: crate::RustExpr) -> crate::RustExpr {
        let state_name = "__sifr_for_iter_state".to_string();
        crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec![
                "std".to_string(),
                "iter".to_string(),
                "from_fn".to_string(),
            ])),
            args: vec![crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Let {
                    mutable: true,
                    name: state_name.clone(),
                    ty: None,
                    value: source_expr,
                }],
                expr: Some(Box::new(crate::RustExpr::Closure {
                    params: vec![],
                    body: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(state_name)),
                        method: "__next__".to_string(),
                        args: vec![],
                    }),
                    is_move: true,
                })),
            }],
        }
    }

    pub(crate) fn apply_copy_clone_yield_mode_for_ir(
        iter_expr: crate::RustExpr,
        yield_mode: crate::helpers::YieldMode,
    ) -> crate::RustExpr {
        match yield_mode {
            crate::helpers::YieldMode::Copy => crate::RustExpr::MethodCall {
                receiver: Box::new(iter_expr),
                method: "copied".to_string(),
                args: vec![],
            },
            crate::helpers::YieldMode::Clone => crate::RustExpr::MethodCall {
                receiver: Box::new(iter_expr),
                method: "cloned".to_string(),
                args: vec![],
            },
            crate::helpers::YieldMode::Move | crate::helpers::YieldMode::Borrow => iter_expr,
        }
    }

    pub(crate) fn wrap_iterator_expr_for_mode_for_ir(
        iterator_expr: crate::RustExpr,
        prefer_boxed_iterator: bool,
    ) -> crate::RustExpr {
        if prefer_boxed_iterator {
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "Box".to_string(),
                    "new".to_string(),
                ])),
                args: vec![iterator_expr],
            }
        } else {
            iterator_expr
        }
    }

    pub(crate) fn lower_homogeneous_tuple_iter_expr(
        lowered_source: crate::RustExpr,
        tuple_len: usize,
        source_access_mode: crate::helpers::SourceAccessMode,
        yield_mode: crate::helpers::YieldMode,
    ) -> crate::RustExpr {
        let tuple_binding = "__sifr_tuple_iter_src".to_string();
        let bound_value = match source_access_mode {
            crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_source))),
                method: "clone".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Consume => lowered_source,
        };
        let tuple_items = (0..tuple_len)
            .map(|index| {
                let field_expr = crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Ident(tuple_binding.clone())),
                    field: index.to_string(),
                };
                match yield_mode {
                    crate::helpers::YieldMode::Copy | crate::helpers::YieldMode::Move => field_expr,
                    crate::helpers::YieldMode::Clone | crate::helpers::YieldMode::Borrow => {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(field_expr),
                            method: "clone".to_string(),
                            args: vec![],
                        }
                    }
                }
            })
            .collect();
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::Let {
                mutable: false,
                name: tuple_binding,
                ty: None,
                value: bound_value,
            }],
            expr: Some(Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Vec(tuple_items)),
                method: "into_iter".to_string(),
                args: vec![],
            })),
        }
    }

    pub(crate) fn lower_iter_source_expr_for_ir_with_mode(
        &mut self,
        source: &HirExpr,
        prefer_boxed_iterator: bool,
        element_type_hint: Option<&Type>,
        source_access_mode_override: Option<crate::helpers::SourceAccessMode>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::RangeLiteral {
            start, end, step, ..
        } = source
        {
            if let Some(lowered_range_iter) =
                self.try_lower_range_iter_expr_for_ir(start, end, step.as_deref())?
            {
                return Ok(Some(lowered_range_iter));
            }
        }

        if let Some(lowered_dict_list_iter) = self
            .try_lower_dict_indexed_list_iter_source_expr_for_ir(
                source,
                prefer_boxed_iterator,
                element_type_hint,
            )?
        {
            return Ok(Some(lowered_dict_list_iter));
        }

        let Some(lowered_source) = self.lower_rendered_expr_for_ir(source)? else {
            return Ok(None);
        };
        let lowered_source = Self::normalize_for_loop_iter_expr(lowered_source);
        let source_ty = Self::resolve_alias_type_for_loop_iter(source.ty());
        let mut plan =
            crate::helpers::plan_iterator_ownership_with_element_hint(source, element_type_hint);
        if let Some(source_access_mode) = source_access_mode_override {
            plan.source_access_mode = source_access_mode;
            if matches!(
                source_access_mode,
                crate::helpers::SourceAccessMode::Consume
            ) {
                plan.yield_mode = crate::helpers::YieldMode::Move;
            }
        }

        if matches!(source_ty, Type::Iterator(_))
            || matches!(source, HirExpr::GeneratorExpr { .. })
            || self.is_generator_call(source)
            || Self::is_iterator_like_expr_for_ir(&lowered_source)
        {
            return Ok(Some(lowered_source));
        }

        if let Type::Class { name, methods, .. } = source_ty {
            let class_source = match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_source.clone()))),
                    method: "clone".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Consume => lowered_source.clone(),
            };
            if let Some(iter_ft) = Self::class_method_signature_for_iter_for_ir(methods, "__iter__")
            {
                if iter_ft.params.is_empty() {
                    let iter_call = crate::RustExpr::MethodCall {
                        receiver: Box::new(class_source.clone()),
                        method: "__iter__".to_string(),
                        args: vec![],
                    };
                    if matches!(
                        iter_ft.return_type.as_ref().resolve_alias(),
                        Type::Class { name: ret_name, .. } if ret_name == name
                    ) && Self::class_has_next_for_iter_for_ir(methods)
                    {
                        return Ok(Some(Self::class_next_iter_expr_for_ir(iter_call)));
                    }
                    if let Type::Class {
                        methods: ret_methods,
                        ..
                    } = iter_ft.return_type.as_ref().resolve_alias()
                    {
                        if Self::class_has_next_for_iter_for_ir(ret_methods) {
                            return Ok(Some(Self::class_next_iter_expr_for_ir(iter_call)));
                        }
                    }
                    return Ok(Some(iter_call));
                }
            }
            if Self::class_has_next_for_iter_for_ir(methods) {
                return Ok(Some(Self::class_next_iter_expr_for_ir(class_source)));
            }
            return Ok(Some(lowered_source));
        }

        let iterator_expr = match source_ty {
            Type::List(_) | Type::Set(_) | Type::Iterable(_) => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "into_iter".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Preserve => {
                    Self::apply_copy_clone_yield_mode_for_ir(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_source),
                            method: "iter".to_string(),
                            args: vec![],
                        },
                        plan.yield_mode,
                    )
                }
            },
            Type::Bytes => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_source),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__byte".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Ident("__byte".to_string())),
                            ty: crate::RustType::Named("u8".to_string()),
                        }),
                        is_move: false,
                    }],
                },
                crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_source),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__byte".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Deref(Box::new(
                                crate::RustExpr::Ident("__byte".to_string()),
                            ))),
                            ty: crate::RustType::Named("u8".to_string()),
                        }),
                        is_move: false,
                    }],
                },
            },
            Type::Dict(_, _) => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "into_keys".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Preserve => {
                    Self::apply_copy_clone_yield_mode_for_ir(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_source),
                            method: "keys".to_string(),
                            args: vec![],
                        },
                        plan.yield_mode,
                    )
                }
            },
            Type::Str => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "c".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                    is_move: false,
                }],
            },
            Type::Range => lowered_source,
            Type::Tuple(elems)
                if !elems.is_empty() && elems.iter().all(|elem| elem == &elems[0]) =>
            {
                Self::lower_homogeneous_tuple_iter_expr(
                    lowered_source,
                    elems.len(),
                    plan.source_access_mode,
                    plan.yield_mode,
                )
            }
            _ => return Ok(Some(lowered_source)),
        };
        Ok(Some(Self::wrap_iterator_expr_for_mode_for_ir(
            iterator_expr,
            prefer_boxed_iterator,
        )))
    }

    fn try_lower_dict_indexed_list_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
        prefer_boxed_iterator: bool,
        element_type_hint: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let HirExpr::Index { object, index, .. } = source else {
            return Ok(None);
        };
        let Type::Dict(_, value_ty) = crate::resolve_alias_type_for_plain_call(object.ty()) else {
            return Ok(None);
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
            Type::List(_)
        ) {
            return Ok(None);
        }

        let Some(lowered_object) = self.lower_rendered_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_rendered_expr_for_ir(index)? else {
            return Ok(None);
        };
        let plan =
            crate::helpers::plan_iterator_ownership_with_element_hint(source, element_type_hint);
        let iter_binding = "__sifr_dict_iter_source".to_string();
        let iterator = Self::apply_copy_clone_yield_mode_for_ir(
            crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(iter_binding.clone())),
                method: "iter".to_string(),
                args: vec![],
            },
            plan.yield_mode,
        );
        let iterator = Self::wrap_iterator_expr_for_mode_for_ir(iterator, prefer_boxed_iterator);
        Ok(Some(crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::LetElse {
                pattern: format!("Some({iter_binding})"),
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "get".to_string(),
                    args: vec![Self::build_dict_lookup_key_arg_for_ir(lowered_index)],
                },
                else_body: vec![crate::RustStmt::Expr(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "std".to_string(),
                        "process".to_string(),
                        "abort".to_string(),
                    ])),
                    args: vec![],
                })],
            }],
            expr: Some(Box::new(iterator)),
        }))
    }

    pub(crate) fn is_collect_call_expr(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::MethodCall { method, .. } => {
                method == "collect" || method.starts_with("collect::<")
            }
            crate::RustExpr::Paren(inner) => Self::is_collect_call_expr(inner),
            _ => false,
        }
    }

    pub(crate) fn normalize_for_loop_iter_expr(expr: crate::RustExpr) -> crate::RustExpr {
        if let crate::RustExpr::MethodCall {
            receiver,
            method,
            args,
        } = expr
        {
            if method == "cloned" && args.is_empty() && Self::is_collect_call_expr(&receiver) {
                return *receiver;
            }
            return crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            };
        }
        expr
    }

    pub(crate) fn is_iterator_like_expr_for_ir(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } => {
                matches!(
                    method.as_str(),
                    "into_iter"
                        | "into_keys"
                        | "map"
                        | "filter"
                        | "filter_map"
                        | "zip"
                        | "chain"
                        | "enumerate"
                        | "copied"
                        | "cloned"
                ) || Self::is_iterator_like_expr_for_ir(receiver)
            }
            crate::RustExpr::FnCall { func, args } => {
                Self::is_iterator_like_expr_for_ir(func)
                    || args.iter().any(Self::is_iterator_like_expr_for_ir)
            }
            crate::RustExpr::Paren(inner)
            | crate::RustExpr::Try(inner)
            | crate::RustExpr::Await(inner)
            | crate::RustExpr::Deref(inner)
            | crate::RustExpr::Clone(inner) => Self::is_iterator_like_expr_for_ir(inner),
            _ => false,
        }
    }

    pub(crate) fn rust_stmts_contain_await(stmts: &[RustStmt]) -> bool {
        stmts.iter().any(Self::rust_stmt_contains_await)
    }

    pub(crate) fn rust_stmt_contains_await(stmt: &RustStmt) -> bool {
        match stmt {
            RustStmt::CompilerFragment(_) => true,
            RustStmt::Let { value, .. }
            | RustStmt::LetPattern { value, .. }
            | RustStmt::Expr(value)
            | RustStmt::TailExpr(value)
            | RustStmt::Return(Some(value)) => Self::rust_expr_contains_await(value),
            RustStmt::LetElse {
                value, else_body, ..
            } => Self::rust_expr_contains_await(value) || Self::rust_stmts_contain_await(else_body),
            RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
                Self::rust_expr_contains_await(target) || Self::rust_expr_contains_await(value)
            }
            RustStmt::Assert { cond, msg } => {
                Self::rust_expr_contains_await(cond)
                    || msg.as_ref().is_some_and(Self::rust_expr_contains_await)
            }
            RustStmt::LetDecl { .. }
            | RustStmt::Return(None)
            | RustStmt::Break
            | RustStmt::Continue => false,
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                Self::rust_expr_contains_await(cond)
                    || Self::rust_stmts_contain_await(then_body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| Self::rust_stmts_contain_await(body))
            }
            RustStmt::IfLet {
                expr,
                then_body,
                else_body,
                ..
            } => {
                Self::rust_expr_contains_await(expr)
                    || Self::rust_stmts_contain_await(then_body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| Self::rust_stmts_contain_await(body))
            }
            RustStmt::Match { expr, arms } => {
                Self::rust_expr_contains_await(expr)
                    || arms.iter().any(Self::rust_match_arm_contains_await)
            }
            RustStmt::For { iter, body, .. } => {
                Self::rust_expr_contains_await(iter) || Self::rust_stmts_contain_await(body)
            }
            RustStmt::With { items, body } => {
                items
                    .iter()
                    .any(|item| Self::rust_expr_contains_await(&item.value))
                    || Self::rust_stmts_contain_await(body)
            }
            RustStmt::While { cond, body } => {
                Self::rust_expr_contains_await(cond) || Self::rust_stmts_contain_await(body)
            }
            RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
                Self::rust_stmts_contain_await(body)
            }
        }
    }

    pub(crate) fn rust_match_arm_contains_await(arm: &crate::RustMatchArm) -> bool {
        arm.guard
            .as_ref()
            .is_some_and(Self::rust_expr_contains_await)
            || Self::rust_stmts_contain_await(&arm.body)
    }

    pub(crate) fn rust_expr_contains_await(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::Await(_) | crate::RustExpr::TimeoutAwait { .. } => true,
            crate::RustExpr::Ident(value) => value.contains(".await"),
            crate::RustExpr::CompilerFragment(source) => source.contains(".await"),
            crate::RustExpr::Literal(_) | crate::RustExpr::Path(_) => false,
            crate::RustExpr::MethodCall { receiver, args, .. } => {
                Self::rust_expr_contains_await(receiver)
                    || args.iter().any(Self::rust_expr_contains_await)
            }
            crate::RustExpr::FnCall { func, args } => {
                Self::rust_expr_contains_await(func)
                    || args.iter().any(Self::rust_expr_contains_await)
            }
            crate::RustExpr::MacroCall { args, .. }
            | crate::RustExpr::FormatMacro { args, .. }
            | crate::RustExpr::Tuple(args)
            | crate::RustExpr::Array(args)
            | crate::RustExpr::Vec(args) => args.iter().any(Self::rust_expr_contains_await),
            crate::RustExpr::BinOp { left, right, .. } => {
                Self::rust_expr_contains_await(left) || Self::rust_expr_contains_await(right)
            }
            crate::RustExpr::UnaryOp { operand, .. }
            | crate::RustExpr::Deref(operand)
            | crate::RustExpr::Clone(operand)
            | crate::RustExpr::Cast { expr: operand, .. }
            | crate::RustExpr::Ref { expr: operand, .. }
            | crate::RustExpr::Try(operand)
            | crate::RustExpr::Paren(operand) => Self::rust_expr_contains_await(operand),
            crate::RustExpr::Field { expr, .. } => Self::rust_expr_contains_await(expr),
            crate::RustExpr::Index { expr, index } => {
                Self::rust_expr_contains_await(expr) || Self::rust_expr_contains_await(index)
            }
            crate::RustExpr::Slice { expr, start, stop } => {
                Self::rust_expr_contains_await(expr)
                    || start
                        .as_ref()
                        .is_some_and(|start| Self::rust_expr_contains_await(start))
                    || stop
                        .as_ref()
                        .is_some_and(|stop| Self::rust_expr_contains_await(stop))
            }
            crate::RustExpr::Block { stmts, expr } => {
                Self::rust_stmts_contain_await(stmts)
                    || expr
                        .as_ref()
                        .is_some_and(|expr| Self::rust_expr_contains_await(expr))
            }
            crate::RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => {
                Self::rust_expr_contains_await(cond)
                    || Self::rust_expr_contains_await(then_expr)
                    || else_expr
                        .as_ref()
                        .is_some_and(|expr| Self::rust_expr_contains_await(expr))
            }
            crate::RustExpr::Match { expr, arms } => {
                Self::rust_expr_contains_await(expr)
                    || arms.iter().any(Self::rust_match_arm_contains_await)
            }
            crate::RustExpr::Closure { body, .. } => Self::rust_expr_contains_await(body),
            crate::RustExpr::ClosureBlock { body, .. }
            | crate::RustExpr::AsyncBlock { body, .. } => Self::rust_stmts_contain_await(body),
            crate::RustExpr::StructInit { fields, .. } => fields
                .iter()
                .any(|(_, value)| Self::rust_expr_contains_await(value)),
            crate::RustExpr::Range { start, end } => {
                Self::rust_expr_contains_await(start) || Self::rust_expr_contains_await(end)
            }
        }
    }
}
