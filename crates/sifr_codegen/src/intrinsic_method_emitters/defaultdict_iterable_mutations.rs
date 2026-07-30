use super::{registry_iterable_to_owned_iter_expr_from_lowered, HirExpr, RustEmitter, Type};

impl RustEmitter {
    fn try_lower_defaultdict_iterable_collection(
        &mut self,
        iterable: &HirExpr,
        element_ty: &Type,
        collect_method: &str,
    ) -> Option<crate::RustExpr> {
        let lowered = self
            .try_lower_registry_expr_strict(iterable)
            .or_else(|| self.lower_stmt_expr_for_ir(iterable).ok().flatten())?;
        let lowered = Self::clone_moved_names_in_borrowed_aggregate(iterable, lowered);
        let owned_iter =
            registry_iterable_to_owned_iter_expr_from_lowered(iterable, Some(element_ty), lowered)?;
        Some(crate::RustExpr::MethodCall {
            receiver: Box::new(owned_iter),
            method: collect_method.to_string(),
            args: vec![],
        })
    }

    pub(crate) fn try_lower_defaultdict_list_extend_expr(
        &mut self,
        key_expr: crate::RustExpr,
        entry_expr: crate::RustExpr,
        iterable: &HirExpr,
        value_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let Type::List(element_ty) = crate::resolve_alias_type_for_plain_call(value_ty) else {
            return None;
        };
        let items_name = "__sifr_defaultdict_items".to_string();
        let bucket_name = "__sifr_defaultdict_bucket".to_string();
        Some(crate::RustExpr::Block {
            stmts: vec![
                crate::RustStmt::Let {
                    mutable: false,
                    name: "__sifr_defaultdict_key".to_string(),
                    ty: None,
                    value: key_expr,
                },
                crate::RustStmt::Let {
                    mutable: false,
                    name: items_name.clone(),
                    ty: None,
                    value: self.try_lower_defaultdict_iterable_collection(
                        iterable,
                        element_ty,
                        "collect::<Vec<_>>",
                    )?,
                },
                crate::RustStmt::Let {
                    mutable: false,
                    name: bucket_name.clone(),
                    ty: None,
                    value: entry_expr,
                },
                crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident(bucket_name)),
                    method: "extend".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(items_name)),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }],
                }),
            ],
            expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
        })
    }

    pub(crate) fn try_lower_defaultdict_set_update_expr(
        &mut self,
        key_expr: crate::RustExpr,
        entry_expr: crate::RustExpr,
        method: &str,
        args: &[HirExpr],
        value_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let Type::Set(element_ty) = crate::resolve_alias_type_for_plain_call(value_ty) else {
            return None;
        };
        if method == "symmetric_difference_update" && args.len() != 1 {
            return None;
        }

        let mut stmts = Vec::with_capacity(args.len() + 4);
        stmts.push(crate::RustStmt::Let {
            mutable: false,
            name: "__sifr_defaultdict_key".to_string(),
            ty: None,
            value: key_expr,
        });
        let mut item_names = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let items_name = format!("__sifr_defaultdict_set_items_{index}");
            stmts.push(crate::RustStmt::Let {
                mutable: false,
                name: items_name.clone(),
                ty: None,
                value: self.try_lower_defaultdict_iterable_collection(
                    arg,
                    element_ty,
                    "collect::<std::collections::HashSet<_>>",
                )?,
            });
            item_names.push(items_name);
        }

        let bucket_name = "__sifr_defaultdict_bucket".to_string();
        stmts.push(crate::RustStmt::Let {
            mutable: false,
            name: bucket_name.clone(),
            ty: None,
            value: entry_expr,
        });

        match method {
            "update" => {
                for items_name in item_names {
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(bucket_name.clone())),
                        method: "extend".to_string(),
                        args: vec![crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(items_name)),
                            method: "into_iter".to_string(),
                            args: vec![],
                        }],
                    }));
                }
            }
            "intersection_update" | "difference_update" => {
                let keep_on_match = method == "intersection_update";
                for items_name in item_names {
                    let contains = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(items_name)),
                        method: "contains".to_string(),
                        args: vec![crate::RustExpr::Ident("__item".to_string())],
                    };
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(bucket_name.clone())),
                        method: "retain".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__item".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(if keep_on_match {
                                contains
                            } else {
                                crate::RustExpr::UnaryOp {
                                    op: "!".to_string(),
                                    operand: Box::new(contains),
                                }
                            }),
                            is_move: false,
                        }],
                    }));
                }
            }
            "symmetric_difference_update" => {
                let items_name = item_names.into_iter().next()?;
                let new_bucket_name = "__sifr_defaultdict_new_bucket".to_string();
                stmts.push(crate::RustStmt::Let {
                    mutable: false,
                    name: new_bucket_name.clone(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(bucket_name.clone())),
                                method: "symmetric_difference".to_string(),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident(items_name)),
                                }],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }),
                        method: "collect::<std::collections::HashSet<_>>".to_string(),
                        args: vec![],
                    },
                });
                stmts.push(crate::RustStmt::Assign {
                    target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(bucket_name))),
                    value: crate::RustExpr::Ident(new_bucket_name),
                });
            }
            _ => return None,
        }

        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
        })
    }
}
