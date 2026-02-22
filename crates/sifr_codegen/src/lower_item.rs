//! Item lowering scaffolds for the IR migration.

use crate::{try_lower_leaf_expr, CodegenError, RustItem, Visibility};
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
    if !matches!(ty, Type::Int | Type::Float | Type::Bool) {
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
}
