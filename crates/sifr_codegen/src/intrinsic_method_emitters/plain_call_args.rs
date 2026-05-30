use super::{
    registry_can_construct_error_from_message, registry_ensure_some_box_inner,
    registry_is_box_new_ctor, registry_is_some_expr, registry_is_string_like_type,
    registry_iterable_to_vec_expr, registry_option_inner_type, HirExpr, ParamConvention,
    RustEmitter, RustExpr, Type,
};
impl RustEmitter {
    pub(crate) fn try_lower_registry_plain_call_with_signature(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let param_info = self.resolve_plain_call_param_info(func, args.len())?;
        if param_info.len() != args.len() {
            return None;
        }

        let mut lowered_args = Vec::with_capacity(args.len());
        let ctor_class_name = func.strip_suffix("::new");
        for (idx, ((param_ty, convention), arg)) in param_info.iter().zip(args.iter()).enumerate() {
            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
            let effective_arg_ty = self.effective_registry_expr_ty(arg);
            let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
            let mut lowered_arg = self.try_lower_registry_expr_strict(arg)?;
            let borrowed_name_arg = matches!(arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));
            if let Some(aligned_callable) = self
                .try_build_registry_callable_convention_alignment_expr(
                    arg,
                    resolved_param,
                    lowered_arg.clone(),
                )
            {
                lowered_arg = aligned_callable;
            }

            if matches!(resolved_param, Type::Iterable(_)) {
                lowered_arg = registry_iterable_to_vec_expr(self, arg)?;
            }

            if let Type::Union(members) = resolved_param {
                if !crate::helpers::is_option_type(resolved_param)
                    && !matches!(
                        crate::resolve_alias_type_for_plain_call(&effective_arg_ty),
                        Type::Union(_)
                    )
                {
                    if let Some(variant) =
                        crate::helpers::find_union_variant(members, &effective_arg_ty)
                    {
                        lowered_arg = crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                resolved_param.union_enum_name(),
                                variant,
                            ])),
                            args: vec![lowered_arg],
                        };
                    }
                }
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
                if !arg_is_option && !matches!(arg, HirExpr::NoneLiteral) {
                    let param_rust_type = param_ty.rust_type();
                    let param_is_owned_rust_value =
                        convention.is_owned() && !param_rust_type.starts_with('&');
                    if (!param_is_owned_rust_value || borrowed_name_arg)
                        && !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty)
                    {
                        lowered_arg = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                            method: "clone".to_string(),
                            args: vec![],
                        };
                    }
                    lowered_arg = if needs_box_inner {
                        registry_ensure_some_box_inner(lowered_arg)
                    } else {
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![lowered_arg],
                        }
                    };
                } else if needs_box_inner && registry_is_some_expr(&lowered_arg) {
                    lowered_arg = registry_ensure_some_box_inner(lowered_arg);
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

            let param_rust_type = param_ty.rust_type();
            if param_rust_type.starts_with("Box<")
                && !matches!(&lowered_arg, RustExpr::FnCall { func, .. } if registry_is_box_new_ctor(func.as_ref()))
            {
                lowered_arg = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![lowered_arg],
                };
            }

            if let Type::Result(_, param_err_ty) = resolved_param {
                if let Type::Result(_, arg_err_ty) =
                    crate::resolve_alias_type_for_plain_call(arg.ty())
                {
                    let param_err_name =
                        crate::render_type(&crate::sifr_type_to_rust_type(param_err_ty));
                    let arg_err_name =
                        crate::render_type(&crate::sifr_type_to_rust_type(arg_err_ty));
                    if param_err_name != arg_err_name
                        && registry_can_construct_error_from_message(&param_err_name)
                    {
                        let ctor_func = if param_err_name.contains("::") {
                            let mut path: Vec<String> =
                                param_err_name.split("::").map(str::to_string).collect();
                            path.push("new".to_string());
                            crate::RustExpr::Path(path)
                        } else {
                            crate::RustExpr::Path(vec![param_err_name.clone(), "new".to_string()])
                        };
                        lowered_arg = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FnCall {
                                    func: Box::new(ctor_func),
                                    args: vec![crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "__e".to_string(),
                                        )),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }],
                                }),
                                is_move: false,
                            }],
                        };
                    }
                }
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
            let requires_shared_borrow = expects_shared_ref_type
                || (convention.is_shared_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));
            let requires_mut_borrow = expects_mut_ref_type
                || (convention.is_mut_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));

            if requires_shared_borrow || requires_mut_borrow {
                lowered_arg = Self::clone_moved_names_in_borrowed_aggregate(arg, lowered_arg);
            }
            if (requires_shared_borrow || requires_mut_borrow)
                && matches!(arg, HirExpr::FieldAccess { object, .. }
                    if matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            {
                lowered_arg = Self::strip_redundant_borrowed_self_field_clone(lowered_arg);
            }

            if requires_shared_borrow
                && !self.arg_is_already_borrowed_for_registry_call(arg, &lowered_arg)
            {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(lowered_arg),
                };
            } else if requires_mut_borrow
                && !self.arg_is_already_mut_borrowed_for_registry_call(arg, &lowered_arg)
            {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(lowered_arg),
                };
            }

            lowered_args.push(lowered_arg);
        }

        if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
            for capture in captures {
                lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
            }
        }

        Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Ident(func.to_string())),
            args: lowered_args,
        })
    }

    pub(crate) fn resolve_plain_call_param_info(
        &self,
        func: &str,
        arg_len: usize,
    ) -> Option<Vec<(Type, ParamConvention)>> {
        if let Some((params, _)) = self.func_signatures.get(func) {
            return Some(params.clone());
        }
        if let Some(params) = self.callable_var_conventions.get(func) {
            return Some(params.clone());
        }

        let mut candidate: Option<Vec<(Type, ParamConvention)>> = None;
        for (name, (params, _)) in &self.func_signatures {
            if name.rsplit("::").next() != Some(func) || params.len() < arg_len {
                continue;
            }
            if candidate.is_some() {
                return None;
            }
            candidate = Some(params.clone());
        }
        candidate
    }

    pub(crate) fn try_build_registry_callable_convention_alignment_expr(
        &self,
        arg: &HirExpr,
        param_ty: &Type,
        lowered_arg: crate::RustExpr,
    ) -> Option<crate::RustExpr> {
        let Type::Callable(_, expected_conventions, _) =
            crate::resolve_alias_type_for_plain_call(param_ty)
        else {
            return None;
        };
        let HirExpr::Name { name: callee, .. } = arg else {
            return None;
        };
        let provided_params = self
            .func_signatures
            .get(callee)
            .map(|(params, _)| params.clone())
            .or_else(|| self.callable_var_conventions.get(callee).cloned())?;
        if provided_params.len() != expected_conventions.len() {
            return None;
        }
        if !provided_params
            .iter()
            .zip(expected_conventions.iter())
            .any(|((_, provided), expected)| *provided != *expected)
        {
            return None;
        }

        let mut closure_params = Vec::with_capacity(provided_params.len());
        let mut call_args = Vec::with_capacity(provided_params.len());
        for (idx, ((_, provided), expected)) in provided_params
            .iter()
            .zip(expected_conventions.iter())
            .enumerate()
        {
            let arg_name = format!("__arg{idx}");
            closure_params.push(crate::RustParam::Named {
                name: arg_name.clone(),
                ty: crate::RustType::Named("_".to_string()),
            });

            let base_arg = crate::RustExpr::Ident(arg_name.clone());
            let adapted = if provided.is_owned() && expected.is_borrowed() {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(base_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                }
            } else if expected.is_owned() && provided.is_shared_borrow() {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(base_arg),
                }
            } else if expected.is_owned() && provided.is_mut_borrow() {
                crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(base_arg),
                }
            } else if expected.is_shared_borrow() && provided.is_mut_borrow() {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(crate::RustExpr::Deref(Box::new(base_arg))),
                }
            } else {
                base_arg
            };
            call_args.push(adapted);
        }

        Some(crate::RustExpr::Closure {
            params: closure_params,
            body: Box::new(crate::RustExpr::FnCall {
                func: Box::new(lowered_arg),
                args: call_args,
            }),
            is_move: false,
        })
    }

    pub(crate) fn resolve_registry_method_params(
        &self,
        object_ty: &Type,
        method: &str,
    ) -> Option<Vec<(Type, ParamConvention)>> {
        let Type::Class { name, methods, .. } = crate::resolve_alias_type_for_plain_call(object_ty)
        else {
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

    pub(crate) fn apply_registry_method_arg_convention(
        &self,
        arg: &HirExpr,
        param_ty: &Type,
        convention: ParamConvention,
        mut lowered_arg: crate::RustExpr,
    ) -> crate::RustExpr {
        let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
        let effective_arg_ty = self.effective_registry_expr_ty(arg);
        let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
        if crate::helpers::is_option_type(param_ty)
            && !arg_is_option
            && !matches!(arg, HirExpr::NoneLiteral)
        {
            if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                lowered_arg = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            lowered_arg = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_arg],
            };
        } else if arg_is_option && !crate::helpers::is_option_type(param_ty) {
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

        let requires_shared_borrow = convention.is_shared_borrow()
            && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                || matches!(resolved_param, Type::TypeVar(_)));
        let requires_mut_borrow = convention.is_mut_borrow()
            && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                || matches!(resolved_param, Type::TypeVar(_)));

        if requires_shared_borrow
            && !self.arg_is_already_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_arg),
            };
        } else if requires_mut_borrow
            && !self.arg_is_already_mut_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(lowered_arg),
            };
        }
        lowered_arg
    }

    pub(crate) fn clone_moved_names_in_borrowed_aggregate(
        arg: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        Self::clone_moved_names_in_borrowed_aggregate_inner(arg, lowered, false)
    }

    pub(crate) fn clone_moved_names_in_borrowed_aggregate_inner(
        arg: &HirExpr,
        lowered: crate::RustExpr,
        in_aggregate: bool,
    ) -> crate::RustExpr {
        match (arg, lowered) {
            (HirExpr::ListLiteral { elements, .. }, crate::RustExpr::Vec(items)) => {
                crate::RustExpr::Vec(
                    elements
                        .iter()
                        .zip(items)
                        .map(|(element, item)| {
                            Self::clone_moved_names_in_borrowed_aggregate_inner(element, item, true)
                        })
                        .collect(),
                )
            }
            (HirExpr::TupleLiteral { elements, .. }, crate::RustExpr::Tuple(items)) => {
                crate::RustExpr::Tuple(
                    elements
                        .iter()
                        .zip(items)
                        .map(|(element, item)| {
                            Self::clone_moved_names_in_borrowed_aggregate_inner(element, item, true)
                        })
                        .collect(),
                )
            }
            (HirExpr::Name { ty, .. }, lowered_expr)
                if in_aggregate && ty.ownership() != sifr_type_system::OwnershipKind::Copy =>
            {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_expr))),
                    method: "clone".to_string(),
                    args: vec![],
                }
            }
            (_, lowered_expr) => lowered_expr,
        }
    }

    pub(crate) fn arg_is_already_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if matches!(lowered, crate::RustExpr::Ref { .. }) {
            return true;
        }
        if let (HirExpr::Name { name, .. }, crate::RustExpr::Ident(lowered_name)) = (arg, lowered) {
            if lowered_name != name {
                return false;
            }
            return self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name);
        }
        false
    }

    pub(crate) fn arg_is_already_mut_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if let crate::RustExpr::Ref { mutable, .. } = lowered {
            return *mutable;
        }
        if let (HirExpr::Name { name, .. }, crate::RustExpr::Ident(lowered_name)) = (arg, lowered) {
            if lowered_name != name {
                return false;
            }
            return self.mut_borrowed_params.contains(name);
        }
        false
    }

    pub(crate) fn try_lower_registry_compare_expr(
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
            let lhs_ty = self.effective_registry_expr_ty(lhs_expr);
            let rhs_ty = self.effective_registry_expr_ty(rhs_expr);
            let lowered_op = match op.as_str() {
                "==" | "!=" | "<" | "<=" | ">" | ">=" => op.clone(),
                "is" => "==".to_string(),
                "is not" => "!=".to_string(),
                _ => return None,
            };
            let lhs_none_like = matches!(lhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(&lhs_ty),
                    Type::None
                );
            let rhs_none_like = matches!(rhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(&rhs_ty),
                    Type::None
                );
            if (op == "is" || op == "is not") && lhs_none_like && rhs_none_like {
                let comparison = crate::RustExpr::Literal(crate::RustLiteral::Bool(op == "is"));
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
                continue;
            }
            let mut lowered_left = self.try_lower_registry_expr_strict(lhs_expr)?;
            let mut lowered_right = self.try_lower_registry_expr_strict(rhs_expr)?;

            let is_comparison_op =
                matches!(lowered_op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=");
            if is_comparison_op
                && registry_option_inner_type(&lhs_ty).is_some()
                && registry_option_inner_type(&rhs_ty).is_none()
                && !rhs_none_like
            {
                lowered_right = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_right],
                };
            } else if is_comparison_op
                && registry_option_inner_type(&lhs_ty).is_none()
                && registry_option_inner_type(&rhs_ty).is_some()
                && !lhs_none_like
            {
                lowered_left = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_left],
                };
            } else if registry_is_string_like_type(&lhs_ty) && registry_is_string_like_type(&rhs_ty)
            {
                lowered_left = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
                lowered_right = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
            }
            let comparison = crate::RustExpr::BinOp {
                left: Box::new(lowered_left),
                op: lowered_op,
                right: Box::new(lowered_right),
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

    pub(crate) fn registry_detect_is_some_guard_name(expr: &HirExpr) -> Option<String> {
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        {
            if method != "is_some" || !args.is_empty() {
                return None;
            }
            let HirExpr::Name { name, .. } = object.as_ref() else {
                return None;
            };
            return Some(name.clone());
        }
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        else {
            return None;
        };
        if ops.len() != 1 || comparators.len() != 1 || !matches!(ops[0].as_str(), "is not" | "!=") {
            return None;
        }
        let rhs = comparators.first()?;
        match (left.as_ref(), rhs) {
            (HirExpr::Name { name, .. }, HirExpr::NoneLiteral)
            | (HirExpr::NoneLiteral, HirExpr::Name { name, .. }) => Some(name.clone()),
            _ => None,
        }
    }

    pub(crate) fn try_lower_registry_guarded_option_compare_expr(
        &mut self,
        expr: &HirExpr,
        guarded_name: &str,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        else {
            return None;
        };
        if ops.len() != 1 || comparators.len() != 1 {
            return None;
        }
        let lowered_op = match ops[0].as_str() {
            "==" | "!=" => ops[0].clone(),
            "is" => "==".to_string(),
            "is not" => "!=".to_string(),
            _ => return None,
        };
        let rhs_expr = comparators.first()?;
        let (option_side, other_side, option_is_left) = match (left.as_ref(), rhs_expr) {
            (HirExpr::Name { name, .. }, other) if name == guarded_name => {
                (left.as_ref(), other, true)
            }
            (other, HirExpr::Name { name, .. }) if name == guarded_name => (rhs_expr, other, false),
            _ => return None,
        };
        if !crate::helpers::is_option_type(option_side.ty()) {
            return None;
        }
        if matches!(other_side, HirExpr::NoneLiteral) {
            return None;
        }

        let lowered_option = if let HirExpr::Name { name, .. } = option_side {
            crate::RustExpr::Ident(name.clone())
        } else {
            self.try_lower_registry_expr_strict(option_side)?
        };
        let mut lowered_other = self.try_lower_registry_expr_strict(other_side)?;
        if !crate::helpers::is_copy_type_for_codegen(other_side.ty()) {
            lowered_other = crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_other))),
                method: "clone".to_string(),
                args: vec![],
            };
        }
        let lowered_some = crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_other],
        };
        let (left_expr, right_expr) = if option_is_left {
            (lowered_option, lowered_some)
        } else {
            (lowered_some, lowered_option)
        };
        Some(crate::RustExpr::BinOp {
            left: Box::new(left_expr),
            op: lowered_op,
            right: Box::new(right_expr),
        })
    }

    pub(crate) fn try_eval_const_int_expr(expr: &HirExpr) -> Option<i64> {
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

    pub(crate) fn usize_cast_literal(value: i64) -> crate::RustExpr {
        crate::RustExpr::Cast {
            expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(value))),
            ty: crate::RustType::Named("usize".to_string()),
        }
    }

    pub(crate) fn try_lower_registry_string_slice_expr(
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
