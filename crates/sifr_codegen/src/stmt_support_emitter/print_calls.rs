use super::{
    call_expr_parts, canonical_plain_call_name_for_ir, HirExpr, HirFStringPart, RustEmitter,
};

fn is_none_type(ty: &sifr_type_system::Type) -> bool {
    matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        sifr_type_system::Type::None
    )
}

fn none_display_expr() -> crate::RustExpr {
    crate::RustExpr::Literal(crate::RustLiteral::Str("None".to_string()))
}

impl RustEmitter {
    pub(crate) fn try_lower_stmt_expr_statement_only(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::IntrinsicCall {
            intrinsic, args, ..
        } = expr
        {
            return Ok(self.try_lower_registry_intrinsic_call_expr(*intrinsic, args, expr.ty()));
        }
        if let Some((func, args)) = call_expr_parts(expr) {
            if func == "print" {
                return self.lower_print_call_expr_for_ir(args);
            }
            if let Some(lowered_builtin) =
                self.try_lower_registry_builtin_call_expr(func, args, Some(expr.ty()))
            {
                return Ok(Some(lowered_builtin));
            }
            if let Some(lowered_plain) =
                self.try_lower_registry_plain_call_with_signature(func, args)
            {
                return Ok(Some(lowered_plain));
            }
            if func == "iter" && args.len() == 1 {
                return self.lower_iter_source_expr_for_ir(&args[0]);
            }
            if func == "next" && args.len() == 1 {
                let Some(lowered_iterator) = self.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "next".to_string(),
                    args: vec![],
                }));
            }
            if func == "anext" && args.len() == 1 {
                let Some(lowered_iterator) = self.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "anext".to_string(),
                    args: vec![],
                }));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(self.rewrite_stdlib_constant_idents_in_expr(lowered_arg));
            }
            let canonical_func = canonical_plain_call_name_for_ir(func);
            lowered_args = self.adapt_plain_call_args_with_signature_for_ir(
                canonical_func,
                args,
                lowered_args,
            );
            if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                for capture in captures {
                    lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            let lowered_func = if canonical_func.contains("::") {
                crate::RustExpr::Path(canonical_func.split("::").map(str::to_string).collect())
            } else {
                crate::RustExpr::Ident(canonical_func.to_string())
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(lowered_func),
                args: lowered_args,
            }));
        }

        match expr {
            HirExpr::MethodCall {
                object,
                method,
                args,
                receiver_convention,
                ..
            } => {
                let needs_self_field_clone_suppression =
                    self.method_call_needs_field_clone_suppression(object, *receiver_convention);
                let suppression_prev = self.pending_self_field_clone_suppression;
                if needs_self_field_clone_suppression {
                    self.pending_self_field_clone_suppression += 1;
                }

                let lowered =
                    self.try_lower_registry_method_call_expr(object, method, args, expr.ty())?;

                if needs_self_field_clone_suppression
                    && self.pending_self_field_clone_suppression > suppression_prev
                {
                    self.pending_self_field_clone_suppression -= 1;
                }

                Ok(lowered)
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn lower_print_call_expr_for_ir(
        &mut self,
        args: &[HirExpr],
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if args.is_empty() {
            return Ok(Some(crate::RustExpr::MacroCall {
                name: "println".to_string(),
                args: vec![],
            }));
        }

        if args.len() == 1 {
            let arg = &args[0];
            if is_none_type(arg.ty()) {
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str: "None".to_string(),
                    args: vec![],
                }));
            }
            if let HirExpr::StringLiteral(value) = arg {
                let escaped = value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('{', "{{")
                    .replace('}', "}}");
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str: escaped,
                    args: vec![],
                }));
            }
            if let HirExpr::FString { parts, .. } = arg {
                let mut format_str = String::new();
                let mut lowered_args = Vec::new();
                for part in parts {
                    match part {
                        HirFStringPart::Literal(text) => {
                            format_str.push_str(&text.replace('{', "{{").replace('}', "}}"));
                        }
                        HirFStringPart::Expr(inner) => {
                            if is_none_type(inner.ty()) {
                                format_str.push_str("{}");
                                lowered_args.push(none_display_expr());
                                continue;
                            }
                            let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                                return Ok(None);
                            };
                            format_str.push_str("{}");
                            if let Some(option_inner_ty) =
                                Self::option_inner_type_for_ir(inner.ty())
                            {
                                let option_format_str =
                                    if Self::uses_debug_display_format_for_ir(option_inner_ty) {
                                        "{:?}".to_string()
                                    } else {
                                        "{}".to_string()
                                    };
                                lowered_args.push(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_inner,
                                    ))),
                                    method: "map_or".to_string(),
                                    args: vec![
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str("None".to_string()),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                        crate::RustExpr::Closure {
                                            params: vec![crate::RustParam::Named {
                                                name: "__v".to_string(),
                                                ty: crate::RustType::Named("_".to_string()),
                                            }],
                                            body: Box::new(crate::RustExpr::FormatMacro {
                                                name: "format".to_string(),
                                                format_str: option_format_str,
                                                args: vec![crate::RustExpr::Ident(
                                                    "__v".to_string(),
                                                )],
                                            }),
                                            is_move: false,
                                        },
                                    ],
                                });
                            } else {
                                lowered_args.push(lowered_inner);
                            }
                        }
                    }
                }
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str,
                    args: lowered_args,
                }));
            }

            let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                return Ok(None);
            };
            if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                let option_format_str = if Self::uses_debug_display_format_for_ir(inner) {
                    "{:?}".to_string()
                } else {
                    "{}".to_string()
                };
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str: "{}".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: option_format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }],
                }));
            }
            let format_str = if Self::uses_debug_display_format_for_ir(arg.ty()) {
                "{:?}"
            } else {
                "{}"
            };
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: format_str.to_string(),
                args: vec![lowered_arg],
            }));
        }

        if let HirExpr::StringLiteral(fmt) = &args[0] {
            let mut lowered_args = Vec::with_capacity(args.len().saturating_sub(1));
            for arg in args.iter().skip(1) {
                if is_none_type(arg.ty()) {
                    lowered_args.push(none_display_expr());
                    continue;
                }
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: fmt.clone(),
                args: lowered_args,
            }));
        }

        let mut format_parts = Vec::with_capacity(args.len());
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            if is_none_type(arg.ty()) {
                lowered_args.push(none_display_expr());
                format_parts.push("{}");
                continue;
            }
            let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                return Ok(None);
            };
            if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                let option_format_str = if Self::uses_debug_display_format_for_ir(inner) {
                    "{:?}".to_string()
                } else {
                    "{}".to_string()
                };
                lowered_args.push(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "map_or".to_string(),
                    args: vec![
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                                "None".to_string(),
                            ))),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                        crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__v".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: option_format_str,
                                args: vec![crate::RustExpr::Ident("__v".to_string())],
                            }),
                            is_move: false,
                        },
                    ],
                });
                format_parts.push("{}");
            } else {
                lowered_args.push(lowered_arg);
                format_parts.push(if Self::uses_debug_display_format_for_ir(arg.ty()) {
                    "{:?}"
                } else {
                    "{}"
                });
            }
        }
        Ok(Some(crate::RustExpr::FormatMacro {
            name: "println".to_string(),
            format_str: format_parts.join(" "),
            args: lowered_args,
        }))
    }

    pub(crate) fn object_name_expr_for_ir(object: &str) -> crate::RustExpr {
        if object.contains("::") {
            return crate::RustExpr::Path(object.split("::").map(ToString::to_string).collect());
        }
        crate::RustExpr::Ident(object.to_string())
    }

    pub(crate) fn is_some_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Some")
        )
    }

    pub(crate) fn is_box_new_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Box::new")
        )
    }

    pub(crate) fn ensure_some_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    && args.len() == 1 =>
            {
                let mut args_iter = args.into_iter();
                let Some(inner) = args_iter.next() else {
                    unreachable!("Some(_) call must have exactly one argument");
                };
                if Self::is_box_new_call_expr_for_ir(&inner) {
                    crate::RustExpr::FnCall {
                        func,
                        args: vec![inner],
                    }
                } else {
                    crate::RustExpr::FnCall {
                        func,
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![inner],
                        }],
                    }
                }
            }
            other => crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![other],
                }],
            },
        }
    }

    pub(crate) fn ensure_option_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
        if matches!(expr, crate::RustExpr::Literal(crate::RustLiteral::None)) {
            return expr;
        }
        if Self::is_some_call_expr_for_ir(&expr) {
            return Self::ensure_some_box_inner_for_ir(expr);
        }
        crate::RustExpr::MethodCall {
            receiver: Box::new(expr),
            method: "map".to_string(),
            args: vec![crate::RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_option_value".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![crate::RustExpr::Ident("__sifr_option_value".to_string())],
                }),
                is_move: false,
            }],
        }
    }
}
