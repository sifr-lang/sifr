use super::narrowing_helpers::{
    is_narrowable_pop_call_for_codegen, supports_nonempty_pop_narrowing_type_for_codegen,
};
use super::{
    methods, registry_box_iterator_expr, registry_defaultdict_alias_parts,
    registry_defaultdict_default_expr, registry_defaultdict_key_arg, registry_expr_is_vec_like,
    registry_iterable_to_owned_iter_expr, registry_iterable_to_set_expr, HirExpr, RustEmitter,
    RustExpr, Type,
};

fn is_defaultdict_list_bucket_mutator(method: &str) -> bool {
    matches!(
        method,
        "append" | "extend" | "insert" | "clear" | "reverse" | "sort" | "pop" | "remove"
    )
}

fn is_defaultdict_set_bucket_mutator(method: &str) -> bool {
    matches!(
        method,
        "add"
            | "update"
            | "intersection_update"
            | "difference_update"
            | "symmetric_difference_update"
            | "remove"
            | "discard"
            | "clear"
            | "pop"
    )
}

impl RustEmitter {
    pub(crate) fn effective_method_object_ty(&self, object: &HirExpr) -> Type {
        if let HirExpr::Name { name, ty, .. } = object {
            if self.none_widened_local_bindings.contains(name) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
        }
        object.ty().clone()
    }

    pub(crate) fn effective_registry_expr_ty(&self, expr: &HirExpr) -> Type {
        if let HirExpr::Name { name, ty, .. } = expr {
            if self.none_widened_local_bindings.contains(name) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = self.local_binding_types.get(name) {
                    return bound_ty.clone();
                }
            }
        }
        expr.ty().clone()
    }

    pub(crate) fn try_lower_registry_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let effective_object_ty = self.effective_method_object_ty(object);
        let object_ty = crate::resolve_alias_type_for_plain_call(&effective_object_ty);
        if let Some(lowered) = crate::python_buffer_codegen::lower_python_buffer_method(
            self,
            object,
            method,
            args,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered) = crate::python_arrow_codegen::lower_python_arrow_method(
            self,
            object,
            method,
            args,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered) = crate::python_dlpack_codegen::lower_python_dlpack_method(
            self,
            object,
            method,
            args,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if let Some(lowered) = self.try_lower_defaultdict_index_method_call_expr(
            object,
            method,
            args,
            method_return_ty,
        ) {
            return Some(lowered);
        }
        if method == "append" && args.len() == 1 {
            if let HirExpr::Index {
                object: index_object,
                index,
                ..
            } = object
            {
                if matches!(
                    crate::resolve_alias_type_for_plain_call(index_object.ty()),
                    Type::Dict(_, _)
                ) && matches!(object_ty, Type::List(_))
                {
                    let lowered_object =
                        self.try_lower_dict_indexed_list_mutation_object(index_object)?;
                    let lowered_index = self.try_lower_registry_expr_strict(index)?;
                    let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
                    let key_arg = Self::build_dict_lookup_key_arg_for_ir(
                        Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                    );
                    let pushed_arg =
                        Self::clone_owned_append_arg_expr_for_ir(&args[0], lowered_arg);
                    return Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::IfLet {
                            pattern: "Some(__elem)".to_string(),
                            expr: crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get_mut".to_string(),
                                args: vec![key_arg],
                            },
                            then_body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                                method: "push".to_string(),
                                args: vec![pushed_arg],
                            })],
                            else_body: None,
                        }],
                        expr: None,
                    });
                }
            }
        }
        if method == "len" && args.is_empty() {
            if let HirExpr::Index {
                object: index_object,
                index,
                ..
            } = object
            {
                let effective_index_object_ty = self.effective_registry_expr_ty(index_object);
                if let Type::Dict(_, value_ty) =
                    crate::resolve_alias_type_for_plain_call(&effective_index_object_ty)
                {
                    if matches!(
                        crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                        Type::List(_)
                    ) {
                        let lowered_object =
                            self.try_lower_dict_indexed_list_mutation_object(index_object)?;
                        let lowered_index = self.try_lower_registry_expr_strict(index)?;
                        let key_arg = Self::list_indexed_dict_lookup_key_arg(index, lowered_index);
                        return Some(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "map_or".to_string(),
                            args: vec![
                                RustExpr::Literal(crate::RustLiteral::Int(0)),
                                RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__sifr_bucket".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "__sifr_bucket".to_string(),
                                            )),
                                            method: "len".to_string(),
                                            args: vec![],
                                        }),
                                        ty: crate::RustType::I64,
                                    }),
                                    is_move: false,
                                },
                            ],
                        });
                    }
                }
            }
        }
        let is_deque_data_field = self.is_deque_data_field(object);
        let object_expr = self.try_lower_registry_expr_strict(object)?;
        if method == "len"
            && args.is_empty()
            && matches!(object_ty, Type::Str | Type::LiteralStr(_))
        {
            return Some(self.lower_string_len_with_cache(object, object_expr));
        }
        let mut arg_exprs = self.try_lower_registry_exprs_strict(args)?;

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
            && matches!(args[0].ty(), Type::TypeVar(_))
        {
            // Clone TypeVar list args to avoid move issues.
            arg_exprs[0] = crate::RustExpr::MethodCall {
                receiver: Box::new(arg_exprs[0].clone()),
                method: "clone".to_string(),
                args: vec![],
            };
        }
        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
        {
            arg_exprs[0] = Self::clone_owned_append_arg_expr_for_ir(&args[0], arg_exprs[0].clone());
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            // Clone borrowed/mut-borrowed move-owned values.
            let needs_clone = if let HirExpr::Name { name, ty, .. } = &args[1] {
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

        if matches!(object_ty, Type::Dict(key_ty, _) if matches!(crate::resolve_alias_type_for_plain_call(key_ty.as_ref()), Type::Str | Type::LiteralStr(_)))
            && matches!(method, "get" | "contains" | "remove" | "pop")
            && !args.is_empty()
        {
            let key_arg_ty = crate::resolve_alias_type_for_plain_call(args[0].ty());
            let key_is_string_like = matches!(key_arg_ty, Type::Str | Type::LiteralStr(_));
            let already_as_str =
                matches!(&arg_exprs[0], RustExpr::MethodCall { method, .. } if method == "as_str");
            if key_is_string_like && !already_as_str {
                arg_exprs[0] = RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(arg_exprs[0].clone()))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
            }
        }

        if let Type::Class {
            fields, methods, ..
        } = crate::resolve_alias_type_for_plain_call(object_ty)
        {
            let is_callable_field = !methods.iter().any(|(name, _)| name == method)
                && fields
                    .iter()
                    .any(|(name, ty)| name == method && matches!(ty, Type::Callable(..)));
            if is_callable_field {
                return Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Field {
                        expr: Box::new(object_expr),
                        field: method.to_string(),
                    }))),
                    args: arg_exprs,
                });
            }
        }

        if matches!(object_ty, Type::List(_)) && method == "extend" && args.len() == 1 {
            return Some(crate::RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: "extend".to_string(),
                args: vec![registry_iterable_to_owned_iter_expr(self, &args[0])?],
            });
        }

        if matches!(object_ty, Type::Set(_)) {
            if let Some(lowered) =
                self.try_lower_registry_set_method_call_expr(&object_expr, method, args)
            {
                return Some(lowered);
            }
        }

        let lowered = methods::lower_method_with_context(
            object_ty,
            method,
            &object_expr,
            &arg_exprs,
            is_deque_data_field,
        )?;
        let lowered_expr = Self::unwrap_compiler_verified_nonempty_pop_result(
            object_ty,
            method,
            args,
            method_return_ty,
            lowered.expr,
        );
        if matches!(
            crate::resolve_alias_type_for_plain_call(method_return_ty),
            Type::Iterator(_)
        ) && registry_expr_is_vec_like(&lowered_expr)
        {
            return Some(registry_box_iterator_expr(RustExpr::MethodCall {
                receiver: Box::new(lowered_expr),
                method: "into_iter".to_string(),
                args: vec![],
            }));
        }
        Some(lowered_expr)
    }

    pub(crate) fn unwrap_compiler_verified_nonempty_pop_result(
        object_ty: &Type,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
        lowered_expr: crate::RustExpr,
    ) -> crate::RustExpr {
        if !supports_nonempty_pop_narrowing_type_for_codegen(object_ty) {
            return lowered_expr;
        }
        if !is_narrowable_pop_call_for_codegen(method, args) {
            return lowered_expr;
        }
        if crate::helpers::is_option_type(method_return_ty) {
            return lowered_expr;
        }
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::LetElse {
                pattern: "Some(__sifr_nonempty_pop_value)".to_string(),
                value: lowered_expr,
                else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                        "compiler-verified non-empty pop should return Some".to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(crate::RustExpr::Ident(
                "__sifr_nonempty_pop_value".to_string(),
            ))),
        }
    }

    pub(crate) fn try_lower_registry_set_method_call_expr(
        &mut self,
        object_expr: &crate::RustExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        match method {
            "update" => {
                let mut stmts = Vec::with_capacity(args.len());
                for arg in args {
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr.clone()),
                        method: "extend".to_string(),
                        args: vec![registry_iterable_to_owned_iter_expr(self, arg)
                            .map(|expr| crate::RustExpr::Paren(Box::new(expr)))?],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                });
            }
            "union" => {
                let mut stmts = vec![crate::RustStmt::Let {
                    mutable: true,
                    name: "__result".to_string(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr.clone()),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                }];
                for arg in args {
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__result".to_string())),
                        method: "extend".to_string(),
                        args: vec![registry_iterable_to_owned_iter_expr(self, arg)
                            .map(|expr| crate::RustExpr::Paren(Box::new(expr)))?],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(crate::RustExpr::Ident("__result".to_string()))),
                });
            }
            "intersection" | "difference" | "intersection_update" | "difference_update" => {
                let result_name = if method.ends_with("_update") {
                    None
                } else {
                    Some("__result".to_string())
                };
                let mut stmts = Vec::new();
                if let Some(result_name) = result_name.as_ref() {
                    stmts.push(crate::RustStmt::Let {
                        mutable: true,
                        name: result_name.clone(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(object_expr.clone()),
                            method: "clone".to_string(),
                            args: vec![],
                        },
                    });
                }
                let target = result_name.as_ref().map_or_else(
                    || object_expr.clone(),
                    |name| crate::RustExpr::Ident(name.clone()),
                );
                let keep_on_match = method.starts_with("intersection");
                for (index, arg) in args.iter().enumerate() {
                    let temp_name = format!("__set_arg_{index}");
                    stmts.push(crate::RustStmt::Let {
                        mutable: false,
                        name: temp_name.clone(),
                        ty: None,
                        value: registry_iterable_to_set_expr(self, arg)?,
                    });
                    stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(target.clone()),
                        method: "retain".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__item".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(if keep_on_match {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(temp_name)),
                                    method: "contains".to_string(),
                                    args: vec![crate::RustExpr::Ident("__item".to_string())],
                                }
                            } else {
                                crate::RustExpr::UnaryOp {
                                    op: "!".to_string(),
                                    operand: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(temp_name)),
                                        method: "contains".to_string(),
                                        args: vec![crate::RustExpr::Ident("__item".to_string())],
                                    }),
                                }
                            }),
                            is_move: false,
                        }],
                    }));
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(result_name.map_or_else(
                        || crate::RustExpr::Literal(crate::RustLiteral::Unit),
                        crate::RustExpr::Ident,
                    ))),
                });
            }
            "symmetric_difference" | "symmetric_difference_update" => {
                if args.len() != 1 {
                    return None;
                }
                let other = registry_iterable_to_set_expr(self, &args[0])?;
                let diff_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(if method.ends_with("_update") {
                            object_expr.clone()
                        } else {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(object_expr.clone()),
                                method: "clone".to_string(),
                                args: vec![],
                            }
                        }),
                        method: "symmetric_difference".to_string(),
                        args: vec![crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__other".to_string())),
                        }],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                };
                let new_set_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(diff_expr),
                    method: "collect::<std::collections::HashSet<_>>".to_string(),
                    args: vec![],
                };
                let mut stmts = vec![crate::RustStmt::Let {
                    mutable: false,
                    name: "__other".to_string(),
                    ty: None,
                    value: other,
                }];
                if method.ends_with("_update") {
                    stmts.push(crate::RustStmt::Assign {
                        target: object_expr.clone(),
                        value: new_set_expr,
                    });
                    return Some(crate::RustExpr::Block {
                        stmts,
                        expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                    });
                }
                return Some(crate::RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(new_set_expr)),
                });
            }
            _ => {}
        }
        None
    }

    pub(crate) fn try_lower_defaultdict_index_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        method_return_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = object
        else {
            return None;
        };
        let (alias_name, key_ty, value_ty) = registry_defaultdict_alias_parts(base_object.ty())?;
        let is_bucket_mutator = match alias_name {
            "__sifr_defaultdict_list" => is_defaultdict_list_bucket_mutator(method),
            "__sifr_defaultdict_set" => is_defaultdict_set_bucket_mutator(method),
            _ => false,
        };
        if !is_bucket_mutator {
            return None;
        }
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let entry_expr = crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "entry".to_string(),
                args: vec![registry_defaultdict_key_arg(index, lowered_index, key_ty)],
            }),
            method: "or_insert".to_string(),
            args: vec![registry_defaultdict_default_expr(alias_name)],
        };

        if alias_name == "__sifr_defaultdict_list" && method == "extend" {
            let [iterable] = args else {
                return None;
            };
            return Some(crate::RustExpr::MethodCall {
                receiver: Box::new(entry_expr),
                method: "extend".to_string(),
                args: vec![registry_iterable_to_owned_iter_expr(self, iterable)?],
            });
        }

        if alias_name == "__sifr_defaultdict_set"
            && matches!(
                method,
                "update"
                    | "intersection_update"
                    | "difference_update"
                    | "symmetric_difference_update"
            )
        {
            let entry_target =
                crate::RustExpr::Paren(Box::new(crate::RustExpr::Deref(Box::new(entry_expr))));
            return self.try_lower_registry_set_method_call_expr(&entry_target, method, args);
        }

        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(
                self.try_lower_registry_expr_strict(arg)
                    .or_else(|| self.lower_stmt_expr_for_ir(arg).ok().flatten())?,
            );
        }

        match (alias_name, method, args, lowered_args.as_mut_slice()) {
            ("__sifr_defaultdict_list", "append", [value], [lowered_value]) => {
                let owned_value =
                    Self::clone_owned_append_arg_expr_for_ir(value, lowered_value.clone());
                Some(crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(entry_expr),
                        method: "push".to_string(),
                        args: vec![owned_value],
                    })],
                    expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                })
            }
            ("__sifr_defaultdict_set", "add", [value], [lowered_value]) => {
                let owned_value =
                    Self::clone_owned_append_arg_expr_for_ir(value, lowered_value.clone());
                Some(crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(entry_expr),
                        method: "insert".to_string(),
                        args: vec![owned_value],
                    })],
                    expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
                })
            }
            ("__sifr_defaultdict_list", "insert", [_, value], [_, lowered_value]) => {
                *lowered_value =
                    Self::clone_owned_append_arg_expr_for_ir(value, lowered_value.clone());
                methods::lower_method(value_ty, method, &entry_expr, &lowered_args)
                    .map(|lowered| lowered.expr)
            }
            (
                "__sifr_defaultdict_list" | "__sifr_defaultdict_set",
                "remove" | "discard",
                [value],
                [lowered_value],
            ) => {
                *lowered_value =
                    Self::clone_owned_append_arg_expr_for_ir(value, lowered_value.clone());
                methods::lower_method(value_ty, method, &entry_expr, &lowered_args)
                    .map(|lowered| lowered.expr)
            }
            _ => {
                let lowered = methods::lower_method(value_ty, method, &entry_expr, &lowered_args)?;
                Some(Self::unwrap_compiler_verified_nonempty_pop_result(
                    value_ty,
                    method,
                    args,
                    method_return_ty,
                    lowered.expr,
                ))
            }
        }
    }

    pub(crate) fn try_lower_defaultdict_index_contains_expr(
        &mut self,
        element: &HirExpr,
        collection: &HirExpr,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = collection
        else {
            return None;
        };
        let (alias_name, key_ty, _) = registry_defaultdict_alias_parts(base_object.ty())?;
        if !crate::intrinsics::is_collection_defaultdict_storage_alias(alias_name) {
            return None;
        }
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_element = self.lower_stmt_expr_for_ir(element).ok()??;
        let element_arg = if matches!(
            element,
            HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
        ) {
            lowered_element
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
            }
        };
        let entry_expr = crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "entry".to_string(),
                args: vec![registry_defaultdict_key_arg(index, lowered_index, key_ty)],
            }),
            method: "or_insert".to_string(),
            args: vec![registry_defaultdict_default_expr(alias_name)],
        };
        Some(crate::RustExpr::MethodCall {
            receiver: Box::new(entry_expr),
            method: "contains".to_string(),
            args: vec![element_arg],
        })
    }

    pub(crate) fn try_lower_last_use_set_add_expr(
        &mut self,
        expr: &HirExpr,
        preceding_stmts: &[crate::HirStmt],
        following_stmts: &[crate::HirStmt],
    ) -> Option<crate::RustExpr> {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        else {
            return None;
        };
        if method != "add" || args.len() != 1 {
            return None;
        }
        if self.stmt_block_depth > 2 {
            return None;
        }
        if !matches!(
            crate::resolve_alias_type_for_plain_call(object.ty()),
            Type::Set(_)
        ) {
            return None;
        }
        let HirExpr::Name { name, ty, .. } = &args[0] else {
            return None;
        };
        if self.borrowed_params.contains(name)
            || self.mut_borrowed_params.contains(name)
            || crate::helpers::is_copy_type_for_codegen(ty)
        {
            return None;
        }
        if !preceding_stmts.iter().any(|stmt| {
            matches!(
                stmt,
                crate::HirStmt::Let {
                    name: binding_name,
                    ..
                } if binding_name == name
            )
        }) {
            return None;
        }
        if crate::collect_referenced_vars_with_types(following_stmts)
            .iter()
            .any(|(referenced, _)| referenced == name)
        {
            return None;
        }
        Some(crate::RustExpr::MethodCall {
            receiver: Box::new(self.try_lower_registry_expr_strict(object)?),
            method: "insert".to_string(),
            args: vec![crate::RustExpr::Ident(name.clone())],
        })
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

    pub(crate) fn try_lower_registry_expr_strict(
        &mut self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        match self.try_lower_registry_expr_result(expr) {
            Ok(Some(lowered_expr)) => Some(lowered_expr),
            Ok(None) => self.try_lower_registry_expr_recursive(expr),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                None
            }
        }
    }
}
