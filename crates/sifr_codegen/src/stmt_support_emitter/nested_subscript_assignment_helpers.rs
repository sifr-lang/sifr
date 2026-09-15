use super::{HirExpr, RustEmitter};

impl RustEmitter {
    pub(crate) fn clone_field_storage_name_expr_for_ir(
        &self,
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        let HirExpr::Name { name, .. } = expr else {
            return lowered;
        };
        if expr.ty().contains_affine_resource()
            || crate::helpers::is_copy_type_for_codegen(expr.ty())
        {
            return lowered;
        }
        if crate::helpers::is_logically_copy_rust_move_type(expr.ty())
            || self.borrowed_params.contains(name)
            || self.mut_borrowed_params.contains(name)
        {
            crate::ownership_plan::materialize_owned_value(expr.ty(), lowered)
        } else {
            lowered
        }
    }

    pub(crate) fn clone_non_copy_name_expr_for_ir(
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if expr.ty().contains_affine_resource() {
            return lowered;
        }
        if matches!(expr, HirExpr::Name { .. })
            && !crate::helpers::is_copy_type_for_codegen(expr.ty())
            && Self::rust_expr_is_reusable_place_for_ir(&lowered)
        {
            crate::ownership_plan::materialize_owned_value(expr.ty(), lowered)
        } else {
            lowered
        }
    }

    pub(crate) fn rust_expr_is_reusable_place_for_ir(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::Ident(_)
            | crate::RustExpr::Field { .. }
            | crate::RustExpr::Index { .. } => true,
            crate::RustExpr::Paren(inner) => Self::rust_expr_is_reusable_place_for_ir(inner),
            _ => false,
        }
    }

    /// Materialize a source value at an owned Rust boundary. Preserve reusable
    /// places unless the body analysis proves this exact expression is the last use.
    pub(crate) fn materialize_reusable_value_for_ir(
        &self,
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if expr.ty().contains_affine_resource() {
            return lowered;
        }
        let lowered = self.clone_moved_names_in_borrowed_aggregate(expr, lowered);
        if matches!(expr, HirExpr::Name { .. })
            && !crate::helpers::is_copy_type_for_codegen(expr.ty())
            && Self::rust_expr_is_reusable_place_for_ir(&lowered)
        {
            if self
                .last_use_move_exprs
                .contains(&crate::body_analysis::expr_key(expr))
            {
                return lowered;
            }
            crate::ownership_plan::materialize_owned_value(expr.ty(), lowered)
        } else {
            lowered
        }
    }

    pub(crate) fn build_dict_lookup_key_arg_for_ir(
        lowered_index: crate::RustExpr,
    ) -> crate::RustExpr {
        crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_index),
        }
    }

    pub(crate) fn build_dict_lookup_key_arg_for_hir(
        &self,
        index: &HirExpr,
        lowered_index: crate::RustExpr,
    ) -> crate::RustExpr {
        if matches!(index, HirExpr::Name { name, .. }
            if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
        {
            lowered_index
        } else {
            Self::build_dict_lookup_key_arg_for_ir(lowered_index)
        }
    }

    pub(crate) fn build_subscript_augassign_elem_stmt_for_ir(
        op: &str,
        source_value: &HirExpr,
        lowered_value: crate::RustExpr,
        exact_integer: bool,
    ) -> Option<crate::RustStmt> {
        if exact_integer {
            let method = match op {
                "//=" => Some("floor_div_known_nonzero"),
                "%=" => Some("floor_mod_known_nonzero"),
                "**=" => Some("pow_known_valid"),
                "<<=" => Some("shl_known_valid"),
                ">>=" => Some("shr_known_valid"),
                _ => None,
            };
            if let Some(method) = method {
                let method_arg = if matches!(op, "**=" | "<<=" | ">>=") {
                    let primitive_ty = if op == "**=" { "u32" } else { "usize" };
                    let literal = crate::integer_literal_decimal(source_value)?
                        .parse::<i64>()
                        .ok()?;
                    crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(literal))),
                        ty: crate::RustType::Named(primitive_ty.to_string()),
                    }
                } else {
                    crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(lowered_value),
                    }
                };
                return Some(crate::RustStmt::Assign {
                    target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "__elem".to_string(),
                    ))),
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                        method: method.to_string(),
                        args: vec![method_arg],
                    },
                });
            }
        }
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
