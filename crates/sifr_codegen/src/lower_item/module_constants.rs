use crate::{
    CodegenError, RustExpr, RustItem, RustLiteral, RustStmt, RustType, Visibility,
    try_lower_leaf_expr, try_lower_leaf_expr_result,
};
use sifr_ir::HirExpr;
use sifr_type_system::Type;
use std::str::FromStr as _;

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
    matches!(resolve_alias_type(ty), Type::Int | Type::LiteralInt(_))
}

pub fn try_lower_simple_module_constant_item_result(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Result<Option<(RustItem, String)>, CodegenError> {
    validate_module_constant_shape(name)?;
    try_lower_simple_module_constant_item_result_impl(name, ty, value)
}

pub(crate) fn module_constant_rust_reference(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Result<String, CodegenError> {
    Ok(
        match try_lower_simple_module_constant_item_result(name, ty, value)? {
            Some((_, rust_reference)) => rust_reference,
            None => format!("__const_{name}()"),
        },
    )
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
        return Ok(Some(lower_large_module_int_const_item(
            name,
            &decimal_text,
        )?));
    }

    if is_exact_module_int_type(ty) {
        // SifrInt is not Copy and exact constant expressions are not generally const-evaluable
        // by Rust. Defer them to the stateful helper-function path, which can also rewrite
        // dependencies on earlier Sifr module constants.
        return Ok(None);
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
        } else if let HirExpr::Name { name, ty, .. } = value {
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

/// Conservative dispatcher for simple module-constant item lowering.
pub fn try_lower_simple_module_constant_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if let Some(decimal_text) = large_module_int_literal_decimal(ty, value) {
        return lower_large_module_int_const_item(name, &decimal_text).ok();
    }
    if is_exact_module_int_type(ty) {
        return None;
    }
    try_lower_simple_module_const_item(name, ty, value)
        .or_else(|| try_lower_simple_module_string_const_item(name, ty, value))
        .or_else(|| try_lower_simple_module_none_const_item(name, ty, value))
        .or_else(|| try_lower_simple_module_helper_const_item(name, ty, value))
}

pub(super) fn lower_large_module_int_const_item(
    name: &str,
    decimal_text: &str,
) -> Result<(RustItem, String), CodegenError> {
    let integer = bigdecimal::num_bigint::BigInt::from_str(decimal_text).map_err(|_| {
        CodegenError::new(format!(
            "invalid compiler integer literal for module constant {name}"
        ))
    })?;
    let bytes = integer
        .to_signed_bytes_be()
        .into_iter()
        .map(|byte| RustExpr::Literal(RustLiteral::Int(i64::from(byte))))
        .collect();
    let rust_name = format!("__const_{name}");
    Ok((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Named("SifrInt".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "SifrInt".to_string(),
                    "from_signed_bytes_be".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Vec(bytes)),
                }],
            }))],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

/// Conservatively lowers module-level primitive constants via IR.
/// Falls back for non-primitive or non-leaf/non-name values.
pub fn try_lower_simple_module_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if is_exact_module_int_type(ty) || !is_simple_module_primitive_const_type(ty) {
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
    } else if let HirExpr::Name { name, ty, .. } = value {
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
