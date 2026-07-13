use sifr_ir::{
    HirFunction, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::Type;
use std::collections::HashMap;

use crate::python_interop_direct::{mapped_try, runtime_call};
use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

pub(crate) fn async_python_function_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind != PythonInteropDecoratorKind::Coroutine || !func.is_async {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let target = declaration.target.as_ref()?;
    if target.segments.len() < 2 || target.segments[0] == "Self" {
        return None;
    }

    let mut body = argument_frame(func, declaration, error_type, opaque_classes)?;
    let request = RustExpr::FnCall {
        func: Box::new(python_path(&["PythonAsyncRequest", "function"])),
        args: vec![
            RustExpr::Vec(
                target
                    .segments
                    .iter()
                    .map(|value| owned_string(value))
                    .collect(),
            ),
            RustExpr::Ident("__sifr_python_args".to_string()),
            RustExpr::Ident("__sifr_python_kwargs".to_string()),
            output_schema(ok_type, opaque_classes)?,
        ],
    };
    body.push(RustStmt::Let {
        mutable: false,
        name: "__sifr_python_request".to_string(),
        ty: None,
        value: request,
    });
    append_submission(&mut body, ok_type, error_type, opaque_classes)?;
    Some(body)
}

pub(crate) fn async_python_method_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind != PythonInteropDecoratorKind::Coroutine || !func.is_async {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let member = declaration.target.as_ref()?.segments.get(1)?.to_string();
    let mut body = argument_frame(func, declaration, error_type, opaque_classes)?;
    let receiver = RustExpr::Field {
        expr: Box::new(RustExpr::Ident("self".to_string())),
        field: "__sifr_python_object".to_string(),
    };
    let constructor = if declaration.consumes_receiver {
        "owned_method"
    } else {
        "borrowed_method"
    };
    let receiver = if declaration.consumes_receiver {
        receiver
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(receiver),
        }
    };
    let request = RustExpr::FnCall {
        func: Box::new(python_path(&["PythonAsyncRequest", constructor])),
        args: vec![
            receiver,
            owned_string(&member),
            RustExpr::Ident("__sifr_python_args".to_string()),
            RustExpr::Ident("__sifr_python_kwargs".to_string()),
            output_schema(ok_type, opaque_classes)?,
        ],
    };
    body.push(RustStmt::Let {
        mutable: false,
        name: "__sifr_python_request".to_string(),
        ty: None,
        value: mapped_try(request, error_type),
    });
    append_submission(&mut body, ok_type, error_type, opaque_classes)?;
    Some(body)
}

fn argument_frame(
    func: &HirFunction,
    declaration: &PythonInteropDeclaration,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let mut body = vec![
        vector_let("__sifr_python_args"),
        vector_let("__sifr_python_kwargs"),
    ];
    let mut forward_positional_by_name = false;
    for (index, (param, shape)) in func.params.iter().zip(&declaration.parameters).enumerate() {
        let value_name = format!("__sifr_python_arg_{index}");
        if shape.omit_when_absent {
            if shape.kind == PythonParameterKind::Positional {
                forward_positional_by_name = true;
            }
            let present_name = format!("__sifr_python_value_{index}");
            let converted = async_input_conversion(&present_name, &param.ty, opaque_classes)?;
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
                    async_input_conversion(&param.name, &param.ty, opaque_classes)?,
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
                            async_input_conversion_borrowed(&item_name, item_type, opaque_classes)?,
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
                            async_input_conversion_borrowed(
                                &item_name,
                                value_type,
                                opaque_classes,
                            )?,
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
    Some(body)
}

fn append_submission(
    body: &mut Vec<RustStmt>,
    ok_type: &Type,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<()> {
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
    let submit = runtime_call(
        "submit_async_declaration",
        vec![
            RustExpr::Ident("__sifr_python_request".to_string()),
            method(
                RustExpr::Ident("__sifr_python_cancellation".to_string()),
                "as_ref",
                Vec::new(),
            ),
        ],
    );
    body.push(mapped_let(
        "__sifr_python_result",
        RustExpr::Await(Box::new(submit)),
        error_type,
    ));
    let converted =
        async_output_value("__sifr_python_result", ok_type, error_type, opaque_classes)?;
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![converted],
    })));
    Some(())
}

fn async_input_conversion(
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
        let inner_value = method(
            if borrowed_inner {
                method(receiver.clone(), "as_ref", Vec::new())
            } else {
                receiver.clone()
            },
            "unwrap",
            Vec::new(),
        );
        return Some(RustExpr::If {
            cond: Box::new(method(receiver, "is_some", Vec::new())),
            then_expr: Box::new(async_input_value(inner_value, inner, opaque_classes)?),
            else_expr: Some(Box::new(runtime_call("async_from_none", Vec::new()))),
        });
    }
    if is_object(ty) {
        return Some(runtime_call("async_from_object", vec![reference(name)]));
    }
    if matches!(ty.resolve_alias(), Type::Class { name: class_name, .. } if opaque_classes.contains_key(class_name))
    {
        return Some(runtime_call(
            "async_from_object",
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
            let converted = mapped_results(
                method(RustExpr::Ident(name.to_string()), "iter", Vec::new()),
                "__sifr_python_item",
                async_input_conversion_borrowed("__sifr_python_item", item, opaque_classes)?,
            );
            Some(runtime_call("async_from_list_results", vec![converted]))
        }
        Type::Tuple(items) => Some(runtime_call(
            "async_from_tuple_results",
            vec![RustExpr::Vec(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        async_input_conversion(&format!("{name}.{index}"), item, opaque_classes)
                    })
                    .collect::<Option<Vec<_>>>()?,
            )],
        )),
        Type::Dict(key, value) if key.resolve_alias() == &Type::Str => {
            let pair = RustExpr::Tuple(vec![
                RustExpr::Clone(Box::new(RustExpr::Ident("__sifr_python_key".to_string()))),
                async_input_conversion_borrowed("__sifr_python_value", value, opaque_classes)?,
            ]);
            Some(runtime_call(
                "async_from_dict_results",
                vec![mapped_results(
                    method(RustExpr::Ident(name.to_string()), "iter", Vec::new()),
                    "(__sifr_python_key, __sifr_python_value)",
                    pair,
                )],
            ))
        }
        Type::Class {
            name: class_name,
            fields,
            ..
        } if !fields.is_empty() && !opaque_classes.contains_key(class_name) => Some(runtime_call(
            "async_from_record_results",
            vec![RustExpr::Vec(
                fields
                    .iter()
                    .map(|(field, field_type)| {
                        Some(RustExpr::Tuple(vec![
                            owned_string(field),
                            async_input_conversion(
                                &format!("{name}.{field}"),
                                field_type,
                                opaque_classes,
                            )?,
                        ]))
                    })
                    .collect::<Option<Vec<_>>>()?,
            )],
        )),
        Type::None => Some(runtime_call("async_from_none", Vec::new())),
        Type::Bool => Some(runtime_call(
            "async_from_bool",
            vec![RustExpr::Ident(name.to_string())],
        )),
        Type::Int => Some(runtime_call(
            "async_from_int",
            vec![RustExpr::Ident(name.to_string())],
        )),
        Type::Float => Some(runtime_call(
            "async_from_float",
            vec![RustExpr::Ident(name.to_string())],
        )),
        Type::Str => Some(runtime_call("async_from_str", vec![reference(name)])),
        Type::Bytes => Some(runtime_call("async_from_bytes", vec![reference(name)])),
        _ => None,
    }
}

fn async_input_conversion_borrowed(
    name: &str,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if matches!(ty.resolve_alias(), Type::Bool | Type::Int | Type::Float) {
        let function = match ty.resolve_alias() {
            Type::Bool => "async_from_bool",
            Type::Int => "async_from_int",
            Type::Float => "async_from_float",
            _ => return None,
        };
        return Some(runtime_call(
            function,
            vec![RustExpr::Deref(Box::new(RustExpr::Ident(name.to_string())))],
        ));
    }
    async_input_conversion(name, ty, opaque_classes)
}

fn async_input_value(
    value: RustExpr,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if matches!(ty.resolve_alias(), Type::Bool | Type::Int | Type::Float) {
        let function = match ty.resolve_alias() {
            Type::Bool => "async_from_bool",
            Type::Int => "async_from_int",
            Type::Float => "async_from_float",
            _ => return None,
        };
        return Some(runtime_call(function, vec![value]));
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_python_nested".to_string(),
            ty: None,
            value,
        }],
        expr: Some(Box::new(async_input_conversion(
            "__sifr_python_nested",
            ty,
            opaque_classes,
        )?)),
    })
}

fn output_schema(
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        return Some(schema_variant(
            "Option",
            vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![output_schema(inner, opaque_classes)?],
            }],
        ));
    }
    if is_object(ty) {
        return Some(schema_variant("Object", Vec::new()));
    }
    if let Type::Class { name, .. } = ty.resolve_alias() {
        if let Some(declaration) = opaque_classes.get(name) {
            let target = declaration.target.as_ref()?;
            return Some(schema_variant(
                "Opaque",
                vec![RustExpr::Vec(
                    target
                        .segments
                        .iter()
                        .map(|segment| owned_string(segment))
                        .collect(),
                )],
            ));
        }
    }
    match ty.resolve_alias() {
        Type::None => Some(schema_variant("None", Vec::new())),
        Type::Bool => Some(schema_variant("Bool", Vec::new())),
        Type::Int => Some(schema_variant("Int", Vec::new())),
        Type::Float => Some(schema_variant("Float", Vec::new())),
        Type::Str => Some(schema_variant("Str", Vec::new())),
        Type::Bytes => Some(schema_variant("Bytes", Vec::new())),
        Type::List(item) => Some(schema_variant(
            "List",
            vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![output_schema(item, opaque_classes)?],
            }],
        )),
        Type::Tuple(items) => Some(schema_variant(
            "Tuple",
            vec![RustExpr::Vec(
                items
                    .iter()
                    .map(|item| output_schema(item, opaque_classes))
                    .collect::<Option<Vec<_>>>()?,
            )],
        )),
        Type::Dict(key, value) if key.resolve_alias() == &Type::Str => Some(schema_variant(
            "Dict",
            vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![output_schema(value, opaque_classes)?],
            }],
        )),
        Type::Class { fields, .. } if !fields.is_empty() => Some(schema_variant(
            "Record",
            vec![RustExpr::Vec(
                fields
                    .iter()
                    .map(|(field, field_type)| {
                        Some(RustExpr::Tuple(vec![
                            owned_string(field),
                            output_schema(field_type, opaque_classes)?,
                        ]))
                    })
                    .collect::<Option<Vec<_>>>()?,
            )],
        )),
        _ => None,
    }
}

fn async_output_value(
    name: &str,
    ty: &Type,
    error_type: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        return Some(RustExpr::If {
            cond: Box::new(runtime_call("async_value_is_none", vec![reference(name)])),
            then_expr: Box::new(RustExpr::Literal(RustLiteral::None)),
            else_expr: Some(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![async_output_value(name, inner, error_type, opaque_classes)?],
            })),
        });
    }
    if is_object(ty) {
        return Some(mapped_try(
            runtime_call("async_to_object", vec![RustExpr::Ident(name.to_string())]),
            error_type,
        ));
    }
    if let Type::Class {
        name: class_name, ..
    } = ty.resolve_alias()
    {
        if opaque_classes.contains_key(class_name) {
            return Some(RustExpr::StructInit {
                name: class_name.clone(),
                fields: vec![(
                    "__sifr_python_object".to_string(),
                    mapped_try(
                        runtime_call("async_to_object", vec![RustExpr::Ident(name.to_string())]),
                        error_type,
                    ),
                )],
            });
        }
    }
    match ty.resolve_alias() {
        Type::List(item) => {
            let converted =
                async_output_value("__sifr_python_item", item, error_type, opaque_classes)?;
            Some(RustExpr::Block {
                stmts: vec![
                    mapped_let(
                        "__sifr_python_items",
                        runtime_call("async_list_items", vec![RustExpr::Ident(name.to_string())]),
                        error_type,
                    ),
                    vector_let("__sifr_python_values"),
                    RustStmt::For {
                        var: "__sifr_python_item".to_string(),
                        iter: method(
                            RustExpr::Ident("__sifr_python_items".to_string()),
                            "into_iter",
                            Vec::new(),
                        ),
                        body: vec![
                            RustStmt::Let {
                                mutable: false,
                                name: "__sifr_python_value".to_string(),
                                ty: None,
                                value: converted,
                            },
                            push_to("__sifr_python_values", "__sifr_python_value"),
                        ],
                    },
                ],
                expr: Some(Box::new(RustExpr::Ident(
                    "__sifr_python_values".to_string(),
                ))),
            })
        }
        Type::Tuple(items) => {
            let mut stmts = vec![mapped_mutable_let(
                "__sifr_python_items",
                runtime_call("async_tuple_items", vec![RustExpr::Ident(name.to_string())]),
                error_type,
            )];
            let mut values = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let binding = format!("__sifr_python_tuple_{index}");
                stmts.push(RustStmt::Let {
                    mutable: false,
                    name: binding.clone(),
                    ty: None,
                    value: method(
                        RustExpr::Ident("__sifr_python_items".to_string()),
                        "remove",
                        vec![RustExpr::Literal(RustLiteral::Int(0))],
                    ),
                });
                values.push(async_output_value(
                    &binding,
                    item,
                    error_type,
                    opaque_classes,
                )?);
            }
            Some(RustExpr::Block {
                stmts,
                expr: Some(Box::new(RustExpr::Tuple(values))),
            })
        }
        Type::Dict(key, value) if key.resolve_alias() == &Type::Str => {
            let converted =
                async_output_value("__sifr_python_item", value, error_type, opaque_classes)?;
            Some(RustExpr::Block {
                stmts: vec![
                    mapped_let(
                        "__sifr_python_items",
                        runtime_call("async_dict_items", vec![RustExpr::Ident(name.to_string())]),
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
                        iter: RustExpr::Ident("__sifr_python_items".to_string()),
                        body: vec![RustStmt::Expr(method(
                            RustExpr::Ident("__sifr_python_values".to_string()),
                            "insert",
                            vec![RustExpr::Ident("__sifr_python_key".to_string()), converted],
                        ))],
                    },
                ],
                expr: Some(Box::new(RustExpr::Ident(
                    "__sifr_python_values".to_string(),
                ))),
            })
        }
        Type::Class {
            name: class_name,
            fields,
            ..
        } if !fields.is_empty() => {
            let mut stmts = vec![RustStmt::Let {
                mutable: true,
                name: "__sifr_python_record".to_string(),
                ty: None,
                value: RustExpr::Ident(name.to_string()),
            }];
            let mut converted = Vec::with_capacity(fields.len());
            for (field, field_type) in fields {
                let binding = format!("__sifr_python_field_{field}");
                stmts.push(mapped_let(
                    &binding,
                    runtime_call(
                        "async_record_field",
                        vec![
                            RustExpr::Ref {
                                mutable: true,
                                expr: Box::new(RustExpr::Ident("__sifr_python_record".to_string())),
                            },
                            RustExpr::Literal(RustLiteral::Str(field.clone())),
                        ],
                    ),
                    error_type,
                ));
                converted.push((
                    field.clone(),
                    async_output_value(&binding, field_type, error_type, opaque_classes)?,
                ));
            }
            Some(RustExpr::Block {
                stmts,
                expr: Some(Box::new(RustExpr::StructInit {
                    name: class_name.clone(),
                    fields: converted,
                })),
            })
        }
        Type::None => primitive_output(name, "async_to_none", error_type),
        Type::Bool => primitive_output(name, "async_to_bool", error_type),
        Type::Int => primitive_output(name, "async_to_int", error_type),
        Type::Float => primitive_output(name, "async_to_float", error_type),
        Type::Str => primitive_output(name, "async_to_str", error_type),
        Type::Bytes => primitive_output(name, "async_to_bytes", error_type),
        _ => None,
    }
}

fn primitive_output(name: &str, function: &str, error_type: &Type) -> Option<RustExpr> {
    Some(mapped_try(
        runtime_call(function, vec![RustExpr::Ident(name.to_string())]),
        error_type,
    ))
}

fn schema_variant(name: &str, args: Vec<RustExpr>) -> RustExpr {
    if args.is_empty() {
        python_path(&["PythonAsyncType", name])
    } else {
        RustExpr::FnCall {
            func: Box::new(python_path(&["PythonAsyncType", name])),
            args,
        }
    }
}

fn python_path(parts: &[&str]) -> RustExpr {
    RustExpr::Path(
        ["sifr_runtime", "python"]
            .into_iter()
            .chain(parts.iter().copied())
            .map(str::to_string)
            .collect(),
    )
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

fn is_object(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Class { name, .. } if name == "Object")
}

fn mapped_results(iter: RustExpr, parameter: &str, body: RustExpr) -> RustExpr {
    method(
        method(
            iter,
            "map",
            vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: parameter.to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(body),
                is_move: false,
            }],
        ),
        "collect",
        Vec::new(),
    )
}

fn mapped_let(name: &str, value: RustExpr, error_type: &Type) -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: name.to_string(),
        ty: None,
        value: mapped_try(value, error_type),
    }
}

fn mapped_mutable_let(name: &str, value: RustExpr, error_type: &Type) -> RustStmt {
    RustStmt::Let {
        mutable: true,
        name: name.to_string(),
        ty: None,
        value: mapped_try(value, error_type),
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

fn push_positional(value: &str) -> RustStmt {
    push_to("__sifr_python_args", value)
}

fn push_keyword(name: &str, value: &str) -> RustStmt {
    push_keyword_expr(owned_string(name), value)
}

fn push_keyword_expr(key: RustExpr, value: &str) -> RustStmt {
    RustStmt::Expr(method(
        RustExpr::Ident("__sifr_python_kwargs".to_string()),
        "push",
        vec![RustExpr::Tuple(vec![
            key,
            RustExpr::Ident(value.to_string()),
        ])],
    ))
}

fn push_to(vector: &str, value: &str) -> RustStmt {
    RustStmt::Expr(method(
        RustExpr::Ident(vector.to_string()),
        "push",
        vec![RustExpr::Ident(value.to_string())],
    ))
}

fn reference(name: &str) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Ident(name.to_string())),
    }
}

fn owned_string(value: &str) -> RustExpr {
    method(
        RustExpr::Literal(RustLiteral::Str(value.to_string())),
        "to_string",
        Vec::new(),
    )
}

fn method(receiver: RustExpr, method: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: method.to_string(),
        args,
    }
}
