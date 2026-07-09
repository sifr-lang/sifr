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
                name: name.to_string(),
                fields: vec![("message".to_string(), to_string_expr(value))],
            };
        }
    }
    match err_type.resolve_alias() {
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
        "ProcessError" | "NetError" | "TlsError" | "HeaderError" | "HttpError" | "SignalError"
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
