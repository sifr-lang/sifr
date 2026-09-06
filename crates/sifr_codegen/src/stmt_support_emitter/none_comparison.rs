use crate::{CodegenError, HirExpr, RustEmitter, RustExpr, RustLiteral, RustStmt, Type};

impl RustEmitter {
    pub(crate) fn lower_static_none_comparison(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
    ) -> Result<Option<RustExpr>, CodegenError> {
        if !matches!(op, "is" | "is not" | "==" | "!=") {
            return Ok(None);
        }
        let other = match (left, right) {
            (_, HirExpr::NoneLiteral) => left,
            (HirExpr::NoneLiteral, _) => right,
            _ => return Ok(None),
        };
        if crate::helpers::is_option_type(other.ty())
            || matches!(
                other.ty().resolve_alias(),
                Type::Any | Type::Unknown | Type::TypeVar(_) | Type::Union(_)
            )
        {
            return Ok(None);
        }
        let is_none = matches!(other.ty().resolve_alias(), Type::None);
        let result = RustExpr::Literal(RustLiteral::Bool(is_none == matches!(op, "is" | "==")));
        if matches!(other, HirExpr::NoneLiteral) {
            return Ok(Some(result));
        }
        // The type determines the comparison, not whether its operand executes.
        // Borrow the result as the comparison would, without consuming a place.
        let Some(value) = self.lower_stmt_expr_for_ir(other)? else {
            return Ok(None);
        };
        Ok(Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(value),
                },
            }],
            expr: Some(Box::new(result)),
        }))
    }
}
