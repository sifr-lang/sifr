//! Item lowering scaffolds for the IR lowering.

use crate::{
    try_lower_leaf_expr, try_lower_leaf_expr_result, CodegenError, RustExpr, RustItem, RustLiteral,
    RustMatchArm, RustStmt, RustType, Visibility,
};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

fn is_simple_module_primitive_const_type(ty: &Type) -> bool {
    matches!(
        resolve_alias_type(ty),
        Type::Int
            | Type::FixedInt(_)
            | Type::Float
            | Type::Bool
            | Type::LiteralInt(_)
            | Type::LiteralBool(_)
    )
}

fn is_simple_module_string_const_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Str | Type::LiteralStr(_))
}

fn is_simple_module_none_const_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

fn is_exact_module_int_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Int)
}

pub fn try_lower_simple_module_constant_item_result(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Result<Option<(RustItem, String)>, CodegenError> {
    validate_module_constant_shape(name)?;
    try_lower_simple_module_constant_item_result_impl(name, ty, value)
}

fn validate_module_constant_shape(name: &str) -> Result<(), CodegenError> {
    if name.trim().is_empty() {
        return Err(CodegenError::new(
            "invalid module constant shape: empty name",
        ));
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(CodegenError::new(
            "invalid module constant shape: empty name",
        ));
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(CodegenError::new(
            "invalid module constant shape: name must start with ASCII letter or underscore",
        ));
    }
    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        return Err(CodegenError::new(
            "invalid module constant shape: name must be ASCII identifier",
        ));
    }
    Ok(())
}

fn try_lower_leaf_or_name_expr_result(value: &HirExpr) -> Result<Option<RustExpr>, CodegenError> {
    if let Some(lowered) = try_lower_leaf_expr_result(value)? {
        return Ok(Some(lowered));
    }
    if let HirExpr::Name { name, .. } = value {
        return Ok(Some(RustExpr::Ident(name.clone())));
    }
    Ok(None)
}

fn try_lower_simple_module_constant_item_result_impl(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Result<Option<(RustItem, String)>, CodegenError> {
    if let Some(decimal_text) = large_module_int_literal_decimal(ty, value) {
        return Ok(Some(lower_large_module_int_const_item(name, &decimal_text)));
    }

    if is_simple_module_primitive_const_type(ty) {
        let lowered_value =
            if let Some(lowered) = crate::fixed_width_literal_expr_for_target(ty, value) {
                lowered
            } else {
                let Some(lowered) = try_lower_leaf_or_name_expr_result(value)? else {
                    return Ok(None);
                };
                lowered
            };
        let rust_name = name.to_uppercase();
        return Ok(Some((
            RustItem::Const {
                name: rust_name.clone(),
                visibility: Visibility::Private,
                ty: crate::sifr_type_to_rust_type(ty),
                value: lowered_value,
            },
            rust_name,
        )));
    }

    if is_simple_module_string_const_type(ty) {
        let Some(lowered_value) = try_lower_leaf_or_name_expr_result(value)? else {
            return Ok(None);
        };
        let rust_name = format!("__const_{name}");
        return Ok(Some((
            RustItem::Fn {
                name: rust_name.clone(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(RustType::String_),
                body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_value),
                    method: "to_string".to_string(),
                    args: vec![],
                }))],
                is_async: false,
            },
            format!("{rust_name}()"),
        )));
    }

    if is_simple_module_none_const_type(ty) {
        let lowered_value = if matches!(value, HirExpr::NoneLiteral) {
            RustExpr::Literal(RustLiteral::Unit)
        } else if let HirExpr::Name { name, ty } = value {
            if !is_simple_module_none_const_type(ty) {
                return Ok(None);
            }
            RustExpr::Ident(name.clone())
        } else {
            return Ok(None);
        };
        let rust_name = format!("__const_{name}");
        return Ok(Some((
            RustItem::Fn {
                name: rust_name.clone(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(RustType::Unit),
                body: vec![RustStmt::Return(Some(lowered_value))],
                is_async: false,
            },
            format!("{rust_name}()"),
        )));
    }

    let Some(lowered_value) = try_lower_leaf_or_name_expr_result(value)? else {
        return Ok(None);
    };
    let rust_name = format!("__const_{name}");
    Ok(Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(crate::sifr_type_to_rust_type(ty)),
            body: vec![RustStmt::Return(Some(lowered_value))],
            is_async: false,
        },
        format!("{rust_name}()"),
    )))
}

fn large_module_int_literal_decimal(ty: &Type, value: &HirExpr) -> Option<String> {
    if !is_exact_module_int_type(ty) {
        return None;
    }
    match value {
        HirExpr::LargeIntLiteral(value) => Some(value.clone()),
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => {
            large_module_int_literal_decimal(ty, operand)
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            let value = large_module_int_literal_decimal(ty, operand)?;
            Some(format!("-{value}"))
        }
        _ => None,
    }
}

fn sifr_int_parse_decimal_call(decimal_text: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "parse_decimal".to_string(),
        ])),
        args: vec![
            RustExpr::Ident(format!("\"{}\"", decimal_text.escape_default())),
            RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                "DEFAULT_MAX_INTEGER_DIGITS".to_string(),
            ]),
        ],
    }
}

/// Conservative dispatcher for simple module-constant item lowering.
pub fn try_lower_simple_module_constant_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if let Some(decimal_text) = large_module_int_literal_decimal(ty, value) {
        return Some(lower_large_module_int_const_item(name, &decimal_text));
    }
    try_lower_simple_module_const_item(name, ty, value)
        .or_else(|| try_lower_simple_module_string_const_item(name, ty, value))
        .or_else(|| try_lower_simple_module_none_const_item(name, ty, value))
        .or_else(|| try_lower_simple_module_helper_const_item(name, ty, value))
}

fn lower_large_module_int_const_item(name: &str, decimal_text: &str) -> (RustItem, String) {
    let rust_name = format!("__const_{name}");
    (
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Named("SifrInt".to_string())),
            body: vec![RustStmt::Match {
                expr: sifr_int_parse_decimal_call(decimal_text),
                arms: vec![
                    RustMatchArm {
                        pattern: "Ok(value)".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::Ident(
                            "value".to_string(),
                        )))],
                    },
                    RustMatchArm {
                        pattern: "Err(err)".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::Expr(RustExpr::FormatMacro {
                            name: "panic".to_string(),
                            format_str: format!(
                                "compiler emitted invalid integer literal for module constant {name}: {{}}"
                            ),
                            args: vec![RustExpr::Ident("err".to_string())],
                        })],
                    },
                ],
            }],
            is_async: false,
        },
        format!("{rust_name}()"),
    )
}

/// Conservatively lowers module-level primitive constants via IR.
/// Falls back for non-primitive or non-leaf/non-name values.
pub fn try_lower_simple_module_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if !is_simple_module_primitive_const_type(ty) {
        return None;
    }
    let rust_name = name.to_uppercase();
    let lowered_value = if let Some(lowered) = try_lower_leaf_expr(value) {
        lowered
    } else if let HirExpr::Name { name, .. } = value {
        RustExpr::Ident(name.clone())
    } else {
        return None;
    };
    Some((
        RustItem::Const {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            ty: crate::sifr_type_to_rust_type(ty),
            value: lowered_value,
        },
        rust_name,
    ))
}

/// Conservatively lowers module-level string-literal constants via IR helper function.
/// Falls back for non-literal/non-name string values.
pub fn try_lower_simple_module_string_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if !is_simple_module_string_const_type(ty) {
        return None;
    }
    let rust_name = format!("__const_{name}");
    let lowered_value = if let Some(lowered) = try_lower_leaf_expr(value) {
        lowered
    } else if let HirExpr::Name { name, .. } = value {
        RustExpr::Ident(name.clone())
    } else {
        return None;
    };
    Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::String_),
            body: vec![RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(lowered_value),
                method: "to_string".to_string(),
                args: vec![],
            }))],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

/// Conservatively lowers module-level `None` constants via IR helper function.
pub fn try_lower_simple_module_none_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if !is_simple_module_none_const_type(ty) {
        return None;
    }
    let lowered_value = if matches!(value, HirExpr::NoneLiteral) {
        RustExpr::Literal(RustLiteral::Unit)
    } else if let HirExpr::Name { name, ty } = value {
        if !is_simple_module_none_const_type(ty) {
            return None;
        }
        RustExpr::Ident(name.clone())
    } else {
        return None;
    };
    let rust_name = format!("__const_{name}");
    Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Unit),
            body: vec![RustStmt::Return(Some(lowered_value))],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

/// Conservatively lowers module-level non-primitive helper constants via IR function items.
/// Falls back for primitive/string/none types or non-leaf/non-name values.
pub fn try_lower_simple_module_helper_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if is_simple_module_primitive_const_type(ty)
        || is_simple_module_string_const_type(ty)
        || is_simple_module_none_const_type(ty)
    {
        return None;
    }
    let rust_name = format!("__const_{name}");
    let lowered_value = if let Some(lowered) = try_lower_leaf_expr(value) {
        lowered
    } else if let HirExpr::Name { name, .. } = value {
        RustExpr::Ident(name.clone())
    } else {
        return None;
    };
    Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(crate::sifr_type_to_rust_type(ty)),
            body: vec![RustStmt::Return(Some(lowered_value))],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_type_system::Type;

    #[test]
    fn dispatcher_lowers_simple_module_constant_item() {
        let (item, rust_name) =
            try_lower_simple_module_constant_item("answer", &Type::Int, &HirExpr::IntLiteral(42))
                .expect("dispatcher should lower simple constant");
        assert_eq!(rust_name, "ANSWER");
        assert!(matches!(item, RustItem::Const { .. }));
    }

    #[test]
    fn dispatcher_result_lowers_simple_module_constant_item() {
        let lowered = try_lower_simple_module_constant_item_result(
            "answer",
            &Type::Int,
            &HirExpr::IntLiteral(42),
        )
        .expect("result dispatcher should validate and lower")
        .expect("dispatcher should lower simple constant");
        assert_eq!(lowered.1, "ANSWER");
        assert!(matches!(lowered.0, RustItem::Const { .. }));
    }

    #[test]
    fn dispatcher_result_reports_invalid_module_constant_name() {
        let err = try_lower_simple_module_constant_item_result(
            "9bad",
            &Type::Int,
            &HirExpr::IntLiteral(42),
        )
        .expect_err("invalid constant name should return error");
        assert!(err
            .message
            .contains("name must start with ASCII letter or underscore"));
    }

    #[test]
    fn dispatcher_result_propagates_leaf_lowering_errors() {
        let err = try_lower_simple_module_constant_item_result(
            "answer",
            &Type::Int,
            &HirExpr::Compare {
                left: Box::new(HirExpr::IntLiteral(1)),
                ops: vec!["==".to_string()],
                comparators: vec![],
                ty: Type::Bool,
            },
        )
        .expect_err("invalid compare shape should propagate as codegen error");
        assert!(err.message.contains("ops/comparators length mismatch"));
    }

    #[test]
    fn lowers_simple_module_int_const_item() {
        let (item, rust_name) =
            try_lower_simple_module_const_item("answer", &Type::Int, &HirExpr::IntLiteral(42))
                .expect("simple const should lower");
        assert_eq!(rust_name, "ANSWER");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                ty: crate::RustType::I64,
                ..
            } if name == "ANSWER"
        ));
    }

    #[test]
    fn dispatcher_result_lowers_large_module_int_const_as_sifr_int_helper() {
        let (item, rust_name_call) = try_lower_simple_module_constant_item_result(
            "limit",
            &Type::Int,
            &HirExpr::LargeIntLiteral("100000000000000000000".to_string()),
        )
        .expect("large exact int module const should validate")
        .expect("large exact int module const should lower");

        assert_eq!(rust_name_call, "__const_limit()");
        assert!(matches!(
            item,
            RustItem::Fn {
                ref name,
                visibility: Visibility::Private,
                ret: Some(RustType::Named(ref ret)),
                ..
            } if name == "__const_limit" && ret == "SifrInt"
        ));

        let rendered = crate::render_items(&[item]);
        assert!(rendered.contains("fn __const_limit() -> SifrInt"));
        assert!(rendered.contains(
            "SifrInt::parse_decimal(\"100000000000000000000\", sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)"
        ));
        assert!(!rendered.contains(".unwrap("));
        assert!(!rendered.contains(".expect("));
    }

    #[test]
    fn lowers_simple_module_name_const_item() {
        let (item, rust_name) = try_lower_simple_module_const_item(
            "answer",
            &Type::Int,
            &HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            },
        )
        .expect("simple name const should lower");
        assert_eq!(rust_name, "ANSWER");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                ty: crate::RustType::I64,
                value: RustExpr::Ident(ident),
            } if name == "ANSWER" && ident == "x"
        ));
    }

    #[test]
    fn does_not_lower_non_primitive_module_const_item() {
        assert!(try_lower_simple_module_const_item(
            "name",
            &Type::Str,
            &HirExpr::StringLiteral("x".to_string()),
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_non_leaf_module_const_item() {
        assert!(try_lower_simple_module_const_item(
            "answer",
            &Type::Int,
            &HirExpr::Call {
                func: "compute_answer".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_module_literal_int_const_item() {
        let (item, rust_name) = try_lower_simple_module_const_item(
            "answer",
            &Type::LiteralInt(42),
            &HirExpr::IntLiteral(42),
        )
        .expect("literal int const should lower");
        assert_eq!(rust_name, "ANSWER");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                ty: crate::RustType::I64,
                ..
            } if name == "ANSWER"
        ));
    }

    #[test]
    fn lowers_simple_module_literal_bool_const_item() {
        let (item, rust_name) = try_lower_simple_module_const_item(
            "enabled",
            &Type::LiteralBool(true),
            &HirExpr::BoolLiteral(true),
        )
        .expect("literal bool const should lower");
        assert_eq!(rust_name, "ENABLED");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                ty: crate::RustType::Bool,
                ..
            } if name == "ENABLED"
        ));
    }

    #[test]
    fn lowers_simple_module_alias_int_const_item() {
        let alias_int = Type::alias("Meters", Type::Int);
        let (item, rust_name) =
            try_lower_simple_module_const_item("answer", &alias_int, &HirExpr::IntLiteral(42))
                .expect("alias int const should lower");
        assert_eq!(rust_name, "ANSWER");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                ..
            } if name == "ANSWER"
        ));
    }

    #[test]
    fn dispatcher_lowers_alias_primitive_module_const_as_const_item() {
        let alias_bool = Type::alias("Flag", Type::Bool);
        let (item, rust_name) = try_lower_simple_module_constant_item(
            "enabled",
            &alias_bool,
            &HirExpr::Name {
                name: "flag".to_string(),
                ty: alias_bool.clone(),
            },
        )
        .expect("dispatcher should lower alias primitive constant");
        assert_eq!(rust_name, "ENABLED");
        assert!(matches!(
            item,
            RustItem::Const {
                name,
                visibility: Visibility::Private,
                value: RustExpr::Ident(ref ident),
                ..
            } if name == "ENABLED" && ident == "flag"
        ));
    }

    #[test]
    fn does_not_lower_alias_primitive_module_helper_const_item() {
        let alias_int = Type::alias("Meters", Type::Int);
        assert!(try_lower_simple_module_helper_const_item(
            "answer",
            &alias_int,
            &HirExpr::IntLiteral(42),
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_module_string_const_item() {
        let (item, rust_name_call) = try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Str,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .expect("simple string const should lower");
        assert_eq!(rust_name_call, "__const_greeting()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::String_),
                ..
            } if name == "__const_greeting"
        ));
    }

    #[test]
    fn lowers_simple_module_alias_string_const_item() {
        let alias_str = Type::alias("Message", Type::Str);
        let (item, rust_name_call) = try_lower_simple_module_string_const_item(
            "greeting",
            &alias_str,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .expect("alias string const should lower");
        assert_eq!(rust_name_call, "__const_greeting()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::String_),
                ..
            } if name == "__const_greeting"
        ));
    }

    #[test]
    fn dispatcher_lowers_alias_string_module_const_as_string_item() {
        let alias_str = Type::alias("Message", Type::Str);
        let (item, rust_name_call) = try_lower_simple_module_constant_item(
            "greeting",
            &alias_str,
            &HirExpr::Name {
                name: "msg".to_string(),
                ty: alias_str.clone(),
            },
        )
        .expect("dispatcher should lower alias string constant");
        assert_eq!(rust_name_call, "__const_greeting()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::String_),
                body,
                ..
            } if name == "__const_greeting"
                && matches!(
                    body.first(),
                    Some(RustStmt::Return(Some(RustExpr::MethodCall { receiver, method, .. })))
                        if matches!(receiver.as_ref(), RustExpr::Ident(n) if n == "msg") && method == "to_string"
                )
        ));
    }

    #[test]
    fn does_not_lower_non_string_module_string_const_item() {
        assert!(try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Int,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_module_string_name_const_item() {
        let (item, rust_name_call) = try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Str,
            &HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Str,
            },
        )
        .expect("string name const should lower");
        assert_eq!(rust_name_call, "__const_greeting()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::String_),
                body,
                ..
            } if name == "__const_greeting"
                && matches!(
                    body.first(),
                    Some(RustStmt::Return(Some(RustExpr::MethodCall { receiver, method, .. })))
                        if matches!(receiver.as_ref(), RustExpr::Ident(n) if n == "msg") && method == "to_string"
                )
        ));
    }

    #[test]
    fn lowers_simple_module_literal_string_const_item() {
        let (item, rust_name_call) = try_lower_simple_module_string_const_item(
            "greeting",
            &Type::LiteralStr("hi".to_string()),
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .expect("literal string const should lower");
        assert_eq!(rust_name_call, "__const_greeting()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::String_),
                ..
            } if name == "__const_greeting"
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_module_string_const_item() {
        assert!(try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Str,
            &HirExpr::BinOp {
                left: Box::new(HirExpr::StringLiteral("a".to_string())),
                op: "+".to_string(),
                right: Box::new(HirExpr::StringLiteral("b".to_string())),
                ty: Type::Str,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_module_none_const_item() {
        let (item, rust_name_call) =
            try_lower_simple_module_none_const_item("nothing", &Type::None, &HirExpr::NoneLiteral)
                .expect("none const should lower");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                ..
            } if name == "__const_nothing"
        ));
    }

    #[test]
    fn lowers_simple_module_alias_none_const_item() {
        let alias_none = Type::alias("Nothing", Type::None);
        let (item, rust_name_call) =
            try_lower_simple_module_none_const_item("nothing", &alias_none, &HirExpr::NoneLiteral)
                .expect("alias none const should lower");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                ..
            } if name == "__const_nothing"
        ));
    }

    #[test]
    fn lowers_simple_module_none_name_const_item() {
        let (item, rust_name_call) = try_lower_simple_module_none_const_item(
            "nothing",
            &Type::None,
            &HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            },
        )
        .expect("none name const should lower");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                body,
                ..
            } if name == "__const_nothing"
                && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
        ));
    }

    #[test]
    fn lowers_simple_module_alias_none_name_const_item() {
        let alias_none = Type::alias("Nothing", Type::None);
        let (item, rust_name_call) = try_lower_simple_module_none_const_item(
            "nothing",
            &alias_none,
            &HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            },
        )
        .expect("alias none name const should lower");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                body,
                ..
            } if name == "__const_nothing"
                && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
        ));
    }

    #[test]
    fn dispatcher_lowers_alias_none_module_const_as_none_item() {
        let alias_none = Type::alias("Nothing", Type::None);
        let (item, rust_name_call) =
            try_lower_simple_module_constant_item("nothing", &alias_none, &HirExpr::NoneLiteral)
                .expect("dispatcher should lower alias none constant");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                ..
            } if name == "__const_nothing"
        ));
    }

    #[test]
    fn dispatcher_lowers_alias_none_name_module_const_as_none_item() {
        let alias_none = Type::alias("Nothing", Type::None);
        let (item, rust_name_call) = try_lower_simple_module_constant_item(
            "nothing",
            &alias_none,
            &HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            },
        )
        .expect("dispatcher should lower alias none name constant");
        assert_eq!(rust_name_call, "__const_nothing()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Unit),
                body,
                ..
            } if name == "__const_nothing"
                && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
        ));
    }

    #[test]
    fn does_not_lower_non_none_module_none_const_item() {
        assert!(try_lower_simple_module_none_const_item(
            "nothing",
            &Type::None,
            &HirExpr::IntLiteral(0),
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_non_none_name_module_none_const_item() {
        assert!(try_lower_simple_module_none_const_item(
            "nothing",
            &Type::None,
            &HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            },
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_alias_none_module_helper_const_item() {
        let alias_none = Type::alias("Nothing", Type::None);
        assert!(try_lower_simple_module_helper_const_item(
            "nothing",
            &alias_none,
            &HirExpr::NoneLiteral
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_module_helper_const_item_for_list_literal() {
        let ty = Type::List(Box::new(Type::Int));
        let value = HirExpr::ListLiteral {
            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
            ty: ty.clone(),
        };
        let (item, rust_name_call) = try_lower_simple_module_helper_const_item("nums", &ty, &value)
            .expect("simple non-primitive helper const should lower");
        assert_eq!(rust_name_call, "__const_nums()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Vec(_)),
                ..
            } if name == "__const_nums"
        ));
    }

    #[test]
    fn lowers_simple_module_helper_name_const_item() {
        let ty = Type::List(Box::new(Type::Int));
        let value = HirExpr::Name {
            name: "nums".to_string(),
            ty: ty.clone(),
        };
        let (item, rust_name_call) = try_lower_simple_module_helper_const_item("data", &ty, &value)
            .expect("simple helper name const should lower");
        assert_eq!(rust_name_call, "__const_data()");
        assert!(matches!(
            item,
            RustItem::Fn {
                name,
                visibility: Visibility::Private,
                ret: Some(RustType::Vec(_)),
                body,
                ..
            } if name == "__const_data"
                && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "nums")
        ));
    }

    #[test]
    fn does_not_lower_primitive_module_helper_const_item() {
        assert!(try_lower_simple_module_helper_const_item(
            "answer",
            &Type::Int,
            &HirExpr::IntLiteral(42),
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_alias_string_module_helper_const_item() {
        let alias_str = Type::alias("Message", Type::Str);
        assert!(try_lower_simple_module_helper_const_item(
            "greeting",
            &alias_str,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_non_leaf_module_helper_const_item() {
        let ty = Type::List(Box::new(Type::Int));
        let value = HirExpr::ListLiteral {
            elements: vec![HirExpr::Call {
                func: "build".to_string(),
                args: vec![],
                ty: Type::Int,
            }],
            ty: ty.clone(),
        };
        assert!(try_lower_simple_module_helper_const_item("nums", &ty, &value).is_none());
    }
}
