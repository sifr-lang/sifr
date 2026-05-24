use super::{
    is_option_like_simple, resolve_alias_type, try_lower_leaf_expr, try_lower_leaf_or_name_expr,
    try_lower_task_duration_expr, HirExpr, ParamConvention, RustExpr, RustParam, RustStmt,
    RustType, Type,
};
pub(super) fn try_lower_simple_divmod_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [left, right] = args else {
        return None;
    };
    let lowered_left = try_lower_leaf_or_name_expr(left)?;
    let lowered_right = try_lower_leaf_or_name_expr(right)?;

    Some(RustExpr::Tuple(vec![
        RustExpr::BinOp {
            left: Box::new(lowered_left.clone()),
            op: "/".to_string(),
            right: Box::new(lowered_right.clone()),
        },
        RustExpr::BinOp {
            left: Box::new(lowered_left),
            op: "%".to_string(),
            right: Box::new(lowered_right),
        },
    ]))
}

pub(super) fn try_lower_simple_callable_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Lambda { params, body, .. } = expr {
        let lowered_params = params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        return Some(RustExpr::Closure {
            params: lowered_params,
            body: Box::new(try_lower_leaf_or_name_expr(body)?),
            is_move: false,
        });
    }
    try_lower_leaf_or_name_expr(expr)
}

pub(super) fn unwrap_simple_iter_source_expr(expr: &HirExpr) -> &HirExpr {
    match expr {
        HirExpr::IteratorCall {
            op: sifr_hir::HirIteratorOp::Iter,
            args,
            ..
        } if args.len() == 1 => &args[0],
        HirExpr::Call { func, args, .. } if func == "iter" && args.len() == 1 => &args[0],
        _ => expr,
    }
}

pub(super) fn apply_simple_copy_clone_yield_mode(
    iter_expr: RustExpr,
    yield_mode: crate::helpers::YieldMode,
) -> RustExpr {
    match yield_mode {
        crate::helpers::YieldMode::Copy => RustExpr::MethodCall {
            receiver: Box::new(iter_expr),
            method: "copied".to_string(),
            args: vec![],
        },
        crate::helpers::YieldMode::Clone => RustExpr::MethodCall {
            receiver: Box::new(iter_expr),
            method: "cloned".to_string(),
            args: vec![],
        },
        crate::helpers::YieldMode::Move | crate::helpers::YieldMode::Borrow => iter_expr,
    }
}

pub(super) fn try_lower_simple_iter_source_expr(iter_expr: &HirExpr) -> Option<RustExpr> {
    let source_expr = unwrap_simple_iter_source_expr(iter_expr);
    let lowered_source = try_lower_leaf_or_name_expr(source_expr)?;
    let source_ty = resolve_alias_type(source_expr.ty());
    let plan = crate::helpers::plan_iterator_ownership(source_expr);

    if matches!(source_ty, Type::Iterator(_)) {
        return Some(lowered_source);
    }

    match source_ty {
        Type::List(_) | Type::Set(_) | Type::Iterable(_) => Some(match plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(lowered_source),
                method: "into_iter".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Preserve => apply_simple_copy_clone_yield_mode(
                RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "iter".to_string(),
                    args: vec![],
                },
                plan.yield_mode,
            ),
        }),
        Type::Dict(_, _) => Some(match plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(lowered_source),
                method: "into_keys".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Preserve => apply_simple_copy_clone_yield_mode(
                RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "keys".to_string(),
                    args: vec![],
                },
                plan.yield_mode,
            ),
        }),
        Type::Bytes => Some(match plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "into_iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__byte".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__byte".to_string())),
                        ty: RustType::Named("u8".to_string()),
                    }),
                    is_move: false,
                }],
            },
            crate::helpers::SourceAccessMode::Preserve => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__byte".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                            "__byte".to_string(),
                        )))),
                        ty: RustType::Named("u8".to_string()),
                    }),
                    is_move: false,
                }],
            },
        }),
        Type::Str => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_source),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_char".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_char".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        Type::Range => Some(lowered_source),
        _ => Some(lowered_source),
    }
}

pub(super) fn try_lower_simple_map_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [callable, iter] = args else {
        return None;
    };
    let lowered_callable = lower_simple_map_callable_expr(callable, iter)?;
    let iter_source = try_lower_simple_iter_source_expr(iter)?;
    let mapped_iter = RustExpr::MethodCall {
        receiver: Box::new(iter_source),
        method: "map".to_string(),
        args: vec![lowered_callable],
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![mapped_iter],
    })
}

pub(super) fn lower_simple_map_callable_expr(
    callable: &HirExpr,
    iter: &HirExpr,
) -> Option<RustExpr> {
    let lowered_callable = try_lower_simple_callable_expr(callable)?;
    let Some((param_types, conventions)) = simple_callable_param_info(callable) else {
        return Some(lowered_callable);
    };
    if param_types.len() != 1 || conventions.len() != 1 {
        return Some(lowered_callable);
    }
    let iter_elem_ty =
        resolve_alias_type(unwrap_simple_iter_source_expr(iter).ty()).iterable_element_type()?;
    let adapted_arg = adapt_simple_map_callable_arg(
        RustExpr::Ident("__sifr_map_item".to_string()),
        &iter_elem_ty,
        &param_types[0],
        conventions[0],
    );
    Some(RustExpr::Closure {
        params: vec![RustParam::Named {
            name: "__sifr_map_item".to_string(),
            ty: RustType::Named("_".to_string()),
        }],
        body: Box::new(RustExpr::FnCall {
            func: Box::new(lowered_callable),
            args: vec![adapted_arg],
        }),
        is_move: false,
    })
}

pub(super) fn simple_callable_param_info(
    callable: &HirExpr,
) -> Option<(Vec<Type>, Vec<ParamConvention>)> {
    match resolve_alias_type(callable.ty()) {
        Type::Function(ft) | Type::AsyncFunction(ft) => Some((
            ft.params
                .iter()
                .map(|(_, ty, _)| ty.clone())
                .collect::<Vec<_>>(),
            ft.params
                .iter()
                .map(|(_, _, convention)| *convention)
                .collect::<Vec<_>>(),
        )),
        Type::Callable(param_types, conventions, _) => {
            Some((param_types.clone(), conventions.clone()))
        }
        _ => None,
    }
}

pub(super) fn adapt_simple_map_callable_arg(
    mut lowered_arg: RustExpr,
    arg_ty: &Type,
    param_ty: &Type,
    convention: ParamConvention,
) -> RustExpr {
    let resolved_param = resolve_alias_type(param_ty);
    let arg_is_option = is_option_like_simple(arg_ty);
    if is_option_like_simple(resolved_param) && !arg_is_option {
        lowered_arg = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_arg],
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

    if requires_shared_borrow {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_arg),
        }
    } else if requires_mut_borrow {
        RustExpr::Ref {
            mutable: true,
            expr: Box::new(lowered_arg),
        }
    } else {
        lowered_arg
    }
}

pub(super) fn try_lower_simple_filter_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [callable, iter] = args else {
        return None;
    };
    let iter_source_expr = unwrap_simple_iter_source_expr(iter);
    let iter_plan = crate::helpers::plan_iterator_ownership(iter_source_expr);
    let predicate_expr = if let HirExpr::Lambda { params, body, .. } = callable {
        if params.len() != 1 {
            return None;
        }
        let param_name = params[0].name.clone();
        let lowered_body = try_lower_leaf_or_name_expr(body)?;
        RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: param_name,
                ty: None,
                value: RustExpr::Ident("__filter_value".to_string()),
            }],
            expr: Some(Box::new(lowered_body)),
        }
    } else {
        let lowered_callable = try_lower_simple_callable_expr(callable)?;
        RustExpr::FnCall {
            func: Box::new(lowered_callable),
            args: vec![RustExpr::Ident("__filter_value".to_string())],
        }
    };

    let iter_source = try_lower_simple_iter_source_expr(iter)?;
    let predicate_input_expr = match iter_plan.element_ownership {
        Some(sifr_type_system::OwnershipKind::Copy) => {
            RustExpr::Deref(Box::new(RustExpr::Ident("__filter_item".to_string())))
        }
        Some(sifr_type_system::OwnershipKind::Move) | None => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__filter_item".to_string())),
            method: "clone".to_string(),
            args: vec![],
        },
    };
    let filtered_iter = RustExpr::MethodCall {
        receiver: Box::new(iter_source),
        method: "filter".to_string(),
        args: vec![RustExpr::ClosureBlock {
            params: vec![RustParam::Named {
                name: "__filter_item".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__filter_value".to_string(),
                    ty: None,
                    value: predicate_input_expr,
                },
                RustStmt::Return(Some(predicate_expr)),
            ],
            is_move: false,
            is_async: false,
        }],
    };

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![filtered_iter],
    })
}

pub(super) fn try_lower_simple_method_call_expr(
    object: &HirExpr,
    method: &str,
    args: &[HirExpr],
) -> Option<RustExpr> {
    if (method == "__sifr_spawn_infallible" || method == "__sifr_spawn_result")
        && matches!(resolve_alias_type(object.ty()), Type::Class { name, .. } if name == "TaskScope" || name == "TaskGroup")
    {
        let lowered_object = try_lower_leaf_or_name_expr(object)?;
        let lowered_args = args
            .iter()
            .map(|arg| {
                if let HirExpr::Call {
                    func,
                    args: call_args,
                    ..
                } = arg
                {
                    let lowered_call_args = call_args
                        .iter()
                        .map(try_lower_leaf_or_name_expr)
                        .collect::<Option<Vec<_>>>()?;
                    return Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Ident(func.clone())),
                        args: lowered_call_args,
                    });
                }
                try_lower_leaf_expr(arg)
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: method.to_string(),
            args: lowered_args,
        });
    }
    if (method == "join" || method == "cancel_and_join")
        && matches!(
            resolve_alias_type(object.ty()),
            Type::Task(_, _) | Type::BlockingTask(_, _)
        )
    {
        let lowered_object = try_lower_leaf_or_name_expr(object)?;
        return Some(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: method.to_string(),
            args: vec![],
        });
    }
    if method == "cancel"
        && matches!(
            resolve_alias_type(object.ty()),
            Type::Task(_, _) | Type::BlockingTask(_, _)
        )
    {
        let lowered_object = try_lower_leaf_or_name_expr(object)?;
        return Some(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: method.to_string(),
            args: vec![],
        });
    }
    if method == "__sifr_timeout" && matches!(resolve_alias_type(object.ty()), Type::Task(_, _)) {
        let lowered_object = try_lower_leaf_or_name_expr(object)?;
        let [duration] = args else {
            return None;
        };
        let lowered_args = vec![try_lower_task_duration_expr(
            duration,
            "__sifr_task_timeout_seconds",
        )?];
        return Some(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: method.to_string(),
            args: lowered_args,
        });
    }
    // Method-call lowering is ownership- and type-convention-sensitive.
    // Keep it on the structured emitter path where binding context is available.
    None
}

pub(super) fn try_lower_dict_get_key_expr(index: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::StringLiteral(value) = index {
        return Some(RustExpr::Ident(format!("{value:?}")));
    }
    Some(RustExpr::Ref {
        mutable: false,
        expr: Box::new(try_lower_leaf_or_name_expr(index)?),
    })
}

pub(super) fn try_lower_simple_constructor_call_expr(
    class_name: &str,
    args: &[HirExpr],
) -> Option<RustExpr> {
    let _ = class_name;
    let _ = args;
    None
}

pub(super) fn try_lower_simple_defaultdict_index_expr(
    object: &HirExpr,
    index: &HirExpr,
) -> Option<RustExpr> {
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = object.ty()
    else {
        return None;
    };
    if !alias_name.starts_with("__compat_defaultdict_") {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
        return None;
    };
    let lowered_object = try_lower_leaf_or_name_expr(object)?;
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    let key_arg = if let HirExpr::StringLiteral(value) = index {
        RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
    } else {
        let _ = key_ty;
        RustExpr::Clone(Box::new(lowered_index))
    };
    let default_expr = match alias_name.as_str() {
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
        _ => return None,
    };
    let entry_expr = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: "entry".to_string(),
            args: vec![key_arg],
        }),
        method: "or_insert".to_string(),
        args: vec![default_expr],
    };
    Some(match resolve_alias_type(value_ty.as_ref()) {
        Type::Int => RustExpr::Deref(Box::new(entry_expr)),
        _ => RustExpr::MethodCall {
            receiver: Box::new(entry_expr),
            method: "clone".to_string(),
            args: vec![],
        },
    })
}
