use super::{
    HirExpr, HirFStringPart, RustEmitter, Type, methods, registry_defaultdict_alias_parts,
    registry_defaultdict_default_expr, registry_defaultdict_key_arg,
    registry_iterator_op_func_name, registry_option_inner_type, registry_uses_debug_display_format,
};
use crate::place_emitter::MethodCallPlaces;
use crate::stmt_support_emitter::canonical_constructor_class_name;
impl RustEmitter {
    pub(crate) fn try_lower_registry_expr_recursive(
        &mut self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        match expr {
            HirExpr::Name { name, .. } => Some(crate::RustExpr::Ident(name.clone())),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => Some(crate::RustExpr::If {
                cond: Box::new(self.try_lower_registry_expr_strict(condition)?),
                then_expr: Box::new(self.try_lower_registry_expr_strict(then_expr)?),
                else_expr: Some(Box::new(self.try_lower_registry_expr_strict(else_expr)?)),
            }),
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
            HirExpr::IteratorCall {
                op,
                args,
                mutable_arg_places,
                ..
            } => {
                let func = registry_iterator_op_func_name(op);
                if let Some(lowered) = self.try_lower_registry_builtin_call_expr(
                    func,
                    args,
                    Some(expr.ty()),
                    mutable_arg_places,
                ) {
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
            HirExpr::IntrinsicCall {
                intrinsic, args, ..
            } => self.try_lower_registry_intrinsic_call_expr(*intrinsic, args, expr.ty()),
            HirExpr::Call {
                func,
                args,
                mutable_arg_places,
                ..
            } => {
                if let Some(lowered) = self.try_lower_registry_builtin_call_expr(
                    func,
                    args,
                    Some(expr.ty()),
                    mutable_arg_places,
                ) {
                    return Some(lowered);
                }
                if let Some(lowered) =
                    self.try_lower_registry_plain_call_with_places(func, args, mutable_arg_places)
                {
                    return Some(lowered);
                }
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::stmt_support_emitter::plain_call_target_for_ir(func)),
                    args: self.try_lower_registry_exprs_strict(args)?,
                })
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
                receiver_convention,
                receiver_target,
                mutable_arg_places,
                ..
            } => {
                let places = MethodCallPlaces::new(
                    *receiver_convention,
                    receiver_target.as_ref(),
                    mutable_arg_places,
                );
                if let Some(lowered) = crate::python_buffer_codegen::lower_python_buffer_method(
                    self, object, method, args, places, ty,
                ) {
                    return Some(lowered);
                }
                if let Some(lowered) = crate::python_arrow_codegen::lower_python_arrow_method(
                    self, object, method, args, places, ty,
                ) {
                    return Some(lowered);
                }
                if let Some(lowered) = crate::python_dlpack_codegen::lower_python_dlpack_method(
                    self, object, method, args, places, ty,
                ) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_recursive_indexed_list_append(
                    object,
                    method,
                    args,
                    receiver_target.as_ref(),
                ) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_dict_indexed_list_append_expr(expr) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_dict_indexed_list_pop_expr(expr) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_dict_indexed_list_len_expr(expr) {
                    return Some(lowered);
                }
                let (object_expr, effective_object_ty, mut arg_exprs) = self
                    .lower_recursive_method_receiver_and_args(
                        object,
                        method,
                        args,
                        *receiver_convention,
                        receiver_target.as_ref(),
                        mutable_arg_places,
                    )?;
                let method_params =
                    self.resolve_registry_method_params(&effective_object_ty, method);
                self.adapt_owned_mapping_default(
                    &effective_object_ty,
                    method,
                    args,
                    &mut arg_exprs,
                );
                if matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::Decimal | Type::BigDecimal
                ) && matches!(method.as_str(), "quantize" | "round")
                    && args.len() == 1
                {
                    let scale = crate::integer_literal_decimal(&args[0])
                        .and_then(|value| value.parse::<i64>().ok())?;
                    arg_exprs[0] = crate::RustExpr::Literal(crate::RustLiteral::Int(scale));
                }
                if let Type::List(element_ty) =
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty)
                {
                    if method == "append" && arg_exprs.len() == 1 && args.len() == 1 {
                        let arg_ty = if let HirExpr::Name { name, ty, .. } = &args[0] {
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
                        let adapted = crate::helpers::flatten_option_value_for_target(
                            element_ty.as_ref(),
                            &arg_ty,
                            adjusted.clone(),
                        );
                        let option_value_adapted = adapted != adjusted;
                        adjusted = adapted;
                        if expects_option && !has_option && !matches!(args[0], HirExpr::NoneLiteral)
                        {
                            adjusted = crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![adjusted],
                            };
                        } else if !expects_option && has_option && !option_value_adapted {
                            if !crate::helpers::is_copy_type_for_codegen(&arg_ty) {
                                adjusted = crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(adjusted))),
                                    method: "clone".to_string(),
                                    args: vec![],
                                };
                            }
                        }
                        arg_exprs[0] = self.materialize_reusable_value_for_ir(&args[0], adjusted);
                    }
                }
                if effective_object_ty.callable_field_type(method).is_some() {
                    if let Some(method_params) = method_params.as_deref() {
                        self.apply_registry_method_arg_conventions(
                            args,
                            method_params,
                            &mut arg_exprs,
                        );
                    }
                    return Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Field {
                            expr: Box::new(object_expr),
                            field: method.clone(),
                        }))),
                        args: arg_exprs,
                    });
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
                        object_expr,
                        self.is_deque_data_field(object),
                        lowered.expr,
                    ));
                }
                if let Some(method_params) = method_params {
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
                    object_expr.clone(),
                    self.is_deque_data_field(object),
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(object_expr),
                        method: method.clone(),
                        args: arg_exprs,
                    },
                ))
            }
            HirExpr::ConstructorCall {
                class_name,
                args,
                ty,
            } => {
                let emitted_class_name = canonical_constructor_class_name(class_name, ty);
                let emitted_ctor_key = format!("{emitted_class_name}::new");
                let source_ctor_key = format!("{class_name}::new");
                let registry_ctor_key = if self.func_signatures.contains_key(&emitted_ctor_key) {
                    &emitted_ctor_key
                } else {
                    &source_ctor_key
                };
                if let Some(mut lowered) =
                    self.try_lower_registry_plain_call_with_signature(registry_ctor_key, args)
                {
                    if let crate::RustExpr::FnCall { func, .. } = &mut lowered {
                        **func = crate::RustExpr::Path(vec![emitted_class_name, "new".to_string()]);
                    }
                    return Some(lowered);
                }

                let mut path = emitted_class_name
                    .split("::")
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                path.push("new".to_string());
                let lowered_args = self.try_lower_registry_exprs_strict(args)?;
                let lowered_args = self.adapt_plain_call_args_with_signature_for_ir(
                    registry_ctor_key,
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
                if let Some(witness) = self.checked_place_read_witness(object, index, ty) {
                    return Some(witness);
                }
                if matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::Str | Type::LiteralStr(_)
                ) {
                    let lowered_object = self.try_lower_registry_expr_strict(object)?;
                    let lowered_index = Self::clone_non_copy_name_expr_for_ir(
                        index,
                        self.try_lower_registry_expr_strict(index)?,
                    );
                    let option_expr = self.lower_string_index_option_with_cache(
                        object,
                        lowered_object,
                        lowered_index,
                    );
                    if crate::helpers::is_option_type(ty) {
                        return Some(option_expr);
                    }
                    return None;
                }
                if let Some(lowered) = self.try_lower_nested_list_element_expr(expr) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_list_indexed_dict_element_expr(expr) {
                    return Some(lowered);
                }
                if let Some(lowered) = self.try_lower_dict_indexed_list_element_expr(expr) {
                    return Some(lowered);
                }
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
                                    self.string_view_expr(index, lowered_index)
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
                            Type::Str => self.lower_string_index_option_with_cache(
                                object,
                                crate::RustExpr::Ident("__v".to_string()),
                                lowered_index,
                            ),
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
                                self.string_view_expr(index, lowered_index)
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
                        Type::Str => Some(self.lower_string_index_option_with_cache(
                            object,
                            lowered_object,
                            lowered_index,
                        )),
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
                            if matches!(
                                crate::resolve_alias_type_for_plain_call(expr.ty()),
                                Type::None
                            ) {
                                lowered_args.push(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                ));
                                continue;
                            }
                            let lowered_expr = self.try_lower_registry_expr_strict(expr)?;
                            if let Some(inner_ty) = registry_option_inner_type(expr.ty()) {
                                let inner_format_str =
                                    if registry_uses_debug_display_format(&inner_ty) {
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
            HirExpr::TemplateString(template) => {
                self.try_lower_template_string_expr_for_ir(template)
            }
            HirExpr::BoolOp { ty, .. }
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Bool) =>
            {
                self.lower_condition_expr_for_ir(expr).ok().flatten()
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
                    Type::Int | Type::LiteralInt(_) => None,
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
                && matches!(ty, Type::Float | Type::Int | Type::LiteralInt(_)) =>
            {
                if matches!(op.as_str(), "/" | "//" | "%")
                    && matches!(
                        crate::resolve_alias_type_for_plain_call(left.ty()),
                        Type::Int | Type::LiteralInt(_)
                    )
                    && matches!(
                        crate::resolve_alias_type_for_plain_call(right.ty()),
                        Type::Int | Type::LiteralInt(_)
                    )
                {
                    return None;
                }
                let left_expr = self.try_lower_registry_expr_strict(left)?;
                let right_expr = self.try_lower_registry_expr_strict(right)?;
                let borrow_integer = |value| {
                    if matches!(ty.resolve_alias(), Type::Int | Type::LiteralInt(_)) {
                        self.coerce_expr_to_sifr_int_comparison_operand(value)
                    } else {
                        value
                    }
                };
                Some(crate::RustExpr::BinOp {
                    left: Box::new(borrow_integer(left_expr)),
                    op: if op == "//" {
                        "/".to_string()
                    } else {
                        op.clone()
                    },
                    right: Box::new(borrow_integer(right_expr)),
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
            HirExpr::DictLiteral { keys, values, ty } => {
                self.try_lower_registry_dict_literal_expr(keys, values, ty)
            }
            HirExpr::ListLiteral { elements, ty } => {
                if elements.is_empty() {
                    if let Some(lowered) = crate::lower_expr::typed_empty_list_expr(ty) {
                        return Some(lowered);
                    }
                }
                let list_ty = crate::resolve_alias_type_for_plain_call(ty);
                let mut lowered = elements
                    .iter()
                    .map(|element| {
                        let lowered = self.try_lower_registry_expr_strict(element)?;
                        let lowered = if let Type::List(element_ty) = list_ty {
                            crate::helpers::adapt_collection_value_for_target(
                                element_ty.as_ref(),
                                element,
                                lowered,
                            )
                        } else {
                            lowered
                        };
                        Some(self.materialize_reusable_value_for_ir(element, lowered))
                    })
                    .collect::<Option<Vec<_>>>()?;
                if matches!(list_ty, Type::Bytes) {
                    lowered = lowered
                        .into_iter()
                        .zip(elements)
                        .map(|(lowered, element)| {
                            crate::helpers::adapt_bytes_element_for_storage(element, lowered)
                        })
                        .collect::<Option<Vec<_>>>()?;
                }
                Some(crate::RustExpr::Vec(lowered))
            }
            HirExpr::TupleLiteral { elements, ty } => {
                let lowered = elements
                    .iter()
                    .map(|element| {
                        let lowered = self.try_lower_registry_expr_strict(element)?;
                        Some(self.materialize_reusable_value_for_ir(element, lowered))
                    })
                    .collect::<Option<Vec<_>>>()?;
                if crate::homogeneous_large_tuple_backing_array(ty).is_some() {
                    Some(crate::RustExpr::Array(lowered))
                } else {
                    Some(crate::RustExpr::Tuple(lowered))
                }
            }
            HirExpr::SetLiteral { elements, ty } => {
                self.try_lower_registry_set_literal_expr(elements, ty)
            }
            _ => None,
        }
    }
}
