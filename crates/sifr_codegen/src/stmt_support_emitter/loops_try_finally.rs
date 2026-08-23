use super::{
    HirExceptHandler, HirStmt, RustEmitter, RustExpr, RustStmt, Type,
    declaration_only_try_bindings, first_try_error_type_in_stmts, queries, select_try_error_type,
    successful_try_bindings,
};
impl RustEmitter {
    pub(crate) fn lower_loop_control_stmt_for_ir(&self, stmt: &HirStmt) -> Option<RustStmt> {
        match stmt {
            HirStmt::Break if self.loop_else_stack.last().copied().unwrap_or(false) => {
                Some(RustStmt::Block(vec![
                    RustStmt::Assign {
                        target: RustExpr::Ident("_broke".to_string()),
                        value: RustExpr::Literal(crate::RustLiteral::Bool(true)),
                    },
                    RustStmt::Break,
                ]))
            }
            HirStmt::Break => Some(RustStmt::Break),
            HirStmt::Continue => Some(RustStmt::Continue),
            _ => None,
        }
    }

    pub(crate) fn try_lower_structured_while_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::While {
            condition,
            body,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };
        let has_else = else_body.is_some();
        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(false);
        };
        self.loop_else_stack.push(has_else);
        let lowered_body = self.try_lower_scoped_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_scoped_stmt_block_for_ir(else_body)?
            else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: true,
                    name: "_broke".to_string(),
                    ty: Some(crate::RustType::Bool),
                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                },
                RustStmt::While {
                    cond: lowered_cond,
                    body: lowered_body,
                },
                RustStmt::If {
                    cond: crate::RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident("_broke".to_string()),
                        ))),
                    },
                    then_body: lowered_else_body,
                    else_body: None,
                },
            ]));
            return Ok(true);
        }

        self.push_captured_stmt(&RustStmt::While {
            cond: lowered_cond,
            body: lowered_body,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_for_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::For {
            target,
            target_ty,
            iter,
            body,
            else_body,
            ..
        } = stmt
        else {
            return Ok(false);
        };
        let has_else = else_body.is_some();
        let var = if target.contains(',') {
            let names = target
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            if names.is_empty() {
                return Ok(false);
            }
            format!("({})", names.join(", "))
        } else {
            target.clone()
        };

        self.loop_else_stack.push(has_else);
        let lowered_iter = self.try_lower_for_iter_expr_for_ir(iter, target_ty)?;
        let target_cache_init = if target.contains(',') {
            None
        } else {
            self.string_char_cache_init_stmt_for_loop_target(target, target_ty)
        };
        let lowered_body = self.try_lower_scoped_stmt_block_for_ir(body);
        let lowered_body = lowered_body?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_iter) = lowered_iter else {
            return Ok(false);
        };
        let Some(mut lowered_body) = lowered_body else {
            return Ok(false);
        };
        if let Some(cache_stmt) = target_cache_init {
            lowered_body.insert(0, cache_stmt);
        }

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_scoped_stmt_block_for_ir(else_body)?
            else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: true,
                    name: "_broke".to_string(),
                    ty: Some(crate::RustType::Bool),
                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                },
                RustStmt::For {
                    var,
                    iter: lowered_iter,
                    body: lowered_body,
                },
                RustStmt::If {
                    cond: crate::RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident("_broke".to_string()),
                        ))),
                    },
                    then_body: lowered_else_body,
                    else_body: None,
                },
            ]));
            return Ok(true);
        }

        self.push_captured_stmt(&RustStmt::For {
            var,
            iter: lowered_iter,
            body: lowered_body,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_with_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::With { items, body } = stmt else {
            return Ok(false);
        };
        let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
            return Ok(false);
        };
        self.push_captured_stmt(&lowered_with);
        Ok(true)
    }

    pub(crate) fn try_lower_structured_try_except_stmt(&mut self, stmt: &HirStmt) -> bool {
        self.try_lower_structured_try_except_stmt_with_following(stmt, None)
    }

    pub(crate) fn try_lower_structured_try_except_stmt_with_following(
        &mut self,
        stmt: &HirStmt,
        following_stmts: Option<&[HirStmt]>,
    ) -> bool {
        let lowered =
            match self.try_lower_try_except_hir_stmt_for_ir_with_following(stmt, following_stmts) {
                Ok(Some(lowered)) => lowered,
                Ok(None) | Err(_) => return false,
            };
        for lowered_stmt in lowered {
            self.push_captured_stmt(&lowered_stmt);
        }
        true
    }

    pub(crate) fn try_lower_try_except_hir_stmt_for_ir(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        self.try_lower_try_except_hir_stmt_for_ir_with_following(stmt, None)
    }

    pub(crate) fn try_lower_try_except_hir_stmt_for_ir_with_following(
        &mut self,
        stmt: &HirStmt,
        following_stmts: Option<&[HirStmt]>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } = stmt
        else {
            return Ok(None);
        };
        self.try_lower_try_except_stmt_for_ir(body, handlers, body_error_types, following_stmts)
    }

    pub(crate) fn try_lower_try_except_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        handlers: &[HirExceptHandler],
        body_error_types: &[Type],
        following_stmts: Option<&[HirStmt]>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if handlers.is_empty() {
            return Ok(None);
        }
        let error_carrier = crate::try_error_carrier::try_error_carrier(body_error_types, handlers)
            .or_else(|| {
                handlers
                    .iter()
                    .find_map(|handler| handler.error_resolved_type.clone())
            });
        let err_ty = error_carrier.as_ref().map_or_else(
            || select_try_error_type(handlers),
            |carrier| crate::render_type(&crate::sifr_type_to_rust_type(carrier)),
        );
        // Handler bodies are emitted in the enclosing function's match arm, so
        // only returns from the closure-backed try body need a carrier.
        let capture_returns =
            self.current_return_type.is_some() && queries::body_contains_return(body);
        let body_always_raises = matches!(
            queries::block_control_flow_effect(body),
            sifr_ir::HirControlFlowEffect::AlwaysRaises
        );
        let direct_return_capture = capture_returns
            && queries::block_control_flow_effect(body).always_exits()
            && handlers
                .iter()
                .all(|handler| queries::block_control_flow_effect(&handler.body).always_exits());
        let declaration_only_bindings =
            declaration_only_try_bindings(body, handlers, following_stmts)
                .into_iter()
                .map(|(name, ty)| {
                    let ty = self.local_binding_types.get(&name).cloned().unwrap_or(ty);
                    (name, ty)
                })
                .collect::<Vec<_>>();
        let successful_bindings = successful_try_bindings(body, handlers, following_stmts)
            .into_iter()
            .map(|(name, ty)| {
                let ty = self.local_binding_types.get(&name).cloned().unwrap_or(ty);
                (name, ty)
            })
            .collect::<Vec<_>>();
        let binding_tuple_ty = crate::RustType::Tuple(
            successful_bindings
                .iter()
                .map(|(_, ty)| crate::sifr_type_to_rust_type(ty))
                .collect(),
        );
        let binding_pattern = format!(
            "({})",
            successful_bindings
                .iter()
                .map(|(name, ty)| {
                    if self.mutated_vars.contains(name)
                        || super::should_force_mutable_binding(ty, &self.recursive_fields)
                    {
                        format!("mut {name},")
                    } else {
                        format!("{name},")
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        let ok_ty = if capture_returns {
            if let Some(return_ty) = self.current_return_type.as_ref() {
                if direct_return_capture {
                    crate::sifr_type_to_rust_type(return_ty)
                } else {
                    crate::RustType::Option(Box::new(crate::sifr_type_to_rust_type(return_ty)))
                }
            } else {
                crate::RustType::Unit
            }
        } else if !successful_bindings.is_empty() {
            binding_tuple_ty.clone()
        } else {
            crate::RustType::Unit
        };

        let mut closure_body = {
            let saved_cache_requirements = self.string_char_cache_required_names.clone();
            let body_cache_uses = crate::string_char_cache_scan::string_cache_uses_in_stmts(body);
            for (name, _) in successful_bindings.iter().chain(&declaration_only_bindings) {
                if !body_cache_uses.contains(name) {
                    self.string_char_cache_required_names.remove(name);
                }
            }
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            self.try_closure_error_type_info.push(error_carrier.clone());
            let lowered_result = self.try_lower_scoped_stmt_block_for_ir(body);
            self.string_char_cache_required_names = saved_cache_requirements;
            let lowered = lowered_result?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
            self.try_closure_error_type_info.pop();
            let Some(lowered) = lowered else {
                return Ok(None);
            };
            lowered
        };

        if !successful_bindings.is_empty() && capture_returns {
            closure_body.push(RustStmt::Assign {
                target: RustExpr::Ident("__sifr_successful_try_bindings".to_string()),
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident("Some".to_string())),
                    args: vec![RustExpr::Tuple(
                        successful_bindings
                            .iter()
                            .map(|(name, _)| RustExpr::Ident(name.clone()))
                            .collect(),
                    )],
                },
            });
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::None)],
            })));
        } else if !successful_bindings.is_empty() {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Tuple(
                    successful_bindings
                        .iter()
                        .map(|(name, _)| RustExpr::Ident(name.clone()))
                        .collect(),
                )],
            })));
        } else if body_always_raises {
            closure_body.push(RustStmt::Expr(RustExpr::FormatMacro {
                name: "unreachable".to_string(),
                format_str: "sifr try/except raising body fell through".to_string(),
                args: vec![],
            }));
        } else if !capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else if direct_return_capture {
            closure_body.push(RustStmt::Expr(RustExpr::FormatMacro {
                name: "unreachable".to_string(),
                format_str: "sifr try/except return capture fell through".to_string(),
                args: vec![],
            }));
        } else {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::None)],
            })));
        }

        let closure_is_async = Self::rust_stmts_contain_await(&closure_body);
        let try_call = RustExpr::FnCall {
            func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: false,
                is_async: closure_is_async,
            }))),
            args: vec![],
        };
        let try_value = if closure_is_async {
            RustExpr::Await(Box::new(try_call))
        } else {
            try_call
        };

        let mut lowered = Vec::new();
        lowered.extend(
            declaration_only_bindings
                .iter()
                .map(|(name, ty)| RustStmt::LetDecl {
                    mutable: queries::declaration_only_binding_needs_mutability(
                        following_stmts.unwrap_or_default(),
                        name,
                    ) || super::should_force_mutable_binding(ty, &self.recursive_fields),
                    name: name.clone(),
                    ty: crate::sifr_type_to_rust_type(ty),
                }),
        );
        if capture_returns && !successful_bindings.is_empty() {
            lowered.push(RustStmt::Let {
                mutable: true,
                name: "__sifr_successful_try_bindings".to_string(),
                ty: Some(crate::RustType::Option(Box::new(binding_tuple_ty))),
                value: RustExpr::Literal(crate::RustLiteral::None),
            });
        }
        lowered.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(crate::RustType::Result(
                Box::new(ok_ty),
                Box::new(crate::RustType::Named(err_ty.clone())),
            )),
            value: try_value,
        });

        if !successful_bindings.is_empty() && !capture_returns {
            let Some(handler_chain) = self.lower_try_except_handler_chain_for_ir(
                handlers,
                "__sifr_try_err",
                error_carrier.as_ref(),
                &err_ty,
            )?
            else {
                return Ok(None);
            };
            let value_name = "__sifr_try_bindings";
            let match_value = RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_try_res".to_string())),
                arms: vec![
                    crate::RustMatchArm {
                        pattern: format!("Ok({value_name})"),
                        bindings: vec![value_name.to_string()],
                        guard: None,
                        body: vec![RustStmt::TailExpr(RustExpr::Ident(value_name.to_string()))],
                    },
                    crate::RustMatchArm {
                        pattern: "Err(__sifr_try_err)".to_string(),
                        bindings: vec!["__sifr_try_err".to_string()],
                        guard: None,
                        body: handler_chain,
                    },
                ],
            };
            lowered.push(RustStmt::LetPattern {
                pattern: binding_pattern.clone(),
                value: match_value,
            });
        } else if capture_returns {
            let mut arms = vec![crate::RustMatchArm {
                pattern: if direct_return_capture {
                    "Ok(__sifr_ret_val)".to_string()
                } else {
                    "Ok(Some(__sifr_ret_val))".to_string()
                },
                bindings: vec!["__sifr_ret_val".to_string()],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Ident(
                    "__sifr_ret_val".to_string(),
                )))],
            }];
            if !direct_return_capture {
                arms.push(crate::RustMatchArm {
                    pattern: "Ok(None)".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![],
                });
            }
            let Some(handler_chain) = self.lower_try_except_handler_chain_for_ir(
                handlers,
                "__sifr_try_err",
                error_carrier.as_ref(),
                &err_ty,
            )?
            else {
                return Ok(None);
            };
            arms.push(crate::RustMatchArm {
                pattern: "Err(__sifr_try_err)".to_string(),
                bindings: vec!["__sifr_try_err".to_string()],
                guard: None,
                body: handler_chain,
            });
            lowered.push(RustStmt::Match {
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                arms,
            });
            if !successful_bindings.is_empty() {
                lowered.push(RustStmt::LetElse {
                    pattern: format!("Some({binding_pattern})"),
                    value: RustExpr::Ident("__sifr_successful_try_bindings".to_string()),
                    else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                        name: "unreachable".to_string(),
                        args: vec![RustExpr::Literal(crate::RustLiteral::Str(
                            "successful try fallthrough must initialize promoted bindings"
                                .to_string(),
                        ))],
                    })],
                });
            }
        } else if body_always_raises {
            let Some(handler_chain) = self.lower_try_except_handler_chain_for_ir(
                handlers,
                "__sifr_try_err",
                error_carrier.as_ref(),
                &err_ty,
            )?
            else {
                return Ok(None);
            };
            lowered.push(RustStmt::Match {
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                arms: vec![
                    crate::RustMatchArm {
                        pattern: "Ok(())".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Expr(RustExpr::FormatMacro {
                            name: "unreachable".to_string(),
                            format_str: "sifr try/except raising body returned success".to_string(),
                            args: vec![],
                        })],
                    },
                    crate::RustMatchArm {
                        pattern: "Err(__sifr_try_err)".to_string(),
                        bindings: vec!["__sifr_try_err".to_string()],
                        guard: None,
                        body: handler_chain,
                    },
                ],
            });
        } else {
            let Some(handler_chain) = self.lower_try_except_handler_chain_for_ir(
                handlers,
                "__sifr_try_err",
                error_carrier.as_ref(),
                &err_ty,
            )?
            else {
                return Ok(None);
            };
            lowered.push(RustStmt::IfLet {
                pattern: "Err(__sifr_try_err)".to_string(),
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                then_body: handler_chain,
                else_body: None,
            });
        }
        for (name, ty) in &successful_bindings {
            if let Some(cache_stmt) = self.string_char_cache_init_stmt_for_local(name, ty) {
                lowered.push(cache_stmt);
            }
        }
        Ok(Some(lowered))
    }

    pub(crate) fn current_result_error_type_name_for_ir(&self) -> String {
        self.try_closure_error_type
            .last()
            .cloned()
            .or_else(|| {
                let Type::Result(_, err_ty) = self.current_return_type.as_ref()? else {
                    return None;
                };
                Some(crate::render_type(&crate::sifr_type_to_rust_type(err_ty)))
            })
            .unwrap_or_else(|| "Error".to_string())
    }

    pub(crate) fn timeout_error_for_ir(&self) -> RustExpr {
        let source_type = crate::try_error_carrier::timeout_error_type();
        let source = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "TimeoutError".to_string(),
                "new".to_string(),
            ])),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                    "task timeout expired".to_string(),
                ))),
                method: "to_string".to_string(),
                args: Vec::new(),
            }],
        };
        let target = self
            .try_closure_error_type_info
            .last()
            .and_then(Option::as_ref)
            .or_else(|| {
                let Type::Result(_, error) = self.current_return_type.as_ref()?.resolve_alias()
                else {
                    return None;
                };
                Some(error.as_ref())
            });
        let Some(target) = target else {
            return source;
        };
        let converted =
            self.consuming_value_conversion_for_ir(target, &source_type, source.clone());
        if converted != source {
            return converted;
        }
        if crate::sifr_type_to_rust_type(target) == crate::sifr_type_to_rust_type(&source_type) {
            source
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(source),
                method: "into".to_string(),
                args: Vec::new(),
            }
        }
    }

    pub(crate) fn try_finally_error_type_name_for_ir(
        &self,
        body: &[HirStmt],
        finalbody: &[HirStmt],
    ) -> String {
        self.try_closure_error_type
            .last()
            .cloned()
            .or_else(|| {
                let Type::Result(_, err_ty) = self.current_return_type.as_ref()? else {
                    return None;
                };
                Some(crate::render_type(&crate::sifr_type_to_rust_type(err_ty)))
            })
            .or_else(|| {
                first_try_error_type_in_stmts(body)
                    .or_else(|| first_try_error_type_in_stmts(finalbody))
            })
            .unwrap_or_else(|| "()".to_string())
    }

    pub(crate) fn try_lower_try_finally_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        finalbody: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let err_ty = self.try_finally_error_type_name_for_ir(body, finalbody);
        let can_return_error = !self.try_closure_error_type.is_empty()
            || self.current_return_type.as_ref().is_some_and(|return_ty| {
                matches!(
                    crate::resolve_alias_type_for_plain_call(return_ty),
                    Type::Result(_, _)
                )
            });
        let capture_returns =
            queries::body_contains_return(body) && self.current_return_type.is_some();
        let direct_return_capture =
            capture_returns && queries::block_control_flow_effect(body).always_exits();
        let ok_ty = if capture_returns {
            if let Some(return_ty) = self.current_return_type.as_ref() {
                if direct_return_capture {
                    crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                } else {
                    format!(
                        "Option<{}>",
                        crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                    )
                }
            } else {
                "()".to_string()
            }
        } else {
            "()".to_string()
        };

        let active_error_type = self
            .try_closure_error_type_info
            .last()
            .cloned()
            .flatten()
            .or_else(|| {
                let Type::Result(_, error_type) =
                    self.current_return_type.as_ref()?.resolve_alias()
                else {
                    return None;
                };
                Some(error_type.as_ref().clone())
            });
        let mut closure_body = {
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            self.try_closure_error_type_info.push(active_error_type);
            let lowered = self.try_lower_scoped_stmt_block_for_ir(body)?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
            self.try_closure_error_type_info.pop();
            let Some(lowered) = lowered else {
                return Ok(None);
            };
            lowered
        };

        if !capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else if direct_return_capture {
            closure_body.push(RustStmt::Expr(RustExpr::FormatMacro {
                name: "unreachable".to_string(),
                format_str: "sifr try/finally return capture fell through".to_string(),
                args: vec![],
            }));
        } else {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::None)],
            })));
        }

        let closure_is_async = Self::rust_stmts_contain_await(&closure_body);
        let try_call = RustExpr::FnCall {
            func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: false,
                is_async: closure_is_async,
            }))),
            args: vec![],
        };
        let try_value = if closure_is_async {
            RustExpr::Await(Box::new(try_call))
        } else {
            try_call
        };

        let mut lowered = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_try_finally_res".to_string(),
            ty: Some(crate::RustType::Result(
                Box::new(crate::RustType::Named(ok_ty.clone())),
                Box::new(crate::RustType::Named(err_ty)),
            )),
            value: try_value,
        }];

        let saved_timeout_durations = std::mem::take(&mut self.active_timeout_durations);
        let finalbody_result = self.try_lower_scoped_stmt_block_for_ir(finalbody);
        self.active_timeout_durations = saved_timeout_durations;
        let Some(finalbody_lowered) = finalbody_result? else {
            return Ok(None);
        };
        lowered.extend(finalbody_lowered);

        if capture_returns {
            let mut arms = vec![crate::RustMatchArm {
                pattern: if direct_return_capture {
                    "Ok(__sifr_ret_val)".to_string()
                } else {
                    "Ok(Some(__sifr_ret_val))".to_string()
                },
                bindings: vec!["__sifr_ret_val".to_string()],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Ident(
                    "__sifr_ret_val".to_string(),
                )))],
            }];
            if !direct_return_capture {
                arms.push(crate::RustMatchArm {
                    pattern: "Ok(None)".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![],
                });
            }
            arms.push(crate::RustMatchArm {
                pattern: "Err(__sifr_finally_err)".to_string(),
                bindings: vec!["__sifr_finally_err".to_string()],
                guard: None,
                body: if can_return_error {
                    vec![RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Ident("Err".to_string())),
                        args: vec![RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_finally_err".to_string())),
                            method: "into".to_string(),
                            args: vec![],
                        }],
                    }))]
                } else {
                    vec![RustStmt::Expr(RustExpr::FormatMacro {
                        name: "unreachable".to_string(),
                        format_str: "sifr try/finally error propagation in non-Result function"
                            .to_string(),
                        args: vec![],
                    })]
                },
            });
            lowered.push(RustStmt::Match {
                expr: RustExpr::Ident("__sifr_try_finally_res".to_string()),
                arms,
            });
        } else {
            lowered.push(RustStmt::IfLet {
                pattern: "Err(__sifr_finally_err)".to_string(),
                expr: RustExpr::Ident("__sifr_try_finally_res".to_string()),
                then_body: if can_return_error {
                    vec![RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Ident("Err".to_string())),
                        args: vec![RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_finally_err".to_string())),
                            method: "into".to_string(),
                            args: vec![],
                        }],
                    }))]
                } else {
                    vec![RustStmt::Expr(RustExpr::FormatMacro {
                        name: "unreachable".to_string(),
                        format_str: "sifr try/finally error propagation in non-Result function"
                            .to_string(),
                        args: vec![],
                    })]
                },
                else_body: None,
            });
        }
        Ok(Some(lowered))
    }
}
