use super::{
    body_contains_yield, collect_mutated_vars_with_sigs, collect_reassigned_vars, HirFunction,
    HirStmt, OwnershipKind, RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt,
    RustType, Type, Visibility,
};
impl RustEmitter {
    pub(crate) fn lower_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        let yield_ty = if let Type::Iterator(elem) = func.return_type.resolve_alias() {
            self.rust_ir_type_with_generics(elem)
        } else {
            RustType::I64
        };

        let mut body = Self::emit_mutable_param_shadow_stmts(mutable_param_shadows);
        for param in &func.params {
            if mutable_param_shadows
                .iter()
                .any(|(name, _)| name == &param.name)
                || !param.convention.is_borrowed()
                || param.ty.ownership() == OwnershipKind::Copy
            {
                continue;
            }
            body.push(RustStmt::Let {
                mutable: false,
                name: param.name.clone(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(param.name.clone()))),
            });
        }
        let generator_iter_ty = RustType::Generic {
            base: "std::vec::IntoIter".to_string(),
            params: vec![yield_ty.clone()],
        };
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_generator_initialized".to_string(),
            ty: Some(RustType::Bool),
            value: RustExpr::Literal(RustLiteral::Bool(false)),
        });
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_generator_iter".to_string(),
            ty: Some(generator_iter_ty),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            },
        });

        let cloned_borrowed_param_names: Vec<String> = func
            .params
            .iter()
            .filter(|param| {
                !mutable_param_shadows
                    .iter()
                    .any(|(name, _)| name == &param.name)
                    && param.convention.is_borrowed()
                    && param.ty.ownership() != OwnershipKind::Copy
            })
            .map(|param| param.name.clone())
            .collect();

        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        for name in &cloned_borrowed_param_names {
            self.borrowed_params.remove(name);
            self.mut_borrowed_params.remove(name);
        }

        let mut materialize_body = vec![RustStmt::Let {
            mutable: true,
            name: "_yields".to_string(),
            ty: Some(RustType::Vec(Box::new(yield_ty))),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                args: vec![],
            },
        }];
        for stmt in &func.body {
            materialize_body.extend(self.lower_stmt_strict_for_function(
                stmt,
                "generator materialization statement lowering",
            ));
        }
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        materialize_body.push(RustStmt::Assign {
            target: RustExpr::Ident("__sifr_generator_iter".to_string()),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("_yields".to_string())),
                method: "into_iter".to_string(),
                args: vec![],
            },
        });
        materialize_body.push(RustStmt::Assign {
            target: RustExpr::Ident("__sifr_generator_initialized".to_string()),
            value: RustExpr::Literal(RustLiteral::Bool(true)),
        });

        let closure_body = vec![
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("__sifr_generator_initialized".to_string())),
                },
                then_body: materialize_body,
                else_body: None,
            },
            RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_generator_iter".to_string())),
                method: "next".to_string(),
                args: vec![],
            })),
        ];

        let from_fn_expr = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "iter".to_string(),
                "from_fn".to_string(),
            ])),
            args: vec![RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: true,
                is_async: false,
            }],
        };

        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
            args: vec![from_fn_expr],
        })));
        body
    }

    pub(crate) fn lower_async_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        let yield_ty = if let Type::AsyncGenerator(elem, _) = func.return_type.resolve_alias() {
            self.rust_ir_type_with_generics(elem)
        } else {
            RustType::I64
        };

        let mut body = Self::emit_mutable_param_shadow_stmts(mutable_param_shadows);
        for param in &func.params {
            if mutable_param_shadows
                .iter()
                .any(|(name, _)| name == &param.name)
                || !param.convention.is_borrowed()
                || param.ty.ownership() == OwnershipKind::Copy
            {
                continue;
            }
            body.push(RustStmt::Let {
                mutable: false,
                name: param.name.clone(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(param.name.clone()))),
            });
        }

        let cloned_borrowed_param_names: Vec<String> = func
            .params
            .iter()
            .filter(|param| {
                !mutable_param_shadows
                    .iter()
                    .any(|(name, _)| name == &param.name)
                    && param.convention.is_borrowed()
                    && param.ty.ownership() != OwnershipKind::Copy
            })
            .map(|param| param.name.clone())
            .collect();

        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        for name in &cloned_borrowed_param_names {
            self.borrowed_params.remove(name);
            self.mut_borrowed_params.remove(name);
        }

        let mut materialize_body = Vec::new();
        materialize_body.push(RustStmt::Let {
            mutable: true,
            name: "_yields".to_string(),
            ty: Some(RustType::Vec(Box::new(yield_ty))),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                args: vec![],
            },
        });
        for stmt in &func.body {
            materialize_body.extend(self.lower_stmt_strict_for_function(
                stmt,
                "async generator lazy materialization statement lowering",
            ));
        }
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        materialize_body.push(RustStmt::Return(Some(RustExpr::Ident(
            "_yields".to_string(),
        ))));
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "AsyncGenerator".to_string(),
                "new_lazy".to_string(),
            ])),
            args: vec![RustExpr::ClosureBlock {
                params: vec![],
                body: materialize_body,
                is_move: true,
                is_async: false,
            }],
        })));
        body
    }

    pub(crate) fn emit_function(
        &mut self,
        func: &HirFunction,
        module_public: bool,
        test_mode: bool,
    ) {
        // In test mode, skip the main function
        if test_mode && func.name == "main" {
            return;
        }

        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_callable_var_conventions = self.callable_var_conventions.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_string_char_cache_vars = self.string_char_cache_vars.clone();
        let saved_hoistable_static_dict_locals = self.hoistable_static_dict_locals.clone();
        let saved_none_widened_local_bindings = self.none_widened_local_bindings.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_sifr_int_result_local_bindings =
            self.sifr_int_result_local_bindings.borrow().clone();
        let saved_sifr_int_function_returns = self.sifr_int_function_returns.borrow().clone();
        let saved_sifr_int_result_function_returns =
            self.sifr_int_result_function_returns.borrow().clone();
        let saved_current_sifr_int_return = self.current_sifr_int_return.get();
        let saved_current_sifr_int_result_return = self.current_sifr_int_result_return.get();

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();
        self.local_binding_types.clear();
        self.string_char_cache_vars.clear();
        self.hoistable_static_dict_locals = self.collect_hoistable_static_dict_locals(func);
        self.none_widened_local_bindings.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.sifr_int_result_local_bindings.borrow_mut().clear();
        self.current_sifr_int_return
            .set(self.function_returns_sifr_int(&func.name));
        self.current_sifr_int_result_return.set(
            self.sifr_int_result_function_returns
                .borrow()
                .contains(&func.name),
        );
        self.register_function_scope_params(&func.name, &func.params);
        let active_function_returns = self.function_sifr_int_returns_for_body(&func.body);
        *self.sifr_int_function_returns.borrow_mut() = active_function_returns;
        self.register_local_body_binding_types(&func.body);

        let visibility = if !test_mode && module_public && func.name != "main" {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let is_generator = body_contains_yield(&func.body);
        let is_async_generator =
            is_generator && matches!(func.return_type.resolve_alias(), Type::AsyncGenerator(_, _));
        if is_generator {
            self.generator_functions.insert(func.name.clone());
        }

        let reassigned_vars = collect_reassigned_vars(&func.body);
        let mutable_param_shadows =
            Self::lower_mutable_param_shadows(&func.params, &reassigned_vars);
        self.apply_mutable_param_shadowing(&mutable_param_shadows);

        let params = func
            .params
            .iter()
            .enumerate()
            .map(|(param_idx, param)| {
                let rust_ty = self.lower_module_function_param_type(&func.name, param_idx, param);
                if param.convention.is_owned() && param.convention.is_mutable() {
                    RustParam::NamedMut {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                } else {
                    RustParam::Named {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                }
            })
            .collect::<Vec<_>>();

        let mut lowered_body = if is_async_generator {
            self.lower_async_generator_function_body(func, &mutable_param_shadows)
        } else if is_generator {
            self.lower_generator_function_body(func, &mutable_param_shadows)
        } else {
            let mut lowered = Self::emit_mutable_param_shadow_stmts(&mutable_param_shadows);
            lowered.extend(self.prepare_string_char_cache_stmts(func, &reassigned_vars));
            for stmt in &func.body {
                lowered.extend(
                    self.lower_stmt_strict_for_function(stmt, "function body statement lowering"),
                );
            }
            lowered
        };

        if !is_generator
            && Self::returns_result_none(&func.return_type)
            && !matches!(
                func.body.last(),
                Some(HirStmt::Return { .. } | HirStmt::Raise { .. })
            )
        {
            lowered_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            })));
        }
        if lowered_body.is_empty() {
            if self
                .lower_function_return_type(func, is_generator)
                .is_none()
            {
                lowered_body.push(RustStmt::Return(None));
            } else {
                panic!(
                    "function IR lowering produced empty body for non-unit return: {}",
                    func.name
                );
            }
        }

        for decorator in &func.decorators {
            self.body_items
                .push(RustItem::Attr(format!("// @{decorator}")));
        }
        if test_mode && func.name.starts_with("test_") {
            self.body_items.push(RustItem::Attr("#[test]".to_string()));
        }

        self.body_items.push(RustItem::Fn {
            name: func.name.clone(),
            visibility,
            type_params: Self::lower_function_type_params(func),
            params,
            ret: self.lower_function_return_type(func, is_generator),
            body: lowered_body,
            is_async: func.is_async && !is_async_generator,
        });

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.callable_var_conventions = saved_callable_var_conventions;
        self.local_binding_types = saved_local_binding_types;
        self.string_char_cache_vars = saved_string_char_cache_vars;
        self.hoistable_static_dict_locals = saved_hoistable_static_dict_locals;
        self.none_widened_local_bindings = saved_none_widened_local_bindings;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        *self.sifr_int_result_local_bindings.borrow_mut() = saved_sifr_int_result_local_bindings;
        *self.sifr_int_function_returns.borrow_mut() = saved_sifr_int_function_returns;
        *self.sifr_int_result_function_returns.borrow_mut() =
            saved_sifr_int_result_function_returns;
        self.current_sifr_int_return
            .set(saved_current_sifr_int_return);
        self.current_sifr_int_result_return
            .set(saved_current_sifr_int_result_return);
    }
}
