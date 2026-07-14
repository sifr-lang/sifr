//! Callback ownership and cleanup inside typed async Python declaration frames.

use super::conversions::{
    async_input_conversion, async_input_conversion_borrowed, async_output_value, mapped_let,
    method, push_keyword, push_keyword_expr, push_positional, vector_let,
};
use crate::python_interop_callbacks::{
    append_owner_failure_evidence, append_owner_failure_observer_setup,
    append_owner_failure_reconciliation, append_retained_callback_retention,
    append_retained_failure_slots, append_retained_failure_transfers, callback_cleanup_expr,
    callback_object_expr, callback_outcome_after_cleanup, callback_owner_expr, callback_setup,
    failure_reconciliation_stmt, owner_outcome_with_evidence, retained_cleanup_expr,
    retained_failure_field, retained_slot_source, CallbackSetup,
};
use crate::python_interop_direct::{mapped_try, runtime_call};
use crate::{RustExpr, RustLiteral, RustStmt};
use sifr_ir::{HirFunction, PythonCallbackLifetime, PythonInteropDeclaration, PythonParameterKind};
use sifr_type_system::Type;
use std::collections::HashMap;

pub(super) struct AsyncArgumentFrame<'a> {
    pub(super) body: Vec<RustStmt>,
    pub(super) callbacks: Vec<CallbackSetup>,
    pub(super) retained_result: Option<&'a sifr_ir::PythonCallbackDeclaration>,
}

pub(super) fn argument_frame<'a>(
    func: &HirFunction,
    declaration: &'a PythonInteropDeclaration,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    owner_retained_errors: &[Type],
    has_receiver: bool,
) -> Option<AsyncArgumentFrame<'a>> {
    let mut body = vec![
        vector_let("__sifr_python_args"),
        vector_let("__sifr_python_kwargs"),
    ];
    if has_receiver {
        append_owner_failure_observer_setup(&mut body, owner_retained_errors);
    }
    if declaration
        .callbacks
        .iter()
        .any(|callback| callback.lifetime == PythonCallbackLifetime::Call)
    {
        body.push(mapped_let(
            "__sifr_callback_call_owner",
            owner_outcome_with_evidence(
                runtime_call("CallbackOwnerState::new_call_scoped", Vec::new()),
                owner_retained_errors,
            ),
            error_type,
        ));
    }
    let retained_receiver = declaration
        .callbacks
        .iter()
        .find(|callback| callback.lifetime == PythonCallbackLifetime::Receiver);
    if let Some(callback) = retained_receiver {
        if !has_receiver {
            return None;
        }
        body.push(mapped_let(
            "__sifr_callback_owner",
            owner_outcome_with_evidence(
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "__sifr_python_callbacks".to_string(),
                    }),
                    method: "owner_or_insert".to_string(),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "__sifr_python_object".to_string(),
                            }),
                        },
                        retained_cleanup_expr(callback.owner_cleanup?)?,
                    ],
                },
                owner_retained_errors,
            ),
            error_type,
        ));
    }
    let retained_result = declaration
        .callbacks
        .iter()
        .find(|callback| callback.lifetime == PythonCallbackLifetime::Result);
    if retained_result.is_some() {
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_callback_group".to_string(),
            ty: None,
            value: mapped_try(
                owner_outcome_with_evidence(
                    runtime_call("RetainedCallbackGroup::new", Vec::new()),
                    owner_retained_errors,
                ),
                error_type,
            ),
        });
        append_retained_failure_slots(
            &mut body,
            declaration,
            retained_callback_errors,
            PythonCallbackLifetime::Result,
        );
    }
    let mut callbacks = Vec::new();
    let mut forward_positional_by_name = false;
    for (index, (param, shape)) in func.params.iter().zip(&declaration.parameters).enumerate() {
        let value_name = format!("__sifr_python_arg_{index}");
        if let Some(callback) = declaration
            .callbacks
            .iter()
            .find(|callback| callback.parameter_name == param.name)
        {
            let owner = match callback.lifetime {
                PythonCallbackLifetime::Call => RustExpr::Clone(Box::new(RustExpr::Ident(
                    "__sifr_callback_call_owner".to_string(),
                ))),
                PythonCallbackLifetime::Result => RustExpr::Clone(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_callback_group".to_string())),
                    method: "owner".to_string(),
                    args: Vec::new(),
                })),
                PythonCallbackLifetime::Receiver => RustExpr::Clone(Box::new(RustExpr::Ident(
                    "__sifr_callback_owner".to_string(),
                ))),
            };
            let failure_slot_source = match callback.lifetime {
                PythonCallbackLifetime::Call => None,
                PythonCallbackLifetime::Result => retained_slot_source(
                    callback,
                    retained_callback_errors,
                    PythonCallbackLifetime::Result,
                ),
                PythonCallbackLifetime::Receiver => {
                    retained_failure_field(callback, retained_callback_errors).map(|field| {
                        RustExpr::Clone(Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field,
                        }))
                    })
                }
            };
            let setup = callback_setup(
                callback,
                &param.name,
                index,
                error_type,
                opaque_classes,
                owner,
                failure_slot_source,
                owner_retained_errors,
            )?;
            body.extend(setup.statements.clone());
            body.push(mapped_let(
                &value_name,
                owner_outcome_with_evidence(
                    runtime_call("async_from_object", vec![callback_object_expr(&setup)]),
                    owner_retained_errors,
                ),
                error_type,
            ));
            body.push(if shape.kind == PythonParameterKind::Positional {
                push_positional(&value_name)
            } else {
                push_keyword(&shape.name, &value_name)
            });
            callbacks.push(setup);
            continue;
        }
        if shape.omit_when_absent {
            if shape.kind == PythonParameterKind::Positional {
                forward_positional_by_name = true;
            }
            let present_name = format!("__sifr_python_value_{index}");
            let converted = owner_outcome_with_evidence(
                async_input_conversion(&present_name, &param.ty, opaque_classes)?,
                owner_retained_errors,
            );
            body.push(RustStmt::IfLet {
                pattern: format!("Some({present_name})"),
                expr: RustExpr::Ident(param.name.clone()),
                then_body: vec![
                    mapped_let(&value_name, converted, error_type),
                    push_keyword(&shape.name, &value_name),
                ],
                else_body: None,
            });
            continue;
        }
        match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                body.push(mapped_let(
                    &value_name,
                    owner_outcome_with_evidence(
                        async_input_conversion(&param.name, &param.ty, opaque_classes)?,
                        owner_retained_errors,
                    ),
                    error_type,
                ));
                body.push(
                    if shape.kind == PythonParameterKind::Positional && !forward_positional_by_name
                    {
                        push_positional(&value_name)
                    } else {
                        push_keyword(&shape.name, &value_name)
                    },
                );
            }
            PythonParameterKind::PositionalVariadic => {
                let Type::List(item_type) = param.ty.resolve_alias() else {
                    return None;
                };
                let item_name = format!("__sifr_python_value_{index}");
                body.push(RustStmt::For {
                    var: item_name.clone(),
                    iter: method(RustExpr::Ident(param.name.clone()), "iter", Vec::new()),
                    body: vec![
                        mapped_let(
                            &value_name,
                            owner_outcome_with_evidence(
                                async_input_conversion_borrowed(
                                    &item_name,
                                    item_type,
                                    opaque_classes,
                                )?,
                                owner_retained_errors,
                            ),
                            error_type,
                        ),
                        push_positional(&value_name),
                    ],
                });
            }
            PythonParameterKind::KeywordVariadic => {
                let Type::Dict(key_type, value_type) = param.ty.resolve_alias() else {
                    return None;
                };
                if key_type.resolve_alias() != &Type::Str {
                    return None;
                }
                let key_name = format!("__sifr_python_key_{index}");
                let item_name = format!("__sifr_python_value_{index}");
                body.push(RustStmt::For {
                    var: format!("({key_name}, {item_name})"),
                    iter: method(RustExpr::Ident(param.name.clone()), "iter", Vec::new()),
                    body: vec![
                        mapped_let(
                            &value_name,
                            owner_outcome_with_evidence(
                                async_input_conversion_borrowed(
                                    &item_name,
                                    value_type,
                                    opaque_classes,
                                )?,
                                owner_retained_errors,
                            ),
                            error_type,
                        ),
                        push_keyword_expr(
                            RustExpr::Clone(Box::new(RustExpr::Ident(key_name))),
                            &value_name,
                        ),
                    ],
                });
            }
        }
    }
    Some(AsyncArgumentFrame {
        body,
        callbacks,
        retained_result,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_submission(
    body: &mut Vec<RustStmt>,
    declaration: &PythonInteropDeclaration,
    callbacks: &[CallbackSetup],
    ok_type: &Type,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    retained_result: Option<&sifr_ir::PythonCallbackDeclaration>,
    close_callbacks: Option<RustExpr>,
    owner_retained_errors: &[Type],
) -> Option<()> {
    if !callbacks.is_empty() || close_callbacks.is_some() {
        let mut protected_preamble = if close_callbacks.is_some() {
            std::mem::take(body)
        } else {
            let mut preamble = callbacks
                .iter()
                .filter_map(|callback| callback.provisional_var.as_ref())
                .map(|provisional| RustStmt::Let {
                    mutable: true,
                    name: provisional.clone(),
                    ty: None,
                    value: RustExpr::Literal(RustLiteral::None),
                })
                .collect::<Vec<_>>();
            preamble.append(body);
            *body = preamble;
            Vec::new()
        };
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_retained_parent_cancellation".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident(
                    "__sifr_current_task_cancellation".to_string(),
                )),
                args: Vec::new(),
            },
        });
        body.push(mapped_let(
            "__sifr_retained_cancellation_scope",
            runtime_call(
                "retained_callback_finalization_scope",
                vec![method(
                    RustExpr::Ident("__sifr_retained_parent_cancellation".to_string()),
                    "as_ref",
                    Vec::new(),
                )],
            ),
            error_type,
        ));
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_python_cancellation".to_string(),
            ty: None,
            value: RustExpr::Ident(
                "__sifr_retained_cancellation_scope.as_ref().map(|scope| scope.child().clone()).or(__sifr_retained_parent_cancellation)"
                    .to_string(),
            ),
        });
        let mut finalization = Vec::new();
        finalization.append(&mut protected_preamble);
        append_submission_body(
            &mut finalization,
            declaration,
            callbacks,
            ok_type,
            error_type,
            opaque_classes,
            retained_callback_errors,
            retained_result,
            close_callbacks,
            owner_retained_errors,
            false,
        )?;
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_retained_finalization".to_string(),
            ty: None,
            value: RustExpr::Await(Box::new(RustExpr::AsyncBlock {
                body: finalization,
                is_move: false,
            })),
        });
        for provisional in callbacks
            .iter()
            .filter_map(|callback| callback.provisional_var.as_ref())
        {
            body.push(RustStmt::If {
                cond: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(
                        "__sifr_retained_finalization".to_string(),
                    )),
                    method: "is_err".to_string(),
                    args: Vec::new(),
                },
                then_body: vec![RustStmt::Expr(RustExpr::Ident(format!(
                    "if let Some(__sifr_provisional_callback) = {provisional}.as_ref() {{ if let Err(__sifr_provisional_cleanup_error) = __sifr_provisional_callback.rollback_provisional().await {{ sifr_runtime::python::record_context_cleanup_evidence(\"receiver-callback-registration\", &__sifr_provisional_cleanup_error); }} }}"
                )))],
                else_body: None,
            });
        }
        if retained_result.is_some() {
            body.push(RustStmt::Let {
                mutable: false,
                name: "__sifr_retained_finalized".to_string(),
                ty: None,
                value: RustExpr::Await(Box::new(runtime_call(
                    "finalize_retained_callbacks",
                    vec![
                        RustExpr::Ident("__sifr_retained_finalization".to_string()),
                        RustExpr::Ref {
                            mutable: true,
                            expr: Box::new(RustExpr::Ident("__sifr_callback_group".to_string())),
                        },
                    ],
                ))),
            });
        }
        body.push(RustStmt::Return(Some(RustExpr::Await(Box::new(
            runtime_call(
                "finish_retained_callback_finalization",
                vec![
                    RustExpr::Ident(
                        if retained_result.is_some() {
                            "__sifr_retained_finalized"
                        } else {
                            "__sifr_retained_finalization"
                        }
                        .to_string(),
                    ),
                    RustExpr::Ident("__sifr_retained_cancellation_scope".to_string()),
                ],
            ),
        )))));
        return Some(());
    }
    append_submission_body(
        body,
        declaration,
        callbacks,
        ok_type,
        error_type,
        opaque_classes,
        retained_callback_errors,
        retained_result,
        close_callbacks,
        owner_retained_errors,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn append_submission_body(
    body: &mut Vec<RustStmt>,
    declaration: &PythonInteropDeclaration,
    callbacks: &[CallbackSetup],
    ok_type: &Type,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    retained_result: Option<&sifr_ir::PythonCallbackDeclaration>,
    close_callbacks: Option<RustExpr>,
    owner_retained_errors: &[Type],
    declare_cancellation: bool,
) -> Option<()> {
    if declare_cancellation {
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_python_cancellation".to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident(
                    "__sifr_current_task_cancellation".to_string(),
                )),
                args: Vec::new(),
            },
        });
    }
    let mut submit_args = vec![
        RustExpr::Ident("__sifr_python_request".to_string()),
        method(
            RustExpr::Ident("__sifr_python_cancellation".to_string()),
            "as_ref",
            Vec::new(),
        ),
    ];
    let submit_name = if let Some(callbacks) = close_callbacks {
        submit_args.push(callbacks);
        "submit_async_declaration_with_callbacks"
    } else {
        "submit_async_declaration"
    };
    let submit = RustExpr::Await(Box::new(runtime_call(submit_name, submit_args)));
    if callbacks.is_empty() && owner_retained_errors.is_empty() {
        body.push(mapped_let("__sifr_python_result", submit, error_type));
    } else {
        body.push(RustStmt::Let {
            mutable: !owner_retained_errors.is_empty(),
            name: "__sifr_python_outcome".to_string(),
            ty: None,
            value: submit,
        });
        let mut cleanup_names = Vec::new();
        for (index, setup) in callbacks.iter().enumerate() {
            if setup.lifetime == PythonCallbackLifetime::Call {
                let name = format!("__sifr_callback_cleanup_{index}");
                body.push(RustStmt::Let {
                    mutable: false,
                    name: name.clone(),
                    ty: None,
                    value: callback_cleanup_expr(setup, true),
                });
                cleanup_names.push(name);
            }
        }
        body.push(RustStmt::Let {
            mutable: !owner_retained_errors.is_empty(),
            name: "__sifr_python_outcome".to_string(),
            ty: None,
            value: callback_outcome_after_cleanup(
                RustExpr::Ident("__sifr_python_outcome".to_string()),
                callbacks,
                &cleanup_names,
            ),
        });
        append_owner_failure_evidence(body, owner_retained_errors);
        body.push(mapped_let(
            "__sifr_python_result",
            RustExpr::Ident("__sifr_python_outcome".to_string()),
            error_type,
        ));
        for setup in callbacks {
            if let Some((slot, handler_error_type)) = &setup.failure_slot {
                body.push(failure_reconciliation_stmt(
                    slot,
                    handler_error_type,
                    error_type,
                    callback_owner_expr(setup),
                ));
            }
        }
        append_retained_callback_retention(body, callbacks, error_type);
        append_owner_failure_reconciliation(body, owner_retained_errors, error_type);
    }
    let converted =
        async_output_value("__sifr_python_result", ok_type, error_type, opaque_classes)?;
    if let Some(callback) = retained_result {
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_python_converted".to_string(),
            ty: None,
            value: converted,
        });
        append_retained_failure_transfers(
            body,
            declaration,
            retained_callback_errors,
            "__sifr_python_converted",
        );
        body.push(mapped_let(
            "__sifr_result_callback_owner",
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_callback_group".to_string())),
                method: "commit_for_object".to_string(),
                args: vec![
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("__sifr_python_converted".to_string())),
                            field: "__sifr_python_object".to_string(),
                        }),
                    },
                    retained_cleanup_expr(callback.owner_cleanup?)?,
                ],
            },
            error_type,
        ));
        body.push(RustStmt::Assign {
            target: RustExpr::Field {
                expr: Box::new(RustExpr::Ident("__sifr_python_converted".to_string())),
                field: "__sifr_python_callbacks".to_string(),
            },
            value: runtime_call(
                "CallbackOwnerSlot::from_owner",
                vec![RustExpr::Ident("__sifr_result_callback_owner".to_string())],
            ),
        });
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Ident("__sifr_python_converted".to_string())],
        })));
        return Some(());
    }
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![converted],
    })));
    Some(())
}
