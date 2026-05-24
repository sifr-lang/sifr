use super::{
    first_try_error_type_in_stmts, queries, select_try_error_type, HirExceptHandler, HirStmt,
    RustEmitter, RustExpr, RustStmt, Type,
};
impl RustEmitter {
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
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)? else {
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
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_iter) = lowered_iter else {
            return Ok(false);
        };
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)? else {
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
        let HirStmt::TryExcept { body, handlers, .. } = stmt else {
            return false;
        };
        let lowered = match self.try_lower_try_except_stmt_for_ir(body, handlers) {
            Ok(Some(lowered)) => lowered,
            Ok(None) => return false,
            Err(_) => return false,
        };
        for lowered_stmt in lowered {
            self.push_captured_stmt(&lowered_stmt);
        }
        true
    }

    pub(crate) fn try_lower_try_except_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        handlers: &[HirExceptHandler],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if handlers.is_empty() {
            return Ok(None);
        }
        let err_ty = select_try_error_type(handlers);
        let capture_returns =
            queries::body_contains_return(body) && self.current_return_type.is_some();
        let direct_return_capture = capture_returns
            && queries::block_control_flow_effect(body).always_exits()
            && handlers
                .iter()
                .all(|handler| queries::block_control_flow_effect(&handler.body).always_exits());
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

        let mut closure_body = {
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            let lowered = self.try_lower_stmt_block_for_ir(body)?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
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

        let mut lowered = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(crate::RustType::Result(
                Box::new(crate::RustType::Named(ok_ty.clone())),
                Box::new(crate::RustType::Named(err_ty.clone())),
            )),
            value: try_value,
        }];

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
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
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
        } else {
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
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
            .unwrap_or_else(|| "Error".to_string())
    }

    pub(crate) fn try_lower_try_finally_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        finalbody: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let err_ty = self.try_finally_error_type_name_for_ir(body, finalbody);
        let can_return_error = self.try_closure_depth > 0
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

        let mut closure_body = {
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            let lowered = self.try_lower_stmt_block_for_ir(body)?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
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
        let finalbody_result = self.try_lower_stmt_block_for_ir(finalbody);
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
