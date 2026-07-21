use crate::python_interop_callbacks::{
    append_owner_failure_evidence, append_owner_failure_observer_setup,
    append_owner_failure_reconciliation, append_retained_callback_retention,
    append_retained_failure_slots, append_retained_failure_transfers, callback_cleanup_expr,
    callback_object_expr, callback_outcome_after_cleanup, callback_owner_expr, callback_setup,
    failure_reconciliation_stmt, owner_outcome_with_evidence, retained_cleanup_expr,
    retained_failure_field, retained_slot_source,
};
use crate::python_interop_direct_helpers::{
    ok_return, push_for_shape, push_keyword_expr, push_named_keyword, push_positional,
};
pub(crate) use crate::python_interop_direct_helpers::{push_to, reference, vector_let};
pub(crate) use crate::python_interop_runtime_exprs::{mapped_let, mapped_try, runtime_call};
use crate::python_zero_copy_arguments as zero_copy;
use crate::{RustExpr, RustLiteral, RustStmt};
use sifr_ir::{
    HirFunction, PythonCallbackLifetime, PythonInteropDeclaration, PythonInteropDecoratorKind,
    PythonParameterKind,
};
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) use crate::python_interop_direct_conversions::{
    callback_output_value_expr, input_conversion, input_conversion_borrowed, is_python_object,
    output_value_expr,
};
#[cfg(test)]
pub(crate) use crate::python_interop_entrypoints::{
    python_interop_function_body, python_interop_method_body,
};

pub(crate) fn python_interop_function_body_with_retained_errors(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind == PythonInteropDecoratorKind::Coroutine {
        return crate::python_interop_async::async_python_function_body(
            func,
            opaque_classes,
            retained_callback_errors,
        );
    }
    if !matches!(
        declaration.kind,
        PythonInteropDecoratorKind::Function
            | PythonInteropDecoratorKind::Buffer
            | PythonInteropDecoratorKind::Arrow
            | PythonInteropDecoratorKind::Dlpack
            | PythonInteropDecoratorKind::DlpackStream
    ) {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let target = declaration.target.as_ref()?;
    if target.segments.len() < 2 || matches!(target.segments[0].as_str(), "bridge" | "Self") {
        return None;
    }

    let mut body = Vec::new();
    body.push(mapped_let(
        "__sifr_python_target",
        runtime_call(
            "resolve_target",
            vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Array(
                    target
                        .segments
                        .iter()
                        .map(|segment| RustExpr::Literal(RustLiteral::Str(segment.clone())))
                        .collect(),
                )),
            }],
        ),
        error_type,
    ));

    body.push(vector_let("__sifr_python_args"));
    body.push(vector_let("__sifr_python_kwargs"));
    if declaration
        .callbacks
        .iter()
        .any(|callback| callback.lifetime == PythonCallbackLifetime::Call)
    {
        body.push(mapped_let(
            "__sifr_callback_call_owner",
            runtime_call("CallbackOwnerState::new_call_scoped", Vec::new()),
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
                runtime_call("RetainedCallbackGroup::new", Vec::new()),
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
    let mut forward_positional_by_name = false;
    let mut callback_setups = Vec::new();
    let mut zero_copy_guards = zero_copy::ArgumentGuards::default();
    for (index, shape) in declaration.parameters.iter().enumerate() {
        let param = func.params.iter().find(|param| param.name == shape.name)?;
        let handle_name = format!("__sifr_python_arg_{index}");
        if let Some(callback) = declaration
            .callbacks
            .iter()
            .find(|callback| callback.parameter_name == param.name)
        {
            let owner = match callback.lifetime {
                PythonCallbackLifetime::Call => Some(RustExpr::Clone(Box::new(RustExpr::Ident(
                    "__sifr_callback_call_owner".to_string(),
                )))),
                PythonCallbackLifetime::Result => {
                    Some(RustExpr::Clone(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_callback_group".to_string())),
                        method: "owner".to_string(),
                        args: Vec::new(),
                    })))
                }
                PythonCallbackLifetime::Receiver => None,
            }?;
            let failure_slot_source = retained_slot_source(
                callback,
                retained_callback_errors,
                PythonCallbackLifetime::Result,
            );
            let setup = callback_setup(
                callback,
                &param.name,
                index,
                error_type,
                opaque_classes,
                owner,
                failure_slot_source,
                &[],
            )?;
            body.extend(setup.statements.clone());
            body.push(mapped_let(
                &handle_name,
                runtime_call(
                    "temporary_argument_handle",
                    vec![callback_object_expr(&setup)],
                ),
                error_type,
            ));
            body.push(push_for_shape(shape.kind, &shape.name, &handle_name)?);
            callback_setups.push(setup);
            continue;
        }
        if shape.omit_when_absent {
            if shape.kind == PythonParameterKind::Positional {
                forward_positional_by_name = true;
            }
            let value_name = format!("__sifr_python_value_{index}");
            let mut present = vec![mapped_let(
                &handle_name,
                input_conversion(&value_name, &param.ty, opaque_classes)?,
                error_type,
            )];
            present.push(match shape.kind {
                PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                    push_named_keyword(&shape.name, &handle_name)
                }
                PythonParameterKind::PositionalVariadic | PythonParameterKind::KeywordVariadic => {
                    return None;
                }
            });
            body.push(RustStmt::IfLet {
                pattern: format!("Some({value_name})"),
                expr: RustExpr::Ident(param.name.clone()),
                then_body: present,
                else_body: None,
            });
            continue;
        }
        match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                if zero_copy_guards.append_preparation(
                    &mut body,
                    zero_copy::ArgumentPreparation {
                        parameter_name: &param.name,
                        index,
                        ty: &param.ty,
                        shape_kind: shape.kind,
                        shape_name: &shape.name,
                        forward_positional_by_name,
                        error_type,
                    },
                )? {
                    continue;
                }
                let conversion = if is_python_object(&param.ty) {
                    runtime_call(
                        "__sifr_declaration_object_argument",
                        vec![RustExpr::Ident(param.name.clone())],
                    )
                } else {
                    input_conversion(&param.name, &param.ty, opaque_classes)?
                };
                body.push(mapped_let(&handle_name, conversion, error_type));
                body.push(
                    if shape.kind == PythonParameterKind::Positional && forward_positional_by_name {
                        push_named_keyword(&shape.name, &handle_name)
                    } else {
                        push_for_shape(shape.kind, &shape.name, &handle_name)?
                    },
                );
            }
            PythonParameterKind::PositionalVariadic => {
                let Type::List(element_type) = param.ty.resolve_alias() else {
                    return None;
                };
                let value_name = format!("__sifr_python_value_{index}");
                let loop_handle = format!("__sifr_python_variadic_{index}");
                body.push(RustStmt::For {
                    var: value_name.clone(),
                    iter: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(param.name.clone())),
                        method: "iter".to_string(),
                        args: Vec::new(),
                    },
                    body: vec![
                        mapped_let(
                            &loop_handle,
                            input_conversion_borrowed(&value_name, element_type, opaque_classes)?,
                            error_type,
                        ),
                        push_positional(&loop_handle),
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
                let loop_handle = format!("__sifr_python_variadic_{index}");
                body.push(RustStmt::For {
                    var: format!("(__sifr_python_key_{index}, __sifr_python_value_{index})"),
                    iter: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(param.name.clone())),
                        method: "iter".to_string(),
                        args: Vec::new(),
                    },
                    body: vec![
                        mapped_let(
                            &loop_handle,
                            input_conversion_borrowed(
                                &format!("__sifr_python_value_{index}"),
                                value_type,
                                opaque_classes,
                            )?,
                            error_type,
                        ),
                        push_keyword_expr(
                            RustExpr::Clone(Box::new(RustExpr::Ident(format!(
                                "__sifr_python_key_{index}"
                            )))),
                            &loop_handle,
                        ),
                    ],
                });
            }
        }
    }

    let call = runtime_call(
        "call_object_owned",
        vec![
            reference("__sifr_python_target"),
            reference("__sifr_python_args"),
            reference("__sifr_python_kwargs"),
        ],
    );
    if callback_setups.is_empty() && zero_copy_guards.is_empty() {
        body.push(mapped_let("__sifr_python_result", call, error_type));
    } else {
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_python_outcome".to_string(),
            ty: None,
            value: call,
        });
        if !callback_setups.is_empty() {
            let mut cleanup_names = Vec::new();
            for (index, setup) in callback_setups.iter().enumerate() {
                if setup.lifetime == PythonCallbackLifetime::Call {
                    let name = format!("__sifr_callback_cleanup_{index}");
                    body.push(RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: callback_cleanup_expr(setup, false),
                    });
                    cleanup_names.push(name);
                }
            }
            body.push(RustStmt::Let {
                mutable: false,
                name: "__sifr_python_outcome".to_string(),
                ty: None,
                value: callback_outcome_after_cleanup(
                    RustExpr::Ident("__sifr_python_outcome".to_string()),
                    &callback_setups,
                    &cleanup_names,
                ),
            });
        }
        zero_copy_guards.append_reconciliation(&mut body, "__sifr_python_outcome");
        body.push(mapped_let(
            "__sifr_python_result",
            RustExpr::Ident("__sifr_python_outcome".to_string()),
            error_type,
        ));
        for setup in &callback_setups {
            if let Some((slot, handler_error_type)) = &setup.failure_slot {
                body.push(failure_reconciliation_stmt(
                    slot,
                    handler_error_type,
                    error_type,
                    callback_owner_expr(setup),
                ));
            }
        }
        append_retained_callback_retention(&mut body, &callback_setups, error_type);
    }

    let converted = if declaration.kind == PythonInteropDecoratorKind::Buffer {
        let Type::PythonBuffer(_) = ok_type.resolve_alias() else {
            return None;
        };
        crate::python_buffer_codegen::acquire_python_buffer_from_foreign(
            RustExpr::Ident("__sifr_python_result".to_string()),
            declaration.buffer.as_ref()?,
            error_type,
        )
    } else if declaration.kind == PythonInteropDecoratorKind::Arrow {
        let Type::PythonArrow(_) = ok_type.resolve_alias() else {
            return None;
        };
        crate::python_arrow_codegen::acquire_python_arrow_from_foreign(
            RustExpr::Ident("__sifr_python_result".to_string()),
            declaration.arrow.as_ref()?,
            &target.dotted(),
            error_type,
        )
    } else if matches!(
        declaration.kind,
        PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream
    ) {
        crate::python_dlpack_codegen::acquire_from_foreign(
            RustExpr::Ident("__sifr_python_result".to_string()),
            declaration.dlpack.as_ref()?,
            ok_type,
            error_type,
        )?
    } else {
        output_value_expr("__sifr_python_result", ok_type, error_type, opaque_classes)?
    };
    if let Some(callback) = retained_result {
        let cleanup = retained_cleanup_expr(callback.owner_cleanup?)?;
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_python_converted".to_string(),
            ty: None,
            value: converted,
        });
        append_retained_failure_transfers(
            &mut body,
            declaration,
            retained_callback_errors,
            "__sifr_python_converted",
        );
        body.push(mapped_let(
            "__sifr_callback_owner",
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
                    cleanup,
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
                vec![RustExpr::Ident("__sifr_callback_owner".to_string())],
            ),
        });
        body.push(ok_return(RustExpr::Ident(
            "__sifr_python_converted".to_string(),
        )));
        return Some(body);
    }
    body.push(ok_return(converted));
    Some(body)
}

pub(crate) fn python_interop_method_body_with_retained_errors(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    owner_declaration: Option<&PythonInteropDeclaration>,
    retained_callback_errors: &HashMap<String, Vec<Type>>,
    owner_retained_errors: &[Type],
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind == PythonInteropDecoratorKind::Coroutine {
        return crate::python_interop_async::async_python_method_body(
            func,
            opaque_classes,
            owner_declaration,
            retained_callback_errors,
            owner_retained_errors,
        );
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    if declaration.kind == PythonInteropDecoratorKind::Buffer {
        return crate::python_buffer_codegen::receiver_interop_body(func);
    }
    if declaration.kind == PythonInteropDecoratorKind::Arrow {
        return crate::python_arrow_codegen::receiver_interop_body(func, owner_declaration);
    }
    if matches!(
        declaration.kind,
        PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream
    ) {
        return crate::python_dlpack_codegen::receiver_interop_body(func, owner_declaration);
    }
    let mut body = Vec::new();
    if !declaration.consumes_receiver {
        append_owner_failure_observer_setup(&mut body, owner_retained_errors);
    }
    if declaration.kind != PythonInteropDecoratorKind::Attribute && !declaration.consumes_receiver {
        body.push(vector_let("__sifr_python_args"));
        body.push(vector_let("__sifr_python_kwargs"));
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
    let mut callback_setups = Vec::new();
    let mut zero_copy_guards = zero_copy::ArgumentGuards::default();
    for (index, shape) in declaration.parameters.iter().enumerate() {
        let param = func.params.iter().find(|param| param.name == shape.name)?;
        let handle = format!("__sifr_python_arg_{index}");
        if let Some(callback) = declaration
            .callbacks
            .iter()
            .find(|callback| callback.parameter_name == param.name)
        {
            let retained_owner = match callback.lifetime {
                PythonCallbackLifetime::Call => Some(RustExpr::Clone(Box::new(RustExpr::Ident(
                    "__sifr_callback_call_owner".to_string(),
                )))),
                PythonCallbackLifetime::Result => {
                    Some(RustExpr::Clone(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_callback_group".to_string())),
                        method: "owner".to_string(),
                        args: Vec::new(),
                    })))
                }
                PythonCallbackLifetime::Receiver => Some(RustExpr::Clone(Box::new(
                    RustExpr::Ident("__sifr_callback_owner".to_string()),
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
                retained_owner?,
                failure_slot_source,
                owner_retained_errors,
            )?;
            body.extend(setup.statements.clone());
            body.push(mapped_let(
                &handle,
                owner_outcome_with_evidence(
                    runtime_call(
                        "temporary_argument_handle",
                        vec![callback_object_expr(&setup)],
                    ),
                    owner_retained_errors,
                ),
                error_type,
            ));
            body.push(push_for_shape(shape.kind, &shape.name, &handle)?);
            callback_setups.push(setup);
            continue;
        }
        if zero_copy_guards.append_preparation(
            &mut body,
            zero_copy::ArgumentPreparation {
                parameter_name: &param.name,
                index,
                ty: &param.ty,
                shape_kind: shape.kind,
                shape_name: &shape.name,
                forward_positional_by_name: false,
                error_type,
            },
        )? {
            continue;
        }
        body.push(mapped_let(
            &handle,
            owner_outcome_with_evidence(
                input_conversion(&param.name, &param.ty, opaque_classes)?,
                owner_retained_errors,
            ),
            error_type,
        ));
        body.push(push_for_shape(shape.kind, &shape.name, &handle)?);
    }
    let member = declaration
        .target
        .as_ref()
        .and_then(|target| target.segments.get(1))
        .map(String::as_str);
    if declaration.consumes_receiver {
        if declaration.kind != PythonInteropDecoratorKind::Function
            || ok_type.resolve_alias() != &Type::None
            || !func.params.is_empty()
        {
            return None;
        }
        if !owner_retained_errors.is_empty() {
            body.push(RustStmt::Let {
                mutable: false,
                name: "__sifr_callback_owner_for_failure".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "__sifr_python_callbacks".to_string(),
                    }),
                    method: "owner".to_string(),
                    args: Vec::new(),
                },
            });
            for index in 0..owner_retained_errors.len() {
                body.push(RustStmt::Let {
                    mutable: false,
                    name: format!("__sifr_callback_owner_failure_{index}"),
                    ty: None,
                    value: RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: format!("__sifr_python_callback_failure_{index}"),
                    },
                });
            }
        }
        let closed = mapped_try(
            runtime_call(
                "semantic_close_with_callbacks",
                vec![
                    RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "__sifr_python_object".to_string(),
                    },
                    RustExpr::Literal(RustLiteral::Str(member?.to_string())),
                    RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "__sifr_python_callbacks".to_string(),
                    },
                ],
            ),
            error_type,
        );
        body.push(RustStmt::Let {
            mutable: false,
            name: "__sifr_python_closed".to_string(),
            ty: None,
            value: closed,
        });
        if !owner_retained_errors.is_empty() {
            body.push(RustStmt::IfLet {
                pattern: "Some(__sifr_callback_owner_for_failure_value)".to_string(),
                expr: RustExpr::Ident("__sifr_callback_owner_for_failure".to_string()),
                then_body: owner_retained_errors
                    .iter()
                    .enumerate()
                    .map(|(index, handler_error_type)| {
                        failure_reconciliation_stmt(
                            &format!("__sifr_callback_owner_failure_{index}"),
                            handler_error_type,
                            error_type,
                            RustExpr::Ident("__sifr_callback_owner_for_failure_value".to_string()),
                        )
                    })
                    .collect(),
                else_body: None,
            });
        }
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::Ident("__sifr_python_closed".to_string())],
        })));
        return Some(body);
    }
    let receiver = RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: "__sifr_python_object".to_string(),
        }),
    };
    let operation = match declaration.kind {
        PythonInteropDecoratorKind::Attribute => runtime_call(
            "get_attr",
            vec![
                receiver,
                RustExpr::Literal(RustLiteral::Str(member?.to_string())),
            ],
        ),
        PythonInteropDecoratorKind::Item => {
            let item_callable = "__sifr_python_item_callable";
            body.push(mapped_let(
                item_callable,
                owner_outcome_with_evidence(
                    runtime_call(
                        "get_attr",
                        vec![
                            receiver.clone(),
                            RustExpr::Literal(RustLiteral::Str("__getitem__".to_string())),
                        ],
                    ),
                    owner_retained_errors,
                ),
                error_type,
            ));
            runtime_call(
                "call_object_owned",
                vec![
                    reference(item_callable),
                    reference("__sifr_python_args"),
                    reference("__sifr_python_kwargs"),
                ],
            )
        }
        PythonInteropDecoratorKind::Function => {
            let callable = "__sifr_python_method_callable";
            body.push(mapped_let(
                callable,
                owner_outcome_with_evidence(
                    runtime_call(
                        "get_attr",
                        vec![
                            receiver,
                            RustExpr::Literal(RustLiteral::Str(member?.to_string())),
                        ],
                    ),
                    owner_retained_errors,
                ),
                error_type,
            ));
            runtime_call(
                "call_object_owned",
                vec![
                    reference(callable),
                    reference("__sifr_python_args"),
                    reference("__sifr_python_kwargs"),
                ],
            )
        }
        _ => return None,
    };
    if callback_setups.is_empty() && zero_copy_guards.is_empty() && owner_retained_errors.is_empty()
    {
        body.push(mapped_let("__sifr_python_result", operation, error_type));
    } else {
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_python_outcome".to_string(),
            ty: None,
            value: operation,
        });
        let mut cleanup_names = Vec::new();
        if !callback_setups.is_empty() {
            for (index, setup) in callback_setups.iter().enumerate() {
                if setup.lifetime == PythonCallbackLifetime::Call {
                    let name = format!("__sifr_callback_cleanup_{index}");
                    body.push(RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: callback_cleanup_expr(setup, false),
                    });
                    cleanup_names.push(name);
                }
            }
            body.push(RustStmt::Assign {
                target: RustExpr::Ident("__sifr_python_outcome".to_string()),
                value: callback_outcome_after_cleanup(
                    RustExpr::Ident("__sifr_python_outcome".to_string()),
                    &callback_setups,
                    &cleanup_names,
                ),
            });
        }
        zero_copy_guards.append_reconciliation(&mut body, "__sifr_python_outcome");
        append_owner_failure_evidence(&mut body, owner_retained_errors);
        body.push(mapped_let(
            "__sifr_python_result",
            RustExpr::Ident("__sifr_python_outcome".to_string()),
            error_type,
        ));
        for setup in &callback_setups {
            if let Some((slot, handler_error_type)) = &setup.failure_slot {
                body.push(failure_reconciliation_stmt(
                    slot,
                    handler_error_type,
                    error_type,
                    callback_owner_expr(setup),
                ));
            }
        }
        append_retained_callback_retention(&mut body, &callback_setups, error_type);
        append_owner_failure_reconciliation(&mut body, owner_retained_errors, error_type);
    }
    let converted = output_value_expr("__sifr_python_result", ok_type, error_type, opaque_classes)?;
    if let Some(callback) = retained_result {
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_python_converted".to_string(),
            ty: None,
            value: converted,
        });
        append_retained_failure_transfers(
            &mut body,
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
        body.push(ok_return(RustExpr::Ident(
            "__sifr_python_converted".to_string(),
        )));
        return Some(body);
    }
    body.push(ok_return(converted));
    Some(body)
}
