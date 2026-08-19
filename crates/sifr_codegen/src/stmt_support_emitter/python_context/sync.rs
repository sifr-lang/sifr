use crate::hir_analysis::queries;
use crate::python_interop_callbacks::failure_reconciliation_stmt;
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
        self.python_context_envelope_depth += 1;
        let lowered_body = self.try_lower_stmt_block_for_ir(body);
        self.python_context_envelope_depth -= 1;
        let Some(lowered_body) = lowered_body? else {
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
                    Some(self.render_rust_type_with_generics(item.context.ty()))
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
        let active_is_python_error = active_error_type_info
            .as_ref()
            .is_some_and(Type::is_python_error_contract);
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
        let owner_retained_errors = match item.context.ty().resolve_alias() {
            Type::Class { name, .. } => self
                .python_retained_callback_errors
                .get(name)
                .cloned()
                .unwrap_or_default(),
            _ => Vec::new(),
        };

        let manager_handle = || RustExpr::Field {
            expr: Box::new(RustExpr::Ident(manager.clone())),
            field: "__sifr_python_object".to_string(),
        };
        let manager_callbacks = || RustExpr::Field {
            expr: Box::new(RustExpr::Ident(manager.clone())),
            field: "__sifr_python_callbacks".to_string(),
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
            manager_callbacks(),
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
        let closure_is_async = Self::rust_stmts_contain_await(&closure_body);
        let closure_call = RustExpr::FnCall {
            func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: false,
                is_async: closure_is_async,
            }))),
            args: vec![],
        };
        let outcome_call = if closure_is_async {
            RustExpr::Await(Box::new(closure_call))
        } else {
            closure_call
        };

        let normal_exit = || {
            Self::normal_context_exit_body(
                &RustExpr::Ident(manager.clone()),
                exit_error_type,
                &owner_retained_errors,
                suffix,
            )
        };
        let can_break_or_continue = !self.loop_else_stack.is_empty();
        let active_error_body = if active_is_python_error {
            Self::python_error_exit_body(
                manager_handle(),
                manager_callbacks(),
                &error,
                &cleanup,
                &active_error_type,
            )
        } else {
            Self::non_python_error_exit_body(
                manager_handle(),
                manager_callbacks(),
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
            body: normal_exit(),
        }];
        if self.try_closure_depth > 0 {
            outcome_arms.push(RustMatchArm {
                pattern: "Ok(Ok(Some(__sifr_context_return)))".to_string(),
                bindings: vec!["__sifr_context_return".to_string()],
                guard: None,
                body: {
                    let mut exit = normal_exit();
                    exit.push(RustStmt::Return(Some(RustExpr::Ident(
                        "__sifr_context_return".to_string(),
                    ))));
                    exit
                },
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
                    body: {
                        let mut exit = normal_exit();
                        exit.push(RustStmt::Break);
                        exit
                    },
                },
                RustMatchArm {
                    pattern: "Ok(Err(true))".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: {
                        let mut exit = normal_exit();
                        exit.push(RustStmt::Continue);
                        exit
                    },
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
                    runtime_call(
                        "context_enter_with_callbacks",
                        vec![reference(manager_handle()), reference(manager_callbacks())],
                    ),
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

    fn normal_context_exit_body(
        manager: &RustExpr,
        exit_error_type: &Type,
        owner_retained_errors: &[Type],
        suffix: usize,
    ) -> Vec<RustStmt> {
        let owner = format!("__sifr_python_context_callback_owner_{suffix}");
        let manager_handle = RustExpr::Field {
            expr: Box::new(manager.clone()),
            field: "__sifr_python_object".to_string(),
        };
        let manager_callbacks = RustExpr::Field {
            expr: Box::new(manager.clone()),
            field: "__sifr_python_callbacks".to_string(),
        };
        let mut body = Vec::new();
        if !owner_retained_errors.is_empty() {
            body.push(RustStmt::Let {
                mutable: false,
                name: owner.clone(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(manager_callbacks.clone()),
                    method: "owner".to_string(),
                    args: Vec::new(),
                },
            });
            for index in 0..owner_retained_errors.len() {
                body.push(RustStmt::Let {
                    mutable: false,
                    name: format!("__sifr_python_context_callback_failure_{suffix}_{index}"),
                    ty: None,
                    value: RustExpr::Field {
                        expr: Box::new(manager.clone()),
                        field: format!("__sifr_python_callback_failure_{index}"),
                    },
                });
            }
        }
        body.push(RustStmt::Expr(mapped_try(
            runtime_call(
                "context_exit_normal_with_callbacks",
                vec![manager_handle, manager_callbacks],
            ),
            exit_error_type,
        )));
        if !owner_retained_errors.is_empty() {
            body.push(RustStmt::IfLet {
                pattern: "Some(__sifr_python_context_callback_owner_value)".to_string(),
                expr: RustExpr::Ident(owner),
                then_body: owner_retained_errors
                    .iter()
                    .enumerate()
                    .map(|(index, handler_error_type)| {
                        failure_reconciliation_stmt(
                            &format!("__sifr_python_context_callback_failure_{suffix}_{index}"),
                            handler_error_type,
                            exit_error_type,
                            RustExpr::Ident(
                                "__sifr_python_context_callback_owner_value".to_string(),
                            ),
                        )
                    })
                    .collect(),
                else_body: None,
            });
        }
        body
    }

    fn python_error_exit_body(
        manager_handle: RustExpr,
        manager_callbacks: RustExpr,
        error: &str,
        cleanup: &str,
        primary_type: &str,
    ) -> Vec<RustStmt> {
        vec![RustStmt::IfLet {
            pattern: "Some(__sifr_python_replay)".to_string(),
            expr: method(field(error, "__sifr_python_error"), "as_ref", vec![]),
            then_body: vec![RustStmt::Match {
                expr: runtime_call(
                    "context_exit_python_error_with_callbacks",
                    vec![
                        manager_handle.clone(),
                        RustExpr::Ident("__sifr_python_replay".into()),
                        manager_callbacks.clone(),
                    ],
                ),
                arms: vec![
                    simple_arm(
                        "Ok(::sifr_runtime::python::PythonExitDecision::Suppress)",
                        vec![],
                    ),
                    simple_arm(
                        "Ok(::sifr_runtime::python::PythonExitDecision::Propagate)",
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
                manager_callbacks,
                error,
                cleanup,
                primary_type,
                "OrdinaryError",
            )),
        }]
    }

    fn non_python_error_exit_body(
        manager_handle: RustExpr,
        manager_callbacks: RustExpr,
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
                    name: "::sifr_runtime::python::SifrExitCause".to_string(),
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
                    "context_exit_sifr_cause_with_callbacks",
                    vec![
                        manager_handle,
                        reference(RustExpr::Ident(cause.to_string())),
                        manager_callbacks,
                    ],
                ),
                arms: vec![
                    simple_arm(
                        "Ok(::sifr_runtime::python::PythonExitDecision::Suppress)",
                        vec![RustStmt::Expr(runtime_call(
                            "record_context_ignored_suppression",
                            vec![reference(RustExpr::Literal(RustLiteral::Str(
                                evidence_label.clone(),
                            )))],
                        ))],
                    ),
                    simple_arm(
                        "Ok(::sifr_runtime::python::PythonExitDecision::Propagate)",
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

pub(super) fn classify_cause_kind(error_type: Option<&Type>, _rendered: &str) -> &'static str {
    match error_type.map(Type::resolve_alias) {
        Some(Type::Class {
            identity: Some(identity),
            ..
        }) if identity == "sifr.builtin.CancellationError" => "Cancellation",
        Some(Type::Class {
            identity: Some(identity),
            ..
        }) if identity == "sifr.builtin.TimeoutError" => "Timeout",
        Some(Type::Class {
            identity: Some(identity),
            ..
        }) if identity == "sifr.parallel.WorkerRuntimeError" => "RuntimeFault",
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
