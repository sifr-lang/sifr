use sifr_type_system::Type;

use crate::{RustExpr, RustType};

pub(crate) fn bridge_error_expr(value: RustExpr, err_type: &Type) -> RustExpr {
    if let Type::Alias { name, body, .. } = err_type {
        if matches!(
            body.resolve_alias(),
            Type::Class { fields, .. }
                if is_message_error_alias(name) && message_error_fields(fields).is_some()
        ) {
            return RustExpr::StructInit {
                name: name.clone(),
                fields: vec![("message".to_string(), to_string_expr(value))],
            };
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
                args: vec![bridge_error_expr(value, python_error)],
            }),
        Type::Class {
            name,
            fields,
            parent_class: _,
            ..
        } if is_message_error_alias(name) && message_error_fields(fields).is_some() => {
            RustExpr::StructInit {
                name: name.clone(),
                fields: vec![("message".to_string(), to_string_expr(value))],
            }
        }
        Type::Class { name, .. } if err_type.is_python_error_contract() => {
            python_error_expr(name, value)
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
        Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JSONDecodeError" => {
            json_decode_error_expr(name, value)
        }
        Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JsonLimitError" => {
            json_limit_error_expr(name, value)
        }
        Type::Class {
            name,
            fields: _,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") && name == "JsonIntegerRangeError" => {
            json_integer_range_error_expr(name, value)
        }
        Type::Class {
            name,
            fields,
            parent_class,
            ..
        } if parent_class.as_deref() == Some("Error") => {
            if let Some(error_fields) = message_error_fields(fields) {
                RustExpr::StructInit {
                    name: name.clone(),
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
                bridge_error_method_i64(value.clone(), "line"),
            ),
            (
                "column".to_string(),
                bridge_error_method_i64(value, "column"),
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
            ("limit".to_string(), bridge_error_method_i64(value, "limit")),
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

fn bridge_error_method_i64(value: RustExpr, method: &str) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(value),
            method: method.to_string(),
            args: Vec::new(),
        }),
        ty: RustType::I64,
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
mod tests {
    use super::bridge_error_expr;
    use crate::{render_expr, RustExpr};
    use sifr_type_system::Type;

    #[test]
    fn runtime_diagnostic_string_errors_map_to_the_declared_error() {
        let diagnostic_error = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "DiagnosticError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: None,
        };

        let mapped = bridge_error_expr(
            RustExpr::Ident("__sifr_bridge_error".to_string()),
            &diagnostic_error,
        );

        assert_eq!(
            render_expr(&mapped),
            "DiagnosticError { message: __sifr_bridge_error.to_string() }"
        );
    }

    #[test]
    fn shadow_python_error_does_not_use_the_runtime_struct_mapping() {
        let shadow = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "PythonError".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                ("kind".to_string(), Type::Str),
                ("exception_type".to_string(), Type::Str),
                ("traceback".to_string(), Type::Str),
                ("context".to_string(), Type::Str),
                ("code".to_string(), Type::Int),
            ],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let mapped = bridge_error_expr(RustExpr::Ident("__sifr_bridge_error".to_string()), &shadow);

        assert_eq!(render_expr(&mapped), "__sifr_bridge_error");
    }
}
