impl RustEmitter {
    pub(crate) fn try_lower_stmt_expr_statement_only(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some((func, args)) = call_expr_parts(expr) {
            if func == "print" {
                return self.lower_print_call_expr_for_ir(args);
            }
            if let Some(lowered_intrinsic) = self.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
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
                ..
            } => {
                let needs_self_field_clone_suppression =
                    self.method_call_needs_field_clone_suppression(object, method);
                let suppression_prev = self.pending_self_field_clone_suppression;
                if needs_self_field_clone_suppression {
                    self.pending_self_field_clone_suppression += 1;
                }

                let lowered =
                    self.try_lower_registry_method_call_expr(object, method, args, expr.ty());

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

    fn lower_print_call_expr_for_ir(
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

    fn object_name_expr_for_ir(object: &str) -> crate::RustExpr {
        if object.contains("::") {
            return crate::RustExpr::Path(object.split("::").map(ToString::to_string).collect());
        }
        crate::RustExpr::Ident(object.to_string())
    }

    fn is_some_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Some")
        )
    }

    fn is_box_new_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Box::new")
        )
    }

    fn ensure_some_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
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

    fn ensure_option_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
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

    pub(crate) fn adapt_plain_call_args_with_signature_for_ir(
        &self,
        func: &str,
        hir_args: &[HirExpr],
        lowered_args: Vec<crate::RustExpr>,
    ) -> Vec<crate::RustExpr> {
        let Some(param_info) = self.resolve_plain_call_param_info(func, hir_args.len()) else {
            return lowered_args;
        };
        if param_info.len() < hir_args.len() || lowered_args.len() != hir_args.len() {
            return lowered_args;
        }

        let mut adapted = Vec::with_capacity(lowered_args.len());
        let ctor_class_name = func.strip_suffix("::new");
        for (idx, (((param_ty, convention), hir_arg), mut lowered_arg)) in param_info
            .iter()
            .take(hir_args.len())
            .zip(hir_args.iter())
            .zip(lowered_args.into_iter())
            .enumerate()
        {
            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
            let effective_arg_ty = if let HirExpr::Name { name, ty } = hir_arg {
                if self.none_widened_local_bindings.contains(name) {
                    self.local_binding_types
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| ty.clone())
                } else if matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                ) {
                    self.local_binding_types
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| ty.clone())
                } else {
                    ty.clone()
                }
            } else {
                hir_arg.ty().clone()
            };
            let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
            let borrowed_name_arg = matches!(hir_arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));

            if crate::helpers::is_option_type(resolved_param) {
                let is_recursive_ctor_param = ctor_class_name
                    .and_then(|class_name| {
                        self.class_field_order
                            .get(class_name)
                            .and_then(|fields| fields.get(idx))
                            .map(|field_name| {
                                self.recursive_fields
                                    .contains(&(class_name.to_owned(), field_name.clone()))
                            })
                    })
                    .unwrap_or(false);
                let needs_box_inner =
                    param_ty.rust_type().starts_with("Option<Box<") || is_recursive_ctor_param;
                if !arg_is_option && !matches!(hir_arg, HirExpr::NoneLiteral) {
                    let wrapped_inner = Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg);
                    lowered_arg = if needs_box_inner {
                        Self::ensure_some_box_inner_for_ir(wrapped_inner)
                    } else {
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![wrapped_inner],
                        }
                    };
                } else if needs_box_inner {
                    lowered_arg = Self::ensure_option_box_inner_for_ir(lowered_arg);
                }
            } else if arg_is_option {
                if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                    lowered_arg = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "clone".to_string(),
                        args: vec![],
                    };
                }
                lowered_arg = Self::force_unwrap_option_expr_for_ir(
                    lowered_arg,
                    "compiler-verified option argument should be Some",
                );
            }

            if self.function_param_lowers_to_sifr_int(func, idx) {
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted.push(self.coerce_expr_to_sifr_int_value(lowered_arg));
                continue;
            }
            if self.function_param_lowers_to_sifr_int_result(func, idx) {
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted.push(self.coerce_result_int_expr_to_sifr_int_value(lowered_arg));
                continue;
            }

            let param_rust_type = param_ty.rust_type();
            if param_rust_type.starts_with("Box<")
                && !Self::is_box_new_call_expr_for_ir(&lowered_arg)
            {
                lowered_arg = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![lowered_arg],
                };
            }

            if convention.is_owned() && borrowed_name_arg {
                lowered_arg = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }

            let expects_shared_ref_type =
                param_ty.rust_type().starts_with('&') && !param_ty.rust_type().starts_with("&mut ");
            let expects_mut_ref_type = param_ty.rust_type().starts_with("&mut ");
            let needs_shared_borrow = expects_shared_ref_type
                || (convention.is_shared_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));
            let needs_mut_borrow = expects_mut_ref_type
                || (convention.is_mut_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));
            let already_borrowed = matches!(lowered_arg, crate::RustExpr::Ref { .. })
                || matches!(
                    (hir_arg, &lowered_arg),
                    (
                        HirExpr::Name { name, .. },
                        crate::RustExpr::Ident(lowered_name)
                    ) if lowered_name == name
                        && (self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name))
                );
            let already_mut_borrowed =
                matches!(lowered_arg, crate::RustExpr::Ref { mutable: true, .. })
                    || matches!(
                        (hir_arg, &lowered_arg),
                        (
                            HirExpr::Name { name, .. },
                            crate::RustExpr::Ident(lowered_name)
                        ) if lowered_name == name && self.mut_borrowed_params.contains(name)
                    );

            if needs_shared_borrow || needs_mut_borrow {
                lowered_arg = Self::clone_moved_names_in_borrowed_aggregate(hir_arg, lowered_arg);
            }

            if needs_shared_borrow && !already_borrowed {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(lowered_arg),
                };
            } else if needs_mut_borrow && !already_mut_borrowed {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(lowered_arg),
                };
            }

            adapted.push(lowered_arg);
        }
        adapted
    }

    pub(crate) fn lower_recursive_capture_arg_for_ir(
        &self,
        capture: &crate::NestedFnCapture,
    ) -> crate::RustExpr {
        let ident = crate::RustExpr::Ident(capture.name.clone());
        if self.recursive_capture_lowers_to_sifr_int(capture) {
            let rewritten = self.rewrite_stdlib_constant_idents_in_expr(ident);
            return self.coerce_expr_to_sifr_int_value(rewritten);
        }
        if capture.convention.is_mut_borrow() {
            if self.mut_borrowed_params.contains(&capture.name) {
                return ident;
            }
            return crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(ident),
            };
        }
        if capture.convention.is_shared_borrow() {
            if self.borrowed_params.contains(&capture.name)
                || self.mut_borrowed_params.contains(&capture.name)
            {
                return ident;
            }
            return crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(ident),
            };
        }
        ident
    }

    fn borrowed_return_name_clone_expr_for_ir(&self, value: &HirExpr) -> Option<crate::RustExpr> {
        let HirExpr::Name { name, .. } = value else {
            return None;
        };
        if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)) {
            return None;
        }
        Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
            name.clone(),
        ))))
    }

    fn lower_non_option_index_expr_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(
            object_ty,
            Type::Tuple(_) | Type::List(_) | Type::Bytes | Type::Str
        ) {
            return Ok(None);
        }

        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };

        let lowered = match object_ty {
            Type::Tuple(elements) => {
                let HirExpr::IntLiteral(raw_idx) = index else {
                    return Ok(None);
                };
                let Ok(idx) = usize::try_from(*raw_idx) else {
                    return Ok(None);
                };
                if idx >= elements.len() {
                    return Ok(None);
                }
                crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                    field: idx.to_string(),
                }
            }
            Type::List(element_ty) => {
                let indexed_expr = crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                };
                if crate::helpers::is_copy_type_for_codegen(element_ty.as_ref()) {
                    indexed_expr
                } else {
                    crate::RustExpr::Clone(Box::new(indexed_expr))
                }
            }
            Type::Bytes => crate::RustExpr::Cast {
                expr: Box::new(crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                }),
                ty: crate::RustType::Named("u8".to_string()),
            },
            Type::Str => {
                let nth_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "nth".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                };
                crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::LetElse {
                        pattern: "Some(__indexed_char)".to_string(),
                        value: nth_expr,
                        else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                            name: "unreachable".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                                "compiler-verified string index should be in range".to_string(),
                            ))],
                        })],
                    }],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__indexed_char".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    })),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(lowered))
    }

    fn lower_return_value_expr_for_ir(
        &mut self,
        value: &HirExpr,
        return_ty: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let coerce_return = |this: &mut Self,
                             lowered: crate::RustExpr|
         -> Result<crate::RustExpr, crate::CodegenError> {
            if let Some(target_ty) = return_ty {
                let coerced =
                    this.coerce_local_value_for_target_type_for_ir(target_ty, value, lowered)?;
                if this.current_sifr_int_result_return.get()
                    && is_result_int_division_error_type(target_ty)
                {
                    return Ok(this.coerce_result_int_expr_to_sifr_int_value(coerced));
                }
                return Ok(coerced);
            }
            Ok(lowered)
        };
        if self.current_class_name.is_some()
            && matches!(value, HirExpr::Name { name, .. } if name == "self")
        {
            return Ok(Some(coerce_return(
                self,
                crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident("self".to_string()))),
            )?));
        }

        if let Some(clone_expr) = self.borrowed_return_name_clone_expr_for_ir(value) {
            return Ok(Some(coerce_return(self, clone_expr)?));
        }

        if let Some(target_ty) = return_ty {
            if matches!(
                crate::resolve_alias_type_for_plain_call(target_ty),
                Type::Iterator(_) | Type::Iterable(_)
            ) {
                if let Some(lowered_iter_return) =
                    self.lower_escaping_iter_return_expr_for_ir(value)?
                {
                    return Ok(Some(coerce_return(self, lowered_iter_return)?));
                }
            }

            if matches!(
                crate::resolve_alias_type_for_plain_call(target_ty),
                Type::Iterator(_)
            ) && !matches!(
                crate::resolve_alias_type_for_plain_call(value.ty()),
                Type::Iterator(_)
            ) && crate::resolve_alias_type_for_plain_call(value.ty())
                .iterable_element_type()
                .is_some()
            {
                if let Some(lowered_iter_source) =
                    self.lower_iter_source_expr_for_ir_with_mode(value, true, None, None)?
                {
                    return Ok(Some(coerce_return(self, lowered_iter_source)?));
                }
            }
        }

        if return_ty.is_some_and(|ty| !crate::helpers::is_option_type(ty))
            && matches!(value, HirExpr::Index { .. })
        {
            let HirExpr::Index { object, index, .. } = value else {
                unreachable!();
            };
            if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                return Ok(Some(lowered));
            }
        }

        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(value)? {
            return Ok(Some(coerce_return(self, lowered_leaf)?));
        }
        if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(value)? {
            return Ok(Some(coerce_return(
                self,
                self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
            )?));
        }
        Ok(None)
    }

    pub(super) fn lower_rendered_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Await { .. } = expr {
            if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? {
                return Ok(Some(
                    self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
                ));
            }
        }
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
        {
            if !crate::helpers::is_option_type(ty) {
                if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                    return Ok(Some(lowered));
                }
            }
        }
        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(expr)? {
            return Ok(Some(lowered_leaf));
        }
        if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? {
            return Ok(Some(
                self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
            ));
        }
        Ok(None)
    }

}
