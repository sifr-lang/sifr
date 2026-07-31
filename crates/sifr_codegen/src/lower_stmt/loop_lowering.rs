use super::{
    resolve_alias_type, try_lower_leaf_or_name_expr, try_lower_loop_else_stmts,
    try_lower_simple_condition_test_expr, try_lower_simple_stmt_block, FunctionType, HirExpr,
    HirStmt, RustExpr, RustParam, RustStmt, RustType, SimpleStmtBindings, SimpleStmtLoweringCtx,
    Type,
};
pub(super) fn try_lower_simple_while_stmt(
    condition: &HirExpr,
    body: &[HirStmt],
    else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if let Some(else_body) = else_body {
        return try_lower_loop_else_stmts(
            RustStmt::While {
                cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(body, true, bindings, ctx)?,
            },
            else_body,
            in_loop_with_else,
            bindings,
            ctx,
        );
    }

    Some(vec![RustStmt::While {
        cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
        // Entering a nested while without else resets loop-else break marker context.
        body: try_lower_simple_stmt_block(body, false, bindings, ctx)?,
    }])
}

#[derive(Clone, Copy)]
pub(super) struct SimpleForStmtParts<'a> {
    pub(super) target: &'a str,
    pub(super) target_ty: &'a Type,
    pub(super) iter: &'a HirExpr,
    pub(super) body: &'a [HirStmt],
    pub(super) else_body: Option<&'a [HirStmt]>,
    pub(super) in_loop_with_else: bool,
}

pub(super) fn try_lower_simple_for_stmt(
    parts: SimpleForStmtParts<'_>,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if parts.target.contains(',') {
        return None;
    }

    if let Some(else_body) = parts.else_body {
        let iter = if crate::RustEmitter::should_lower_string_set_loop_target_as_char(
            parts.target,
            parts.target_ty,
            parts.iter,
            parts.body,
        ) {
            try_lower_simple_string_chars_for_iter_expr(parts.iter)?
        } else {
            try_lower_simple_for_iter_expr(parts.iter, parts.target_ty)?
        };
        return try_lower_loop_else_stmts(
            RustStmt::For {
                var: parts.target.to_string(),
                iter,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(parts.body, true, bindings, ctx)?,
            },
            else_body,
            parts.in_loop_with_else,
            bindings,
            ctx,
        );
    }

    let iter = if crate::RustEmitter::should_lower_string_set_loop_target_as_char(
        parts.target,
        parts.target_ty,
        parts.iter,
        parts.body,
    ) {
        try_lower_simple_string_chars_for_iter_expr(parts.iter)?
    } else {
        try_lower_simple_for_iter_expr(parts.iter, parts.target_ty)?
    };
    Some(vec![RustStmt::For {
        var: parts.target.to_string(),
        iter,
        // Entering a nested for without else resets loop-else break marker context.
        body: try_lower_simple_stmt_block(parts.body, false, bindings, ctx)?,
    }])
}

pub(super) fn try_lower_simple_string_chars_for_iter_expr(iter: &HirExpr) -> Option<RustExpr> {
    let iter_source = match iter {
        HirExpr::IteratorCall {
            op: sifr_ir::HirIteratorOp::Iter,
            args,
            ..
        } if args.len() == 1 => &args[0],
        HirExpr::Call { func, args, .. } if func == "iter" && args.len() == 1 => &args[0],
        _ => iter,
    };
    let lowered_iter = try_lower_leaf_or_name_expr(iter_source)?;
    Some(RustExpr::MethodCall {
        receiver: Box::new(lowered_iter),
        method: "chars".to_string(),
        args: vec![],
    })
}

pub(super) fn try_lower_simple_for_iter_expr(iter: &HirExpr, target_ty: &Type) -> Option<RustExpr> {
    fn is_collect_call_expr(expr: &RustExpr) -> bool {
        match expr {
            RustExpr::MethodCall { method, .. } => {
                method == "collect" || method.starts_with("collect::<")
            }
            RustExpr::Paren(inner) => is_collect_call_expr(inner),
            _ => false,
        }
    }

    fn normalize_for_iter_expr(expr: RustExpr) -> RustExpr {
        match expr {
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let normalized_receiver = Box::new(normalize_for_iter_expr(*receiver));
                let normalized_args = args
                    .into_iter()
                    .map(normalize_for_iter_expr)
                    .collect::<Vec<_>>();

                if method == "cloned"
                    && normalized_args.is_empty()
                    && is_collect_call_expr(&normalized_receiver)
                {
                    return *normalized_receiver;
                }

                RustExpr::MethodCall {
                    receiver: normalized_receiver,
                    method,
                    args: normalized_args,
                }
            }
            RustExpr::Paren(inner) => RustExpr::Paren(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Try(inner) => RustExpr::Try(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Await(inner) => RustExpr::Await(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Deref(inner) => RustExpr::Deref(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Clone(inner) => RustExpr::Clone(Box::new(normalize_for_iter_expr(*inner))),
            other => other,
        }
    }

    fn class_method_signature<'a>(
        methods: &'a [(String, FunctionType)],
        method_name: &str,
    ) -> Option<&'a FunctionType> {
        methods.iter().find_map(
            |(name, ft)| {
                if name == method_name {
                    Some(ft)
                } else {
                    None
                }
            },
        )
    }

    fn class_has_next(methods: &[(String, FunctionType)]) -> bool {
        class_method_signature(methods, "__next__").is_some_and(|next_ft| {
            next_ft.params.is_empty()
                && matches!(next_ft.return_type.as_ref().resolve_alias(), Type::Union(members) if {
                    let has_none = members
                        .iter()
                        .any(|member| matches!(member.resolve_alias(), Type::None));
                    let non_none = members
                        .iter()
                        .filter(|member| !matches!(member.resolve_alias(), Type::None))
                        .count();
                    has_none && non_none == 1
                })
        })
    }

    fn class_next_iter_expr(source_expr: RustExpr) -> RustExpr {
        let state_name = "__sifr_for_iter_state".to_string();
        RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: true,
                name: state_name.clone(),
                ty: None,
                value: source_expr,
            }],
            expr: Some(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "iter".to_string(),
                    "from_fn".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(state_name)),
                        method: "__next__".to_string(),
                        args: vec![],
                    }),
                    is_move: true,
                }],
            })),
        }
    }

    let lowered_iter_call = match iter {
        HirExpr::IteratorCall {
            op: sifr_ir::HirIteratorOp::Iter,
            args,
            ..
        } if args.len() == 1 => try_lower_leaf_or_name_expr(iter).map(normalize_for_iter_expr),
        _ => None,
    };

    let iter_source = match iter {
        HirExpr::IteratorCall {
            op: sifr_ir::HirIteratorOp::Iter,
            args,
            ..
        } if args.len() == 1 => &args[0],
        _ => iter,
    };
    if matches!(iter_source, HirExpr::ConstructorCall { .. }) {
        // Constructor-backed iteration often needs full protocol-aware lowering context.
        // Defer to structured lowering instead of emitting a potentially non-iterator source.
        return None;
    }

    let lowered_iter = try_lower_leaf_or_name_expr(iter_source)?;
    let lowered_iter = normalize_for_iter_expr(lowered_iter);
    let fallback_iter_expr = || {
        lowered_iter_call
            .clone()
            .unwrap_or_else(|| lowered_iter.clone())
    };
    let iter_plan =
        crate::helpers::plan_iterator_ownership_with_element_hint(iter_source, Some(target_ty));
    let consumes_task_handle_collection = matches!(
        resolve_alias_type(iter_source.ty()),
        Type::List(element_ty) if matches!(element_ty.resolve_alias(), Type::Task(_, _))
    );
    let apply_copy_clone_yield = |iter_expr: RustExpr| match iter_plan.yield_mode {
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
    };
    Some(match resolve_alias_type(iter_source.ty()) {
        Type::List(_) if consumes_task_handle_collection => RustExpr::MethodCall {
            receiver: Box::new(lowered_iter),
            method: "into_iter".to_string(),
            args: vec![],
        },
        Type::List(_) | Type::Set(_) | Type::Iterable(_) => match iter_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "into_iter".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Preserve => {
                apply_copy_clone_yield(RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "iter".to_string(),
                    args: vec![],
                })
            }
        },
        Type::Dict(_, _) => match iter_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "into_keys".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Preserve => {
                apply_copy_clone_yield(RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "keys".to_string(),
                    args: vec![],
                })
            }
        },
        Type::Str => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "c".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("c".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        },
        Type::Bytes => match iter_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
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
                    receiver: Box::new(lowered_iter),
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
        },
        Type::Tuple(elems) if !elems.is_empty() && elems.iter().all(|elem| elem == &elems[0]) => {
            let tuple_binding = "__sifr_tuple_iter_src".to_string();
            let tuple_items = (0..elems.len())
                .map(|index| {
                    let field_expr = RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(tuple_binding.clone())),
                        field: index.to_string(),
                    };
                    match iter_plan.yield_mode {
                        crate::helpers::YieldMode::Copy | crate::helpers::YieldMode::Move => {
                            field_expr
                        }
                        crate::helpers::YieldMode::Clone | crate::helpers::YieldMode::Borrow => {
                            RustExpr::MethodCall {
                                receiver: Box::new(field_expr),
                                method: "clone".to_string(),
                                args: vec![],
                            }
                        }
                    }
                })
                .collect();
            RustExpr::Block {
                stmts: vec![RustStmt::Let {
                    mutable: false,
                    name: tuple_binding,
                    ty: None,
                    value: match iter_plan.source_access_mode {
                        crate::helpers::SourceAccessMode::Preserve => RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Paren(Box::new(lowered_iter))),
                            method: "clone".to_string(),
                            args: vec![],
                        },
                        crate::helpers::SourceAccessMode::Consume => lowered_iter,
                    },
                }],
                expr: Some(Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Vec(tuple_items)),
                    method: "into_iter".to_string(),
                    args: vec![],
                })),
            }
        }
        Type::Class { name, methods, .. } => {
            let class_source = match iter_plan.source_access_mode {
                crate::helpers::SourceAccessMode::Preserve => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered_iter.clone()))),
                    method: "clone".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Consume => lowered_iter.clone(),
            };
            if let Some(iter_ft) = class_method_signature(methods, "__iter__") {
                if iter_ft.params.is_empty() {
                    let iter_call = RustExpr::MethodCall {
                        receiver: Box::new(class_source.clone()),
                        method: "__iter__".to_string(),
                        args: vec![],
                    };
                    if matches!(
                        iter_ft.return_type.as_ref().resolve_alias(),
                        Type::Class { name: ret_name, .. } if ret_name == name
                    ) && class_has_next(methods)
                    {
                        class_next_iter_expr(iter_call)
                    } else if let Type::Class {
                        methods: ret_methods,
                        ..
                    } = iter_ft.return_type.as_ref().resolve_alias()
                    {
                        if class_has_next(ret_methods) {
                            class_next_iter_expr(iter_call)
                        } else {
                            iter_call
                        }
                    } else {
                        iter_call
                    }
                } else if class_has_next(methods) {
                    class_next_iter_expr(class_source)
                } else {
                    fallback_iter_expr()
                }
            } else if class_has_next(methods) {
                class_next_iter_expr(class_source)
            } else {
                fallback_iter_expr()
            }
        }
        _ => fallback_iter_expr(),
    })
}
