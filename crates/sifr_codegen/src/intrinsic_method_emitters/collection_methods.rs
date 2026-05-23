impl RustEmitter {
    pub(crate) fn effective_method_object_ty(&self, object: &HirExpr) -> Type {
        if let HirExpr::Name { name, ty } = object {
            if self.none_widened_local_bindings.contains(name) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
        }
        object.ty().clone()
    }

    pub(crate) fn effective_registry_expr_ty(&self, expr: &HirExpr) -> Type {
        if let HirExpr::Name { name, ty } = expr {
            if self.none_widened_local_bindings.contains(name) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
        }
        expr.ty().clone()
    }

    /// Check if a name is a stdlib constant.
    pub(crate) fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
            && self.intrinsic_functions.contains(name)
    }

    pub(crate) fn try_lower_registry_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let effective_object_ty = self.effective_method_object_ty(object);
        let object_ty = crate::resolve_alias_type_for_plain_call(&effective_object_ty);
        if let Some(lowered) =
            self.try_lower_defaultdict_index_method_call_expr(object, method, args)
        {
            return Some(lowered);
        }
        let is_deque_data_field = self.is_deque_data_field(object);
        let object_expr = self.try_lower_registry_expr_strict(object)?;
        let mut arg_exprs = self.try_lower_registry_exprs_strict(args)?;

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
            && matches!(args[0].ty(), Type::TypeVar(_))
        {
            // Clone TypeVar list args to avoid move issues.
            arg_exprs[0] = crate::RustExpr::MethodCall {
                receiver: Box::new(arg_exprs[0].clone()),
                method: "clone".to_string(),
                args: vec![],
            };
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            // Clone borrowed/mut-borrowed move-owned values.
            let needs_clone = if let HirExpr::Name { name, ty } = &args[1] {
                (self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()))
                    && ty.ownership() != sifr_type_system::OwnershipKind::Copy
            } else {
                false
            };
            if needs_clone {
                arg_exprs[1] = crate::RustExpr::MethodCall {
                    receiver: Box::new(arg_exprs[1].clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
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
                arg_exprs[0] = RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(arg_exprs[0].clone()))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
            }
        }

        if let Type::Class {
            fields, methods, ..
        } = crate::resolve_alias_type_for_plain_call(object_ty)
        {
            let is_callable_field = !methods.iter().any(|(name, _)| name == method)
                && fields
                    .iter()
                    .any(|(name, ty)| name == method && matches!(ty, Type::Callable(..)));
            if is_callable_field {
                return Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Field {
                        expr: Box::new(object_expr),
                        field: method.to_string(),
                    }))),
                    args: arg_exprs,
                });
            }
        }

        if matches!(object_ty, Type::List(_)) && method == "extend" && args.len() == 1 {
            return Some(crate::RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: "extend".to_string(),
                args: vec![registry_iterable_to_owned_iter_expr(self, &args[0])?],
            });
        }

        if matches!(object_ty, Type::Set(_)) {
            if let Some(lowered) =
                self.try_lower_registry_set_method_call_expr(&object_expr, method, args)
            {
                return Some(lowered);
            }
        }

        let lowered = methods::lower_method_with_context(
            object_ty,
            method,
            &object_expr,
            &arg_exprs,
            is_deque_data_field,
        )?;
        let lowered_expr = Self::unwrap_compiler_verified_nonempty_pop_result(
            object_ty,
            method,
            args,
            method_return_ty,
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

    fn unwrap_compiler_verified_nonempty_pop_result(
        object_ty: &Type,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
        lowered_expr: crate::RustExpr,
    ) -> crate::RustExpr {
        if !supports_nonempty_pop_narrowing_type_for_codegen(object_ty) {
            return lowered_expr;
        }
        if !is_narrowable_pop_call_for_codegen(method, args) {
            return lowered_expr;
        }
        if crate::helpers::is_option_type(method_return_ty) {
            return lowered_expr;
        }
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::LetElse {
                pattern: "Some(__sifr_nonempty_pop_value)".to_string(),
                value: lowered_expr,
                else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                        "compiler-verified non-empty pop should return Some".to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(crate::RustExpr::Ident(
                "__sifr_nonempty_pop_value".to_string(),
            ))),
        }
    }

    fn try_lower_registry_set_method_call_expr(
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
                        args: vec![registry_iterable_to_owned_iter_expr(self, arg)
                            .map(|expr| crate::RustExpr::Paren(Box::new(expr)))?],
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
                        args: vec![registry_iterable_to_owned_iter_expr(self, arg)
                            .map(|expr| crate::RustExpr::Paren(Box::new(expr)))?],
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

    fn try_lower_defaultdict_index_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = object
        else {
            return None;
        };
        let (alias_name, key_ty, _) = registry_defaultdict_alias_parts(base_object.ty())?;
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_args = self.try_lower_registry_exprs_strict(args)?;
        let entry_expr = crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "entry".to_string(),
                args: vec![registry_defaultdict_key_arg(index, lowered_index, key_ty)],
            }),
            method: "or_insert".to_string(),
            args: vec![registry_defaultdict_default_expr(alias_name)],
        };
        match (alias_name, method, lowered_args.as_slice()) {
            ("__compat_defaultdict_list", "append", [value]) => Some(crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(entry_expr),
                    method: "push".to_string(),
                    args: vec![value.clone()],
                })],
                expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
            }),
            ("__compat_defaultdict_set", "add", [value]) => Some(crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(entry_expr),
                    method: "insert".to_string(),
                    args: vec![value.clone()],
                })],
                expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
            }),
            _ => None,
        }
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
