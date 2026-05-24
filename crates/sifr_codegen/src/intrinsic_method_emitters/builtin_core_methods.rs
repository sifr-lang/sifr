use super::{
    intrinsics, registry_box_iterator_expr, registry_call_callable_with_owned_args,
    registry_callable_signature, registry_class_has_next, registry_class_method_signature,
    registry_dict_source_to_map_expr, registry_iter_from_next_method_expr,
    registry_iterable_to_owned_iter_expr, registry_iterable_to_vec_expr,
    registry_iterable_to_vec_expr_with_hint, registry_nested_zip_field_expr,
    registry_zip_iter_expr, HirExpr, RustEmitter, RustExpr, Type,
};
impl RustEmitter {
    pub(crate) fn apply_intrinsic_registry_side_effects(
        &mut self,
        func: &str,
        lowered: &intrinsics::LoweredIntrinsic,
    ) {
        if matches!(
            func,
            "builtin_open"
                | "open_file"
                | "file_read"
                | "file_write"
                | "file_readline"
                | "file_readlines"
                | "file_close"
                | "file_read_bytes"
                | "file_write_bytes"
        ) {
            self.runtime_needs.require(crate::RuntimeNeed::FileHandles);
        }
        if func == "builtin_open" {
            self.used_stdlib_modules.insert("io".to_string());
        }
        if matches!(func, "set_global_level" | "get_global_level") {
            self.runtime_needs.require(crate::RuntimeNeed::LoggingState);
        }
        if matches!(
            func,
            "random_module_state_words"
                | "random_module_state_index"
                | "random_module_state_gauss_next"
                | "random_module_set_state"
        ) {
            self.runtime_needs
                .require(crate::RuntimeNeed::RandomModuleState);
        }

        if let Some(required_crate) = lowered.required_crate {
            self.intrinsic_registry_crates
                .insert(required_crate.to_string());
        }
        for required_crate in lowered.additional_required_crates {
            self.intrinsic_registry_crates
                .insert((*required_crate).to_string());
        }
    }
}

impl RustEmitter {
    pub(crate) fn try_lower_registry_builtin_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
        result_ty: Option<&Type>,
    ) -> Option<crate::RustExpr> {
        if let Some(lowered) =
            self.try_lower_registry_collection_builtin_call_expr(func, args, result_ty)
        {
            return Some(lowered);
        }
        if let Some(lowered) =
            self.try_lower_registry_ordering_builtin_call_expr(func, args, result_ty)
        {
            return Some(lowered);
        }
        self.try_lower_registry_numeric_builtin_call_expr(func, args, result_ty)
    }
}

impl RustEmitter {
    pub(crate) fn try_lower_registry_collection_builtin_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
        result_ty: Option<&Type>,
    ) -> Option<crate::RustExpr> {
        match func {
            "__compat_defaultdict_int"
            | "__compat_defaultdict_list"
            | "__compat_defaultdict_set"
                if args.is_empty() =>
            {
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "HashMap".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                })
            }
            "__compat_defaultdict_int"
            | "__compat_defaultdict_list"
            | "__compat_defaultdict_set"
                if args.len() == 1 =>
            {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                    method: "clone".to_string(),
                    args: vec![],
                })
            }
            "list" if args.is_empty() => Some(RustExpr::Vec(vec![])),
            "list" if args.len() == 1 => {
                let target_elem_hint = result_ty.and_then(|ty| {
                    let Type::List(elem) = crate::resolve_alias_type_for_plain_call(ty) else {
                        return None;
                    };
                    Some(elem.as_ref())
                });
                registry_iterable_to_vec_expr_with_hint(self, &args[0], target_elem_hint)
            }
            "bytes" if args.is_empty() => Some(RustExpr::Vec(vec![])),
            "dict" if args.is_empty() => Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "HashMap".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            }),
            "dict" if args.len() == 1 => registry_dict_source_to_map_expr(self, &args[0]),
            "dict" if args.len() == 2 => {
                let map_name = "__sifr_dict_ctor".to_string();
                let base_map = registry_dict_source_to_map_expr(self, &args[0])?;
                let extra_map = registry_dict_source_to_map_expr(self, &args[1])?;
                Some(RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: true,
                            name: map_name.clone(),
                            ty: None,
                            value: base_map,
                        },
                        crate::RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(map_name.clone())),
                            method: "extend".to_string(),
                            args: vec![extra_map],
                        }),
                    ],
                    expr: Some(Box::new(RustExpr::Ident(map_name))),
                })
            }
            "set" if args.is_empty() => Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "HashSet".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            }),
            "set" if args.len() == 1 => {
                let lowered_iter =
                    registry_iterable_to_owned_iter_expr(self, &args[0]).or_else(|| {
                        // Generator-style sources may fail strict registry lowering.
                        // Fall back to stmt-support lowering for the source expression only.
                        self.lower_stmt_expr_for_ir(&args[0]).ok().flatten()
                    })?;
                Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "collect::<std::collections::HashSet<_>>".to_string(),
                    args: vec![],
                })
            }
            "iter" if args.len() == 1 => {
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Iterator(_)
                ) {
                    self.try_lower_registry_expr_strict(&args[0])
                } else {
                    Some(registry_box_iterator_expr(
                        registry_iterable_to_owned_iter_expr(self, &args[0])?,
                    ))
                }
            }
            "next" if args.len() == 1 => {
                let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Class { methods, .. } if registry_class_has_next(methods) => {
                        Some(RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "__next__".to_string(),
                            args: vec![],
                        })
                    }
                    _ => Some(RustExpr::MethodCall {
                        receiver: Box::new(lowered_arg),
                        method: "next".to_string(),
                        args: vec![],
                    }),
                }
            }
            "sum" if args.len() == 1 => {
                let elem_ty =
                    crate::resolve_alias_type_for_plain_call(args[0].ty()).iterable_element_type();
                let mut iter_expr = registry_iterable_to_owned_iter_expr(self, &args[0])?;
                if matches!(
                    elem_ty.as_ref().map(Type::resolve_alias),
                    Some(Type::FixedInt(fixed)) if fixed.supports_current_int_builtin_widening()
                ) {
                    iter_expr = crate::RustExpr::MethodCall {
                        receiver: Box::new(iter_expr),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__sum_item".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__sum_item".to_string())),
                                ty: crate::RustType::I64,
                            }),
                            is_move: false,
                        }],
                    };
                }
                let sum_method = if let Some(elem_ty) = elem_ty {
                    let sum_ty = match elem_ty.resolve_alias() {
                        Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening() => {
                            crate::RustType::I64
                        }
                        _ => crate::sifr_type_to_rust_type(&elem_ty),
                    };
                    format!("sum::<{}>", crate::render_type(&sum_ty))
                } else {
                    "sum".to_string()
                };
                let iter_chain = crate::RustExpr::MethodCall {
                    receiver: Box::new(iter_expr),
                    method: sum_method,
                    args: vec![],
                };
                Some(iter_chain)
            }
            "any" if args.len() == 1 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                method: "any".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "x".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::Ident("x".to_string())),
                    is_move: false,
                }],
            }),
            "all" if args.len() == 1 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                method: "all".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "x".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::Ident("x".to_string())),
                    is_move: false,
                }],
            }),
            "reversed" if args.len() == 1 => {
                if let Type::Class { name, methods, .. } =
                    crate::resolve_alias_type_for_plain_call(args[0].ty())
                {
                    if let Some(reversed_ft) =
                        registry_class_method_signature(methods, "__reversed__")
                    {
                        if reversed_ft.params.is_empty() {
                            let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
                            let reversed_call = RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Paren(Box::new(lowered_arg))),
                                    method: "clone".to_string(),
                                    args: vec![],
                                }),
                                method: "__reversed__".to_string(),
                                args: vec![],
                            };
                            let reversed_iter = if matches!(
                                reversed_ft.return_type.as_ref().resolve_alias(),
                                Type::Class { name: ret_name, .. } if ret_name == name
                            ) && registry_class_has_next(methods)
                            {
                                registry_iter_from_next_method_expr(reversed_call)
                            } else if let Type::Class {
                                methods: ret_methods,
                                ..
                            } = reversed_ft.return_type.as_ref().resolve_alias()
                            {
                                if registry_class_has_next(ret_methods) {
                                    registry_iter_from_next_method_expr(reversed_call)
                                } else {
                                    RustExpr::MethodCall {
                                        receiver: Box::new(reversed_call),
                                        method: "into_iter".to_string(),
                                        args: vec![],
                                    }
                                }
                            } else {
                                RustExpr::MethodCall {
                                    receiver: Box::new(reversed_call),
                                    method: "into_iter".to_string(),
                                    args: vec![],
                                }
                            };
                            return Some(registry_box_iterator_expr(reversed_iter));
                        }
                    }
                }
                Some(registry_box_iterator_expr(RustExpr::MethodCall {
                    receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                    method: "rev".to_string(),
                    args: vec![],
                }))
            }
            "zip" if args.is_empty() => Some(registry_box_iterator_expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "iter".to_string(),
                    "empty".to_string(),
                ])),
                args: vec![],
            })),
            "zip" if args.len() == 1 => Some(registry_box_iterator_expr(RustExpr::MethodCall {
                receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__zip_item".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Tuple(vec![RustExpr::Ident(
                        "__zip_item".to_string(),
                    )])),
                    is_move: false,
                }],
            })),
            "zip" if args.len() >= 2 => {
                let zip_iter = registry_zip_iter_expr(self, args)?;
                let tuple_items = (0..args.len())
                    .map(|index| {
                        registry_nested_zip_field_expr(
                            RustExpr::Ident("__zip_item".to_string()),
                            args.len(),
                            index,
                        )
                    })
                    .collect();
                Some(registry_box_iterator_expr(RustExpr::MethodCall {
                    receiver: Box::new(zip_iter),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__zip_item".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Tuple(tuple_items)),
                        is_move: false,
                    }],
                }))
            }
            _ => None,
        }
    }
}

impl RustEmitter {
    pub(crate) fn try_lower_registry_ordering_builtin_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
        _result_ty: Option<&Type>,
    ) -> Option<crate::RustExpr> {
        match func {
            "max" | "min" if args.len() == 1 => {
                let method = if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()).iterable_element_type(),
                    Some(Type::Float)
                ) {
                    format!(
                        "{func}_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))"
                    )
                } else {
                    func.to_owned()
                };
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                    method,
                    args: vec![],
                })
            }
            "sorted" if matches!(args.len(), 1 | 3) => {
                let elem_ty = crate::resolve_alias_type_for_plain_call(args[0].ty())
                    .iterable_element_type()?;
                let vec_name = "__sifr_sorted_v".to_string();
                let collect_expr = registry_iterable_to_vec_expr(self, &args[0])?;
                let mut stmts = vec![crate::RustStmt::Let {
                    mutable: true,
                    name: vec_name.clone(),
                    ty: None,
                    value: collect_expr,
                }];
                if args.len() == 3 && !matches!(args[1], HirExpr::NoneLiteral) {
                    let (_param_types, _conventions, key_return_ty) =
                        registry_callable_signature(&args[1])?;
                    if matches!(key_return_ty, Type::Float) {
                        let left_call = registry_call_callable_with_owned_args(
                            self,
                            &args[1],
                            &[("__left_key".to_string(), elem_ty.clone())],
                        )?;
                        let right_call = registry_call_callable_with_owned_args(
                            self,
                            &args[1],
                            &[("__right_key".to_string(), elem_ty.clone())],
                        )?;
                        stmts.push(crate::RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(vec_name.clone())),
                            method: "sort_by".to_string(),
                            args: vec![RustExpr::ClosureBlock {
                                params: vec![
                                    crate::RustParam::Named {
                                        name: "__left".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    },
                                    crate::RustParam::Named {
                                        name: "__right".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    },
                                ],
                                body: vec![
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "__left_key".to_string(),
                                        ty: None,
                                        value: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__left".to_string(),
                                            )),
                                            method: "clone".to_string(),
                                            args: vec![],
                                        },
                                    },
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "__right_key".to_string(),
                                        ty: None,
                                        value: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__right".to_string(),
                                            )),
                                            method: "clone".to_string(),
                                            args: vec![],
                                        },
                                    },
                                    crate::RustStmt::Return(Some(RustExpr::MethodCall {
                                        receiver: Box::new(left_call),
                                        method: "total_cmp".to_string(),
                                        args: vec![RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(right_call),
                                        }],
                                    })),
                                ],
                                is_move: false,
                                is_async: false,
                            }],
                        }));
                    } else {
                        let left_call = registry_call_callable_with_owned_args(
                            self,
                            &args[1],
                            &[("__left_key".to_string(), elem_ty.clone())],
                        )?;
                        let right_call = registry_call_callable_with_owned_args(
                            self,
                            &args[1],
                            &[("__right_key".to_string(), elem_ty.clone())],
                        )?;
                        stmts.push(crate::RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(vec_name.clone())),
                            method: "sort_by".to_string(),
                            args: vec![RustExpr::ClosureBlock {
                                params: vec![
                                    crate::RustParam::Named {
                                        name: "__left".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    },
                                    crate::RustParam::Named {
                                        name: "__right".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    },
                                ],
                                body: vec![
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "__left_key".to_string(),
                                        ty: None,
                                        value: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__left".to_string(),
                                            )),
                                            method: "clone".to_string(),
                                            args: vec![],
                                        },
                                    },
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "__right_key".to_string(),
                                        ty: None,
                                        value: RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__right".to_string(),
                                            )),
                                            method: "clone".to_string(),
                                            args: vec![],
                                        },
                                    },
                                    crate::RustStmt::Return(Some(RustExpr::MethodCall {
                                        receiver: Box::new(left_call),
                                        method: "cmp".to_string(),
                                        args: vec![RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(right_call),
                                        }],
                                    })),
                                ],
                                is_move: false,
                                is_async: false,
                            }],
                        }));
                    }
                } else if matches!(elem_ty, Type::Float) {
                    stmts.push(crate::RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(vec_name.clone())),
                        method: "sort_by".to_string(),
                        args: vec![RustExpr::Path(vec![
                            "f64".to_string(),
                            "total_cmp".to_string(),
                        ])],
                    }));
                } else {
                    stmts.push(crate::RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(vec_name.clone())),
                        method: "sort".to_string(),
                        args: vec![],
                    }));
                }
                if args.len() == 3 {
                    let reverse_expr = self.try_lower_registry_expr_strict(&args[2])?;
                    stmts.push(crate::RustStmt::Expr(RustExpr::If {
                        cond: Box::new(reverse_expr),
                        then_expr: Box::new(RustExpr::Block {
                            stmts: vec![crate::RustStmt::Expr(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident(vec_name.clone())),
                                method: "reverse".to_string(),
                                args: vec![],
                            })],
                            expr: None,
                        }),
                        else_expr: None,
                    }));
                }
                Some(RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(RustExpr::Ident(vec_name))),
                })
            }
            "enumerate" if matches!(args.len(), 1 | 2) => {
                let iter_expr = registry_iterable_to_owned_iter_expr(self, &args[0])?;
                let start_expr = if args.len() == 2 {
                    self.try_lower_registry_expr_strict(&args[1])?
                } else {
                    RustExpr::Literal(crate::RustLiteral::Int(0))
                };
                Some(registry_box_iterator_expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(iter_expr),
                        method: "enumerate".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__pair".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Tuple(vec![
                            RustExpr::BinOp {
                                left: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                        field: "0".to_string(),
                                    }),
                                    ty: crate::RustType::I64,
                                }),
                                op: "+".to_string(),
                                right: Box::new(start_expr),
                            },
                            RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("__pair".to_string())),
                                field: "1".to_string(),
                            },
                        ])),
                        is_move: false,
                    }],
                }))
            }
            "filter" if args.len() == 2 => {
                let item_ty = crate::resolve_alias_type_for_plain_call(args[1].ty())
                    .iterable_element_type()?;
                let filtered_iter = RustExpr::MethodCall {
                    receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[1])?),
                    method: "filter".to_string(),
                    args: vec![RustExpr::ClosureBlock {
                        params: vec![crate::RustParam::Named {
                            name: "__filter_item".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "__filter_value".to_string(),
                                ty: None,
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident(
                                        "__filter_item".to_string(),
                                    )),
                                    method: "clone".to_string(),
                                    args: vec![],
                                },
                            },
                            crate::RustStmt::Return(Some(registry_call_callable_with_owned_args(
                                self,
                                &args[0],
                                &[("__filter_value".to_string(), item_ty)],
                            )?)),
                        ],
                        is_move: false,
                        is_async: false,
                    }],
                };
                Some(registry_box_iterator_expr(filtered_iter))
            }
            "map" if args.len() >= 2 => {
                let iter_expr = registry_zip_iter_expr(self, &args[1..])?;
                let arg_count = args.len() - 1;
                if arg_count == 1 {
                    Some(registry_box_iterator_expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(iter_expr),
                            method: "map".to_string(),
                            args: vec![RustExpr::ClosureBlock {
                                params: vec![crate::RustParam::Named {
                                    name: "__map_item".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: vec![crate::RustStmt::Return(Some(
                                    registry_call_callable_with_owned_args(
                                        self,
                                        &args[0],
                                        &[(
                                            "__map_item".to_string(),
                                            crate::resolve_alias_type_for_plain_call(args[1].ty())
                                                .iterable_element_type()?,
                                        )],
                                    )?,
                                ))],
                                is_move: false,
                                is_async: false,
                            }],
                        }),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }))
                } else {
                    let mut body = Vec::new();
                    let mut bindings = Vec::new();
                    for index in 0..arg_count {
                        let name = format!("__map_arg_{index}");
                        let arg_ty = crate::resolve_alias_type_for_plain_call(args[index + 1].ty())
                            .iterable_element_type()?;
                        bindings.push((name.clone(), arg_ty));
                        body.push(crate::RustStmt::Let {
                            mutable: false,
                            name,
                            ty: None,
                            value: registry_nested_zip_field_expr(
                                RustExpr::Ident("__map_item".to_string()),
                                arg_count,
                                index,
                            ),
                        });
                    }
                    body.push(crate::RustStmt::Return(Some(
                        registry_call_callable_with_owned_args(self, &args[0], &bindings)?,
                    )));
                    Some(registry_box_iterator_expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(iter_expr),
                            method: "map".to_string(),
                            args: vec![RustExpr::ClosureBlock {
                                params: vec![crate::RustParam::Named {
                                    name: "__map_item".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body,
                                is_move: false,
                                is_async: false,
                            }],
                        }),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }))
                }
            }
            _ => None,
        }
    }
}
