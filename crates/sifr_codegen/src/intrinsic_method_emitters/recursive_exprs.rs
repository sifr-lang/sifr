use super::narrowing_helpers::canonicalize_compat_intrinsic_name;
use super::{
    intrinsics, methods, registry_defaultdict_alias_parts, registry_defaultdict_default_expr,
    registry_defaultdict_key_arg, registry_iterator_op_func_name, registry_option_inner_type,
    registry_uses_debug_display_format, HirExpr, HirFStringPart, RustEmitter, Type,
};
impl RustEmitter {
    pub(crate) fn try_lower_registry_expr_recursive(
        &mut self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        match expr {
            HirExpr::Name { name, .. } => Some(crate::RustExpr::Ident(name.clone())),
            HirExpr::FieldAccess { object, field, ty } => {
                if let Ok(Some(lowered)) =
                    self.try_lower_structured_field_access_expr(object, field, ty)
                {
                    return Some(lowered);
                }
                let lowered_object = self.try_lower_registry_expr_strict(object)?;
                Some(self.lower_field_access_expr_with_lowered_object(
                    object,
                    field,
                    ty,
                    lowered_object,
                ))
            }
            HirExpr::IteratorCall { op, args, .. } => {
                let func = registry_iterator_op_func_name(op);
                if let Some(lowered) = self.try_lower_registry_intrinsic_call_expr(func, args) {
                    return Some(lowered);
                }
                if let Some(lowered) =
                    self.try_lower_registry_builtin_call_expr(func, args, Some(expr.ty()))
                {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_registry_plain_call_with_signature(func, args)
                {
                    return Some(lowered);
                }
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Ident(func.to_string())),
                    args: self.try_lower_registry_exprs_strict(args)?,
                })
            }
            HirExpr::Call { func, args, .. } => {
                if let Some(lowered) = self.try_lower_registry_intrinsic_call_expr(func, args) {
                    return Some(lowered);
                }
                if let Some(lowered) =
                    self.try_lower_registry_builtin_call_expr(func, args, Some(expr.ty()))
                {
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
                    func: Box::new(crate::RustExpr::Ident(func.clone())),
                    args: self.try_lower_registry_exprs_strict(args)?,
                })
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
            } => {
                let needs_self_field_clone_suppression =
                    self.method_call_needs_field_clone_suppression(object, method);
                let suppression_prev = self.pending_self_field_clone_suppression;
                if needs_self_field_clone_suppression {
                    self.pending_self_field_clone_suppression += 1;
                }
                let object_expr = self.try_lower_registry_expr_strict(object)?;
                if needs_self_field_clone_suppression
                    && self.pending_self_field_clone_suppression > suppression_prev
                {
                    self.pending_self_field_clone_suppression -= 1;
                }
                let effective_object_ty = self.effective_method_object_ty(object);
                let mut arg_exprs = self.try_lower_registry_exprs_strict(args)?;
                if let Type::List(element_ty) =
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty)
                {
                    if method == "append" && arg_exprs.len() == 1 && args.len() == 1 {
                        let arg_ty = if let HirExpr::Name { name, ty } = &args[0] {
                            self.local_binding_types
                                .get(name)
                                .cloned()
                                .unwrap_or_else(|| ty.clone())
                        } else {
                            args[0].ty().clone()
                        };
                        let expects_option = crate::helpers::is_option_type(element_ty.as_ref());
                        let has_option = crate::helpers::is_option_type(&arg_ty);
                        let mut adjusted = arg_exprs[0].clone();
                        if expects_option && !has_option && !matches!(args[0], HirExpr::NoneLiteral)
                        {
                            adjusted = crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![adjusted],
                            };
                        } else if !expects_option && has_option {
                            if !crate::helpers::is_copy_type_for_codegen(&arg_ty) {
                                adjusted = crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(adjusted))),
                                    method: "clone".to_string(),
                                    args: vec![],
                                };
                            }
                            adjusted = Self::force_unwrap_option_expr_for_ir(
                                adjusted,
                                "compiler-verified list append element should be Some",
                            );
                        }
                        arg_exprs[0] = adjusted;
                    }
                }
                if let Type::Class {
                    fields, methods, ..
                } = crate::resolve_alias_type_for_plain_call(&effective_object_ty)
                {
                    let is_callable_field = !methods.iter().any(|(name, _)| name == method)
                        && fields
                            .iter()
                            .any(|(name, ty)| name == method && matches!(ty, Type::Callable(..)));
                    if is_callable_field {
                        return Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::Field {
                                    expr: Box::new(object_expr),
                                    field: method.clone(),
                                },
                            ))),
                            args: arg_exprs,
                        });
                    }
                }
                if let Some(lowered) = methods::lower_method_with_context(
                    &effective_object_ty,
                    method,
                    &object_expr,
                    &arg_exprs,
                    self.is_deque_data_field(object),
                ) {
                    return Some(Self::unwrap_compiler_verified_nonempty_pop_result(
                        &effective_object_ty,
                        method,
                        args,
                        ty,
                        lowered.expr,
                    ));
                }
                if let Some(method_params) =
                    self.resolve_registry_method_params(&effective_object_ty, method)
                {
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
                Some(Self::unwrap_compiler_verified_nonempty_pop_result(
                    object.ty(),
                    method,
                    args,
                    ty,
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr),
                        method: method.clone(),
                        args: arg_exprs,
                    },
                ))
            }
            HirExpr::ConstructorCall {
                class_name, args, ..
            } => {
                let mut path = class_name
                    .split("::")
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                path.push("new".to_string());
                let lowered_args = self.try_lower_registry_exprs_strict(args)?;
                let lowered_args = self.adapt_plain_call_args_with_signature_for_ir(
                    &path.join("::"),
                    args,
                    lowered_args,
                );
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(path)),
                    args: lowered_args,
                })
            }
            HirExpr::Index {
                object, index, ty, ..
            } => {
                if let Some((alias_name, key_ty, value_ty)) =
                    registry_defaultdict_alias_parts(object.ty())
                {
                    let lowered_object = self.try_lower_registry_expr_strict(object)?;
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
                    let value_expr = match crate::resolve_alias_type_for_plain_call(value_ty) {
                        Type::Int => crate::RustExpr::Deref(Box::new(entry_expr)),
                        _ => crate::RustExpr::MethodCall {
                            receiver: Box::new(entry_expr),
                            method: "clone".to_string(),
                            args: vec![],
                        },
                    };
                    if crate::helpers::is_option_type(ty) {
                        return Some(value_expr);
                    }
                    return Some(value_expr);
                }
                let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
                if let Type::Union(members) = object_ty {
                    let mut option_inner: Option<&Type> = None;
                    for member in members {
                        let resolved_member = crate::resolve_alias_type_for_plain_call(member);
                        if matches!(resolved_member, Type::None) {
                            continue;
                        }
                        if option_inner.is_some() {
                            option_inner = None;
                            break;
                        }
                        option_inner = Some(resolved_member);
                    }
                    if let Some(inner_ty) = option_inner {
                        let lowered_object = self.try_lower_registry_expr_strict(object)?;
                        let lowered_index = self.try_lower_registry_expr_strict(index)?;
                        let inner_expr = match inner_ty {
                            Type::Dict(key_ty, _) => {
                                let key_is_string_like = matches!(
                                    crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                                    Type::Str | Type::LiteralStr(_)
                                );
                                let key_arg = if let HirExpr::StringLiteral(value) = index.as_ref()
                                {
                                    crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                                } else if key_is_string_like {
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered_index,
                                        ))),
                                        method: "as_str".to_string(),
                                        args: vec![],
                                    }
                                } else {
                                    crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(lowered_index),
                                    }
                                };
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "__v".to_string(),
                                        )),
                                        method: "get".to_string(),
                                        args: vec![key_arg],
                                    }),
                                    method: "cloned".to_string(),
                                    args: vec![],
                                }
                            }
                            Type::List(_) => crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Cast {
                                        expr: Box::new(lowered_index),
                                        ty: crate::RustType::Named("usize".to_string()),
                                    }],
                                }),
                                method: "cloned".to_string(),
                                args: vec![],
                            },
                            Type::Bytes => crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Cast {
                                        expr: Box::new(lowered_index),
                                        ty: crate::RustType::Named("usize".to_string()),
                                    }],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__byte".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::Deref(Box::new(
                                            crate::RustExpr::Ident("__byte".to_string()),
                                        ))),
                                        ty: crate::RustType::Named("u8".to_string()),
                                    }),
                                    is_move: false,
                                }],
                            },
                            Type::Str => crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "__v".to_string(),
                                        )),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    }),
                                    method: "nth".to_string(),
                                    args: vec![crate::RustExpr::Cast {
                                        expr: Box::new(lowered_index),
                                        ty: crate::RustType::Named("usize".to_string()),
                                    }],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "c".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }),
                                    is_move: false,
                                }],
                            },
                            _ => return None,
                        };
                        let option_expr = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    lowered_object,
                                ))),
                                method: "as_ref".to_string(),
                                args: vec![],
                            }),
                            method: "and_then".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(inner_expr),
                                is_move: false,
                            }],
                        };
                        if crate::helpers::is_option_type(ty) {
                            return Some(option_expr);
                        }
                        return Some(option_expr);
                    }
                }
                if let Ok(Some(lowered)) = self.try_lower_structured_index_expr(object, index, ty) {
                    Some(lowered)
                } else {
                    let lowered_object = self.try_lower_registry_expr_strict(object)?;
                    let lowered_index = self.try_lower_registry_expr_strict(index)?;
                    match object_ty {
                        Type::Dict(key_ty, value_ty) => {
                            let projection_method =
                                crate::helpers::option_projection_method_for_owned_type(
                                    value_ty.as_ref(),
                                );
                            let key_is_string_like = matches!(
                                crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                                Type::Str | Type::LiteralStr(_)
                            );
                            let key_arg = if let HirExpr::StringLiteral(value) = index.as_ref() {
                                crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                            } else if key_is_string_like {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_index,
                                    ))),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }
                            } else {
                                crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(lowered_index),
                                }
                            };
                            Some(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(lowered_object),
                                    method: "get".to_string(),
                                    args: vec![key_arg],
                                }),
                                method: projection_method.to_string(),
                                args: vec![],
                            })
                        }
                        Type::List(element_ty) => {
                            let projection_method =
                                crate::helpers::option_projection_method_for_owned_type(
                                    element_ty.as_ref(),
                                );
                            Some(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(lowered_object),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Cast {
                                        expr: Box::new(lowered_index),
                                        ty: crate::RustType::Named("usize".to_string()),
                                    }],
                                }),
                                method: projection_method.to_string(),
                                args: vec![],
                            })
                        }
                        Type::Bytes => Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(lowered_index),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }],
                            }),
                            method: "map".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__byte".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Deref(Box::new(
                                        crate::RustExpr::Ident("__byte".to_string()),
                                    ))),
                                    ty: crate::RustType::Named("u8".to_string()),
                                }),
                                is_move: false,
                            }],
                        }),
                        Type::Str => Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
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
                            }),
                            method: "map".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "c".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                }),
                                is_move: false,
                            }],
                        }),
                        _ => None,
                    }
                }
            }
            HirExpr::FString { parts, .. } => {
                let mut format_str = String::new();
                let mut lowered_args = Vec::new();
                for part in parts {
                    match part {
                        HirFStringPart::Literal(s) => {
                            for ch in s.chars() {
                                match ch {
                                    '{' => format_str.push_str("{{"),
                                    '}' => format_str.push_str("}}"),
                                    _ => format_str.push(ch),
                                }
                            }
                        }
                        HirFStringPart::Expr(expr) => {
                            format_str.push_str("{}");
                            let lowered_expr = self.try_lower_registry_expr_strict(expr)?;
                            if let Some(inner_ty) = registry_option_inner_type(expr.ty()) {
                                let inner_format_str =
                                    if registry_uses_debug_display_format(inner_ty) {
                                        "{:?}".to_string()
                                    } else {
                                        "{}".to_string()
                                    };
                                lowered_args.push(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_expr,
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
                                                format_str: inner_format_str,
                                                args: vec![crate::RustExpr::Ident(
                                                    "__v".to_string(),
                                                )],
                                            }),
                                            is_move: false,
                                        },
                                    ],
                                });
                            } else if registry_uses_debug_display_format(expr.ty()) {
                                lowered_args.push(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: "{:?}".to_string(),
                                    args: vec![lowered_expr],
                                });
                            } else {
                                lowered_args.push(lowered_expr);
                            }
                        }
                    }
                }
                Some(crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str,
                    args: lowered_args,
                })
            }
            HirExpr::BoolOp { op, values, .. } if !values.is_empty() => {
                let lowered_op = match op.as_str() {
                    "and" => "&&",
                    "or" => "||",
                    _ => return None,
                };
                if op == "and" && values.len() == 2 {
                    if let Some(guarded_name) = Self::registry_detect_is_some_guard_name(&values[0])
                    {
                        if let Some(guarded_compare) = self
                            .try_lower_registry_guarded_option_compare_expr(
                                &values[1],
                                &guarded_name,
                            )
                        {
                            return Some(crate::RustExpr::BinOp {
                                left: Box::new(self.try_lower_registry_expr_strict(&values[0])?),
                                op: lowered_op.to_string(),
                                right: Box::new(guarded_compare),
                            });
                        }
                    }
                }
                let mut iter = values.iter();
                let mut lowered = self.try_lower_registry_expr_strict(iter.next()?)?;
                for value in iter {
                    lowered = crate::RustExpr::BinOp {
                        left: Box::new(lowered),
                        op: lowered_op.to_string(),
                        right: Box::new(self.try_lower_registry_expr_strict(value)?),
                    };
                }
                Some(lowered)
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
            } if matches!(op.as_str(), "+" | "-" | "*" | "/" | "//" | "%")
                && matches!(
                    ty,
                    Type::Float | Type::Int | Type::LiteralInt(_) | Type::BigInt
                ) =>
            {
                let mut left_expr = self.try_lower_registry_expr_strict(left)?;
                let mut right_expr = self.try_lower_registry_expr_strict(right)?;
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::BigInt) {
                    if matches!(
                        left.as_ref(),
                        HirExpr::Name { .. } | HirExpr::FieldAccess { .. }
                    ) {
                        left_expr = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                            method: "clone".to_string(),
                            args: vec![],
                        };
                    }
                    if matches!(
                        right.as_ref(),
                        HirExpr::Name { .. } | HirExpr::FieldAccess { .. }
                    ) {
                        right_expr = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                            method: "clone".to_string(),
                            args: vec![],
                        };
                    }
                }
                Some(crate::RustExpr::BinOp {
                    left: Box::new(left_expr),
                    op: if op == "//" {
                        "/".to_string()
                    } else {
                        op.clone()
                    },
                    right: Box::new(right_expr),
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
            ) =>
            {
                self.try_lower_registry_string_slice_expr(
                    object,
                    start.as_deref(),
                    stop.as_deref(),
                    step.as_deref(),
                )
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                self.try_lower_registry_dict_literal_expr(keys, values)
            }
            HirExpr::ListLiteral { elements, ty } => {
                let list_ty = crate::resolve_alias_type_for_plain_call(ty);
                let mut lowered = elements
                    .iter()
                    .map(|element| self.try_lower_registry_expr_strict(element))
                    .collect::<Option<Vec<_>>>()?;
                if matches!(list_ty, Type::Bytes) {
                    lowered = lowered
                        .into_iter()
                        .map(|element| crate::RustExpr::Cast {
                            expr: Box::new(element),
                            ty: crate::RustType::Named("u8".to_string()),
                        })
                        .collect();
                }
                Some(crate::RustExpr::Vec(lowered))
            }
            HirExpr::TupleLiteral { elements, .. } => Some(crate::RustExpr::Tuple(
                elements
                    .iter()
                    .map(|element| self.try_lower_registry_expr_strict(element))
                    .collect::<Option<Vec<_>>>()?,
            )),
            HirExpr::SetLiteral { elements, .. } => {
                self.try_lower_registry_set_literal_expr(elements)
            }
            _ => None,
        }
    }

    pub(crate) fn try_lower_registry_dict_literal_expr(
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

    pub(crate) fn try_lower_registry_set_literal_expr(
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

    pub(crate) fn try_lower_registry_intrinsic_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let intrinsic_func = canonicalize_compat_intrinsic_name(func);
        let mut ir_args = if let Some(lowered_args) = self.try_lower_registry_exprs_strict(args) {
            lowered_args
        } else {
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let lowered = self.lower_stmt_expr_for_ir(arg).ok().flatten()?;
                lowered_args.push(self.rewrite_stdlib_constant_idents_in_expr(lowered));
            }
            lowered_args
        };
        if matches!(
            intrinsic_func,
            "assert_eq" | "assert_ne" | "assert_gt" | "assert_lt" | "assert_almost_eq"
        ) {
            for (idx, arg) in args.iter().enumerate() {
                let HirExpr::Name { name, ty } = arg else {
                    continue;
                };
                if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                {
                    continue;
                }
                if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                    continue;
                }
                if let Some(lowered_arg) = ir_args.get(idx).cloned() {
                    ir_args[idx] = crate::RustExpr::Clone(Box::new(lowered_arg));
                }
            }
        }
        let lowered = intrinsics::lower_intrinsic(intrinsic_func, &ir_args)?;
        self.apply_intrinsic_registry_side_effects(intrinsic_func, &lowered);
        Some(lowered.expr)
    }
}
