use super::*;
use crate::{RustLiteral, RustParam, Visibility};

#[test]
fn catches_duplicate_struct_fields() {
    let items = vec![RustItem::Struct {
        name: "User".to_string(),
        visibility: Visibility::Private,
        derives: vec![],
        fields: vec![
            ("id".to_string(), RustType::I64),
            ("id".to_string(), RustType::I64),
        ],
    }];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::DuplicateStructField
            && issue.message.contains("duplicate field")
    }));
}

#[test]
fn catches_empty_function_body() {
    let items = vec![RustItem::Fn {
        name: "empty".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![],
        ret: None,
        body: vec![],
        is_async: false,
    }];

    let issues = validate_items(&items);
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == IrValidationKind::EmptyFunctionBody)
    );
}

#[test]
fn catches_return_outside_function() {
    let items = vec![RustItem::Const {
        name: "X".to_string(),
        visibility: Visibility::Private,
        ty: RustType::I64,
        value: RustExpr::Block {
            stmts: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                1,
            ))))],
            expr: Some(Box::new(RustExpr::Literal(RustLiteral::Int(0)))),
        },
    }];

    let issues = validate_items(&items);
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == IrValidationKind::ReturnOutsideFunction)
    );
}

#[test]
fn accepts_balanced_ir() {
    let items = vec![RustItem::Fn {
        name: "ok".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "n".to_string(),
            ty: RustType::I64,
        }],
        ret: Some(RustType::I64),
        body: vec![RustStmt::Return(Some(RustExpr::Ident("n".to_string())))],
        is_async: false,
    }];

    let issues = validate_items(&items);
    assert!(issues.is_empty());
}

#[test]
fn rejects_raw_syntax_in_identifier_nodes() {
    let items = vec![RustItem::Const {
        name: "BROKEN".to_string(),
        visibility: Visibility::Private,
        ty: RustType::I64,
        value: RustExpr::Ident("std::cmp::max(1, 2)".to_string()),
    }];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::InvalidIdentifier
            && issue.message.contains("std::cmp::max(1, 2)")
    }));
}

#[test]
fn validates_explicit_verbatim_expression_nodes() {
    let valid = vec![RustItem::Const {
        name: "VALUE".to_string(),
        visibility: Visibility::Private,
        ty: RustType::I64,
        value: RustExpr::Verbatim("std::cmp::max(1, 2)".to_string()),
    }];
    assert!(validate_items(&valid).is_empty());

    let invalid = vec![RustItem::Const {
        name: "BROKEN".to_string(),
        visibility: Visibility::Private,
        ty: RustType::I64,
        value: RustExpr::Verbatim("std::cmp::max(".to_string()),
    }];
    assert!(
        validate_items(&invalid)
            .iter()
            .any(|issue| issue.kind == IrValidationKind::InvalidVerbatimExpression)
    );
}

#[test]
fn rejects_structured_failure_discharge_before_rendering() {
    let items = vec![RustItem::Fn {
        name: "broken".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![],
        ret: None,
        body: vec![RustStmt::Expr(RustExpr::MacroCall {
            name: "unreachable".to_string(),
            args: vec![],
        })],
        is_async: false,
    }];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::ForbiddenFailureDischarge
            && issue.message.contains("unreachable!")
    }));
}

#[test]
fn rejects_failure_discharge_in_every_structured_macro_variant() {
    let items = vec![RustItem::Const {
        name: "BROKEN".to_string(),
        visibility: Visibility::Private,
        ty: RustType::String_,
        value: RustExpr::FormatMacro {
            name: "panic".to_string(),
            format_str: "{}".to_string(),
            args: vec![RustExpr::Literal(RustLiteral::Str("boom".to_string()))],
        },
    }];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::ForbiddenFailureDischarge
            && issue.message.contains("panic!")
    }));
}

#[test]
fn permits_source_methods_named_like_failure_discharge() {
    for method in ["unwrap", "expect"] {
        let source_call = RustExpr::SourceMethodCall {
            receiver: Box::new(RustExpr::Ident("value".to_string())),
            method: method.to_string(),
            args: vec![],
        };
        let items = vec![RustItem::Fn {
            name: "source_method".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(source_call)],
            is_async: false,
        }];
        assert!(validate_items(&items).is_empty(), "{method}");
    }
}

#[test]
fn rejects_compiler_owned_method_extraction() {
    for method in ["unwrap", "expect"] {
        let items = vec![RustItem::Fn {
            name: "internal_extraction".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("value".to_string())),
                method: method.to_string(),
                args: vec![],
            })],
            is_async: false,
        }];
        let issues = validate_items(&items);
        assert!(
            issues
                .iter()
                .any(|issue| { issue.kind == IrValidationKind::ForbiddenFailureDischarge }),
            "{method}"
        );
    }
}

#[test]
fn rejects_raw_failure_discharge_before_rendering() {
    let items = vec![RustItem::Attr(
        "fn broken(value: Option<i64>) -> i64 { value.unwrap() }".to_string(),
    )];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::ForbiddenFailureDischarge
            && issue.message.contains("unwrap call")
    }));
}

#[test]
fn rejects_unchecked_index_before_rendering() {
    let items = vec![RustItem::Fn {
        name: "broken".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![],
        ret: None,
        body: vec![RustStmt::Expr(RustExpr::Index {
            expr: Box::new(RustExpr::Ident("values".to_string())),
            index: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        })],
        is_async: false,
    }];

    let issues = validate_items(&items);
    assert!(issues.iter().any(|issue| {
        issue.kind == IrValidationKind::UncheckedIndex
            && issue.message.contains("checked-place plan")
    }));
}
