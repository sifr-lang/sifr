use crate::{intrinsics, methods, RustEmitter, RustExpr};
use sifr_hir::{HirExpr, HirFStringPart, HirIteratorOp};
use sifr_type_system::{ParamConvention, Type};

fn registry_uses_debug_display_format(ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(ty) {
        Type::Int
        | Type::Float
        | Type::Bool
        | Type::Str
        | Type::None
        | Type::Range
        | Type::Union(_)
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Class { .. }
        | Type::Newtype { .. }
        | Type::TypeVar(_)
        | Type::Enum { .. }
        | Type::BigInt
        | Type::Decimal
        | Type::BigDecimal => false,
        Type::List(_)
        | Type::Bytes
        | Type::Dict(_, _)
        | Type::Set(_)
        | Type::Tuple(_)
        | Type::Iterable(_)
        | Type::Iterator(_)
        | Type::Function(_)
        | Type::Callable(..)
        | Type::Result(_, _)
        | Type::Protocol { .. }
        | Type::Any
        | Type::Unknown
        | Type::Intersection(_)
        | Type::Never => true,
        Type::Alias { body, .. } => registry_uses_debug_display_format(body),
    }
}

fn registry_option_inner_type(ty: &Type) -> Option<&Type> {
    let resolved = crate::resolve_alias_type_for_plain_call(ty);
    let Type::Union(members) = resolved else {
        return None;
    };
    if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
        return None;
    }
    members.iter().find(|member| !matches!(member, Type::None))
}

fn registry_is_string_like_type(ty: &Type) -> bool {
    matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        Type::Str | Type::LiteralStr(_)
    )
}

fn registry_defaultdict_alias_parts(ty: &Type) -> Option<(&str, &Type, &Type)> {
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = ty
    else {
        return None;
    };
    if !alias_name.starts_with("__compat_defaultdict_") {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
        return None;
    };
    Some((alias_name.as_str(), key_ty.as_ref(), value_ty.as_ref()))
}

fn registry_defaultdict_default_expr(alias_name: &str) -> RustExpr {
    match alias_name {
        "__compat_defaultdict_int" => RustExpr::Literal(crate::RustLiteral::Int(0)),
        "__compat_defaultdict_list" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
            args: vec![],
        },
        "__compat_defaultdict_set" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "HashSet".to_string(),
                "new".to_string(),
            ])),
            args: vec![],
        },
        _ => RustExpr::Literal(crate::RustLiteral::Unit),
    }
}

fn registry_defaultdict_key_arg(
    index: &HirExpr,
    lowered_index: RustExpr,
    key_ty: &Type,
) -> RustExpr {
    if let HirExpr::StringLiteral(value) = index {
        RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
    } else {
        let _ = key_ty;
        RustExpr::Clone(Box::new(lowered_index))
    }
}

fn registry_callable_signature(expr: &HirExpr) -> Option<(Vec<Type>, Vec<ParamConvention>, Type)> {
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Function(ft) => Some((
            ft.params.iter().map(|(_, ty, _)| ty.clone()).collect(),
            ft.params
                .iter()
                .map(|(_, _, convention)| *convention)
                .collect(),
            *ft.return_type.clone(),
        )),
        Type::Callable(params, conventions, return_type) => {
            Some((params.clone(), conventions.clone(), *return_type.clone()))
        }
        _ => None,
    }
}

fn registry_iterator_op_func_name(op: &HirIteratorOp) -> &'static str {
    match op {
        HirIteratorOp::Iter => "iter",
        HirIteratorOp::Next => "next",
        HirIteratorOp::Reversed => "reversed",
        HirIteratorOp::Map => "map",
        HirIteratorOp::Filter => "filter",
        HirIteratorOp::Zip => "zip",
        HirIteratorOp::Enumerate => "enumerate",
    }
}

fn registry_tuple_homogeneous_iter_expr(lowered: RustExpr, tuple_len: usize) -> Option<RustExpr> {
    if tuple_len == 0 {
        return None;
    }
    let tuple_binding = "__sifr_tuple_iter_src".to_string();
    let tuple_items = (0..tuple_len)
        .map(|index| RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident(tuple_binding.clone())),
                field: index.to_string(),
            }),
            method: "clone".to_string(),
            args: vec![],
        })
        .collect();
    Some(RustExpr::Block {
        stmts: vec![crate::RustStmt::Let {
            mutable: false,
            name: tuple_binding,
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "clone".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Vec(tuple_items)),
            method: "into_iter".to_string(),
            args: vec![],
        })),
    })
}

fn registry_iterable_to_owned_iter_expr(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
) -> Option<RustExpr> {
    let lowered = emitter.try_lower_registry_expr_strict(expr)?;
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::List(_) | Type::Set(_) | Type::Iterable(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }),
        Type::Bytes => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__byte".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                            "__byte".to_string(),
                        )))),
                        ty: crate::RustType::I64,
                    }),
                    is_move: false,
                }],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }),
        Type::Iterator(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
            method: "into_iter".to_string(),
            args: vec![],
        }),
        Type::Range => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
            method: "into_iter".to_string(),
            args: vec![],
        }),
        Type::Str => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_char".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_char".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        Type::Dict(_, _) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "keys".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        Type::Tuple(elems) if !elems.is_empty() && elems.iter().all(|elem| elem == &elems[0]) => {
            registry_tuple_homogeneous_iter_expr(lowered, elems.len())
        }
        _ => None,
    }
}

fn registry_box_iterator_expr(iter_expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![iter_expr],
    }
}

fn registry_iterable_to_vec_expr(emitter: &mut RustEmitter, expr: &HirExpr) -> Option<RustExpr> {
    Some(RustExpr::MethodCall {
        receiver: Box::new(registry_iterable_to_owned_iter_expr(emitter, expr)?),
        method: "collect::<Vec<_>>".to_string(),
        args: vec![],
    })
}

fn registry_iterable_to_set_expr(emitter: &mut RustEmitter, expr: &HirExpr) -> Option<RustExpr> {
    Some(RustExpr::MethodCall {
        receiver: Box::new(registry_iterable_to_owned_iter_expr(emitter, expr)?),
        method: "collect::<std::collections::HashSet<_>>".to_string(),
        args: vec![],
    })
}

fn registry_dict_source_to_map_expr(emitter: &mut RustEmitter, expr: &HirExpr) -> Option<RustExpr> {
    let lowered = emitter.try_lower_registry_expr_strict(expr)?;
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Dict(_, _) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
            method: "clone".to_string(),
            args: vec![],
        }),
        Type::List(_) | Type::Set(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "clone".to_string(),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            }),
            method: "collect::<HashMap<_, _>>".to_string(),
            args: vec![],
        }),
        Type::Any | Type::Unknown => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "into_iter".to_string(),
                args: vec![],
            }),
            method: "collect::<HashMap<_, _>>".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}

fn registry_call_callable_with_owned_args(
    emitter: &mut RustEmitter,
    callable: &HirExpr,
    arg_bindings: &[(String, Type)],
) -> Option<RustExpr> {
    let callable_expr = emitter.try_lower_registry_expr_strict(callable)?;
    let (param_types, conventions, _) = registry_callable_signature(callable)?;
    if param_types.len() != arg_bindings.len() {
        return None;
    }
    let mut lowered_args = Vec::with_capacity(arg_bindings.len());
    for ((name, _arg_ty), convention) in arg_bindings.iter().zip(conventions.iter()) {
        let base_expr = RustExpr::Ident(name.clone());
        let lowered_arg = if convention.is_borrowed() {
            RustExpr::Ref {
                mutable: convention.is_mutable(),
                expr: Box::new(base_expr),
            }
        } else {
            base_expr
        };
        lowered_args.push(lowered_arg);
    }
    Some(RustExpr::FnCall {
        func: Box::new(callable_expr),
        args: lowered_args,
    })
}

fn registry_nested_zip_field_expr(base: RustExpr, total: usize, index: usize) -> RustExpr {
    if total == 1 {
        return base;
    }
    let left_count = total - 1;
    if index < left_count {
        registry_nested_zip_field_expr(
            RustExpr::Field {
                expr: Box::new(base),
                field: "0".to_string(),
            },
            left_count,
            index,
        )
    } else {
        RustExpr::Field {
            expr: Box::new(base),
            field: "1".to_string(),
        }
    }
}

fn registry_zip_iter_expr(emitter: &mut RustEmitter, args: &[HirExpr]) -> Option<RustExpr> {
    let mut iter = args.iter();
    let mut acc = registry_iterable_to_owned_iter_expr(emitter, iter.next()?)?;
    for arg in iter {
        let next_iter = registry_iterable_to_owned_iter_expr(emitter, arg)?;
        acc = RustExpr::MethodCall {
            receiver: Box::new(acc),
            method: "zip".to_string(),
            args: vec![next_iter],
        };
    }
    Some(acc)
}

fn registry_can_construct_error_from_message(ty_name: &str) -> bool {
    matches!(
        ty_name,
        "Error"
            | "ValueError"
            | "TypeError"
            | "NameError"
            | "ParseError"
            | "OverflowError"
            | "ZeroDivisionError"
            | "LookupError"
            | "IndexError"
            | "KeyError"
            | "RuntimeError"
            | "AssertionError"
            | "ImportError"
            | "IOError"
            | "RegexError"
            | "HashlibError"
            | "DecimalConversionError"
    )
}

fn registry_is_some_ctor(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
        || matches!(expr, RustExpr::Ident(name) if name == "Some")
}

fn registry_is_box_new_ctor(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
        || matches!(expr, RustExpr::Ident(name) if name == "Box::new")
}

fn registry_is_some_expr(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::FnCall { func, .. } if registry_is_some_ctor(func.as_ref()))
}

fn registry_ensure_some_box_inner(expr: RustExpr) -> RustExpr {
    match expr {
        RustExpr::FnCall { func, args }
            if registry_is_some_ctor(func.as_ref()) && args.len() == 1 =>
        {
            let mut args_iter = args.into_iter();
            let inner = args_iter
                .next()
                .expect("checked args.len() == 1 for Some(_) call");
            if matches!(&inner, RustExpr::FnCall { func, .. } if registry_is_box_new_ctor(func.as_ref()))
            {
                RustExpr::FnCall {
                    func,
                    args: vec![inner],
                }
            } else {
                RustExpr::FnCall {
                    func,
                    args: vec![RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                        args: vec![inner],
                    }],
                }
            }
        }
        other => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![other],
            }],
        },
    }
}

impl RustEmitter {
    /// Check if a name is a stdlib constant.
    pub(crate) fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
            && self.intrinsic_functions.contains(name)
    }

    pub(crate) fn try_lower_registry_method_call_expr(
        &mut self,
        object_ty: &Type,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        if let Some(lowered) =
            self.try_lower_defaultdict_index_method_call_expr(object, method, args)
        {
            return Some(lowered);
        }
        let is_deque_data_field = self.is_deque_data_field(object);
        let object_expr = self.try_lower_registry_expr_strict(object)?;
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
        Some(lowered.expr)
    }

    fn try_lower_registry_set_method_call_expr(
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

    fn try_lower_defaultdict_index_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = object
        else {
            return None;
        };
        let (alias_name, key_ty, _) = registry_defaultdict_alias_parts(base_object.ty())?;
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_args = self.try_lower_registry_exprs_strict(args)?;
        let entry_expr = crate::RustExpr::MethodCall {
            receiver: Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: "entry".to_string(),
                args: vec![registry_defaultdict_key_arg(index, lowered_index, key_ty)],
            }),
            method: "or_insert".to_string(),
            args: vec![registry_defaultdict_default_expr(alias_name)],
        };
        match (alias_name, method, lowered_args.as_slice()) {
            ("__compat_defaultdict_list", "append", [value]) => Some(crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(entry_expr),
                    method: "push".to_string(),
                    args: vec![value.clone()],
                })],
                expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
            }),
            ("__compat_defaultdict_set", "add", [value]) => Some(crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(entry_expr),
                    method: "insert".to_string(),
                    args: vec![value.clone()],
                })],
                expr: Some(Box::new(crate::RustExpr::Literal(crate::RustLiteral::Unit))),
            }),
            _ => None,
        }
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

    fn try_lower_registry_expr_recursive(&mut self, expr: &HirExpr) -> Option<crate::RustExpr> {
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
                if let Some(lowered) = self.try_lower_registry_builtin_call_expr(func, args) {
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
                    func: Box::new(crate::RustExpr::Ident(func.clone())),
                    args: self.try_lower_registry_exprs_strict(args)?,
                })
            }
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
                let object_expr = self.try_lower_registry_expr_strict(object)?;
                if needs_self_field_clone_suppression
                    && self.pending_self_field_clone_suppression > suppression_prev
                {
                    self.pending_self_field_clone_suppression -= 1;
                }
                let mut arg_exprs = self.try_lower_registry_exprs_strict(args)?;
                if let Type::Class {
                    fields, methods, ..
                } = crate::resolve_alias_type_for_plain_call(object.ty())
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
                    object.ty(),
                    method,
                    &object_expr,
                    &arg_exprs,
                    self.is_deque_data_field(object),
                ) {
                    return Some(lowered.expr);
                }
                if let Some(method_params) =
                    self.resolve_registry_method_params(object.ty(), method)
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
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(object_expr),
                    method: method.clone(),
                    args: arg_exprs,
                })
            }
            HirExpr::ConstructorCall {
                class_name, args, ..
            } => {
                let mut path = class_name
                    .split("::")
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                path.push("new".to_string());
                Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(path)),
                    args: self.try_lower_registry_exprs_strict(args)?,
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
                                        ty: crate::RustType::I64,
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
                        Type::Dict(key_ty, _) => {
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
                                method: "cloned".to_string(),
                                args: vec![],
                            })
                        }
                        Type::List(_) => Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(lowered_index),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }),
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
                                    ty: crate::RustType::I64,
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
            "list" if args.len() == 1 => registry_iterable_to_vec_expr(self, &args[0]),
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
            "set" if args.len() == 1 => Some(RustExpr::MethodCall {
                receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
                method: "collect::<HashSet<_>>".to_string(),
                args: vec![],
            }),
            "iter" if args.len() == 1 => {
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Iterator(_)
                ) {
                    self.try_lower_registry_expr_strict(&args[0])
                } else {
                    Some(registry_box_iterator_expr(registry_iterable_to_owned_iter_expr(
                        self, &args[0],
                    )?))
                }
            }
            "sum" if args.len() == 1 => {
                let iter_chain = crate::RustExpr::MethodCall {
                    receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[0])?),
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
                    }],
                };
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[1].ty()),
                    Type::Iterator(_)
                ) {
                    Some(registry_box_iterator_expr(filtered_iter))
                } else {
                    Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "Vec".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![filtered_iter],
                    })
                }
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
                            }],
                        }),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }))
                }
            }
            "abs" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                    method: "abs".to_string(),
                    args: vec![],
                })
            }
            "ord" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__sifr_ord_chars".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "collect::<Vec<char>>".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__sifr_ord_chars".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(1))),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__sifr_ord_chars".to_string())),
                                    index: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                                }),
                                ty: crate::RustType::I64,
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "ValueError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    RustExpr::Literal(crate::RustLiteral::Str(
                                        "ord() expected a string of length 1".to_string(),
                                    )),
                                )],
                            }],
                        })),
                    })),
                })
            }
            "chr" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "char".to_string(),
                                "from_u32".to_string(),
                            ])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Paren(Box::new(lowered))),
                                ty: crate::RustType::Named("u32".to_string()),
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__sifr_chr".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__sifr_chr".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(RustExpr::StructInit {
                            name: "ValueError".to_string(),
                            fields: vec![(
                                "message".to_string(),
                                RustExpr::Literal(crate::RustLiteral::Str(
                                    "chr() arg not in range(0x110000)".to_string(),
                                )),
                            )],
                        }),
                        is_move: false,
                    }],
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
            "repr" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                if registry_option_inner_type(args[0].ty()).is_some() {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
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
                                    format_str: "{:?}".to_string(),
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    })
                } else {
                    Some(crate::RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{:?}".to_string(),
                        args: vec![lowered],
                    })
                }
            }
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
                    Type::Str | Type::Bytes | Type::List(_) | Type::Dict(_, _) => {
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
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::F64,
                    }),
                    Type::Str => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "parse::<f64>".to_string(),
                            args: vec![],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("e".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Bool => Some(crate::RustExpr::If {
                        cond: Box::new(lowered),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(
                            1.0,
                        ))),
                        else_expr: Some(Box::new(crate::RustExpr::Literal(
                            crate::RustLiteral::Float(0.0),
                        ))),
                    }),
                    _ => Some(lowered),
                }
            }
            "int" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Float => Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::I64,
                    }),
                    Type::Str => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "parse::<i64>".to_string(),
                            args: vec![],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("e".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Bool => Some(crate::RustExpr::If {
                        cond: Box::new(lowered),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(1))),
                        else_expr: Some(Box::new(crate::RustExpr::Literal(
                            crate::RustLiteral::Int(0),
                        ))),
                    }),
                    Type::BigInt => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "i64".to_string(),
                                "try_from".to_string(),
                            ])),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            }],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e_ignored".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "OverflowError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Literal(
                                            crate::RustLiteral::Str(
                                                "bigint value out of range for int".to_string(),
                                            ),
                                        )),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Decimal => Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "__decimal_bigint".to_string(),
                            ty: None,
                            value: crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "BigInt".to_string(),
                                    "from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "trunc".to_string(),
                                        args: vec![],
                                    }),
                                    method: "mantissa".to_string(),
                                    args: vec![],
                                }],
                            },
                        }],
                        expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__decimal_bigint".to_string(),
                                    )),
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e_ignored".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str(
                                                    "decimal value out of range for int"
                                                        .to_string(),
                                                ),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        })),
                    }),
                    Type::BigDecimal => Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "__decimal_bigint".to_string(),
                            ty: None,
                            value: crate::RustExpr::Field {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "with_scale".to_string(),
                                        args: vec![crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )],
                                    }),
                                    method: "into_bigint_and_scale".to_string(),
                                    args: vec![],
                                }),
                                field: "0".to_string(),
                            },
                        }],
                        expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__decimal_bigint".to_string(),
                                    )),
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e_ignored".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str(
                                                    "bigdecimal value out of range for int"
                                                        .to_string(),
                                                ),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        })),
                    }),
                    _ => Some(lowered),
                }
            }
            "bigint" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) | Type::BigInt => {
                        Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "BigInt".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![lowered],
                        })
                    }
                    Type::Decimal => Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigInt".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "trunc".to_string(),
                                args: vec![],
                            }),
                            method: "mantissa".to_string(),
                            args: vec![],
                        }],
                    }),
                    Type::BigDecimal => Some(crate::RustExpr::Field {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "with_scale".to_string(),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                            }),
                            method: "into_bigint_and_scale".to_string(),
                            args: vec![],
                        }),
                        field: "0".to_string(),
                    }),
                    _ => None,
                }
            }
            "Decimal" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered],
                    }),
                    Type::Decimal => Some(lowered),
                    Type::Str | Type::LiteralStr(_) => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "from_str_exact".to_string(),
                            ])),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "as_str".to_string(),
                                args: vec![],
                            }],
                        }),
                        method: "unwrap_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MacroCall {
                                name: "unreachable".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::BigInt | Type::BigDecimal => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "Decimal".to_string(),
                                    "from_str_exact".to_string(),
                                ])),
                                args: vec![crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "e".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "e".to_string(),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__v".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            is_move: false,
                        }],
                    }),
                    _ => None,
                }
            }
            "BigDecimal" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) | Type::BigInt => {
                        Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "BigDecimal".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![lowered],
                        })
                    }
                    Type::Decimal | Type::Str | Type::LiteralStr(_) => {
                        let source = match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                            Type::Decimal => crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            _ => lowered,
                        };
                        Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(source))),
                                method: "parse::<BigDecimal>".to_string(),
                                args: vec![],
                            }),
                            method: "unwrap_or_else".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::MacroCall {
                                    name: "unreachable".to_string(),
                                    args: vec![],
                                }),
                                is_move: false,
                            }],
                        })
                    }
                    Type::BigDecimal => Some(lowered),
                    _ => None,
                }
            }
            "str" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                let call_return_ty = if let HirExpr::Call { func, .. } = &args[0] {
                    self.func_signatures.get(func).map(|(_, ret)| ret.clone())
                } else {
                    None
                };
                let str_arg_ty = call_return_ty.as_ref().unwrap_or_else(|| args[0].ty());
                if let Some(inner) = registry_option_inner_type(str_arg_ty) {
                    let format_str = if registry_uses_debug_display_format(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
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
                                    format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    })
                } else {
                    Some(crate::RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: if registry_uses_debug_display_format(str_arg_ty) {
                            "{:?}".to_string()
                        } else {
                            "{}".to_string()
                        },
                        args: vec![lowered],
                    })
                }
            }
            _ => None,
        }
    }

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
            let mut lowered_arg = self.try_lower_registry_expr_strict(arg)?;
            if let Some(aligned_callable) = self
                .try_build_registry_callable_convention_alignment_expr(
                    arg,
                    resolved_param,
                    lowered_arg.clone(),
                )
            {
                lowered_arg = aligned_callable;
            }

            if let Type::Union(members) = resolved_param {
                if !crate::helpers::is_option_type(resolved_param)
                    && !matches!(
                        crate::resolve_alias_type_for_plain_call(arg.ty()),
                        Type::Union(_)
                    )
                {
                    if let Some(variant) = crate::helpers::find_union_variant(members, arg.ty()) {
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
                if !crate::helpers::is_option_type(arg.ty()) && !matches!(arg, HirExpr::NoneLiteral)
                {
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
            }

            let param_rust_type = param_ty.rust_type();
            if param_rust_type.starts_with("Box<dyn ")
                && !arg.ty().rust_type().starts_with("Box<dyn ")
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

            let borrowed_name_arg = matches!(arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));
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

    fn try_build_registry_callable_convention_alignment_expr(
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

    pub(crate) fn clone_moved_names_in_borrowed_aggregate(
        arg: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        Self::clone_moved_names_in_borrowed_aggregate_inner(arg, lowered, false)
    }

    fn clone_moved_names_in_borrowed_aggregate_inner(
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
            let lhs_none_like = matches!(lhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(lhs_expr.ty()),
                    Type::None
                );
            let rhs_none_like = matches!(rhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(rhs_expr.ty()),
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
                && registry_option_inner_type(lhs_expr.ty()).is_some()
                && registry_option_inner_type(rhs_expr.ty()).is_none()
                && !rhs_none_like
            {
                lowered_right = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_right],
                };
            } else if is_comparison_op
                && registry_option_inner_type(lhs_expr.ty()).is_none()
                && registry_option_inner_type(rhs_expr.ty()).is_some()
                && !lhs_none_like
            {
                lowered_left = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_left],
                };
            } else if registry_is_string_like_type(lhs_expr.ty())
                && registry_is_string_like_type(rhs_expr.ty())
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

fn canonicalize_compat_intrinsic_name(func: &str) -> &str {
    func.strip_prefix("__compat_sifr_math_").unwrap_or(func)
}

#[cfg(test)]
mod tests {
    use super::canonicalize_compat_intrinsic_name;

    #[test]
    fn intrinsic_emit_wrapper_layer_is_absent() {
        let src = include_str!("intrinsic_method_emitters.rs");
        let prod_src = src.split("\n#[cfg(test)]").next().unwrap_or(src);
        assert!(!prod_src.contains("pub(crate) fn emit_intrinsic_call("));
        assert!(!prod_src.contains("pub(crate) fn try_emit_intrinsic_via_registry("));
    }

    #[test]
    fn registry_arg_lowering_avoids_inline_rawcode_paths() {
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
    }

    #[test]
    fn canonicalizes_math_compat_intrinsic_aliases() {
        assert_eq!(
            canonicalize_compat_intrinsic_name("__compat_sifr_math_fmod"),
            "fmod"
        );
        assert_eq!(canonicalize_compat_intrinsic_name("fmod"), "fmod");
        assert_eq!(
            canonicalize_compat_intrinsic_name("__compat_sifr_heapq_heappush"),
            "__compat_sifr_heapq_heappush"
        );
    }
}
