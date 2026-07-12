use sifr_ir::{
    HirFunction, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::Type;

use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustExpr, RustLiteral, RustParam, RustStmt, RustType};

pub(crate) fn python_interop_function_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
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
                input_conversion(&value_name, &param.ty)?,
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
                    input_conversion(&param.name, &param.ty)?
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
                            input_conversion_borrowed(&value_name, element_type)?,
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

    let converted = if is_python_object(ok_type) {
        RustExpr::Ident("__sifr_python_result".to_string())
    } else {
        mapped_try(
            output_conversion("__sifr_python_result", ok_type)?,
            error_type,
        )
    };
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![converted],
    })));
    Some(body)
}

pub(crate) fn python_omit_parameter_indices(
    declaration: &PythonInteropDeclaration,
) -> impl Iterator<Item = usize> + '_ {
    declaration
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.omit_when_absent.then_some(index))
}

fn input_conversion(name: &str, ty: &Type) -> Option<RustExpr> {
    if let Some(inner) = option_inner(ty) {
        let receiver = RustExpr::Ident(name.to_string());
        let borrowed_inner =
            matches!(inner.resolve_alias(), Type::Str | Type::Bytes) || is_python_object(inner);
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
            then_expr: Box::new(input_conversion_value(inner_value, inner)?),
            else_expr: Some(Box::new(runtime_call("from_none", Vec::new()))),
        });
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

fn input_conversion_value(value: RustExpr, ty: &Type) -> Option<RustExpr> {
    if is_python_object(ty) {
        return Some(runtime_call("temporary_argument_handle", vec![value]));
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

fn input_conversion_borrowed(name: &str, ty: &Type) -> Option<RustExpr> {
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

fn runtime_call(function: &str, args: Vec<RustExpr>) -> RustExpr {
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

fn mapped_try(value: RustExpr, error_type: &Type) -> RustExpr {
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
