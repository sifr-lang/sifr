//! Item lowering scaffolds for the IR migration.

use crate::{
    try_lower_leaf_expr, CodegenError, RustExpr, RustItem, RustLiteral, RustStmt, RustType,
    Visibility,
};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

pub fn lower_item_raw(raw: &str) -> Result<Vec<RustItem>, CodegenError> {
    Ok(vec![RustItem::RawCode(raw.to_string())])
}

/// Conservatively lowers module-level primitive constants via IR.
/// Falls back for non-primitive or non-leaf values.
pub fn try_lower_simple_module_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if !matches!(ty, Type::Int | Type::Float | Type::Bool | Type::LiteralInt(_) | Type::LiteralBool(_)) {
        return None;
    }
    let rust_name = name.to_uppercase();
    Some((
        RustItem::Const {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            ty: crate::sifr_type_to_rust_type(ty),
            value: try_lower_leaf_expr(value)?,
        },
        rust_name,
    ))
}

/// Conservatively lowers module-level string-literal constants via IR helper function.
/// Falls back for non-literal string values.
pub fn try_lower_simple_module_string_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if !matches!(ty, Type::Str | Type::LiteralStr(_)) || !matches!(value, HirExpr::StringLiteral(_)) {
        return None;
    }
    let rust_name = format!("__const_{name}");
    let lowered_value = try_lower_leaf_expr(value)?;
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
    if !matches!(ty, Type::None) || !matches!(value, HirExpr::NoneLiteral) {
        return None;
    }
    let rust_name = format!("__const_{name}");
    Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Unit),
            body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Unit)))],
            is_async: false,
        },
        format!("{rust_name}()"),
    ))
}

/// Conservatively lowers module-level non-primitive helper constants via IR function items.
/// Falls back for primitive/string/none types or non-leaf values.
pub fn try_lower_simple_module_helper_const_item(
    name: &str,
    ty: &Type,
    value: &HirExpr,
) -> Option<(RustItem, String)> {
    if matches!(
        ty,
        Type::Int
            | Type::Float
            | Type::Bool
            | Type::LiteralInt(_)
            | Type::LiteralBool(_)
            | Type::Str
            | Type::LiteralStr(_)
            | Type::None
    ) {
        return None;
    }
    let rust_name = format!("__const_{name}");
    Some((
        RustItem::Fn {
            name: rust_name.clone(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(crate::sifr_type_to_rust_type(ty)),
            body: vec![RustStmt::Return(Some(try_lower_leaf_expr(value)?))],
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
    fn lowers_raw_item_placeholder() {
        let items = lower_item_raw("fn helper() {}").expect("placeholder lower should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], RustItem::RawCode(_)));
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
    fn does_not_lower_non_primitive_module_const_item() {
        assert!(try_lower_simple_module_const_item(
            "name",
            &Type::Str,
            &HirExpr::StringLiteral("x".to_string()),
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
    fn does_not_lower_non_literal_module_string_const_item() {
        assert!(try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Str,
            &HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Str,
            },
        )
        .is_none());
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
    fn does_not_lower_non_none_module_none_const_item() {
        assert!(try_lower_simple_module_none_const_item(
            "nothing",
            &Type::None,
            &HirExpr::IntLiteral(0),
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
        let (item, rust_name_call) =
            try_lower_simple_module_helper_const_item("nums", &ty, &value)
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
    fn does_not_lower_primitive_module_helper_const_item() {
        assert!(try_lower_simple_module_helper_const_item(
            "answer",
            &Type::Int,
            &HirExpr::IntLiteral(42),
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_non_leaf_module_helper_const_item() {
        let ty = Type::List(Box::new(Type::Int));
        let value = HirExpr::ListLiteral {
            elements: vec![HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }],
            ty: ty.clone(),
        };
        assert!(try_lower_simple_module_helper_const_item("nums", &ty, &value).is_none());
    }
}
