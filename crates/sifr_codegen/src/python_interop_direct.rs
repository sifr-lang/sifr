use sifr_ir::{
    HirFunction, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::Type;
use std::collections::HashMap;

use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

pub(crate) fn python_interop_function_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind == PythonInteropDecoratorKind::Coroutine {
        return crate::python_interop_async::async_python_function_body(func, opaque_classes);
    }
    if declaration.kind != PythonInteropDecoratorKind::Function {
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
    let mut forward_positional_by_name = false;
    for (index, (param, shape)) in func.params.iter().zip(&declaration.parameters).enumerate() {
        let handle_name = format!("__sifr_python_arg_{index}");
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
                    return None
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
                let conversion = if is_python_object(&param.ty) {
                    runtime_call("temporary_argument_handle", vec![reference(&param.name)])
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

    body.push(mapped_let(
        "__sifr_python_result",
        runtime_call(
            "call_object_owned",
            vec![
                reference("__sifr_python_target"),
                reference("__sifr_python_args"),
                reference("__sifr_python_kwargs"),
            ],
        ),
        error_type,
    ));

    let converted = output_value_expr("__sifr_python_result", ok_type, error_type, opaque_classes)?;
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![converted],
    })));
    Some(body)
}

pub(crate) fn python_interop_method_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    owner_declaration: Option<&PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind == PythonInteropDecoratorKind::Coroutine {
        return crate::python_interop_async::async_python_method_body(
            func,
            opaque_classes,
            owner_declaration,
        );
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let mut body = Vec::new();
    if declaration.kind != PythonInteropDecoratorKind::Attribute && !declaration.consumes_receiver {
        body.push(vector_let("__sifr_python_args"));
        body.push(vector_let("__sifr_python_kwargs"));
    }
    for (index, (param, shape)) in func.params.iter().zip(&declaration.parameters).enumerate() {
        let handle = format!("__sifr_python_arg_{index}");
        body.push(mapped_let(
            &handle,
            input_conversion(&param.name, &param.ty, opaque_classes)?,
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
        let closed = mapped_try(
            runtime_call(
                "semantic_close",
                vec![
                    RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "__sifr_python_object".to_string(),
                    },
                    RustExpr::Literal(RustLiteral::Str(member?.to_string())),
                ],
            ),
            error_type,
        );
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![closed],
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
                runtime_call(
                    "get_attr",
                    vec![
                        receiver.clone(),
                        RustExpr::Literal(RustLiteral::Str("__getitem__".to_string())),
                    ],
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
                runtime_call(
                    "get_attr",
                    vec![
                        receiver,
                        RustExpr::Literal(RustLiteral::Str(member?.to_string())),
                    ],
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
    body.push(mapped_let("__sifr_python_result", operation, error_type));
    let converted = output_value_expr("__sifr_python_result", ok_type, error_type, opaque_classes)?;
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![converted],
    })));
    Some(body)
}

pub(crate) fn input_conversion(
    name: &str,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        let receiver = RustExpr::Ident(name.to_string());
        let borrowed_inner = !matches!(
            inner.resolve_alias(),
            Type::None | Type::Bool | Type::Int | Type::Float
        );
        let inner_value = RustExpr::MethodCall {
            receiver: Box::new(if borrowed_inner {
                RustExpr::MethodCall {
                    receiver: Box::new(receiver.clone()),
                    method: "as_ref".to_string(),
                    args: Vec::new(),
                }
            } else {
                receiver.clone()
            }),
            method: "unwrap".to_string(),
            args: Vec::new(),
        };
        return Some(RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "is_some".to_string(),
                args: Vec::new(),
            }),
            then_expr: Box::new(input_conversion_value(inner_value, inner, opaque_classes)?),
            else_expr: Some(Box::new(runtime_call("from_none", Vec::new()))),
        });
    }
    if matches!(ty.resolve_alias(), Type::Class { name: class_name, .. } if opaque_classes.contains_key(class_name))
    {
        return Some(runtime_call(
            "temporary_argument_handle",
            vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident(name.to_string())),
                    field: "__sifr_python_object".to_string(),
                }),
            }],
        ));
    }
    match ty.resolve_alias() {
        Type::List(item) => {
            let iter = RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.to_string())),
                method: "iter".to_string(),
                args: Vec::new(),
            };
            let converted = mapped_collection_results(
                iter,
                "__sifr_python_item",
                input_conversion_borrowed("__sifr_python_item", item, opaque_classes)?,
            );
            return Some(runtime_call("from_list_results", vec![converted]));
        }
        Type::Tuple(items) => {
            let values = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    input_conversion(&format!("{name}.{index}"), item, opaque_classes)
                })
                .collect::<Option<Vec<_>>>()?;
            return Some(runtime_call(
                "from_tuple_results",
                vec![RustExpr::Vec(values)],
            ));
        }
        Type::Dict(key, value) if key.resolve_alias() == &Type::Str => {
            let iter = RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.to_string())),
                method: "iter".to_string(),
                args: Vec::new(),
            };
            let pair = RustExpr::Tuple(vec![
                RustExpr::Clone(Box::new(RustExpr::Ident("__sifr_python_key".to_string()))),
                input_conversion_borrowed("__sifr_python_value", value, opaque_classes)?,
            ]);
            let converted =
                mapped_collection_results(iter, "(__sifr_python_key, __sifr_python_value)", pair);
            return Some(runtime_call("from_dict_results", vec![converted]));
        }
        Type::Class {
            name: class_name,
            fields,
            ..
        } if !fields.is_empty() && !opaque_classes.contains_key(class_name) => {
            let values = fields
                .iter()
                .map(|(field, field_type)| {
                    Some(RustExpr::Tuple(vec![
                        RustExpr::Literal(RustLiteral::Str(field.clone())),
                        input_conversion(&format!("{name}.{field}"), field_type, opaque_classes)?,
                    ]))
                })
                .collect::<Option<Vec<_>>>()?;
            return Some(runtime_call(
                "from_record_results",
                vec![RustExpr::Vec(values)],
            ));
        }
        _ => {}
    }
    let ident = RustExpr::Ident(name.to_string());
    let (function, value) = match ty.resolve_alias() {
        Type::None => ("from_none", None),
        Type::Bool => ("from_bool", Some(ident)),
        Type::Int => ("from_int", Some(ident)),
        Type::Float => ("from_float", Some(ident)),
        Type::Str => ("from_str", Some(reference(name))),
        Type::Bytes => ("from_bytes", Some(reference(name))),
        _ => return None,
    };
    Some(runtime_call(function, value.into_iter().collect()))
}

pub(crate) fn output_value_expr(
    value_name: &str,
    ty: &Type,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        let is_none = mapped_try(
            runtime_call("object_is_none", vec![reference(value_name)]),
            error_type,
        );
        return Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_is_none".to_string(),
                ty: None,
                value: is_none,
            }],
            expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::Ident("__sifr_python_is_none".to_string())),
                then_expr: Box::new(RustExpr::Literal(RustLiteral::None)),
                else_expr: Some(Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![output_value_expr(
                        value_name,
                        inner,
                        error_type,
                        opaque_classes,
                    )?],
                })),
            })),
        });
    }
    if is_python_object(ty) {
        return Some(RustExpr::Ident(value_name.to_string()));
    }
    if let Type::Class { name, .. } = ty.resolve_alias() {
        if let Some(opaque) = opaque_classes.get(name) {
            let target = opaque.target.as_ref()?;
            let checked = mapped_try(
                runtime_call(
                    "expect_instance",
                    vec![
                        RustExpr::Ident(value_name.to_string()),
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Array(
                                target
                                    .segments
                                    .iter()
                                    .map(|segment| {
                                        RustExpr::Literal(RustLiteral::Str(segment.clone()))
                                    })
                                    .collect(),
                            )),
                        },
                    ],
                ),
                error_type,
            );
            return Some(RustExpr::StructInit {
                name: name.clone(),
                fields: vec![("__sifr_python_object".to_string(), checked)],
            });
        }
    }
    match ty.resolve_alias() {
        Type::List(item) => {
            let mut body = vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_value".to_string(),
                ty: None,
                value: output_value_expr("__sifr_python_item", item, error_type, opaque_classes)?,
            }];
            body.push(push_to("__sifr_python_values", "__sifr_python_value"));
            return Some(RustExpr::Block {
                stmts: vec![
                    mapped_let(
                        "__sifr_python_items",
                        runtime_call("list_items", vec![reference(value_name)]),
                        error_type,
                    ),
                    vector_let("__sifr_python_values"),
                    RustStmt::For {
                        var: "__sifr_python_item".to_string(),
                        iter: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_python_items".to_string())),
                            method: "into_iter".to_string(),
                            args: Vec::new(),
                        },
                        body,
                    },
                ],
                expr: Some(Box::new(RustExpr::Ident(
                    "__sifr_python_values".to_string(),
                ))),
            });
        }
        Type::Tuple(items) => {
            let mut statements = vec![mapped_let(
                "__sifr_python_items",
                runtime_call("tuple_items", vec![reference(value_name)]),
                error_type,
            )];
            let mut values = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let binding = format!("__sifr_python_tuple_{index}");
                statements.push(RustStmt::Let {
                    mutable: false,
                    name: binding.clone(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_python_items".to_string())),
                        method: "remove".to_string(),
                        args: vec![RustExpr::Literal(RustLiteral::Int(0))],
                    },
                });
                values.push(output_value_expr(
                    &binding,
                    item,
                    error_type,
                    opaque_classes,
                )?);
            }
            statements.insert(
                0,
                RustStmt::Let {
                    mutable: true,
                    name: "__sifr_python_items".to_string(),
                    ty: None,
                    value: mapped_try(
                        runtime_call("tuple_items", vec![reference(value_name)]),
                        error_type,
                    ),
                },
            );
            statements.remove(1);
            return Some(RustExpr::Block {
                stmts: statements,
                expr: Some(Box::new(RustExpr::Tuple(values))),
            });
        }
        Type::Dict(key, value) if key.resolve_alias() == &Type::Str => {
            let converted =
                output_value_expr("__sifr_python_item", value, error_type, opaque_classes)?;
            return Some(RustExpr::Block {
                stmts: vec![
                    mapped_let(
                        "__sifr_python_items",
                        runtime_call("dict_str_items", vec![reference(value_name)]),
                        error_type,
                    ),
                    RustStmt::Let {
                        mutable: true,
                        name: "__sifr_python_values".to_string(),
                        ty: None,
                        value: RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Default".to_string(),
                                "default".to_string(),
                            ])),
                            args: Vec::new(),
                        },
                    },
                    RustStmt::For {
                        var: "(__sifr_python_key, __sifr_python_item)".to_string(),
                        iter: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_python_items".to_string())),
                            method: "into_iter".to_string(),
                            args: Vec::new(),
                        },
                        body: vec![RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_python_values".to_string())),
                            method: "insert".to_string(),
                            args: vec![RustExpr::Ident("__sifr_python_key".to_string()), converted],
                        })],
                    },
                ],
                expr: Some(Box::new(RustExpr::Ident(
                    "__sifr_python_values".to_string(),
                ))),
            });
        }
        Type::Class { name, fields, .. } if !fields.is_empty() => {
            let mut statements = Vec::new();
            let mut converted_fields = Vec::new();
            for (field, field_type) in fields {
                let handle = format!("__sifr_python_field_{field}");
                statements.push(mapped_let(
                    &handle,
                    runtime_call(
                        "record_field",
                        vec![
                            reference(value_name),
                            RustExpr::Literal(RustLiteral::Str(field.clone())),
                        ],
                    ),
                    error_type,
                ));
                converted_fields.push((
                    field.clone(),
                    output_value_expr(&handle, field_type, error_type, opaque_classes)?,
                ));
            }
            return Some(RustExpr::Block {
                stmts: statements,
                expr: Some(Box::new(RustExpr::StructInit {
                    name: name.clone(),
                    fields: converted_fields,
                })),
            });
        }
        _ => {}
    }
    Some(mapped_try(output_conversion(value_name, ty)?, error_type))
}

fn input_conversion_value(
    value: RustExpr,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if is_python_object(ty) {
        return Some(runtime_call("temporary_argument_handle", vec![value]));
    }
    if matches!(ty.resolve_alias(), Type::Class { name, .. } if opaque_classes.contains_key(name)) {
        return Some(runtime_call(
            "temporary_argument_handle",
            vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Field {
                    expr: Box::new(value),
                    field: "__sifr_python_object".to_string(),
                }),
            }],
        ));
    }
    if matches!(
        ty.resolve_alias(),
        Type::List(_) | Type::Tuple(_) | Type::Dict(_, _) | Type::Class { .. }
    ) {
        return Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_nested".to_string(),
                ty: None,
                value,
            }],
            expr: Some(Box::new(input_conversion(
                "__sifr_python_nested",
                ty,
                opaque_classes,
            )?)),
        });
    }
    let function = match ty.resolve_alias() {
        Type::Bool => "from_bool",
        Type::Int => "from_int",
        Type::Float => "from_float",
        Type::Str => "from_str",
        Type::Bytes => "from_bytes",
        _ => return None,
    };
    Some(runtime_call(function, vec![value]))
}

fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Union(variants) = ty.resolve_alias() else {
        return None;
    };
    if variants.len() != 2
        || !variants
            .iter()
            .any(|variant| variant.resolve_alias() == &Type::None)
    {
        return None;
    }
    variants
        .iter()
        .find(|variant| variant.resolve_alias() != &Type::None)
}

fn input_conversion_borrowed(
    name: &str,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if option_inner(ty).is_some() {
        return input_conversion(name, ty, opaque_classes);
    }
    if matches!(
        ty.resolve_alias(),
        Type::List(_) | Type::Tuple(_) | Type::Dict(_, _) | Type::Class { .. }
    ) && !is_python_object(ty)
        && !matches!(ty.resolve_alias(), Type::Class { name, .. } if opaque_classes.contains_key(name))
    {
        return input_conversion(name, ty, opaque_classes);
    }
    let value = match ty.resolve_alias() {
        Type::Bool | Type::Int | Type::Float => {
            RustExpr::Deref(Box::new(RustExpr::Ident(name.to_string())))
        }
        Type::Str | Type::Bytes => RustExpr::Ident(name.to_string()),
        _ if is_python_object(ty) => {
            return Some(runtime_call(
                "temporary_argument_handle",
                vec![RustExpr::Ident(name.to_string())],
            ));
        }
        Type::Class {
            name: class_name, ..
        } if opaque_classes.contains_key(class_name) => {
            return Some(runtime_call(
                "temporary_argument_handle",
                vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(name.to_string())),
                        field: "__sifr_python_object".to_string(),
                    }),
                }],
            ));
        }
        _ => return None,
    };
    let function = match ty.resolve_alias() {
        Type::Bool => "from_bool",
        Type::Int => "from_int",
        Type::Float => "from_float",
        Type::Str => "from_str",
        Type::Bytes => "from_bytes",
        _ => return None,
    };
    Some(runtime_call(function, vec![value]))
}

fn mapped_collection_results(iter: RustExpr, parameter: &str, body: RustExpr) -> RustExpr {
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(iter),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: parameter.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(body),
            is_move: false,
        }],
    };
    RustExpr::MethodCall {
        receiver: Box::new(mapped),
        method: "collect".to_string(),
        args: Vec::new(),
    }
}

fn output_conversion(name: &str, ty: &Type) -> Option<RustExpr> {
    let function = match ty.resolve_alias() {
        Type::None => "to_none",
        Type::Bool => "to_bool",
        Type::Int => "to_int",
        Type::Float => "to_float",
        Type::Str => "to_str",
        Type::Bytes => "to_bytes",
        _ => return None,
    };
    Some(runtime_call(function, vec![reference(name)]))
}

fn is_python_object(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "Object")
}

pub(crate) fn runtime_call(function: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "python".to_string(),
            function.to_string(),
        ])),
        args,
    }
}

fn mapped_let(name: &str, value: RustExpr, error_type: &Type) -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: name.to_string(),
        ty: None,
        value: mapped_try(value, error_type),
    }
}

pub(crate) fn mapped_try(value: RustExpr, error_type: &Type) -> RustExpr {
    let error_name = "__sifr_python_error";
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: error_name.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_error_expr(
                RustExpr::Ident(error_name.to_string()),
                error_type,
            )),
            is_move: false,
        }],
    }))
}

fn reference(name: &str) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Ident(name.to_string())),
    }
}

fn vector_let(name: &str) -> RustStmt {
    RustStmt::Let {
        mutable: true,
        name: name.to_string(),
        ty: None,
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
            args: Vec::new(),
        },
    }
}

fn push_for_shape(kind: PythonParameterKind, name: &str, handle: &str) -> Option<RustStmt> {
    match kind {
        PythonParameterKind::Positional => Some(push_positional(handle)),
        PythonParameterKind::KeywordOnly => Some(push_named_keyword(name, handle)),
        PythonParameterKind::PositionalVariadic | PythonParameterKind::KeywordVariadic => None,
    }
}

fn push_named_keyword(name: &str, handle: &str) -> RustStmt {
    push_keyword_expr(
        RustExpr::Literal(RustLiteral::Str(name.to_string())),
        handle,
    )
}

fn push_positional(handle: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__sifr_python_args".to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Ident(handle.to_string())],
    })
}

fn push_to(vector: &str, value: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(vector.to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Ident(value.to_string())],
    })
}

fn push_keyword_expr(key: RustExpr, handle: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__sifr_python_kwargs".to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Tuple(vec![
            key,
            RustExpr::Ident(handle.to_string()),
        ])],
    })
}
