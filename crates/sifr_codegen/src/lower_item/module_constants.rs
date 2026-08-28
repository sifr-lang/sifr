#[cfg(test)]
use crate::try_lower_leaf_expr;
use crate::{
    CodegenError, RustExpr, RustItem, RustLiteral, RustStmt, RustType, Visibility,
    try_lower_leaf_expr_result,
};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

const EXACT_INT_CHUNK_DIGITS: usize = 18;
const EXACT_INT_CHUNK_BASE: i64 = 1_000_000_000_000_000_000;
// Keep compiler-owned literals under the same resource contract as runtime
// decimal parsing. Unlike the former generated panic path, reject over-limit
// source through the structured codegen diagnostic.
const EXACT_INT_LITERAL_DIGIT_LIMIT: usize = 4096;

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

pub(crate) fn try_lower_simple_module_constant_item_result(
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
        return lower_large_module_int_const_item(name, &decimal_text).map(Some);
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

fn sifr_int_from_i64(value: i64) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from_i64".to_string(),
        ])),
        args: vec![RustExpr::Literal(RustLiteral::Int(value))],
    }
}

fn exact_sifr_int_literal_expr(decimal_text: &str) -> Result<RustExpr, CodegenError> {
    let (negative, digits) = decimal_text
        .strip_prefix('-')
        .map_or((false, decimal_text), |digits| (true, digits));
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CodegenError::new(
            "invalid exact integer literal reached code generation",
        ));
    }
    if digits.len() > EXACT_INT_LITERAL_DIGIT_LIMIT {
        return Err(CodegenError::new(format!(
            "exact integer literal exceeds the {EXACT_INT_LITERAL_DIGIT_LIMIT}-digit limit"
        )));
    }

    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(sifr_int_from_i64(0));
    }

    let first_len = match digits.len() % EXACT_INT_CHUNK_DIGITS {
        0 => EXACT_INT_CHUNK_DIGITS,
        remainder => remainder,
    };
    let parse_chunk = |chunk: &str| {
        chunk.parse::<i64>().map_err(|_| {
            CodegenError::new("invalid exact integer literal chunk reached code generation")
        })
    };
    let mut expression = sifr_int_from_i64(parse_chunk(&digits[..first_len])?);
    for chunk in digits.as_bytes()[first_len..].chunks(EXACT_INT_CHUNK_DIGITS) {
        let chunk = std::str::from_utf8(chunk).map_err(|_| {
            CodegenError::new("non-ASCII exact integer literal reached code generation")
        })?;
        let shifted = RustExpr::BinOp {
            left: Box::new(expression),
            op: "*".to_string(),
            right: Box::new(sifr_int_from_i64(EXACT_INT_CHUNK_BASE)),
        };
        expression = RustExpr::BinOp {
            left: Box::new(shifted),
            op: "+".to_string(),
            right: Box::new(sifr_int_from_i64(parse_chunk(chunk)?)),
        };
    }
    if negative {
        expression = RustExpr::UnaryOp {
            op: "-".to_string(),
            operand: Box::new(expression),
        };
    }
    Ok(expression)
}

/// Conservative dispatcher for simple module-constant item lowering.
#[cfg(test)]
pub(crate) fn try_lower_simple_module_constant_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if let Some(decimal_text) = large_module_int_literal_decimal(ty, value) {
        return lower_large_module_int_const_item(name, &decimal_text).ok();
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
    let rust_name = format!("__const_{name}");
    Ok((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Named("SifrInt".to_string())),
            body: vec![RustStmt::TailExpr(exact_sifr_int_literal_expr(
                decimal_text,
            )?)],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

/// Conservatively lowers module-level primitive constants via IR.
/// Falls back for non-primitive or non-leaf/non-name values.
#[cfg(test)]
pub(crate) fn try_lower_simple_module_const_item(
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
#[cfg(test)]
pub(crate) fn try_lower_simple_module_string_const_item(
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
#[cfg(test)]
pub(crate) fn try_lower_simple_module_none_const_item(
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
#[cfg(test)]
pub(crate) fn try_lower_simple_module_helper_const_item(
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
