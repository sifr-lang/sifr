use crate::{intrinsics, methods, RustEmitter};
use sifr_hir::HirExpr;
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    pub(crate) fn write_registry_expr(&mut self, expr: &crate::RustExpr) {
        self.output.push_str(&crate::render_expr(expr));
    }

    /// Check if a name is a stdlib constant.
    pub(crate) fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
            && self.intrinsic_functions.contains(name)
    }

    /// Emit a stdlib constant value.
    pub(crate) fn emit_stdlib_constant(&mut self, name: &str) {
        let lowered = match name {
            "pi" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "PI".to_string(),
            ]),
            "e" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "E".to_string(),
            ]),
            "tau" => crate::RustExpr::Path(vec![
                "std".to_string(),
                "f64".to_string(),
                "consts".to_string(),
                "TAU".to_string(),
            ]),
            "inf" => crate::RustExpr::Path(vec!["f64".to_string(), "INFINITY".to_string()]),
            "nan" => crate::RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
            _ => crate::RustExpr::Ident(name.to_string()),
        };
        self.write_registry_expr(&lowered);
    }

    fn emit_registry_plain_call_expr(&mut self, func: &str, args: &[HirExpr]) {
        let lowered_args = self
            .try_lower_registry_exprs_strict(args)
            .unwrap_or_else(|| panic!("structured intrinsic-call lowering missing for args: {args:?}"));
        let lowered = if func.contains("::") {
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(
                    func.split("::").map(str::to_string).collect(),
                )),
                args: lowered_args,
            }
        } else {
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Ident(func.to_string())),
                args: lowered_args,
            }
        };
        self.write_registry_expr(&lowered);
    }

    /// Emit an intrinsic function call with the correct Rust code.
    pub(crate) fn emit_intrinsic_call(&mut self, func: &str, args: &[HirExpr]) {
        if self.try_emit_intrinsic_via_registry(func, args) {
            return;
        }

        // Unknown intrinsic name: still lower and emit as a normal call expression.
        self.emit_registry_plain_call_expr(func, args);
    }

    pub(crate) fn try_emit_intrinsic_via_registry(&mut self, func: &str, args: &[HirExpr]) -> bool {
        let Some(lowered_expr) = self.try_lower_registry_intrinsic_call_expr(func, args) else {
            return false;
        };
        self.write_registry_expr(&lowered_expr);
        true
    }

    pub(crate) fn try_emit_method_via_registry(
        &mut self,
        object_ty: &Type,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> bool {
        let is_deque_data_field = self.is_deque_data_field(object);
        let Some(object_expr) = self.try_lower_registry_expr_strict(object) else {
            return false;
        };
        let Some(mut arg_exprs) = self.try_lower_registry_exprs_strict(args) else {
            return false;
        };

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
        {
            // Clone TypeVar list args to avoid move issues.
            if matches!(args[0].ty(), Type::TypeVar(_)) {
                arg_exprs[0] = crate::RustExpr::MethodCall {
                    receiver: Box::new(arg_exprs[0].clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            // Clone borrowed/mut-borrowed move-owned values.
            let needs_clone = if let HirExpr::Name { name, ty } = &args[1] {
                (self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()))
                    && ty.ownership() != sifr_type_system::OwnershipKind::Copy
            } else {
                false
            };
            if needs_clone {
                arg_exprs[1] = crate::RustExpr::MethodCall {
                    receiver: Box::new(arg_exprs[1].clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }

        let Some(lowered) = methods::lower_method_with_context(
            object_ty,
            method,
            &object_expr,
            &arg_exprs,
            is_deque_data_field,
        ) else {
            return false;
        };
        self.write_registry_expr(&lowered.expr);
        true
    }

    pub(crate) fn try_lower_registry_exprs_strict(
        &mut self,
        exprs: &[HirExpr],
    ) -> Option<Vec<crate::RustExpr>> {
        let mut lowered = Vec::with_capacity(exprs.len());
        for expr in exprs {
            lowered.push(self.try_lower_registry_expr_strict(expr)?);
        }
        Some(lowered)
    }

    pub(crate) fn try_lower_registry_expr_strict(&mut self, expr: &HirExpr) -> Option<crate::RustExpr> {
        match self.try_lower_registry_expr_result(expr) {
            Ok(Some(lowered_expr)) => Some(lowered_expr),
            Ok(None) => self.try_lower_registry_expr_recursive(expr),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                None
            }
        }
    }

    fn try_lower_registry_expr_recursive(&mut self, expr: &HirExpr) -> Option<crate::RustExpr> {
        match expr {
            HirExpr::Name { name, .. } => Some(crate::RustExpr::Ident(name.clone())),
            HirExpr::FieldAccess { object, field, .. } => Some(crate::RustExpr::Field {
                expr: Box::new(self.try_lower_registry_expr_strict(object)?),
                field: field.clone(),
            }),
            HirExpr::Call { func, args, .. } => {
                if let Some(lowered) = self.try_lower_registry_intrinsic_call_expr(func, args) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_registry_builtin_call_expr(func, args) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_registry_plain_call_with_signature(func, args)
                {
                    return Some(lowered);
                }
                if func.contains("::") {
                    return Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(
                            func.split("::").map(str::to_string).collect(),
                        )),
                        args: self.try_lower_registry_exprs_strict(args)?,
                    });
                }
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Ident(func.to_string())),
                    args: self.try_lower_registry_exprs_strict(args)?,
                })
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                let object_expr = self.try_lower_registry_expr_strict(object)?;
                let mut arg_exprs = self.try_lower_registry_exprs_strict(args)?;
                if let Some(lowered) = methods::lower_method_with_context(
                    object.ty(),
                    method,
                    &object_expr,
                    &arg_exprs,
                    self.is_deque_data_field(object),
                ) {
                    return Some(lowered.expr);
                }
                if let Some(method_params) = self.resolve_registry_method_params(object.ty(), method) {
                    for (idx, arg_expr) in arg_exprs.iter_mut().enumerate() {
                        if let (Some((param_ty, convention)), Some(arg)) =
                            (method_params.get(idx), args.get(idx))
                        {
                            let adjusted = self.apply_registry_method_arg_convention(
                                arg,
                                param_ty,
                                *convention,
                                arg_expr.clone(),
                            );
                            *arg_expr = adjusted;
                        }
                    }
                }
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(object_expr),
                    method: method.clone(),
                    args: arg_exprs,
                })
            }
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => self.try_lower_registry_compare_expr(left, ops, comparators),
            HirExpr::BinOp {
                left,
                op,
                right,
                ty,
            } if op == "**" => {
                let left_expr = self.try_lower_registry_expr_strict(left)?;
                let right_expr = self.try_lower_registry_expr_strict(right)?;
                match crate::resolve_alias_type_for_plain_call(ty) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(left_expr),
                        method: "pow".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(right_expr),
                            ty: crate::RustType::Named("u32".to_string()),
                        }],
                    }),
                    Type::Float => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(left_expr),
                            ty: crate::RustType::F64,
                        }),
                        method: "powf".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(right_expr),
                            ty: crate::RustType::F64,
                        }],
                    }),
                    _ => None,
                }
            }
            HirExpr::BinOp {
                left,
                op,
                right,
                ty,
            } if op == "+" && matches!(ty, Type::Str | Type::LiteralStr(_)) => {
                Some(crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}{}".to_string(),
                    args: vec![
                        self.try_lower_registry_expr_strict(left)?,
                        self.try_lower_registry_expr_strict(right)?,
                    ],
                })
            }
            HirExpr::BinOp {
                left,
                op,
                right,
                ty,
            } if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%")
                && matches!(ty, Type::Float | Type::Int | Type::LiteralInt(_)) =>
            {
                Some(crate::RustExpr::BinOp {
                    left: Box::new(self.try_lower_registry_expr_strict(left)?),
                    op: op.clone(),
                    right: Box::new(self.try_lower_registry_expr_strict(right)?),
                })
            }
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } if matches!(
                crate::resolve_alias_type_for_plain_call(object.ty()),
                Type::Str | Type::LiteralStr(_)
            ) => self.try_lower_registry_string_slice_expr(
                object,
                start.as_deref(),
                stop.as_deref(),
                step.as_deref(),
            ),
            HirExpr::DictLiteral { keys, values, .. } => {
                self.try_lower_registry_dict_literal_expr(keys, values)
            }
            HirExpr::SetLiteral { elements, .. } => {
                self.try_lower_registry_set_literal_expr(elements)
            }
            _ => None,
        }
    }

    fn try_lower_registry_dict_literal_expr(
        &mut self,
        keys: &[HirExpr],
        values: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        if keys.len() != values.len() {
            return None;
        }

        let map_ident = "__sifr_registry_dict_literal".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: map_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashMap".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];

        for (key, value) in keys.iter().zip(values.iter()) {
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(map_ident.clone())),
                method: "insert".to_string(),
                args: vec![
                    self.try_lower_registry_expr_strict(key)?,
                    self.try_lower_registry_expr_strict(value)?,
                ],
            }));
        }

        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(map_ident))),
        })
    }

    fn try_lower_registry_set_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let set_ident = "__sifr_registry_set_literal".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: set_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashSet".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];

        for element in elements {
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(set_ident.clone())),
                method: "insert".to_string(),
                args: vec![self.try_lower_registry_expr_strict(element)?],
            }));
        }

        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(set_ident))),
        })
    }

    fn try_lower_registry_intrinsic_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let ir_args = self.try_lower_registry_exprs_strict(args)?;
        let lowered = intrinsics::lower_intrinsic(func, &ir_args)?;
        self.apply_intrinsic_registry_side_effects(func, &lowered);
        Some(lowered.expr)
    }

    fn apply_intrinsic_registry_side_effects(
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
            self.runtime_needs.needs_file_handles = true;
        }
        if func == "builtin_open" {
            self.used_stdlib_modules.insert("io".to_string());
        }
        if matches!(func, "set_global_level" | "get_global_level") {
            self.runtime_needs.needs_logging_state = true;
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

    pub(crate) fn try_lower_registry_builtin_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        match func {
            "sum" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                let iter_chain = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }),
                    method: if let Type::List(elem_ty) =
                        crate::resolve_alias_type_for_plain_call(args[0].ty())
                    {
                        format!(
                            "sum::<{}>",
                            crate::render_type(&crate::sifr_type_to_rust_type(elem_ty))
                        )
                    } else {
                        "sum".to_string()
                    },
                    args: vec![],
                };
                Some(iter_chain)
            }
            "any" if args.len() == 1 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                        self.try_lower_registry_expr_strict(&args[0])?,
                    ))),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "any".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "x".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "x".to_string(),
                    )))),
                    is_move: false,
                }],
            }),
            "all" if args.len() == 1 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                        self.try_lower_registry_expr_strict(&args[0])?,
                    ))),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "all".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "x".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "x".to_string(),
                    )))),
                    is_move: false,
                }],
            }),
            "reversed" if args.len() == 1 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                self.try_lower_registry_expr_strict(&args[0])?,
                            ))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }),
                    method: "rev".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<_>>".to_string(),
                args: vec![],
            }),
            "zip" if args.len() == 2 => Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                self.try_lower_registry_expr_strict(&args[0])?,
                            ))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }),
                    method: "zip".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                self.try_lower_registry_expr_strict(&args[1])?,
                            ))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }],
                }),
                method: "collect::<Vec<_>>".to_string(),
                args: vec![],
            }),
            "abs" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                    method: "abs".to_string(),
                    args: vec![],
                })
            }
            "round" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                        method: "round".to_string(),
                        args: vec![],
                    }),
                    ty: crate::RustType::I64,
                })
            }
            "repr" if args.len() == 1 => Some(crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "{:?}".to_string(),
                args: vec![self.try_lower_registry_expr_strict(&args[0])?],
            }),
            "max" | "min" if args.len() == 2 => {
                let left = self.try_lower_registry_expr_strict(&args[0])?;
                let right = self.try_lower_registry_expr_strict(&args[1])?;
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Float
                ) || matches!(
                    crate::resolve_alias_type_for_plain_call(args[1].ty()),
                    Type::Float
                ) {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(left),
                        method: func.to_string(),
                        args: vec![right],
                    })
                } else {
                    Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "std".to_string(),
                            "cmp".to_string(),
                            func.to_string(),
                        ])),
                        args: vec![left, right],
                    })
                }
            }
            "pow" if args.len() == 2 => {
                let base = self.try_lower_registry_expr_strict(&args[0])?;
                let exp = self.try_lower_registry_expr_strict(&args[1])?;
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Int | Type::LiteralInt(_)
                ) && matches!(
                    crate::resolve_alias_type_for_plain_call(args[1].ty()),
                    Type::Int | Type::LiteralInt(_)
                ) {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(base),
                        method: "pow".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(exp),
                            ty: crate::RustType::Named("u32".to_string()),
                        }],
                    })
                } else {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(base),
                            ty: crate::RustType::F64,
                        }),
                        method: "powf".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(exp),
                            ty: crate::RustType::F64,
                        }],
                    })
                }
            }
            "bool" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::BinOp {
                        left: Box::new(lowered),
                        op: "!=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    }),
                    Type::Float => Some(crate::RustExpr::BinOp {
                        left: Box::new(lowered),
                        op: "!=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(0.0))),
                    }),
                    Type::Str | Type::List(_) | Type::Dict(_, _) => {
                        Some(crate::RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered),
                                method: "is_empty".to_string(),
                                args: vec![],
                            }),
                        })
                    }
                    Type::Tuple(elems) => Some(crate::RustExpr::Literal(crate::RustLiteral::Bool(
                        !elems.is_empty(),
                    ))),
                    Type::Bool => Some(lowered),
                    Type::None => Some(crate::RustExpr::Literal(crate::RustLiteral::Bool(false))),
                    _ => Some(lowered),
                }
            }
            "float" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                if matches!(args[0].ty(), Type::Float) {
                    Some(lowered)
                } else {
                    Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::F64,
                    })
                }
            }
            "int" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                if matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    Some(lowered)
                } else {
                    Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::I64,
                    })
                }
            }
            "bigint" if args.len() == 1 => Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "BigInt".to_string(),
                    "from".to_string(),
                ])),
                args: vec![self.try_lower_registry_expr_strict(&args[0])?],
            }),
            "str" if args.len() == 1 => Some(crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "{}".to_string(),
                args: vec![self.try_lower_registry_expr_strict(&args[0])?],
            }),
            _ => None,
        }
    }

    fn try_lower_registry_plain_call_with_signature(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let param_info = self
            .func_signatures
            .get(func)
            .map(|(pts, _)| pts.clone())
            .or_else(|| self.callable_var_conventions.get(func).cloned())?;
        if param_info.len() != args.len() {
            return None;
        }

        let mut lowered_args = Vec::with_capacity(args.len());
        for ((param_ty, convention), arg) in param_info.iter().zip(args.iter()) {
            let mut lowered_arg = self.try_lower_registry_expr_strict(arg)?;
            if *convention == ParamConvention::Borrow
                && !self.arg_is_already_borrowed_for_registry_call(arg, &lowered_arg)
            {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(lowered_arg),
                };
            } else if *convention == ParamConvention::MutBorrow
                && !self.arg_is_already_mut_borrowed_for_registry_call(arg, &lowered_arg)
            {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(lowered_arg),
                };
            }

            if crate::helpers::is_option_type(param_ty)
                && !crate::helpers::is_option_type(arg.ty())
                && !matches!(arg, HirExpr::NoneLiteral)
            {
                lowered_arg = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_arg],
                };
            }

            lowered_args.push(lowered_arg);
        }

        Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Ident(func.to_string())),
            args: lowered_args,
        })
    }

    fn resolve_registry_method_params(
        &self,
        object_ty: &Type,
        method: &str,
    ) -> Option<Vec<(Type, ParamConvention)>> {
        let Type::Class { name, methods, .. } = crate::resolve_alias_type_for_plain_call(object_ty) else {
            return None;
        };
        self.func_signatures
            .get(&format!("{name}::{method}"))
            .map(|(params, _)| params.clone())
            .or_else(|| {
                methods
                    .iter()
                    .find(|(method_name, _)| method_name == method)
                    .map(|(_, fty)| {
                        let self_offset = usize::from(
                            fty.params
                                .first()
                                .is_some_and(|(param_name, _, _)| param_name == "self"),
                        );
                        fty.params
                            .iter()
                            .skip(self_offset)
                            .map(|(_, ty, conv)| (ty.clone(), *conv))
                            .collect::<Vec<_>>()
                    })
            })
    }

    fn apply_registry_method_arg_convention(
        &self,
        arg: &HirExpr,
        param_ty: &Type,
        convention: ParamConvention,
        mut lowered_arg: crate::RustExpr,
    ) -> crate::RustExpr {
        if convention == ParamConvention::Borrow
            && !self.arg_is_already_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_arg),
            };
        } else if convention == ParamConvention::MutBorrow
            && !self.arg_is_already_mut_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(lowered_arg),
            };
        }

        if crate::helpers::is_option_type(param_ty)
            && !crate::helpers::is_option_type(arg.ty())
            && !matches!(arg, HirExpr::NoneLiteral)
        {
            lowered_arg = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_arg],
            };
        }
        lowered_arg
    }

    fn arg_is_already_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if matches!(lowered, crate::RustExpr::Ref { .. }) {
            return true;
        }
        if let HirExpr::Name { name, .. } = arg {
            return self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name);
        }
        false
    }

    fn arg_is_already_mut_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if let crate::RustExpr::Ref { mutable, .. } = lowered {
            return *mutable;
        }
        if let HirExpr::Name { name, .. } = arg {
            return self.mut_borrowed_params.contains(name);
        }
        false
    }

    fn try_lower_registry_compare_expr(
        &mut self,
        left: &HirExpr,
        ops: &[String],
        comparators: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        if ops.is_empty() || ops.len() != comparators.len() {
            return None;
        }
        let mut lhs_expr = left;
        let mut chained: Option<crate::RustExpr> = None;
        for (idx, op) in ops.iter().enumerate() {
            let rhs_expr = comparators.get(idx)?;
            let lowered_op = match op.as_str() {
                "==" | "!=" | "<" | "<=" | ">" | ">=" => op.clone(),
                "is" => "==".to_string(),
                "is not" => "!=".to_string(),
                _ => return None,
            };
            let comparison = crate::RustExpr::BinOp {
                left: Box::new(self.try_lower_registry_expr_strict(lhs_expr)?),
                op: lowered_op,
                right: Box::new(self.try_lower_registry_expr_strict(rhs_expr)?),
            };
            chained = Some(if let Some(prev) = chained {
                crate::RustExpr::BinOp {
                    left: Box::new(prev),
                    op: "&&".to_string(),
                    right: Box::new(comparison),
                }
            } else {
                comparison
            });
            lhs_expr = rhs_expr;
        }
        chained
    }

    fn try_eval_const_int_expr(expr: &HirExpr) -> Option<i64> {
        match expr {
            HirExpr::IntLiteral(value) => Some(*value),
            HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
                if let HirExpr::IntLiteral(value) = operand.as_ref() {
                    Some(-*value)
                } else {
                    None
                }
            }
            HirExpr::UnaryOp { op, operand, .. } if op == "+" => {
                if let HirExpr::IntLiteral(value) = operand.as_ref() {
                    Some(*value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn usize_cast_literal(value: i64) -> crate::RustExpr {
        crate::RustExpr::Cast {
            expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(value))),
            ty: crate::RustType::Named("usize".to_string()),
        }
    }

    fn try_lower_registry_string_slice_expr(
        &mut self,
        object: &HirExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
        step: Option<&HirExpr>,
    ) -> Option<crate::RustExpr> {
        let object_expr = self.try_lower_registry_expr_strict(object)?;
        if let Some(step_expr) = step {
            // Structured lowering for full-string step slicing used by display refs, e.g. s[::2], s[::-1].
            if start.is_none() && stop.is_none() {
                let step_value = Self::try_eval_const_int_expr(step_expr)?;
                if step_value == 0 {
                    return None;
                }
                let mut iter_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(object_expr),
                    method: "chars".to_string(),
                    args: vec![],
                };
                if step_value < 0 {
                    iter_expr = crate::RustExpr::MethodCall {
                        receiver: Box::new(iter_expr),
                        method: "rev".to_string(),
                        args: vec![],
                    };
                }
                let magnitude = step_value.checked_abs()?;
                if magnitude > 1 {
                    iter_expr = crate::RustExpr::MethodCall {
                        receiver: Box::new(iter_expr),
                        method: "step_by".to_string(),
                        args: vec![Self::usize_cast_literal(magnitude)],
                    };
                }
                return Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(iter_expr),
                    method: "collect::<String>".to_string(),
                    args: vec![],
                });
            }
            return None;
        }

        let chars_count_usize = crate::RustExpr::Cast {
            expr: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(object_expr.clone()),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "count".to_string(),
                args: vec![],
            }),
            ty: crate::RustType::Named("usize".to_string()),
        };
        let start_usize = if let Some(start_expr) = start {
            crate::RustExpr::Cast {
                expr: Box::new(self.try_lower_registry_expr_strict(start_expr)?),
                ty: crate::RustType::Named("usize".to_string()),
            }
        } else {
            crate::RustExpr::Cast {
                expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                ty: crate::RustType::Named("usize".to_string()),
            }
        };
        let stop_usize = if let Some(stop_expr) = stop {
            crate::RustExpr::Cast {
                expr: Box::new(self.try_lower_registry_expr_strict(stop_expr)?),
                ty: crate::RustType::Named("usize".to_string()),
            }
        } else {
            chars_count_usize
        };
        let take_len = crate::RustExpr::BinOp {
            left: Box::new(stop_usize),
            op: "-".to_string(),
            right: Box::new(start_usize.clone()),
        };

        Some(crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "skip".to_string(),
                    args: vec![start_usize],
                }),
                method: "take".to_string(),
                args: vec![take_len],
            }),
            method: "collect::<String>".to_string(),
            args: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn emit_intrinsic_call_has_no_pre_registry_match_dispatch() {
        let src = include_str!("intrinsic_method_emitters.rs");
        let start = src
            .find("pub(crate) fn emit_intrinsic_call")
            .expect("emit_intrinsic_call should exist");
        let end = src
            .find("pub(crate) fn try_emit_intrinsic_via_registry")
            .expect("try_emit_intrinsic_via_registry should exist");
        let emit_block = &src[start..end];
        assert!(!emit_block.contains("match func"));
    }

    #[test]
    fn registry_arg_lowering_avoids_inline_rawcode_shims() {
        let src = include_str!("intrinsic_method_emitters.rs");
        let prod_src = src.split("\n#[cfg(test)]").next().unwrap_or(src);
        assert!(prod_src.contains("fn try_lower_registry_expr_strict("));
        assert!(prod_src.contains("fn try_lower_registry_exprs_strict("));
        assert!(prod_src.contains("fn try_lower_registry_expr_recursive("));
        let helper_defs = prod_src
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("fn try_lower_registry_expr")
                    || trimmed.starts_with("pub(crate) fn try_lower_registry_expr")
            })
            .count();
        assert_eq!(helper_defs, 3, "unexpected registry expr helper set");
        assert!(!prod_src.contains("lower_registry_expr_with_string_path"));
        assert!(!prod_src.contains("render_expr_via_string_only("));
        assert!(!prod_src.contains("RustExpr::RawCode("));
    }
}
