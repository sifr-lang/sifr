use super::{bridge_error_contract_expr, bridge_error_expr};
use crate::{RustExpr, render_expr};
use sifr_type_system::Type;

#[test]
fn runtime_diagnostic_string_errors_map_to_the_declared_error() {
    let diagnostic_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "DiagnosticError".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
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
fn reserved_message_errors_initialize_every_declared_string_field() {
    let http_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "HttpError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("detail".to_string(), Type::Str),
        ]
        .into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };

    let mapped = bridge_error_contract_expr(
        RustExpr::Ident("__sifr_bridge_error".to_string()),
        &http_error,
    );

    assert_eq!(
        render_expr(&mapped),
        "HttpError { message: __sifr_bridge_error.to_string(), detail: __sifr_bridge_error.to_string() }"
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
        ]
        .into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let mapped = bridge_error_expr(RustExpr::Ident("__sifr_bridge_error".to_string()), &shadow);

    assert_eq!(render_expr(&mapped), "__sifr_bridge_error");
}

#[test]
fn bridge_error_mapping_uses_the_nominal_rust_identity() {
    let declared = Type::Class {
        identity: Some("local.__SifrBridgeError".to_string()),
        type_args: Vec::new(),
        name: "__SifrBridgeError".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };

    let mapped = bridge_error_expr(
        RustExpr::Ident("__sifr_bridge_error".to_string()),
        &declared,
    );

    assert_eq!(
        render_expr(&mapped),
        format!(
            "{} {{ message: __sifr_bridge_error.to_string() }}",
            super::rust_type_name(&declared)
        )
    );
}

#[test]
fn multi_field_errors_initialize_every_declared_string_field() {
    let declared = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "DetailedError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("context".to_string(), Type::Str),
        ]
        .into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };

    let mapped = bridge_error_expr(
        RustExpr::Ident("__sifr_bridge_error".to_string()),
        &declared,
    );

    assert_eq!(
        render_expr(&mapped),
        "DetailedError { message: __sifr_bridge_error.to_string(), context: __sifr_bridge_error.to_string() }"
    );
}

#[test]
fn ordinary_message_records_are_not_rewritten_as_errors() {
    let record = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "MessageRecord".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: None,
    };

    let mapped = bridge_error_expr(RustExpr::Ident("__sifr_bridge_value".to_string()), &record);

    assert_eq!(render_expr(&mapped), "__sifr_bridge_value");
}

#[test]
fn declared_error_contract_maps_when_parent_metadata_is_erased() {
    let declared = Type::Class {
        identity: Some("app.PanicMapped".to_string()),
        type_args: Vec::new(),
        name: "PanicMapped".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: None,
    };

    let mapped = bridge_error_contract_expr(
        RustExpr::Ident("__sifr_bridge_error".to_string()),
        &declared,
    );

    assert_eq!(
        render_expr(&mapped),
        format!(
            "{} {{ message: __sifr_bridge_error.to_string() }}",
            super::rust_type_name(&declared)
        )
    );
}

#[test]
fn ordinary_error_aliases_construct_the_resolved_nominal_type() {
    let declared = Type::Class {
        identity: Some("app.PayloadError".to_string()),
        type_args: Vec::new(),
        name: "PayloadError".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let alias = Type::Alias {
        name: "PayloadAlias".to_string(),
        type_args: Vec::new(),
        body: Box::new(declared.clone()),
    };

    let mapped = bridge_error_expr(RustExpr::Ident("__sifr_bridge_error".to_string()), &alias);

    assert_eq!(
        render_expr(&mapped),
        format!(
            "{} {{ message: __sifr_bridge_error.to_string() }}",
            super::rust_type_name(&declared)
        )
    );
}
