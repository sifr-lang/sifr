use sifr_ir::HirExpr;

use crate::{
    RustEmitter, RustExpr, RustParam, RustStmt, RustType, Type, resolve_alias_type_for_plain_call,
};

impl RustEmitter {
    pub(crate) fn lower_borrowed_string_name_for_compare(expr: &HirExpr) -> Option<RustExpr> {
        let HirExpr::Name { name, ty, .. } = expr else {
            return None;
        };
        if !matches!(
            resolve_alias_type_for_plain_call(ty),
            Type::Str | Type::LiteralStr(_)
        ) {
            return None;
        }
        Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(name.clone())),
                method: "as_str".to_string(),
                args: vec![],
            }],
        })
    }

    pub(crate) fn try_lower_borrowed_string_lookup_for_compare(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let HirExpr::Index { object, index, .. } = expr else {
            return Ok(None);
        };
        match resolve_alias_type_for_plain_call(object.ty()) {
            Type::Str | Type::LiteralStr(_) => {
                self.lower_borrowed_string_index_char_for_compare(object, index)
            }
            Type::List(element_ty) if matches!(element_ty.as_ref().resolve_alias(), Type::Str) => {
                self.lower_borrowed_list_string_index_for_compare(object, index)
            }
            Type::Dict(_, value_ty) if matches!(value_ty.as_ref().resolve_alias(), Type::Str) => {
                self.lower_borrowed_dict_string_index_for_compare(object, index)
            }
            _ => Ok(None),
        }
    }

    fn lower_borrowed_string_index_char_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(cache_name) = self.string_char_cache_for_expr(object) else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
        Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_cmp_chars".to_string(),
                    ty: None,
                    value: RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident(cache_name)),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_cmp_i".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_cmp_norm".to_string(),
                    ty: None,
                    value: RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__sifr_cmp_i".to_string())),
                            op: "<".to_string(),
                            right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Paren(Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident(
                                            "__sifr_cmp_chars".to_string(),
                                        )),
                                        method: "len".to_string(),
                                        args: vec![],
                                    }),
                                    ty: RustType::I64,
                                }),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Ident("__sifr_cmp_i".to_string())),
                            }))),
                            ty: RustType::Named("usize".to_string()),
                        }),
                        else_expr: Some(Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__sifr_cmp_i".to_string())),
                            ty: RustType::Named("usize".to_string()),
                        })),
                    },
                },
            ],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_cmp_chars".to_string())),
                    method: "get".to_string(),
                    args: vec![RustExpr::Ident("__sifr_cmp_norm".to_string())],
                }),
                method: "copied".to_string(),
                args: vec![],
            })),
        }))
    }

    fn lower_borrowed_list_string_index_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if let HirExpr::Index {
            object: outer_object,
            index: outer_index,
            ..
        } = object
        {
            if matches!(
                crate::resolve_alias_type_for_plain_call(outer_object.ty()),
                crate::Type::List(_)
            ) {
                let Some(lowered_outer_object) = self.lower_stmt_expr_for_ir(outer_object)? else {
                    return Ok(None);
                };
                let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
                    return Ok(None);
                };
                let lowered_outer_index =
                    Self::clone_non_copy_name_expr_for_ir(outer_index, lowered_outer_index);
                let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let lowered_inner_index =
                    Self::clone_non_copy_name_expr_for_ir(index, lowered_inner_index);
                return Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_list".to_string(),
                            ty: None,
                            value: RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_outer_object),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_i".to_string(),
                            ty: None,
                            value: lowered_outer_index,
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_norm".to_string(),
                            ty: None,
                            value: crate::build_normalized_list_index_i64_expr(
                                RustExpr::Ident("__sifr_cmp_outer_list".to_string()),
                                "__sifr_cmp_outer_i",
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_i".to_string(),
                            ty: None,
                            value: lowered_inner_index,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(
                                "__sifr_cmp_outer_list".to_string(),
                            )),
                            method: "get".to_string(),
                            args: vec![RustExpr::Ident("__sifr_cmp_outer_norm".to_string())],
                        }),
                        method: "and_then".to_string(),
                        args: vec![RustExpr::ClosureBlock {
                            params: vec![RustParam::Named {
                                name: "__sifr_cmp_list".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__sifr_cmp_norm".to_string(),
                                    ty: None,
                                    value: crate::build_normalized_list_index_i64_expr(
                                        RustExpr::Ident("__sifr_cmp_list".to_string()),
                                        "__sifr_cmp_i",
                                    ),
                                },
                                RustStmt::Return(Some(Self::as_str_option(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident(
                                        "__sifr_cmp_list".to_string(),
                                    )),
                                    method: "get".to_string(),
                                    args: vec![RustExpr::Ident("__sifr_cmp_norm".to_string())],
                                }))),
                            ],
                            is_move: false,
                            is_async: false,
                        }],
                    })),
                }));
            }
        }
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let mut stmts = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_cmp_list".to_string(),
            ty: None,
            value: RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_object),
            },
        }];
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
        stmts.extend([
            RustStmt::Let {
                mutable: false,
                name: "__sifr_cmp_i".to_string(),
                ty: None,
                value: lowered_index,
            },
            RustStmt::Let {
                mutable: false,
                name: "__sifr_cmp_norm".to_string(),
                ty: None,
                value: crate::build_normalized_list_index_i64_expr(
                    RustExpr::Ident("__sifr_cmp_list".to_string()),
                    "__sifr_cmp_i",
                ),
            },
        ]);
        Ok(Some(RustExpr::Block {
            stmts,
            expr: Some(Box::new(Self::as_str_option(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_cmp_list".to_string())),
                method: "get".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__sifr_cmp_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
                }],
            }))),
        }))
    }

    fn lower_borrowed_dict_string_index_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let key = if matches!(
            resolve_alias_type_for_plain_call(index.ty()),
            Type::Str | Type::LiteralStr(_)
        ) {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered_index))),
                method: "as_str".to_string(),
                args: Vec::new(),
            }
        } else {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Paren(Box::new(lowered_index))),
            }
        };
        Ok(Some(Self::as_str_option(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: "get".to_string(),
            args: vec![key],
        })))
    }

    fn as_str_option(option_expr: RustExpr) -> RustExpr {
        RustExpr::MethodCall {
            receiver: Box::new(option_expr),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_cmp_s".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_cmp_s".to_string())),
                    method: "as_str".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }
    }
}
