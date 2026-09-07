use super::{
    HirExpr, RecursiveOptionConstructorArgContext, RustEmitter, Type,
    is_result_int_division_error_type,
};
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
            .zip(lowered_args)
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
            let effective_arg_ty = if let HirExpr::Name { name, ty, .. } = hir_arg {
                if self.option_unwrapped_vars.contains(name)
                    && let Some(inner) = ty.optional_member_type()
                {
                    inner
                } else if self.none_widened_local_bindings.contains(name) {
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
            if let Type::AsyncCallable(params, _, _) = resolved_param {
                lowered_arg = Self::send_async_callable_adapter(lowered_arg, params.len());
            }
            let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
            let borrowed_name_arg = matches!(hir_arg, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name));

            if let Some(borrowed_view) = self.adapt_recursive_option_borrowed_argument(
                param_ty,
                *convention,
                hir_arg,
                &effective_arg_ty,
                lowered_arg.clone(),
            ) {
                adapted.push(borrowed_view);
                continue;
            }

            let unadapted_option_arg = lowered_arg.clone();
            if matches!(hir_arg, HirExpr::NoneLiteral)
                && matches!(resolved_param, Type::None | Type::TypeVar(_))
            {
                lowered_arg = crate::RustExpr::Literal(crate::RustLiteral::Unit);
            }

            let mut consuming_value_adapted = false;
            if convention.is_owned() {
                (lowered_arg, consuming_value_adapted) = self.adapt_consuming_call_argument_for_ir(
                    param_ty,
                    &effective_arg_ty,
                    lowered_arg,
                    borrowed_name_arg,
                );
            } else {
                lowered_arg = Self::flatten_option_argument_for_ir(
                    hir_arg,
                    param_ty,
                    &effective_arg_ty,
                    *convention,
                    lowered_arg,
                );
                if let Some(wrapped) = crate::helpers::wrap_union_member_expr(
                    param_ty,
                    if matches!(hir_arg, HirExpr::NoneLiteral) {
                        &Type::None
                    } else {
                        &effective_arg_ty
                    },
                    lowered_arg.clone(),
                ) {
                    lowered_arg = wrapped;
                }
            }
            let option_value_adapted = lowered_arg != unadapted_option_arg;

            let mut recursive_option_adapted = false;
            if crate::helpers::is_option_type(resolved_param) {
                if let Some(adapted) = self.try_adapt_recursive_option_constructor_arg_for_ir(
                    &RecursiveOptionConstructorArgContext {
                        ctor_class_name,
                        index: idx,
                        param_ty,
                        arg: hir_arg,
                        effective_arg_ty: &effective_arg_ty,
                        convention: *convention,
                        borrowed_name_arg,
                        borrowed_name_materialized: false,
                    },
                    lowered_arg.clone(),
                ) {
                    lowered_arg = adapted;
                    recursive_option_adapted = true;
                } else if !arg_is_option && !matches!(hir_arg, HirExpr::NoneLiteral) {
                    let param_is_owned_rust_value = convention.is_owned();
                    let wrapped_inner = if param_is_owned_rust_value && !borrowed_name_arg {
                        lowered_arg
                    } else if matches!(hir_arg, HirExpr::Name { .. })
                        && !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty)
                    {
                        crate::ownership_plan::materialize_owned_value(
                            &effective_arg_ty,
                            lowered_arg,
                        )
                    } else {
                        Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg)
                    };
                    lowered_arg = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![wrapped_inner],
                    };
                }
            } else if arg_is_option && !option_value_adapted {
                if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                    lowered_arg = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "clone".to_string(),
                        args: vec![],
                    };
                }
            }

            if self.function_param_lowers_to_sifr_int(func, idx) {
                let lowered_arg = Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg);
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted
                    .push(self.coerce_typed_expr_to_sifr_int_value(lowered_arg, &effective_arg_ty));
                continue;
            }
            if self.function_param_lowers_to_sifr_int_result(func, idx) {
                let lowered_arg = Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg);
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted.push(self.coerce_result_int_expr_to_sifr_int_value(lowered_arg));
                continue;
            }

            if matches!(
                crate::sifr_type_to_rust_type(param_ty),
                crate::RustType::Boxed(_)
            ) && !Self::is_box_new_call_expr_for_ir(&lowered_arg)
            {
                lowered_arg = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![lowered_arg],
                };
            }

            if convention.is_owned()
                && !recursive_option_adapted
                && !consuming_value_adapted
                && (borrowed_name_arg
                    || (crate::helpers::is_logically_copy_rust_move_type(&effective_arg_ty)
                        && matches!(
                            crate::helpers::classify_value_category(hir_arg),
                            crate::helpers::ValueCategory::Place
                        )
                        && Self::rust_expr_is_reusable_place_for_ir(&lowered_arg)))
            {
                lowered_arg =
                    crate::ownership_plan::materialize_owned_value(&effective_arg_ty, lowered_arg);
            }

            let needs_shared_borrow = convention.is_shared_borrow()
                && (!crate::helpers::is_copy_type_for_codegen(param_ty)
                    || matches!(
                        resolved_param,
                        Type::TypeVar(_) | Type::Any | Type::Callable(..) | Type::AsyncCallable(..)
                    ));
            let needs_mut_borrow = convention.is_mut_borrow()
                && (!crate::helpers::is_copy_type_for_codegen(param_ty)
                    || matches!(resolved_param, Type::TypeVar(_) | Type::Any));
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
                lowered_arg = self.clone_moved_names_in_borrowed_aggregate(hir_arg, lowered_arg);
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

    pub(crate) fn send_async_callable_adapter(
        callable: crate::RustExpr,
        arity: usize,
    ) -> crate::RustExpr {
        let params = (0..arity)
            .map(|index| crate::RustParam::Named {
                name: format!("__sifr_async_arg_{index}"),
                ty: crate::RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        let args = (0..arity)
            .map(|index| crate::RustExpr::Ident(format!("__sifr_async_arg_{index}")))
            .collect::<Vec<_>>();
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::Let {
                mutable: false,
                name: "__sifr_send_async_callable".to_string(),
                ty: None,
                value: crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "std".to_string(),
                        "sync".to_string(),
                        "Arc".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![callable],
                },
            }],
            expr: Some(Box::new(crate::RustExpr::ClosureBlock {
                params,
                body: vec![
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "__sifr_send_async_callable".to_string(),
                        ty: None,
                        value: crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "std".to_string(),
                                "sync".to_string(),
                                "Arc".to_string(),
                                "clone".to_string(),
                            ])),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident(
                                    "__sifr_send_async_callable".to_string(),
                                )),
                            }],
                        },
                    },
                    crate::RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Box".to_string(),
                            "pin".to_string(),
                        ])),
                        args: vec![crate::RustExpr::AsyncBlock {
                            body: vec![crate::RustStmt::Return(Some(crate::RustExpr::Await(
                                Box::new(crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Ident(
                                        "__sifr_send_async_callable".to_string(),
                                    )),
                                    args,
                                }),
                            )))],
                            is_move: true,
                        }],
                    })),
                ],
                is_move: true,
                is_async: false,
            })),
        }
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
        Some(crate::ownership_plan::materialize_owned_value(
            value.ty(),
            crate::RustExpr::Ident(name.clone()),
        ))
    }

    pub(crate) fn lower_non_option_index_expr_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some(witness) = self.checked_place_read_witness(
            object,
            index,
            &object
                .ty()
                .index_result_type(index.ty())
                .and_then(|ty| ty.optional_member_type())
                .unwrap_or(Type::Unknown),
        ) {
            return Ok(Some(witness));
        }
        if let Some(lowered) = self.lower_proven_read_with_error_carrier(object, index)? {
            return Ok(Some(lowered));
        }
        let witnessed_object_ty = match object {
            HirExpr::Index {
                object: parent,
                index: parent_index,
                ..
            } if self
                .checked_place_read_borrow_witness(parent, parent_index)
                .is_some() =>
            {
                object.ty().optional_member_type()
            }
            _ => None,
        };
        let object_ty = crate::resolve_alias_type_for_plain_call(
            witnessed_object_ty.as_ref().unwrap_or_else(|| object.ty()),
        );
        if !matches!(object_ty, Type::Tuple(_)) {
            return Ok(None);
        }

        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
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
                if crate::helpers::is_copy_type_for_codegen(element_ty)
                    || !element_ty.supports_derived_clone()
                {
                    field_expr
                } else {
                    crate::RustExpr::Clone(Box::new(field_expr))
                }
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
        if let Some(return_ty) = return_ty
            && let Some(lowered) =
                self.lower_checked_place_option_value_for_target(return_ty, value)?
        {
            return Ok(Some(lowered));
        }
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

        if return_ty.is_some_and(|ty| self.recursive_option_borrowed_type(ty).is_some())
            && self.expr_is_recursive_option_borrowed_view(value)
            && let Some(lowered) = self.lower_stmt_expr_for_ir(value)?
        {
            return Ok(Some(coerce_return(
                self,
                crate::ownership_plan::materialize_borrowed_option_value(lowered),
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
            let HirExpr::Index {
                object, index, ty, ..
            } = value
            else {
                unreachable!();
            };
            if let Some(witness) = self.checked_place_read_witness(object, index, ty) {
                return Ok(Some(coerce_return(self, witness)?));
            }
            if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                return Ok(Some(lowered));
            }
        }

        if !matches!(
            value,
            HirExpr::OkWrap { .. }
                | HirExpr::ErrWrap { .. }
                | HirExpr::Compare { .. }
                | HirExpr::BinOp { .. }
        ) {
            if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(value)? {
                return Ok(Some(coerce_return(self, lowered_leaf)?));
            }
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
                if let Some(witness) = self.checked_place_read_witness(object, index, ty) {
                    return Ok(Some(witness));
                }
                if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                    return Ok(Some(lowered));
                }
            }
        }
        if !matches!(expr, HirExpr::Compare { .. } | HirExpr::BinOp { .. })
            && let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(expr)?
        {
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
