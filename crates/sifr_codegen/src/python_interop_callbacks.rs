use crate::python_interop_direct::{callback_output_value_expr, input_conversion, runtime_call};
use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_ir::{
    PythonCallbackConcurrency, PythonCallbackDeclaration, PythonCallbackDispatch,
    PythonCallbackLifetime, PythonCleanupPolicy, PythonInteropDeclaration,
};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;

pub(crate) struct CallbackSetup {
    pub(crate) statements: Vec<RustStmt>,
    pub(crate) callback_var: String,
    pub(crate) failure_slot: Option<(String, Type)>,
    pub(crate) dispatch: PythonCallbackDispatch,
    pub(crate) lifetime: PythonCallbackLifetime,
}

pub(crate) fn callback_setup(
    callback: &PythonCallbackDeclaration,
    handler_name: &str,
    callback_index: usize,
    error_type: &Type,
    opaque_classes: &HashMap<String, sifr_ir::PythonInteropDeclaration>,
    owner: RustExpr,
    failure_slot_source: Option<RustExpr>,
) -> Option<CallbackSetup> {
    if callback.dispatch == PythonCallbackDispatch::Current
        && callback.lifetime != PythonCallbackLifetime::Call
    {
        return None;
    }
    let callback_var = format!("__sifr_callback_{callback_index}");
    let handler_capture_var = format!("__sifr_callback_handler_{callback_index}");
    let slot_var = format!("__sifr_callback_failure_{callback_index}");
    let handler_slot_var = format!("__sifr_callback_failure_for_handler_{callback_index}");
    let mut statements = Vec::new();
    if callback.dispatch == PythonCallbackDispatch::Asyncio {
        statements.push(RustStmt::Let {
            mutable: false,
            name: handler_capture_var.clone(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "Arc".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Ident(handler_name.to_string())],
            },
        });
    }
    if callback.handler_error_type.is_some() {
        statements.push(RustStmt::Let {
            mutable: false,
            name: slot_var.clone(),
            ty: None,
            value: failure_slot_source.unwrap_or_else(|| RustExpr::FnCall {
                func: Box::new(callback_runtime_path("CallbackFailureSlot::new")),
                args: Vec::new(),
            }),
        });
        statements.push(RustStmt::Let {
            mutable: false,
            name: handler_slot_var.clone(),
            ty: None,
            value: RustExpr::Clone(Box::new(RustExpr::Ident(slot_var.clone()))),
        });
    }

    let factory = match callback.dispatch {
        PythonCallbackDispatch::Current => "current_callback_with_owner",
        PythonCallbackDispatch::Foreign if callback.lifetime == PythonCallbackLifetime::Call => {
            "foreign_callback_scoped_with_owner"
        }
        PythonCallbackDispatch::Foreign => "foreign_callback_with_owner",
        PythonCallbackDispatch::Asyncio if callback.lifetime == PythonCallbackLifetime::Call => {
            "asyncio_callback_scoped_with_owner"
        }
        PythonCallbackDispatch::Asyncio => "asyncio_callback_with_owner",
    };
    let mut factory_args = vec![owner];
    factory_args.extend([
        RustExpr::Literal(RustLiteral::Int(
            i64::try_from(callback_index.checked_add(1)?).ok()?,
        )),
        RustExpr::Literal(RustLiteral::Int(
            i64::try_from(callback.argument_types.len()).ok()?,
        )),
    ]);
    if matches!(
        callback.dispatch,
        PythonCallbackDispatch::Foreign | PythonCallbackDispatch::Asyncio
    ) {
        factory_args.push(concurrency_expr(callback.dispatch, callback.concurrency?));
    }
    factory_args.push(decoder(callback, opaque_classes)?);
    factory_args.push(handler_adapter(
        callback,
        if callback.dispatch == PythonCallbackDispatch::Asyncio {
            &handler_capture_var
        } else {
            handler_name
        },
        callback
            .handler_error_type
            .as_ref()
            .map(|_| handler_slot_var.as_str()),
    ));
    factory_args.push(encoder(callback, opaque_classes)?);
    statements.push(RustStmt::Let {
        mutable: false,
        name: callback_var.clone(),
        ty: None,
        value: mapped_try(runtime_call(factory, factory_args), error_type),
    });
    Some(CallbackSetup {
        statements,
        callback_var,
        failure_slot: callback
            .handler_error_type
            .clone()
            .map(|error| (slot_var, error)),
        dispatch: callback.dispatch,
        lifetime: callback.lifetime,
    })
}

pub(crate) fn retained_failure_field(
    callback: &PythonCallbackDeclaration,
    errors: &HashMap<String, Vec<Type>>,
) -> Option<String> {
    let owner = callback.owner_class.as_ref()?;
    let error = callback.handler_error_type.as_ref()?;
    let index = errors
        .get(owner)?
        .iter()
        .position(|candidate| candidate == error)?;
    Some(format!("__sifr_python_callback_failure_{index}"))
}

pub(crate) fn append_retained_failure_slots(
    body: &mut Vec<RustStmt>,
    declaration: &PythonInteropDeclaration,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    lifetime: PythonCallbackLifetime,
) {
    let mut fields = Vec::new();
    for callback in &declaration.callbacks {
        if callback.lifetime != lifetime {
            continue;
        }
        let Some(field) = retained_failure_field(callback, retained_callback_errors) else {
            continue;
        };
        if fields.contains(&field) {
            continue;
        }
        body.push(RustStmt::Let {
            mutable: false,
            name: retained_slot_var(&field),
            ty: None,
            value: runtime_call("CallbackFailureSlot::new", Vec::new()),
        });
        fields.push(field);
    }
}

pub(crate) fn retained_slot_source(
    callback: &PythonCallbackDeclaration,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    lifetime: PythonCallbackLifetime,
) -> Option<RustExpr> {
    if callback.lifetime != lifetime {
        return None;
    }
    retained_failure_field(callback, retained_callback_errors)
        .map(|field| RustExpr::Clone(Box::new(RustExpr::Ident(retained_slot_var(&field)))))
}

pub(crate) fn append_retained_failure_transfers(
    body: &mut Vec<RustStmt>,
    declaration: &PythonInteropDeclaration,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    converted: &str,
) {
    let mut fields = Vec::new();
    for callback in &declaration.callbacks {
        if callback.lifetime != PythonCallbackLifetime::Result {
            continue;
        }
        let Some(field) = retained_failure_field(callback, retained_callback_errors) else {
            continue;
        };
        if fields.contains(&field) {
            continue;
        }
        body.push(RustStmt::Assign {
            target: RustExpr::Field {
                expr: Box::new(RustExpr::Ident(converted.to_string())),
                field: field.clone(),
            },
            value: RustExpr::Ident(retained_slot_var(&field)),
        });
        fields.push(field);
    }
}

fn retained_slot_var(field: &str) -> String {
    format!("__sifr_retained{field}")
}

pub(crate) fn retained_cleanup_expr(cleanup: PythonCleanupPolicy) -> Option<RustExpr> {
    let variant = match cleanup {
        PythonCleanupPolicy::Close => "Close",
        PythonCleanupPolicy::Context => "Context",
        PythonCleanupPolicy::AsyncClose => "AsyncClose",
        PythonCleanupPolicy::AsyncContext => "AsyncContext",
        PythonCleanupPolicy::Drop => return None,
    };
    Some(RustExpr::Path(vec![
        "sifr_runtime".to_string(),
        "python".to_string(),
        "RetainedCallbackCleanup".to_string(),
        variant.to_string(),
    ]))
}

pub(crate) fn callback_object_expr(setup: &CallbackSetup) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(setup.callback_var.clone())),
        method: "object".to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn callback_owner_expr(setup: &CallbackSetup) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(setup.callback_var.clone())),
        method: "owner".to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn callback_outcome_with_evidence(
    outcome: RustExpr,
    callbacks: &[CallbackSetup],
) -> RustExpr {
    runtime_call(
        "attach_callback_failure_evidence",
        vec![
            outcome,
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Array(
                    callbacks
                        .iter()
                        .map(|callback| RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(callback_owner_expr(callback)),
                        })
                        .collect(),
                )),
            },
        ],
    )
}

pub(crate) fn callback_cleanup_expr(setup: &CallbackSetup) -> RustExpr {
    debug_assert_eq!(setup.lifetime, PythonCallbackLifetime::Call);
    let method = match setup.dispatch {
        PythonCallbackDispatch::Current => "close",
        PythonCallbackDispatch::Foreign => "close_call_scope",
        PythonCallbackDispatch::Asyncio => "close_call_scope",
    };
    let close = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(setup.callback_var.clone())),
        method: method.to_string(),
        args: Vec::new(),
    };
    if setup.dispatch == PythonCallbackDispatch::Asyncio {
        RustExpr::Await(Box::new(close))
    } else {
        close
    }
}

pub(crate) fn append_retained_callback_retention(
    body: &mut Vec<RustStmt>,
    setups: &[CallbackSetup],
    error_type: &Type,
) {
    for setup in setups {
        if setup.lifetime == PythonCallbackLifetime::Call {
            continue;
        }
        body.push(RustStmt::Expr(mapped_try(
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(setup.callback_var.clone())),
                method: "retain_in_owner".to_string(),
                args: Vec::new(),
            },
            error_type,
        )));
    }
}

pub(crate) fn failure_reconciliation_stmt(
    slot: &str,
    handler_error_type: &Type,
    enclosing_error_type: &Type,
    owner: RustExpr,
) -> RustStmt {
    RustStmt::IfLet {
        pattern: "Some(__sifr_callback_handler_error)".to_string(),
        expr: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(slot.to_string())),
            method: "take_if_owner_first".to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(owner),
            }],
        },
        then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![error_channel_value(
                RustExpr::Ident("__sifr_callback_handler_error".to_string()),
                handler_error_type,
                enclosing_error_type,
            )],
        }))],
        else_body: None,
    }
}

fn decoder(
    callback: &PythonCallbackDeclaration,
    opaque_classes: &HashMap<String, sifr_ir::PythonInteropDeclaration>,
) -> Option<RustExpr> {
    let mut body = Vec::new();
    let mut converted = Vec::new();
    for (index, ty) in callback.argument_types.iter().enumerate() {
        let handle = format!("__sifr_callback_arg_{index}");
        body.push(RustStmt::Let {
            mutable: false,
            name: handle.clone(),
            ty: None,
            value: RustExpr::Clone(Box::new(RustExpr::Index {
                expr: Box::new(RustExpr::Ident("__sifr_callback_args".to_string())),
                index: Box::new(RustExpr::Literal(RustLiteral::Int(
                    i64::try_from(index).ok()?,
                ))),
            })),
        });
        converted.push(callback_output_value_expr(&handle, ty, opaque_classes)?);
    }
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::Tuple(converted)],
    })));
    Some(RustExpr::ClosureBlock {
        params: vec![RustParam::Named {
            name: "__sifr_callback_args".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body,
        is_move: true,
        is_async: false,
    })
}

fn handler_adapter(
    callback: &PythonCallbackDeclaration,
    handler_name: &str,
    failure_slot: Option<&str>,
) -> RustExpr {
    let handler_args = callback
        .argument_conventions
        .iter()
        .enumerate()
        .map(|(index, convention)| convention_arg(index, *convention))
        .collect();
    let invoked_handler = if callback.dispatch == PythonCallbackDispatch::Asyncio {
        "__sifr_callback_handler"
    } else {
        handler_name
    };
    let mut handler_call = RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(invoked_handler.to_string())),
        args: handler_args,
    };
    if callback.dispatch == PythonCallbackDispatch::Asyncio {
        handler_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_TASK_CANCELLATION".to_string())),
            method: "scope".to_string(),
            args: vec![
                RustExpr::Ident("__sifr_callback_cancellation".to_string()),
                RustExpr::AsyncBlock {
                    body: vec![RustStmt::Return(Some(RustExpr::Await(Box::new(
                        handler_call,
                    ))))],
                    is_move: true,
                },
            ],
        }));
    }
    let result = if let (Some(error_type), Some(slot)) =
        (&callback.handler_error_type, failure_slot)
    {
        RustExpr::MethodCall {
            receiver: Box::new(handler_call),
            method: "map_err".to_string(),
            args: vec![RustExpr::ClosureBlock {
                params: vec![RustParam::Named {
                    name: "__sifr_callback_error".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: vec![
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(slot.to_string())),
                        method: "record".to_string(),
                        args: vec![
                            RustExpr::Ident("__sifr_callback_sequence".to_string()),
                            RustExpr::Ident("__sifr_callback_error".to_string()),
                        ],
                    }),
                    RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(callback_runtime_path("CallbackExecutionError::Handler")),
                        args: vec![RustExpr::FnCall {
                            func: Box::new(callback_runtime_path("CallbackHandlerFailure::new")),
                            args: vec![
                                RustExpr::Literal(RustLiteral::Str(error_type.to_string())),
                                RustExpr::Literal(RustLiteral::Str(
                                    "Sifr callback handler returned an error".to_string(),
                                )),
                            ],
                        }],
                    })),
                ],
                is_move: true,
                is_async: false,
            }],
        }
    } else {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![handler_call],
        }
    };
    let mut params = vec![
        RustParam::Named {
            name: "__sifr_callback_sequence".to_string(),
            ty: RustType::Named("_".to_string()),
        },
        if callback
            .argument_conventions
            .iter()
            .any(|convention| convention.is_mut_borrow())
        {
            RustParam::NamedMut {
                name: "__sifr_callback_values".to_string(),
                ty: RustType::Named("_".to_string()),
            }
        } else {
            RustParam::Named {
                name: "__sifr_callback_values".to_string(),
                ty: RustType::Named("_".to_string()),
            }
        },
    ];
    if callback.dispatch == PythonCallbackDispatch::Asyncio {
        params.push(RustParam::Named {
            name: "__sifr_callback_cancellation".to_string(),
            ty: RustType::Named("_".to_string()),
        });
    }
    RustExpr::Closure {
        params,
        body: Box::new(if callback.dispatch == PythonCallbackDispatch::Asyncio {
            RustExpr::Block {
                stmts: vec![RustStmt::Let {
                    mutable: false,
                    name: "__sifr_callback_handler".to_string(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "std".to_string(),
                            "sync".to_string(),
                            "Arc".to_string(),
                            "clone".to_string(),
                        ])),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident(handler_name.to_string())),
                        }],
                    },
                }],
                expr: Some(Box::new(RustExpr::AsyncBlock {
                    body: vec![RustStmt::Return(Some(result))],
                    is_move: true,
                })),
            }
        } else {
            result
        }),
        is_move: true,
    }
}

fn encoder(
    callback: &PythonCallbackDeclaration,
    opaque_classes: &HashMap<String, sifr_ir::PythonInteropDeclaration>,
) -> Option<RustExpr> {
    Some(RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "__sifr_callback_result".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body: Box::new(input_conversion(
            "__sifr_callback_result",
            &callback.success_type,
            opaque_classes,
        )?),
        is_move: true,
    })
}

fn convention_arg(index: usize, convention: ParamConvention) -> RustExpr {
    let field = RustExpr::Field {
        expr: Box::new(RustExpr::Ident("__sifr_callback_values".to_string())),
        field: index.to_string(),
    };
    if convention.is_borrowed() {
        RustExpr::Ref {
            mutable: convention.is_mut_borrow(),
            expr: Box::new(field),
        }
    } else {
        field
    }
}

fn concurrency_expr(
    dispatch: PythonCallbackDispatch,
    concurrency: PythonCallbackConcurrency,
) -> RustExpr {
    let family = if dispatch == PythonCallbackDispatch::Asyncio {
        "AsyncioCallbackConcurrency"
    } else {
        "ForeignCallbackConcurrency"
    };
    let variant = match concurrency {
        PythonCallbackConcurrency::Serial => "Serial",
        PythonCallbackConcurrency::Parallel => "Parallel",
    };
    callback_runtime_path(&format!("{family}::{variant}"))
}

fn error_channel_value(value: RustExpr, source: &Type, target: &Type) -> RustExpr {
    if source.resolve_alias() == target.resolve_alias() {
        return value;
    }
    if let Type::Union(members) = target.resolve_alias() {
        if let Some(variant) = crate::helpers::find_union_variant(members, source) {
            return RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![target.union_enum_name(), variant])),
                args: vec![value],
            };
        }
    }
    value
}

fn mapped_try(value: RustExpr, error_type: &Type) -> RustExpr {
    crate::python_interop_direct::mapped_try(value, error_type)
}

fn callback_runtime_path(item: &str) -> RustExpr {
    let mut path = vec!["sifr_runtime".to_string(), "python".to_string()];
    path.extend(item.split("::").map(str::to_string));
    RustExpr::Path(path)
}
