use super::{HirExpr, RustEmitter};

impl RustEmitter {
    pub(crate) fn clone_non_copy_name_expr_for_ir(
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if expr.ty().contains_affine_resource() {
            return lowered;
        }
        if matches!(expr, HirExpr::Name { .. })
            && !crate::helpers::is_copy_type_for_codegen(expr.ty())
        {
            crate::RustExpr::Clone(Box::new(lowered))
        } else {
            lowered
        }
    }

    pub(crate) fn clone_owned_append_arg_expr_for_ir(
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if expr.ty().contains_affine_resource() {
            return lowered;
        }
        Self::clone_non_copy_name_expr_for_ir(
            expr,
            Self::clone_moved_names_in_borrowed_aggregate(expr, lowered),
        )
    }

    pub(crate) fn build_dict_lookup_key_arg_for_ir(
        lowered_index: crate::RustExpr,
    ) -> crate::RustExpr {
        crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_index),
        }
    }

    pub(crate) fn build_subscript_augassign_elem_stmt_for_ir(
        op: &str,
        lowered_value: crate::RustExpr,
    ) -> Option<crate::RustStmt> {
        if op == "**=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_value),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                },
            });
        }
        if op == "//=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "__elem".to_string(),
                    )))),
                    op: "/".to_string(),
                    right: Box::new(lowered_value),
                },
            });
        }
        let rust_op = op.strip_suffix('=')?;
        Some(crate::RustStmt::AugAssign {
            target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident("__elem".to_string()))),
            op: rust_op.to_string(),
            value: lowered_value,
        })
    }
}
