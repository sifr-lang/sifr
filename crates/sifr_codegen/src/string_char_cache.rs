use crate::hir_analysis::traversal::{self, TraversalConfig};
use crate::string_char_cache_scan::{collect_string_cache_uses, function_calls_itself};
use crate::{HirExpr, HirFunction, RustEmitter, RustExpr, RustLiteral, RustStmt, RustType, Type};
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn prepare_string_char_cache_stmts(
        &mut self,
        func: &HirFunction,
        _reassigned_vars: &HashSet<String>,
    ) -> Vec<RustStmt> {
        let mut used_string_names = HashSet::new();
        traversal::walk_stmts(
            &func.body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                collect_string_cache_uses(expr, &mut used_string_names);
            },
        );
        self.string_char_cache_required_names
            .clone_from(&used_string_names);
        if function_calls_itself(func) {
            for param in &func.params {
                used_string_names.remove(&param.name);
                self.string_char_cache_required_names.remove(&param.name);
            }
        }

        func.params
            .iter()
            .filter(|param| matches!(param.ty.resolve_alias(), Type::Str | Type::LiteralStr(_)))
            .filter(|param| used_string_names.contains(&param.name))
            .map(|param| {
                let cache_name = format!("__sifr_chars_{}", param.name);
                self.string_char_cache_vars
                    .insert(param.name.clone(), cache_name.clone());
                RustStmt::Let {
                    mutable: true,
                    name: cache_name,
                    ty: Some(RustType::Vec(Box::new(RustType::Named("char".to_string())))),
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(param.name.clone())),
                            method: "chars".to_string(),
                            args: vec![],
                        }),
                        method: "collect::<Vec<char>>".to_string(),
                        args: vec![],
                    },
                }
            })
            .collect()
    }

    pub(crate) fn lower_string_len_with_cache(
        &self,
        object: &HirExpr,
        lowered_object: RustExpr,
    ) -> RustExpr {
        if let Some(cache_name) = self.string_char_cache_for_expr(object) {
            return RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(cache_name)),
                    method: "len".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            };
        }
        RustExpr::Cast {
            expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "count".to_string(),
                args: vec![],
            }),
            ty: RustType::I64,
        }
    }

    pub(crate) fn lower_string_index_option_with_cache(
        &self,
        object: &HirExpr,
        lowered_object: RustExpr,
        lowered_index: RustExpr,
    ) -> RustExpr {
        let index = RustExpr::Cast {
            expr: Box::new(lowered_index),
            ty: RustType::Named("usize".to_string()),
        };
        if let Some(cache_name) = self.string_char_cache_for_expr(object) {
            return char_option_to_string(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(cache_name)),
                method: "get".to_string(),
                args: vec![index],
            });
        }
        char_option_to_string(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "nth".to_string(),
            args: vec![index],
        })
    }

    pub(crate) fn lower_string_index_unwrapped_with_cache(
        &self,
        object: &HirExpr,
        lowered_object: RustExpr,
        lowered_index: RustExpr,
    ) -> RustExpr {
        RustExpr::Block {
            stmts: vec![RustStmt::LetElse {
                pattern: "Some(__indexed_char)".to_string(),
                value: self.lower_string_index_option_with_cache(
                    object,
                    lowered_object,
                    lowered_index,
                ),
                else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(
                        "compiler-verified string index should be in range".to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(RustExpr::Ident("__indexed_char".to_string()))),
        }
    }

    pub(crate) fn try_lower_dict_indexed_list_append_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        else {
            return None;
        };
        if method != "append" || args.len() != 1 {
            return None;
        }
        let HirExpr::Index {
            object: index_object,
            index,
            ..
        } = object.as_ref()
        else {
            return None;
        };
        let effective_index_object_ty = self.effective_registry_expr_ty(index_object);
        let Type::Dict(_, value_ty) =
            crate::resolve_alias_type_for_plain_call(&effective_index_object_ty)
        else {
            return None;
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
            Type::List(_)
        ) {
            return None;
        }

        let lowered_object = self.try_lower_dict_indexed_list_mutation_object(index_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
        let key_arg = Self::build_dict_lookup_key_arg_for_ir(
            Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
        );
        let pushed_arg = Self::clone_owned_append_arg_expr_for_ir(&args[0], lowered_arg);
        Some(RustExpr::Block {
            stmts: vec![RustStmt::IfLet {
                pattern: "Some(__elem)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "get_mut".to_string(),
                    args: vec![key_arg],
                },
                then_body: vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__elem".to_string())),
                    method: "push".to_string(),
                    args: vec![pushed_arg],
                })],
                else_body: None,
            }],
            expr: None,
        })
    }

    pub(crate) fn try_lower_list_indexed_dict_element_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::Index {
            object: inner_object,
            index: dict_index,
            ..
        } = expr
        else {
            return None;
        };
        let HirExpr::Index {
            object: list_object,
            index: list_index,
            ..
        } = inner_object.as_ref()
        else {
            return None;
        };
        let effective_list_ty = self.effective_registry_expr_ty(list_object);
        let Type::List(row_ty) = crate::resolve_alias_type_for_plain_call(&effective_list_ty)
        else {
            return None;
        };
        let Type::Dict(_, value_ty) = crate::resolve_alias_type_for_plain_call(row_ty.as_ref())
        else {
            return None;
        };

        let lowered_object = self.try_lower_registry_expr_strict(list_object)?;
        let lowered_list_index = self.try_lower_registry_expr_strict(list_index)?;
        let lowered_dict_index = self.try_lower_registry_expr_strict(dict_index)?;
        let key_arg = Self::list_indexed_dict_lookup_key_arg(dict_index, lowered_dict_index);
        let projection_method = if crate::helpers::is_copy_type_for_codegen(value_ty.as_ref()) {
            "copied"
        } else {
            "cloned"
        };
        Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_list_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        lowered_object.clone(),
                        "__idx_raw",
                    ),
                },
            ],
            expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                    op: ">=".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                }),
                then_expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "get".to_string(),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                                ty: RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "and_then".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__bucket".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__bucket".to_string())),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            is_move: false,
                        }],
                    }),
                    method: projection_method.to_string(),
                    args: vec![],
                }),
                else_expr: Some(Box::new(RustExpr::Literal(RustLiteral::None))),
            })),
        })
    }

    pub(crate) fn try_lower_list_indexed_dict_contains_expr(
        &mut self,
        element: &HirExpr,
        collection: &HirExpr,
        lowered_element: RustExpr,
    ) -> Option<RustExpr> {
        let HirExpr::Index {
            object: list_object,
            index: list_index,
            ..
        } = collection
        else {
            return None;
        };
        let effective_list_ty = self.effective_registry_expr_ty(list_object);
        let Type::List(row_ty) = crate::resolve_alias_type_for_plain_call(&effective_list_ty)
        else {
            return None;
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(row_ty.as_ref()),
            Type::Dict(_, _)
        ) {
            return None;
        }

        let lowered_object = self.try_lower_registry_expr_strict(list_object)?;
        let lowered_list_index = self.try_lower_registry_expr_strict(list_index)?;
        let key_arg = Self::list_indexed_dict_lookup_key_arg(element, lowered_element);
        Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_list_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        lowered_object.clone(),
                        "__idx_raw",
                    ),
                },
            ],
            expr: Some(Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                    op: ">=".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                }),
                op: "&&".to_string(),
                right: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "get".to_string(),
                        args: vec![RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                            ty: RustType::Named("usize".to_string()),
                        }],
                    }),
                    method: "map_or".to_string(),
                    args: vec![
                        RustExpr::Literal(RustLiteral::Bool(false)),
                        RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__bucket".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__bucket".to_string())),
                                method: "contains_key".to_string(),
                                args: vec![key_arg],
                            }),
                            is_move: false,
                        },
                    ],
                }),
            })),
        })
    }

    fn list_indexed_dict_lookup_key_arg(expr: &HirExpr, lowered: RustExpr) -> RustExpr {
        if matches!(
            expr,
            HirExpr::Name { ty, .. }
                if matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Str | Type::LiteralStr(_)
                )
        ) {
            return RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "as_str".to_string(),
                args: vec![],
            };
        }
        Self::build_dict_lookup_key_arg_for_ir(lowered)
    }

    pub(crate) fn try_lower_dict_indexed_list_pop_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        else {
            return None;
        };
        if method != "pop" || !args.is_empty() {
            return None;
        }
        let HirExpr::Index {
            object: index_object,
            index,
            ..
        } = object.as_ref()
        else {
            return None;
        };
        let effective_index_object_ty = self.effective_registry_expr_ty(index_object);
        let Type::Dict(_, value_ty) =
            crate::resolve_alias_type_for_plain_call(&effective_index_object_ty)
        else {
            return None;
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
            Type::List(_)
        ) {
            return None;
        }

        let lowered_object = self.try_lower_dict_indexed_list_mutation_object(index_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let key_arg = Self::build_dict_lookup_key_arg_for_ir(
            Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
        );
        let option_pop_expr = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "get_mut".to_string(),
                args: vec![key_arg],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_bucket".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_bucket".to_string())),
                    method: "pop".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        };
        if crate::helpers::is_option_type(expr.ty()) {
            return Some(option_pop_expr);
        }
        Some(RustExpr::Block {
            stmts: vec![RustStmt::LetElse {
                pattern: "Some(__sifr_popped)".to_string(),
                value: option_pop_expr,
                else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(
                        "compiler-verified non-empty dict list pop should return Some".to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(RustExpr::Ident("__sifr_popped".to_string()))),
        })
    }

    pub(crate) fn try_lower_dict_indexed_list_len_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        else {
            return None;
        };
        if method != "len" || !args.is_empty() {
            return None;
        }
        let HirExpr::Index {
            object: index_object,
            index,
            ..
        } = object.as_ref()
        else {
            return None;
        };
        let effective_index_object_ty = self.effective_registry_expr_ty(index_object);
        let Type::Dict(_, value_ty) =
            crate::resolve_alias_type_for_plain_call(&effective_index_object_ty)
        else {
            return None;
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
            Type::List(_)
        ) {
            return None;
        }

        let lowered_object = self.try_lower_dict_indexed_list_mutation_object(index_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let key_arg = Self::build_dict_lookup_key_arg_for_ir(
            Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
        );
        Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "get".to_string(),
                args: vec![key_arg],
            }),
            method: "map_or".to_string(),
            args: vec![
                RustExpr::Literal(RustLiteral::Int(0)),
                RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__sifr_bucket".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_bucket".to_string())),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        ty: RustType::I64,
                    }),
                    is_move: false,
                },
            ],
        })
    }

    pub(crate) fn try_lower_dict_indexed_list_element_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        let HirExpr::Index {
            object,
            index: element_index,
            ..
        } = expr
        else {
            return None;
        };
        let HirExpr::Index {
            object: dict_object,
            index: dict_index,
            ..
        } = object.as_ref()
        else {
            return None;
        };
        let effective_dict_ty = self.effective_registry_expr_ty(dict_object);
        let Type::Dict(_, value_ty) = crate::resolve_alias_type_for_plain_call(&effective_dict_ty)
        else {
            return None;
        };
        let Type::List(element_ty) = crate::resolve_alias_type_for_plain_call(value_ty.as_ref())
        else {
            return None;
        };

        let lowered_object = self.try_lower_dict_indexed_list_mutation_object(dict_object)?;
        let lowered_dict_index = self.try_lower_registry_expr_strict(dict_index)?;
        let lowered_element_index = self.try_lower_registry_expr_strict(element_index)?;
        let key_arg = Self::build_dict_lookup_key_arg_for_ir(
            Self::clone_non_copy_name_expr_for_ir(dict_index, lowered_dict_index),
        );
        let projection_method = crate::helpers::option_projection_method_for_owned_type(element_ty);
        Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "get".to_string(),
                args: vec![key_arg],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::ClosureBlock {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_bucket".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__sifr_index_i".to_string(),
                        ty: None,
                        value: lowered_element_index,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__sifr_index_norm".to_string(),
                        ty: None,
                        value: RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__sifr_index_i".to_string())),
                                op: "<".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                            }),
                            then_expr: Box::new(RustExpr::Cast {
                                expr: Box::new(RustExpr::Paren(Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__sifr_bucket".to_string(),
                                            )),
                                            method: "len".to_string(),
                                            args: vec![],
                                        }),
                                        ty: RustType::I64,
                                    }),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::Ident("__sifr_index_i".to_string())),
                                }))),
                                ty: RustType::Named("usize".to_string()),
                            }),
                            else_expr: Some(Box::new(RustExpr::Cast {
                                expr: Box::new(RustExpr::Ident("__sifr_index_i".to_string())),
                                ty: RustType::Named("usize".to_string()),
                            })),
                        },
                    },
                    RustStmt::Return(Some(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_bucket".to_string())),
                            method: "get".to_string(),
                            args: vec![RustExpr::Ident("__sifr_index_norm".to_string())],
                        }),
                        method: projection_method.to_string(),
                        args: vec![],
                    })),
                ],
                is_move: false,
                is_async: false,
            }],
        })
    }

    pub(crate) fn string_char_cache_for_expr(&self, expr: &HirExpr) -> Option<String> {
        let HirExpr::Name { name, .. } = expr else {
            return None;
        };
        self.string_char_cache_vars.get(name).cloned()
    }

    pub(crate) fn string_char_cache_init_stmt_for_local(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        let should_cache = self.string_char_cache_required_names.contains(name);
        if !should_cache
            || self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    pub(crate) fn force_string_char_cache_init_stmt_for_local(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        if self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    pub(crate) fn string_char_cache_init_stmt_for_loop_target(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        if !self.string_char_cache_required_names.contains(name)
            || self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    fn build_string_char_cache_init_stmt(&mut self, name: &str) -> RustStmt {
        let cache_name = format!("__sifr_chars_{name}");
        self.string_char_cache_vars
            .insert(name.to_string(), cache_name.clone());
        RustStmt::Let {
            mutable: true,
            name: cache_name,
            ty: Some(RustType::Vec(Box::new(RustType::Named("char".to_string())))),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<char>>".to_string(),
                args: vec![],
            },
        }
    }

    pub(crate) fn string_char_cache_rebuild_stmt_for_local(&self, name: &str) -> Option<RustStmt> {
        let cache_name = self.string_char_cache_vars.get(name)?;
        Some(RustStmt::Assign {
            target: RustExpr::Ident(cache_name.clone()),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<char>>".to_string(),
                args: vec![],
            },
        })
    }

    pub(crate) fn try_lower_dict_indexed_list_mutation_object(
        &mut self,
        expr: &HirExpr,
    ) -> Option<RustExpr> {
        if let HirExpr::FieldAccess { object, field, .. } = expr {
            let lowered_object = self.try_lower_registry_expr_strict(object)?;
            return Some(RustExpr::Field {
                expr: Box::new(lowered_object),
                field: field.clone(),
            });
        }
        let suppress_self_field_clone = matches!(expr, HirExpr::FieldAccess { .. })
            && self.method_call_needs_field_clone_suppression(expr, "append");
        let suppression_prev = self.pending_self_field_clone_suppression;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }
        let lowered = self.try_lower_registry_expr_strict(expr);
        if suppress_self_field_clone && self.pending_self_field_clone_suppression > suppression_prev
        {
            self.pending_self_field_clone_suppression -= 1;
        }
        lowered
    }
}

fn char_option_to_string(option_expr: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(option_expr),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![crate::RustParam::Named {
                name: "c".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("c".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            }),
            is_move: false,
        }],
    }
}
