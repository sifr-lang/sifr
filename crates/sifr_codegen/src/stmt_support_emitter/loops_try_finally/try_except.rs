use super::{
    HirExceptHandler, HirStmt, RustEmitter, RustExpr, RustStmt, Type,
    declaration_only_try_bindings, queries, select_try_error_type, successful_try_bindings,
};

impl RustEmitter {
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
        let infallible_success =
            body_always_raises && !capture_returns && successful_bindings.is_empty();
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
                        || crate::stmt_support_emitter::should_force_mutable_binding(
                            ty,
                            &self.recursive_fields,
                        )
                    {
                        format!("mut {name},")
                    } else {
                        format!("{name},")
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        let return_wrap = if !successful_bindings.is_empty() && capture_returns {
            crate::TryClosureReturnWrap::ControlFlow {
                continue_type: crate::render_type(&binding_tuple_ty),
            }
        } else if direct_return_capture {
            crate::TryClosureReturnWrap::Direct
        } else {
            crate::TryClosureReturnWrap::Optional
        };
        let ok_ty = if infallible_success {
            crate::RustType::Named("std::convert::Infallible".to_string())
        } else if capture_returns {
            if let Some(return_ty) = self.current_return_type.as_ref() {
                match &return_wrap {
                    crate::TryClosureReturnWrap::Direct => crate::sifr_type_to_rust_type(return_ty),
                    crate::TryClosureReturnWrap::Optional => {
                        crate::RustType::Option(Box::new(crate::sifr_type_to_rust_type(return_ty)))
                    }
                    crate::TryClosureReturnWrap::ControlFlow { continue_type } => {
                        crate::RustType::Named(format!(
                            "std::ops::ControlFlow<{}, {}>",
                            crate::render_type(&crate::sifr_type_to_rust_type(return_ty)),
                            continue_type
                        ))
                    }
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
                self.try_closure_return_wrap.push(return_wrap.clone());
            }
            self.try_closure_error_type.push(err_ty.clone());
            self.try_closure_error_type_info.push(error_carrier.clone());
            let lowered_result = self.try_lower_scoped_stmt_block_for_ir(body);
            self.string_char_cache_required_names = saved_cache_requirements;
            let lowered = lowered_result?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_return_wrap.pop();
            }
            self.try_closure_error_type.pop();
            self.try_closure_error_type_info.pop();
            let Some(lowered) = lowered else {
                return Ok(None);
            };
            lowered
        };

        if !successful_bindings.is_empty() && capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "ops".to_string(),
                        "ControlFlow".to_string(),
                        "Continue".to_string(),
                    ])),
                    args: vec![RustExpr::Tuple(
                        successful_bindings
                            .iter()
                            .map(|(name, _)| RustExpr::Ident(name.clone()))
                            .collect(),
                    )],
                }],
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
        } else if !body_always_raises && !capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else if !body_always_raises && !direct_return_capture {
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
                    ) || crate::stmt_support_emitter::should_force_mutable_binding(
                        ty,
                        &self.recursive_fields,
                    ),
                    name: name.clone(),
                    ty: crate::sifr_type_to_rust_type(ty),
                }),
        );
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
        } else if capture_returns
            && matches!(return_wrap, crate::TryClosureReturnWrap::ControlFlow { .. })
        {
            let Some(handler_chain) = self.lower_try_except_handler_chain_for_ir(
                handlers,
                "__sifr_try_err",
                error_carrier.as_ref(),
                &err_ty,
            )?
            else {
                return Ok(None);
            };
            let continued_bindings = "__sifr_try_bindings";
            let match_value = RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_try_res".to_string())),
                arms: vec![
                    crate::RustMatchArm {
                        pattern: "Ok(std::ops::ControlFlow::Break(__sifr_ret_val))".to_string(),
                        bindings: vec!["__sifr_ret_val".to_string()],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::Ident(
                            "__sifr_ret_val".to_string(),
                        )))],
                    },
                    crate::RustMatchArm {
                        pattern: format!(
                            "Ok(std::ops::ControlFlow::Continue({continued_bindings}))"
                        ),
                        bindings: vec![continued_bindings.to_string()],
                        guard: None,
                        body: vec![RustStmt::TailExpr(RustExpr::Ident(
                            continued_bindings.to_string(),
                        ))],
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
        } else if infallible_success {
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
                        pattern: "Ok(__sifr_infallible)".to_string(),
                        bindings: vec!["__sifr_infallible".to_string()],
                        guard: None,
                        body: vec![RustStmt::Expr(RustExpr::Verbatim(
                            "match __sifr_infallible {}".to_string(),
                        ))],
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
}
