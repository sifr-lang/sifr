use super::{HirExpr, HirStmt, RustEmitter, RustStmt, Type, queries};
impl RustEmitter {
    pub(crate) fn try_lower_with_stmt_for_ir(
        &mut self,
        items: &[sifr_ir::HirWithItem],
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if items
            .iter()
            .any(|item| matches!(item.kind, sifr_ir::HirWithItemKind::Python { .. }))
        {
            return self.try_lower_python_context_with_for_ir(items, body);
        }
        let mut lowered_items = Vec::with_capacity(items.len());
        for item in items {
            let (has_cm, value) = match &item.kind {
                sifr_ir::HirWithItemKind::Native {
                    has_context_manager_protocol,
                } => (*has_context_manager_protocol, &item.context),
                sifr_ir::HirWithItemKind::Python { .. } => {
                    unreachable!("Python context items are delegated before native with lowering")
                }
            };
            let var = &item.target;
            let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                return Ok(None);
            };
            let binding = if queries::stmts_reference_var(body, var)
                || items
                    .iter()
                    .any(|other| other.target != *var && other.target.contains(var))
            {
                var.clone()
            } else {
                format!("_{var}")
            };
            let class_name = if has_cm {
                if !matches!(value.ty(), Type::Class { .. }) {
                    return Ok(None);
                }
                Some(self.render_rust_type_with_generics(value.ty()))
            } else {
                None
            };
            lowered_items.push(crate::RustWithItem {
                mutable: self.mutated_vars.contains(var),
                binding,
                value: lowered_value,
                has_cm,
                class_name,
            });
        }
        let Some(lowered_body) = self.try_lower_scoped_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::With {
            items: lowered_items,
            body: lowered_body,
        }))
    }

    pub(crate) fn try_lower_async_with_stmt_for_ir(
        &mut self,
        kind: &sifr_ir::HirAsyncWithKind,
        target: Option<&str>,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if let sifr_ir::HirAsyncWithKind::Python {
            context,
            manager_class,
            entered_type,
            enter_error_type,
            exit_error_type,
            entered_is_opaque_borrow,
            active_error_type,
            ..
        } = kind
        {
            return self.try_lower_python_async_context_for_ir(
                context,
                manager_class,
                entered_type,
                enter_error_type,
                exit_error_type,
                *entered_is_opaque_borrow,
                active_error_type,
                target,
                body,
            );
        }
        if let sifr_ir::HirAsyncWithKind::UserDefined {
            context,
            exit_error_ty,
            active_error_ty,
            ..
        } = kind
        {
            return self.try_lower_native_async_context_for_ir(
                context,
                exit_error_ty,
                active_error_ty,
                target,
                body,
            );
        }

        let timeout_duration = if let sifr_ir::HirAsyncWithKind::TaskTimeout { duration } = kind {
            let Some(duration) =
                crate::try_lower_task_duration_expr(duration, "__sifr_task_timeout_seconds")
            else {
                return Ok(None);
            };
            Some(duration)
        } else {
            None
        };
        if let Some(duration) = timeout_duration.clone() {
            self.active_timeout_durations.push(duration);
        }
        let lowered_body_result = self.try_lower_scoped_stmt_block_for_ir(body);
        if timeout_duration.is_some() {
            let _ = self.active_timeout_durations.pop();
        }
        let Some(mut lowered_body) = lowered_body_result? else {
            return Ok(None);
        };
        if let Some(target) = target {
            let (constructor, constructor_args) = match kind {
                sifr_ir::HirAsyncWithKind::TaskGroup {
                    context: Some(context),
                } => {
                    let Some(context) = crate::try_lower_leaf_or_name_expr_result(context)? else {
                        return Ok(None);
                    };
                    ("new_task_group_with_context", vec![context])
                }
                sifr_ir::HirAsyncWithKind::TaskGroup { context: None } => {
                    ("new_task_group", vec![])
                }
                _ => ("new", vec![]),
            };
            lowered_body.insert(
                0,
                crate::RustStmt::Let {
                    mutable: true,
                    name: target.to_string(),
                    ty: None,
                    value: crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "__SifrTaskScope".to_string(),
                            constructor.to_string(),
                        ])),
                        args: constructor_args,
                    },
                },
            );
        }
        if let (true, Some(target)) = (
            matches!(
                kind,
                sifr_ir::HirAsyncWithKind::TaskScope | sifr_ir::HirAsyncWithKind::TaskGroup { .. }
            ),
            target,
        ) {
            let propagates_scope_failure = self.current_return_type.as_ref().is_some_and(|ty| {
                matches!(ty.resolve_alias(), Type::Result(_, err) if matches!(err.resolve_alias(), Type::Class { name, .. } if name == "ScopeFailure" || name == "Error"))
            });
            let join_expr = format!("{target}.__sifr_join_all().await");
            let stmt = if propagates_scope_failure {
                format!(
                    "if let Err(__sifr_scope_failure) = {join_expr} {{ return Err(__sifr_scope_failure.into()); }}"
                )
            } else {
                format!("let _ = {join_expr};")
            };
            lowered_body.push(crate::RustStmt::Verbatim(stmt));
        }
        Ok(Some(RustStmt::Block(lowered_body)))
    }

    pub(crate) fn try_lower_async_for_stmt_for_ir(
        &mut self,
        target: &str,
        iter: &HirExpr,
        iter_error_ty: &Type,
        close_error_ty: Option<&Type>,
        active_error_ty: &Type,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if let Some(close_error_ty) = close_error_ty {
            return self.try_lower_closable_native_async_for_for_ir(
                target,
                iter,
                iter_error_ty,
                close_error_ty,
                active_error_ty,
                body,
            );
        }
        let Some(lowered_body) = self.try_lower_scoped_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        let target_pattern = if queries::stmts_reference_var(body, target) {
            target.to_string()
        } else {
            format!("_{target}")
        };
        let infallible_iter = matches!(iter_error_ty.resolve_alias(), Type::Never);
        let loop_body = |receiver: crate::RustExpr| {
            let lowered_body = lowered_body.clone();
            let next_call = crate::RustExpr::Await(Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "anext".to_string(),
                args: vec![],
            }));
            let next_value = if infallible_iter {
                next_call
            } else {
                crate::RustExpr::Try(Box::new(next_call))
            };
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_async_next".to_string(),
                    ty: None,
                    value: next_value,
                },
                RustStmt::Match {
                    expr: crate::RustExpr::Ident("__sifr_async_next".to_string()),
                    arms: vec![
                        crate::RustMatchArm {
                            pattern: format!("Some({target_pattern})"),
                            bindings: vec![target.to_string()],
                            guard: None,
                            body: lowered_body.clone(),
                        },
                        crate::RustMatchArm {
                            pattern: "None".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Break],
                        },
                    ],
                },
            ]
        };

        if let HirExpr::Name { name, .. } = iter {
            return Ok(Some(RustStmt::Loop {
                body: loop_body(crate::RustExpr::Ident(name.clone())),
            }));
        }

        let Some(lowered_iter) = self.lower_stmt_expr_for_ir(iter)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: true,
                name: "__sifr_async_iter".to_string(),
                ty: None,
                value: lowered_iter,
            },
            RustStmt::Loop {
                body: loop_body(crate::RustExpr::Ident("__sifr_async_iter".to_string())),
            },
        ])))
    }
}
