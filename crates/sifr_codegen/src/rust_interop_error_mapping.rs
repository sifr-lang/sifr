use sifr_type_system::Type;

use crate::RustExpr;

fn rust_type_name(ty: &Type) -> String {
    crate::render_type(&crate::sifr_type_to_rust_type(ty))
}

pub(crate) fn bridge_error_expr(value: RustExpr, err_type: &Type) -> RustExpr {
    bridge_error_expr_with_contract(value, err_type, false)
}

pub(crate) fn bridge_error_contract_expr(value: RustExpr, err_type: &Type) -> RustExpr {
    bridge_error_expr_with_contract(value, err_type, true)
}

fn bridge_error_expr_with_contract(
    value: RustExpr,
    err_type: &Type,
    declared_error_contract: bool,
) -> RustExpr {
    if let Type::Alias { name, body, .. } = err_type {
        if let Type::Class { fields, .. } = body.resolve_alias() {
            if is_message_error_alias(name) {
                if let Some(error_fields) = message_error_fields(fields) {
                    return RustExpr::StructInit {
                        name: name.clone(),
                        fields: error_fields
                            .into_iter()
                            .map(|field| (field, to_string_expr(value.clone())))
                            .collect(),
                    };
                }
            }
        }
    }
    match err_type.resolve_alias() {
        Type::Union(members) => members
            .iter()
            .find(|member| member.is_python_error_contract())
            .map_or(value.clone(), |python_error| RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    err_type.resolve_alias().union_enum_name(),
                    python_error.union_variant_name(),
                ])),
                args: vec![bridge_error_expr_with_contract(
                    value,
                    python_error,
                    declared_error_contract,
                )],
            }),
        class @ Type::Class { name, fields, .. }
            if is_message_error_alias(name) && message_error_fields(fields).is_some() =>
        {
            RustExpr::StructInit {
                name: rust_type_name(class),
                fields: message_error_fields(fields)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|field| (field, to_string_expr(value.clone())))
                    .collect(),
            }
        }
        class @ Type::Class { .. } if err_type.is_python_error_contract() => {
            python_error_expr(&rust_type_name(class), value)
        }
        Type::Class {
            name,
            fields,
            parent_class,
            ..
        } if name == "IOError"
            && parent_class.as_deref() == Some("Error")
            && io_error_fields(fields) =>
        {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("__io_err".to_string())),
                args: vec![value],
            }
        }
        class @ Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JSONDecodeError" => {
            json_decode_error_expr(&rust_type_name(class), value)
        }
        class @ Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JsonLimitError" => {
            json_limit_error_expr(&rust_type_name(class), value)
        }
        class @ Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JsonIntegerRangeError" => {
            json_integer_range_error_expr(&rust_type_name(class), value)
        }
        class @ Type::Class {
            name: _,
            fields,
            parent_class,
            ..
        } if declared_error_contract || parent_class.as_deref() == Some("Error") => {
            if let Some(error_fields) = message_error_fields(fields) {
                RustExpr::StructInit {
                    name: rust_type_name(class),
                    fields: error_fields
                        .into_iter()
                        .map(|field| (field, to_string_expr(value.clone())))
                        .collect(),
                }
            } else {
                value
            }
        }
        _ => value,
    }
}

fn is_message_error_alias(name: &str) -> bool {
    matches!(
        name,
        "DiagnosticError"
            | "ProcessError"
            | "NetError"
            | "TlsError"
            | "HeaderError"
            | "HttpError"
            | "SignalError"
    )
}

fn json_decode_error_expr(name: &str, value: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: name.to_string(),
        fields: vec![
            (
                "message".to_string(),
                bridge_error_method_string(value.clone(), "message"),
            ),
            (
                "line".to_string(),
                bridge_error_method_int(value.clone(), "line"),
            ),
            (
                "column".to_string(),
                bridge_error_method_int(value, "column"),
            ),
        ],
    }
}

fn json_limit_error_expr(name: &str, value: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: name.to_string(),
        fields: vec![
            (
                "message".to_string(),
                bridge_error_method_string(value.clone(), "message"),
            ),
            ("limit".to_string(), bridge_error_method_int(value, "limit")),
        ],
    }
}

fn json_integer_range_error_expr(name: &str, value: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: name.to_string(),
        fields: vec![
            (
                "message".to_string(),
                bridge_error_method_string(value.clone(), "message"),
            ),
            (
                "path".to_string(),
                bridge_error_method_string(value.clone(), "path"),
            ),
            (
                "profile".to_string(),
                bridge_error_method_string(value, "profile"),
            ),
        ],
    }
}

fn python_error_expr(name: &str, value: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: name.to_string(),
        fields: vec![
            (
                "message".to_string(),
                bridge_error_field_string(value.clone(), "message"),
            ),
            (
                "kind".to_string(),
                bridge_error_field_string(value.clone(), "kind"),
            ),
            (
                "exception_type".to_string(),
                bridge_error_field_string(value.clone(), "exception_type"),
            ),
            (
                "traceback".to_string(),
                bridge_error_field_string(value.clone(), "traceback"),
            ),
            (
                "context".to_string(),
                bridge_error_field_string(value.clone(), "context"),
            ),
            (
                "__sifr_python_error".to_string(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![value],
                },
            ),
        ],
    }
}

fn io_error_fields(fields: &[(String, Type)]) -> bool {
    ["message", "kind"].iter().all(|expected_name| {
        fields
            .iter()
            .any(|(name, ty)| name == expected_name && ty.resolve_alias() == &Type::Str)
    })
}

fn bridge_error_field_string(value: RustExpr, field: &str) -> RustExpr {
    to_string_expr(RustExpr::Field {
        expr: Box::new(value),
        field: field.to_string(),
    })
}

fn bridge_error_method_string(value: RustExpr, method: &str) -> RustExpr {
    to_string_expr(RustExpr::MethodCall {
        receiver: Box::new(value),
        method: method.to_string(),
        args: Vec::new(),
    })
}

fn bridge_error_method_int(value: RustExpr, method: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(value),
            method: method.to_string(),
            args: Vec::new(),
        }],
    }
}

fn message_error_fields(fields: &[(String, Type)]) -> Option<Vec<String>> {
    let all_fields_are_strings = fields
        .iter()
        .all(|(_name, ty)| ty.resolve_alias() == &Type::Str);
    if !all_fields_are_strings {
        return None;
    }
    let field_names = fields
        .iter()
        .map(|(name, _ty)| name.clone())
        .collect::<Vec<_>>();
    if field_names.iter().any(|name| name == "message") {
        Some(field_names)
    } else {
        None
    }
}

fn to_string_expr(expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "to_string".to_string(),
        args: Vec::new(),
    }
}

#[cfg(test)]
#[path = "../tests/unit/rust_interop_error_mapping/tests.rs"]
mod tests;
