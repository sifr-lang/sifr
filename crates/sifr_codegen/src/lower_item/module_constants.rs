use crate::{
    try_lower_leaf_expr, try_lower_leaf_expr_result, CodegenError, RustExpr, RustItem, RustLiteral,
    RustMatchArm, RustStmt, RustType, Visibility,
};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

pub(super) fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

pub(super) fn is_simple_module_primitive_const_type(ty: &Type) -> bool {
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

pub(super) fn is_simple_module_string_const_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Str | Type::LiteralStr(_))
}

pub(super) fn is_simple_module_none_const_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

pub(super) fn is_exact_module_int_type(ty: &Type) -> bool {
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

pub(super) fn validate_module_constant_shape(name: &str) -> Result<(), CodegenError> {
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

pub(super) fn try_lower_leaf_or_name_expr_result(
    value: &HirExpr,
) -> Result<Option<RustExpr>, CodegenError> {
    if let Some(lowered) = try_lower_leaf_expr_result(value)? {
        return Ok(Some(lowered));
    }
    if let HirExpr::Name { name, .. } = value {
        return Ok(Some(RustExpr::Ident(name.clone())));
    }
    Ok(None)
}

pub(super) fn try_lower_simple_module_constant_item_result_impl(
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

pub(super) fn large_module_int_literal_decimal(ty: &Type, value: &HirExpr) -> Option<String> {
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

pub(super) fn sifr_int_parse_decimal_call(decimal_text: &str) -> RustExpr {
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

pub(super) fn lower_large_module_int_const_item(
    name: &str,
    decimal_text: &str,
) -> (RustItem, String) {
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
