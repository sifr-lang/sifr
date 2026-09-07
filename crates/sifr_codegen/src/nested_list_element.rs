use crate::{HirExpr, RustEmitter, RustExpr, RustParam, RustStmt, RustType, Type};

impl RustEmitter {
    pub(crate) fn try_lower_nested_list_element_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::Index {
            object: inner_object,
            index: inner_index,
            ty,
        } = expr
        else {
            return None;
        };
        let HirExpr::Index {
            object: outer_object,
            index: outer_index,
            ..
        } = inner_object.as_ref()
        else {
            return None;
        };
        let effective_outer_ty = self.effective_registry_expr_ty(outer_object);
        let Type::List(row_ty) = crate::resolve_alias_type_for_plain_call(&effective_outer_ty)
        else {
            return None;
        };
        let Type::List(element_ty) = crate::resolve_alias_type_for_plain_call(row_ty.as_ref())
        else {
            return None;
        };

        let lowered_outer_object = self.try_lower_registry_expr_strict(outer_object)?;
        let lowered_outer_index = self.try_lower_registry_expr_strict(outer_index)?;
        let lowered_outer_index =
            self.materialize_reusable_value_for_ir(outer_index, lowered_outer_index);
        let lowered_inner_index = self.try_lower_registry_expr_strict(inner_index)?;
        let lowered_inner_index =
            self.materialize_reusable_value_for_ir(inner_index, lowered_inner_index);
        if crate::helpers::is_option_type(ty) {
            Some(option_nested_list_element(
                lowered_outer_object,
                lowered_outer_index,
                lowered_inner_index,
                element_ty,
            ))
        } else {
            None
        }
    }
}

fn common_index_stmts(
    lowered_outer_object: RustExpr,
    lowered_outer_index: RustExpr,
) -> Vec<RustStmt> {
    vec![
        RustStmt::Let {
            mutable: false,
            name: "__sifr_outer_list".to_string(),
            ty: None,
            value: RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_outer_object),
            },
        },
        RustStmt::Let {
            mutable: false,
            name: "__sifr_outer_i".to_string(),
            ty: None,
            value: lowered_outer_index,
        },
        RustStmt::Let {
            mutable: false,
            name: "__sifr_outer_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident("__sifr_outer_list".to_string()),
                "__sifr_outer_i",
            ),
        },
    ]
}

fn option_nested_list_element(
    lowered_outer_object: RustExpr,
    lowered_outer_index: RustExpr,
    lowered_inner_index: RustExpr,
    element_ty: &Type,
) -> RustExpr {
    let projection_method = crate::helpers::option_projection_method_for_owned_type(element_ty);
    RustExpr::Block {
        stmts: common_index_stmts(lowered_outer_object, lowered_outer_index),
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_outer_list".to_string())),
                method: "get".to_string(),
                args: vec![RustExpr::Ident("__sifr_outer_norm".to_string())],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::ClosureBlock {
                params: vec![RustParam::Named {
                    name: "__sifr_row".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__sifr_inner_i".to_string(),
                        ty: None,
                        value: lowered_inner_index,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__sifr_inner_norm".to_string(),
                        ty: None,
                        value: crate::build_normalized_list_index_i64_expr(
                            RustExpr::Ident("__sifr_row".to_string()),
                            "__sifr_inner_i",
                        ),
                    },
                    RustStmt::Return(Some(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_row".to_string())),
                            method: "get".to_string(),
                            args: vec![RustExpr::Ident("__sifr_inner_norm".to_string())],
                        }),
                        method: projection_method.to_string(),
                        args: vec![],
                    })),
                ],
                is_move: false,
                is_async: false,
            }],
        })),
    }
}
