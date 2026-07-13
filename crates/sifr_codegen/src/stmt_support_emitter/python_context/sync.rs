use crate::hir_analysis::queries;
use crate::python_interop_direct::{mapped_try, output_value_expr, runtime_call};
use crate::{HirStmt, RustEmitter, RustExpr, RustStmt, Type};
use crate::{RustLiteral, RustMatchArm, RustType, RustWithItem};
use sifr_ir::{HirWithItem, HirWithItemKind};

impl RustEmitter {
    pub(crate) fn try_lower_python_context_with_for_ir(
        &mut self,
        items: &[HirWithItem],
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        let wrapped = self.wrap_context_items(items, body, lowered_body)?;
        Ok(Some(RustStmt::Block(wrapped)))
    }

    fn wrap_context_items(
        &mut self,
        items: &[HirWithItem],
        source_body: &[HirStmt],
        body: Vec<RustStmt>,
    ) -> Result<Vec<RustStmt>, crate::CodegenError> {
        let Some((item, remaining)) = items.split_first() else {
            return Ok(body);
        };
        let inner = self.wrap_context_items(remaining, source_body, body)?;
        match &item.kind {
            HirWithItemKind::Native {
                has_context_manager_protocol,
            } => {
                let Some(value) = self.lower_rendered_expr_for_ir(&item.context)? else {
                    return Err(crate::CodegenError::new(
                        "native item in mixed Python context could not be lowered",
                    ));
                };
                let binding = if queries::stmts_reference_var(source_body, &item.target) {
                    item.target.clone()
                } else {
                    format!("_{}", item.target)
                };
                let class_name = if *has_context_manager_protocol {
                    if !matches!(item.context.ty(), Type::Class { .. }) {
                        return Err(crate::CodegenError::new(
                            "native context-manager item does not have a class type",
                        ));
                    }
                    Some(self.rust_type_with_generics(item.context.ty()))
                } else {
                    None
                };
                Ok(vec![RustStmt::With {
                    items: vec![RustWithItem {
                        mutable: self.mutated_vars.contains(&item.target),
                        binding,
                        value,
                        has_cm: *has_context_manager_protocol,
                        class_name,
                    }],
                    body: inner,
                }])
            }
            HirWithItemKind::Python {
                entered_type,
                enter_error_type,
                exit_error_type,
                ..
            } => self.wrap_python_context_item(
                item,
                entered_type,
                enter_error_type,
                exit_error_type,
                inner,
            ),
        }
    }

    fn wrap_python_context_item(
        &mut self,
        item: &HirWithItem,
        entered_type: &Type,
        enter_error_type: &Type,
        exit_error_type: &Type,
        body: Vec<RustStmt>,
    ) -> Result<Vec<RustStmt>, crate::CodegenError> {
        let Some(active_error_type) = self.try_closure_error_type.last().cloned() else {
            return Err(crate::CodegenError::new(
                "Python context code generation requires an enclosing try error type",
            ));
        };
        let active_error_type_info = self
            .try_closure_error_type_info
            .last()
            .and_then(Option::as_ref)
            .cloned();
        let active_cause_kind =
            classify_cause_kind(active_error_type_info.as_ref(), &active_error_type);
        let active_is_python_error = matches!(
            active_error_type_info.as_ref().map(Type::resolve_alias),
            Some(Type::Class { name, .. }) if name == "PythonError"
        ) || (active_error_type_info.is_none()
            && active_error_type == "PythonError");
        let Some(manager_value) = self.lower_rendered_expr_for_ir(&item.context)? else {
            return Err(crate::CodegenError::new(
                "Python context manager expression could not be lowered",
            ));
        };

        let suffix = self.python_context_counter;
        self.python_context_counter += 1;
        let manager = format!("__sifr_python_context_manager_{suffix}");
        let entered_raw = format!("__sifr_python_context_entered_{suffix}");
        let conversion = format!("__sifr_python_context_conversion_{suffix}");
        let entered_slot = format!("__sifr_python_context_entered_slot_{suffix}");
        let outcome = format!("__sifr_python_context_outcome_{suffix}");
        let error = format!("__sifr_python_context_error_{suffix}");
        let cleanup = format!("__sifr_python_context_cleanup_{suffix}");

        let manager_handle = || RustExpr::Field {
            expr: Box::new(RustExpr::Ident(manager.clone())),
            field: "__sifr_python_object".to_string(),
        };
        let entered = output_value_expr(
            &entered_raw,
            entered_type,
            enter_error_type,
            &self.python_opaque_classes,
        )
        .ok_or_else(|| {
            crate::CodegenError::new("Python context entered value cannot be converted")
        })?;
        let entered_rust_type = crate::sifr_type_to_rust_type(entered_type);
        let conversion_type = RustType::Result(
            Box::new(entered_rust_type.clone()),
            Box::new(crate::sifr_type_to_rust_type(enter_error_type)),
        );

        let conversion_call = RustExpr::FnCall {
            func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                params: vec![],
                body: vec![RustStmt::Return(Some(call("Ok", entered)))],
                is_move: false,
                is_async: false,
            }))),
            args: vec![],
        };

        let conversion_error_body = Self::non_python_error_exit_body(
            manager_handle(),
            &error,
            &cleanup,
            &active_error_type,
            "OrdinaryError",
        );

        let mut closure_body = rewrite_context_control_flow(body, 0);
        closure_body.push(RustStmt::Return(Some(call(
            "Ok",
            call("Ok", RustExpr::Literal(RustLiteral::None)),
        ))));
        let return_expression_type = self.context_return_expression_type(&active_error_type);
        let outcome_type = RustType::Named(format!(
            "Result<Result<Option<{return_expression_type}>, bool>, {active_error_type}>"
        ));
        let outcome_call = RustExpr::FnCall {
            func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: false,
                is_async: false,
            }))),
            args: vec![],
        };

        let normal_exit = || {
            RustStmt::Expr(mapped_try(
                runtime_call("context_exit_normal", vec![manager_handle()]),
                exit_error_type,
            ))
        };
        let can_break_or_continue = !self.loop_else_stack.is_empty();
        let active_error_body = if active_is_python_error {
            Self::python_error_exit_body(manager_handle(), &error, &cleanup, &active_error_type)
        } else {
            Self::non_python_error_exit_body(
                manager_handle(),
                &error,
                &cleanup,
                &active_error_type,
                active_cause_kind,
            )
        };

        let mut outcome_arms = vec![RustMatchArm {
            pattern: "Ok(Ok(None))".to_string(),
            bindings: vec![],
            guard: None,
            body: vec![normal_exit()],
        }];
        if self.try_closure_depth > 0 {
            outcome_arms.push(RustMatchArm {
                pattern: "Ok(Ok(Some(__sifr_context_return)))".to_string(),
                bindings: vec!["__sifr_context_return".to_string()],
                guard: None,
                body: vec![
                    normal_exit(),
                    RustStmt::Return(Some(RustExpr::Ident("__sifr_context_return".to_string()))),
                ],
            });
        } else {
            outcome_arms.push(RustMatchArm {
                pattern: "Ok(Ok(Some(_)))".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Expr(RustExpr::FormatMacro {
                    name: "unreachable".to_string(),
                    format_str: "Python context captured a return in a non-returning try"
                        .to_string(),
                    args: vec![],
                })],
            });
        }
        if can_break_or_continue {
            outcome_arms.extend([
                RustMatchArm {
                    pattern: "Ok(Err(false))".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![normal_exit(), RustStmt::Break],
                },
                RustMatchArm {
                    pattern: "Ok(Err(true))".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![normal_exit(), RustStmt::Continue],
                },
            ]);
        } else {
            outcome_arms.push(RustMatchArm {
                pattern: "Ok(Err(_))".to_string(),
                bindings: vec![],
                guard: None,
                body: vec![RustStmt::Expr(RustExpr::FormatMacro {
                    name: "unreachable".to_string(),
                    format_str: "Python context emitted loop control outside a loop".to_string(),
                    args: vec![],
                })],
            });
        }
        outcome_arms.push(RustMatchArm {
            pattern: format!("Err(mut {error})"),
            bindings: vec![error.clone()],
            guard: None,
            body: active_error_body,
        });

        Ok(vec![
            RustStmt::Let {
                mutable: false,
                name: manager.clone(),
                ty: None,
                value: manager_value,
            },
            RustStmt::Let {
                mutable: false,
                name: entered_raw.clone(),
                ty: None,
                value: mapped_try(
                    runtime_call("enter_context", vec![reference(manager_handle())]),
                    enter_error_type,
                ),
            },
            RustStmt::Let {
                mutable: false,
                name: conversion.clone(),
                ty: Some(conversion_type),
                value: conversion_call,
            },
            RustStmt::Let {
                mutable: true,
                name: entered_slot.clone(),
                ty: Some(RustType::Option(Box::new(entered_rust_type))),
                value: RustExpr::Literal(RustLiteral::None),
            },
            RustStmt::Match {
                expr: RustExpr::Ident(conversion),
                arms: vec![
                    RustMatchArm {
                        pattern: "Ok(__sifr_entered_value)".to_string(),
                        bindings: vec!["__sifr_entered_value".to_string()],
                        guard: None,
                        body: vec![RustStmt::Assign {
                            target: RustExpr::Ident(entered_slot.clone()),
                            value: call(
                                "Some",
                                RustExpr::Ident("__sifr_entered_value".to_string()),
                            ),
                        }],
                    },
                    RustMatchArm {
                        pattern: format!("Err(mut {error})"),
                        bindings: vec![error.clone()],
                        guard: None,
                        body: conversion_error_body,
                    },
                ],
            },
            RustStmt::LetElse {
                pattern: format!(
                    "Some({}{})",
                    if self.mutated_vars.contains(&item.target) {
                        "mut "
                    } else {
                        ""
                    },
                    item.target
                ),
                value: RustExpr::Ident(entered_slot),
                else_body: vec![RustStmt::Expr(RustExpr::FormatMacro {
                    name: "unreachable".to_string(),
                    format_str: "validated Python context conversion produced no entered value"
                        .to_string(),
                    args: vec![],
                })],
            },
            RustStmt::Let {
                mutable: false,
                name: outcome.clone(),
                ty: Some(outcome_type),
                value: outcome_call,
            },
            RustStmt::Match {
                expr: RustExpr::Ident(outcome),
                arms: outcome_arms,
            },
        ])
    }

    pub(super) fn context_return_expression_type(&self, error_type: &str) -> String {
        if self.try_closure_depth == 0 {
            return "()".to_string();
        }
        let function_return = self.current_return_type.as_ref().map_or_else(
            || "()".to_string(),
            |ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)),
        );
        let ok_type = if self
            .try_closure_option_wrap
            .last()
            .copied()
            .unwrap_or(false)
        {
            format!("Option<{function_return}>")
        } else {
            function_return
        };
        format!("Result<{ok_type}, {error_type}>")
    }

    fn python_error_exit_body(
        manager_handle: RustExpr,
        error: &str,
        cleanup: &str,
        primary_type: &str,
    ) -> Vec<RustStmt> {
        vec![RustStmt::IfLet {
            pattern: "Some(__sifr_python_replay)".to_string(),
            expr: method(field(error, "__sifr_python_error"), "as_ref", vec![]),
            then_body: vec![RustStmt::Match {
                expr: runtime_call(
                    "context_exit_python_error",
                    vec![
                        manager_handle.clone(),
                        RustExpr::Ident("__sifr_python_replay".into()),
                    ],
                ),
                arms: vec![
                    simple_arm(
                        "Ok(sifr_runtime::python::PythonExitDecision::Suppress)",
                        vec![],
                    ),
                    simple_arm(
                        "Ok(sifr_runtime::python::PythonExitDecision::Propagate)",
                        vec![return_error(error)],
                    ),
                    RustMatchArm {
                        pattern: format!("Err({cleanup})"),
                        bindings: vec![cleanup.to_string()],
                        guard: None,
                        body: vec![
                            RustStmt::IfLet {
                                pattern: "Some(__sifr_python_primary)".to_string(),
                                expr: method(field(error, "__sifr_python_error"), "as_mut", vec![]),
                                then_body: vec![
                                    RustStmt::Expr(runtime_call(
                                        "attach_secondary_python_error",
                                        vec![
                                            RustExpr::Ident("__sifr_python_primary".into()),
                                            reference(RustExpr::Ident(cleanup.to_string())),
                                        ],
                                    )),
                                    RustStmt::Assign {
                                        target: field(error, "context"),
                                        value: method(
                                            field("__sifr_python_primary", "context"),
                                            "to_string",
                                            vec![],
                                        ),
                                    },
                                ],
                                else_body: Some(vec![RustStmt::Expr(runtime_call(
                                    "record_context_cleanup_evidence",
                                    vec![
                                        reference(RustExpr::Literal(RustLiteral::Str(
                                            primary_type.to_string(),
                                        ))),
                                        reference(RustExpr::Ident(cleanup.to_string())),
                                    ],
                                ))]),
                            },
                            return_error(error),
                        ],
                    },
                ],
            }],
            else_body: Some(Self::non_python_error_exit_body(
                manager_handle,
                error,
                cleanup,
                primary_type,
                "OrdinaryError",
            )),
        }]
    }

    fn non_python_error_exit_body(
        manager_handle: RustExpr,
        error: &str,
        cleanup: &str,
        primary_type: &str,
        cause_kind: &str,
    ) -> Vec<RustStmt> {
        let evidence_label = format!("{}:{primary_type}", cause_kind_label(cause_kind));
        let cause = "__sifr_python_context_cause";
        vec![
            RustStmt::Let {
                mutable: false,
                name: cause.to_string(),
                ty: None,
                value: RustExpr::StructInit {
                    name: "sifr_runtime::python::SifrExitCause".to_string(),
                    fields: vec![
                        (
                            "kind".to_string(),
                            RustExpr::Path(vec![
                                "sifr_runtime".to_string(),
                                "python".to_string(),
                                "SifrExitCauseKind".to_string(),
                                cause_kind.to_string(),
                            ]),
                        ),
                        (
                            "sifr_type".to_string(),
                            method(
                                RustExpr::Literal(RustLiteral::Str(primary_type.to_string())),
                                "to_string",
                                vec![],
                            ),
                        ),
                        (
                            "message".to_string(),
                            RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: "{}".to_string(),
                                args: vec![RustExpr::Ident(error.to_string())],
                            },
                        ),
                    ],
                },
            },
            RustStmt::Match {
                expr: runtime_call(
                    "context_exit_sifr_cause",
                    vec![
                        manager_handle,
                        reference(RustExpr::Ident(cause.to_string())),
                    ],
                ),
                arms: vec![
                    simple_arm(
                        "Ok(sifr_runtime::python::PythonExitDecision::Suppress)",
                        vec![RustStmt::Expr(runtime_call(
                            "record_context_ignored_suppression",
                            vec![reference(RustExpr::Literal(RustLiteral::Str(
                                evidence_label.clone(),
                            )))],
                        ))],
                    ),
                    simple_arm(
                        "Ok(sifr_runtime::python::PythonExitDecision::Propagate)",
                        vec![],
                    ),
                    RustMatchArm {
                        pattern: format!("Err({cleanup})"),
                        bindings: vec![cleanup.to_string()],
                        guard: None,
                        body: vec![RustStmt::Expr(runtime_call(
                            "record_context_cleanup_evidence",
                            vec![
                                reference(RustExpr::Literal(RustLiteral::Str(evidence_label))),
                                reference(RustExpr::Ident(cleanup.to_string())),
                            ],
                        ))],
                    },
                ],
            },
            return_error(error),
        ]
    }
}

pub(super) fn rewrite_context_control_flow(
    stmts: Vec<RustStmt>,
    loop_depth: usize,
) -> Vec<RustStmt> {
    stmts
        .into_iter()
        .map(|stmt| match stmt {
            RustStmt::Return(Some(expr)) if !is_error_return(&expr) => {
                RustStmt::Return(Some(call("Ok", call("Ok", call("Some", expr)))))
            }
            RustStmt::Break if loop_depth == 0 => RustStmt::Return(Some(call(
                "Ok",
                call("Err", RustExpr::Literal(RustLiteral::Bool(false))),
            ))),
            RustStmt::Continue if loop_depth == 0 => RustStmt::Return(Some(call(
                "Ok",
                call("Err", RustExpr::Literal(RustLiteral::Bool(true))),
            ))),
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => RustStmt::If {
                cond,
                then_body: rewrite_context_control_flow(then_body, loop_depth),
                else_body: else_body.map(|body| rewrite_context_control_flow(body, loop_depth)),
            },
            RustStmt::IfLet {
                pattern,
                expr,
                then_body,
                else_body,
            } => RustStmt::IfLet {
                pattern,
                expr,
                then_body: rewrite_context_control_flow(then_body, loop_depth),
                else_body: else_body.map(|body| rewrite_context_control_flow(body, loop_depth)),
            },
            RustStmt::LetElse {
                pattern,
                value,
                else_body,
            } => RustStmt::LetElse {
                pattern,
                value,
                else_body: rewrite_context_control_flow(else_body, loop_depth),
            },
            RustStmt::Match { expr, arms } => RustStmt::Match {
                expr,
                arms: arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.body = rewrite_context_control_flow(arm.body, loop_depth);
                        arm
                    })
                    .collect(),
            },
            RustStmt::For { var, iter, body } => RustStmt::For {
                var,
                iter,
                body: rewrite_context_control_flow(body, loop_depth + 1),
            },
            RustStmt::While { cond, body } => RustStmt::While {
                cond,
                body: rewrite_context_control_flow(body, loop_depth + 1),
            },
            RustStmt::Loop { body } => RustStmt::Loop {
                body: rewrite_context_control_flow(body, loop_depth + 1),
            },
            RustStmt::With { items, body } => RustStmt::With {
                items,
                body: rewrite_context_control_flow(body, loop_depth),
            },
            RustStmt::Block(body) => {
                RustStmt::Block(rewrite_context_control_flow(body, loop_depth))
            }
            other => other,
        })
        .collect()
}

fn is_error_return(expr: &RustExpr) -> bool {
    matches!(
        expr,
        RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["Err"])
    )
}

pub(super) fn classify_cause_kind(error_type: Option<&Type>, rendered: &str) -> &'static str {
    let class_name = match error_type.map(Type::resolve_alias) {
        Some(Type::Class { name, .. }) => name.as_str(),
        _ => rendered,
    };
    match class_name {
        "CancellationError" => "Cancellation",
        "TimeoutError" => "Timeout",
        "RuntimeFault" | "WorkerRuntimeError" => "RuntimeFault",
        _ => "OrdinaryError",
    }
}

fn cause_kind_label(cause_kind: &str) -> &'static str {
    match cause_kind {
        "Cancellation" => "cancellation",
        "Timeout" => "timeout",
        "RuntimeFault" => "runtime-fault",
        _ => "ordinary-error",
    }
}

fn call(name: &str, value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![name.to_string()])),
        args: vec![value],
    }
}

fn field(name: &str, field_name: &str) -> RustExpr {
    RustExpr::Field {
        expr: Box::new(RustExpr::Ident(name.to_string())),
        field: field_name.to_string(),
    }
}

fn method(receiver: RustExpr, name: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: name.to_string(),
        args,
    }
}

fn reference(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn return_error(name: &str) -> RustStmt {
    RustStmt::Return(Some(call("Err", RustExpr::Ident(name.to_string()))))
}

fn simple_arm(pattern: &str, body: Vec<RustStmt>) -> RustMatchArm {
    RustMatchArm {
        pattern: pattern.to_string(),
        bindings: vec![],
        guard: None,
        body,
    }
}
