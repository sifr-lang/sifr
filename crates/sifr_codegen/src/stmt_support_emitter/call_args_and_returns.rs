use super::{is_result_int_division_error_type, HirExpr, RustEmitter, Type};
impl RustEmitter {
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
            if matches!(
                hir_arg,
                HirExpr::Call { func, .. }
                    if matches!(
                        func.as_str(),
                        "__sifr_python_present_argument" | "__sifr_python_omitted_argument"
                    )
            ) {
                adapted.push(lowered_arg);
                continue;
            }
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

            if matches!(hir_arg, HirExpr::NoneLiteral)
                && matches!(resolved_param, Type::None | Type::TypeVar(_))
            {
                lowered_arg = crate::RustExpr::Literal(crate::RustLiteral::Unit);
            }

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
                    let param_rust_type = param_ty.rust_type();
                    let param_is_owned_rust_value =
                        convention.is_owned() && !param_rust_type.starts_with('&');
                    let wrapped_inner = if param_is_owned_rust_value && !borrowed_name_arg {
                        lowered_arg
                    } else if matches!(hir_arg, HirExpr::Name { .. })
                        && !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty)
                    {
                        crate::RustExpr::Clone(Box::new(lowered_arg))
                    } else {
                        Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg)
                    };
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
                        || matches!(
                            resolved_param,
                            Type::TypeVar(_)
                                | Type::Any
                                | Type::Callable(..)
                                | Type::AsyncCallable(..)
                        )));
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
            if (needs_shared_borrow || needs_mut_borrow)
                && matches!(hir_arg, HirExpr::FieldAccess { object, .. }
                    if matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            {
                lowered_arg = Self::strip_redundant_borrowed_self_field_clone(lowered_arg);
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

    pub(crate) fn strip_redundant_borrowed_self_field_clone(
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } if method == "clone" && args.is_empty() => *receiver,
            other => other,
        }
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

    pub(crate) fn borrowed_return_name_clone_expr_for_ir(
        &self,
        value: &HirExpr,
    ) -> Option<crate::RustExpr> {
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

    pub(crate) fn lower_non_option_index_expr_for_ir(
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
                let Some(element_ty) = elements.get(idx) else {
                    return Ok(None);
                };
                let field_expr = crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                    field: idx.to_string(),
                };
                if crate::helpers::is_copy_type_for_codegen(element_ty) {
                    field_expr
                } else {
                    crate::RustExpr::Clone(Box::new(field_expr))
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
                self.lower_string_index_unwrapped_with_cache(object, lowered_object, lowered_index)
            }
            _ => return Ok(None),
        };
        Ok(Some(lowered))
    }

    pub(crate) fn lower_return_value_expr_for_ir(
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

    pub(crate) fn lower_rendered_expr_for_ir(
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
