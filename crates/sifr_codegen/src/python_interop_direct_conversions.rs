use crate::python_interop_direct::{
    mapped_try, push_to, reference, runtime_call, value_place, vector_let,
};
use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_ir::PythonInteropDeclaration;
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) fn input_conversion(
    name: &str,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        let receiver = value_place(name);
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
                    expr: Box::new(value_place(name)),
                    field: "__sifr_python_object".to_string(),
                }),
            }],
        ));
    }
    match ty.resolve_alias() {
        Type::List(item) => {
            let iter = RustExpr::MethodCall {
                receiver: Box::new(value_place(name)),
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
                receiver: Box::new(value_place(name)),
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
    let ident = value_place(name);
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
    output_value_expr_with(value_name, ty, Some(error_type), opaque_classes)
}

pub(crate) fn callback_output_value_expr(
    value_name: &str,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    output_value_expr_with(value_name, ty, None, opaque_classes)
}

fn output_value_expr_with(
    value_name: &str,
    ty: &Type,
    error_type: Option<&Type>,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        let is_none = conversion_try(
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
                    args: vec![output_value_expr_with(
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
        return Some(runtime_call(
            "__sifr_declaration_object_result",
            vec![RustExpr::Ident(value_name.to_string())],
        ));
    }
    if let class @ Type::Class { name, .. } = ty.resolve_alias() {
        if let Some(opaque) = opaque_classes.get(name) {
            let target = opaque.target.as_ref()?;
            let checked = conversion_try(
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
            return Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    crate::render_type(&crate::sifr_type_to_rust_type(class)),
                    "__sifr_from_python_object".to_string(),
                ])),
                args: vec![checked],
            });
        }
    }
    match ty.resolve_alias() {
        Type::List(item) => {
            let mut body = vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_value".to_string(),
                ty: None,
                value: output_value_expr_with(
                    "__sifr_python_item",
                    item,
                    error_type,
                    opaque_classes,
                )?,
            }];
            body.push(push_to("__sifr_python_values", "__sifr_python_value"));
            return Some(RustExpr::Block {
                stmts: vec![
                    conversion_let(
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
            let mut statements = vec![conversion_let(
                "__sifr_python_items",
                runtime_call(
                    "tuple_items_exact",
                    vec![
                        reference(value_name),
                        RustExpr::Literal(RustLiteral::Int(i64::try_from(items.len()).ok()?)),
                    ],
                ),
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
                values.push(output_value_expr_with(
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
                    value: conversion_try(
                        runtime_call(
                            "tuple_items_exact",
                            vec![
                                reference(value_name),
                                RustExpr::Literal(RustLiteral::Int(
                                    i64::try_from(items.len()).ok()?,
                                )),
                            ],
                        ),
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
                output_value_expr_with("__sifr_python_item", value, error_type, opaque_classes)?;
            return Some(RustExpr::Block {
                stmts: vec![
                    conversion_let(
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
        class @ Type::Class { fields, .. } if !fields.is_empty() => {
            let mut statements = Vec::new();
            let mut converted_fields = Vec::new();
            for (field, field_type) in fields {
                let handle = format!("__sifr_python_field_{field}");
                statements.push(conversion_let(
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
                    output_value_expr_with(&handle, field_type, error_type, opaque_classes)?,
                ));
            }
            return Some(RustExpr::Block {
                stmts: statements,
                expr: Some(Box::new(RustExpr::StructInit {
                    name: crate::render_type(&crate::sifr_type_to_rust_type(class)),
                    fields: converted_fields,
                })),
            });
        }
        _ => {}
    }
    Some(conversion_try(
        output_conversion(value_name, ty)?,
        error_type,
    ))
}

fn conversion_try(value: RustExpr, error_type: Option<&Type>) -> RustExpr {
    match error_type {
        Some(error_type) => mapped_try(value, error_type),
        None => RustExpr::Try(Box::new(value)),
    }
}

fn conversion_let(name: &str, value: RustExpr, error_type: Option<&Type>) -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: name.to_string(),
        ty: None,
        value: conversion_try(value, error_type),
    }
}

fn input_conversion_value(
    value: RustExpr,
    ty: &Type,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<RustExpr> {
    if is_python_object(ty) {
        return Some(runtime_call(
            "__sifr_declaration_object_argument",
            vec![value],
        ));
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

pub(crate) fn input_conversion_borrowed(
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
        Type::Bool | Type::Int | Type::Float => RustExpr::Deref(Box::new(value_place(name))),
        Type::Str | Type::Bytes => value_place(name),
        _ if is_python_object(ty) => {
            return Some(runtime_call(
                "__sifr_declaration_object_argument",
                vec![value_place(name)],
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
                        expr: Box::new(value_place(name)),
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

pub(crate) fn output_conversion(name: &str, ty: &Type) -> Option<RustExpr> {
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

pub(crate) fn is_python_object(ty: &Type) -> bool {
    ty.is_python_object_contract()
}
