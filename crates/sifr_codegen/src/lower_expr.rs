//! Expression lowering scaffolds for the IR migration.

use crate::{CodegenError, RustExpr, RustLiteral};
use sifr_hir::HirExpr;

pub fn lower_expr_raw(raw: &str) -> Result<RustExpr, CodegenError> {
    Ok(RustExpr::RawCode(raw.to_string()))
}

/// Lowers leaf expressions that don't require emitter state.
/// This is the first incremental bridge from `emit_expr` string writes
/// to IR + renderer output.
pub fn try_lower_leaf_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::IntLiteral(v) => Some(RustExpr::RawCode(format!("{v}_i64"))),
        HirExpr::FloatLiteral(v) => {
            let mut s = v.to_string();
            if !s.contains('.') {
                s.push_str(".0");
            }
            Some(RustExpr::RawCode(format!("{s}_f64")))
        }
        HirExpr::StringLiteral(s) => Some(RustExpr::Literal(RustLiteral::Str(s.clone()))),
        HirExpr::BoolLiteral(v) => Some(RustExpr::Literal(RustLiteral::Bool(*v))),
        HirExpr::NoneLiteral => Some(RustExpr::Literal(RustLiteral::None)),
        HirExpr::EnumVariant { enum_name, variant, .. } => {
            Some(RustExpr::Path(vec![enum_name.clone(), variant.clone()]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_expr_placeholder() {
        let expr = lower_expr_raw("a + b").expect("placeholder lower should succeed");
        assert!(matches!(expr, RustExpr::RawCode(_)));
    }

    #[test]
    fn lowers_leaf_expr_variants() {
        let int_expr = try_lower_leaf_expr(&HirExpr::IntLiteral(7)).expect("int lowered");
        let str_expr =
            try_lower_leaf_expr(&HirExpr::StringLiteral("ok".to_string())).expect("str lowered");
        let bool_expr = try_lower_leaf_expr(&HirExpr::BoolLiteral(true)).expect("bool lowered");
        let none_expr = try_lower_leaf_expr(&HirExpr::NoneLiteral).expect("none lowered");
        let enum_expr = try_lower_leaf_expr(&HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: sifr_type_system::Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1))],
            },
        })
        .expect("enum variant lowered");

        assert!(matches!(int_expr, RustExpr::RawCode(_)));
        assert!(matches!(str_expr, RustExpr::Literal(RustLiteral::Str(_))));
        assert!(matches!(bool_expr, RustExpr::Literal(RustLiteral::Bool(true))));
        assert!(matches!(none_expr, RustExpr::Literal(RustLiteral::None)));
        assert!(matches!(enum_expr, RustExpr::Path(_)));
    }
}
