macro_rules! stmt_expr_unit_slice {
    ($emitter:ident, $object:ident, $start:ident, $stop:ident, $lowered_object:ident) => {{
        let normalize_bound = |raw_opt: Option<crate::RustExpr>, default_value: crate::RustExpr| {
            let Some(raw) = raw_opt else {
                return default_value;
            };
            crate::RustExpr::MethodCall {
                receiver: Box::new(raw),
                method: "clamp_slice_bound".to_string(),
                args: vec![crate::RustExpr::Ident("__sifr_slice_len".to_string())],
            }
        };

        match crate::resolve_alias_type_for_plain_call($object.ty()) {
            Type::Str => {
                let cached_chars = $emitter.string_char_cache_for_expr($object);
                let cache_override = if cached_chars.is_none() {
                    if let HirExpr::Name { name, .. } = $object.as_ref() {
                        Some((
                            name.clone(),
                            $emitter
                                .string_char_cache_vars
                                .insert(name.clone(), "__sifr_slice_src".to_string()),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let lowered_bounds = (|| -> Result<_, crate::CodegenError> {
                    let lowered_start_raw = if let Some(start_expr) = $start {
                        let Some(start_lowered) = $emitter.lower_stmt_expr_for_ir(start_expr)?
                        else {
                            return Ok(None);
                        };
                        Some(start_lowered)
                    } else {
                        None
                    };
                    let lowered_stop_raw = if let Some(stop_expr) = $stop {
                        let Some(stop_lowered) = $emitter.lower_stmt_expr_for_ir(stop_expr)? else {
                            return Ok(None);
                        };
                        Some(stop_lowered)
                    } else {
                        None
                    };
                    Ok(Some((lowered_start_raw, lowered_stop_raw)))
                })();
                if let Some((name, previous)) = cache_override {
                    if let Some(previous) = previous {
                        $emitter.string_char_cache_vars.insert(name, previous);
                    } else {
                        $emitter.string_char_cache_vars.remove(&name);
                    }
                }
                let Some((lowered_start_raw, lowered_stop_raw)) = lowered_bounds? else {
                    return Ok(None);
                };
                let start_bound = normalize_bound(
                    lowered_start_raw,
                    crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                );
                let stop_bound = normalize_bound(
                    lowered_stop_raw,
                    crate::RustExpr::Ident("__sifr_slice_len".to_string()),
                );
                let take_count = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__sifr_slice_stop".to_string())),
                    method: "saturating_sub".to_string(),
                    args: vec![crate::RustExpr::Ident("__sifr_slice_start".to_string())],
                };
                let slice_src_value = if let Some(cache_name) = cached_chars.as_ref() {
                    crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Ident(cache_name.clone())),
                    }
                } else {
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new($lowered_object),
                            method: "chars".to_string(),
                            args: vec![],
                        }),
                        method: "collect::<Vec<char>>".to_string(),
                        args: vec![],
                    }
                };
                let slice_len_value = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__sifr_slice_src".to_string())),
                    method: "len".to_string(),
                    args: vec![],
                };
                let slice_iter_source = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__sifr_slice_src".to_string())),
                    method: "iter".to_string(),
                    args: vec![],
                };
                let iter = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(slice_iter_source),
                        method: "skip".to_string(),
                        args: vec![crate::RustExpr::Ident("__sifr_slice_start".to_string())],
                    }),
                    method: "take".to_string(),
                    args: vec![take_count],
                };
                let iter = crate::RustExpr::MethodCall {
                    receiver: Box::new(iter),
                    method: "copied".to_string(),
                    args: vec![],
                };
                let slice_expr = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "String".to_string(),
                        "from_iter".to_string(),
                    ])),
                    args: vec![iter],
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_src".to_string(),
                            ty: None,
                            value: slice_src_value,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_len".to_string(),
                            ty: None,
                            value: slice_len_value,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_start".to_string(),
                            ty: None,
                            value: start_bound,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_stop".to_string(),
                            ty: None,
                            value: stop_bound,
                        },
                    ],
                    expr: Some(Box::new(slice_expr)),
                }));
            }
            Type::List(_) | Type::Bytes => {
                let lowered_start_raw = if let Some(start_expr) = $start {
                    let Some(start_lowered) = $emitter.lower_stmt_expr_for_ir(start_expr)? else {
                        return Ok(None);
                    };
                    Some(start_lowered)
                } else {
                    None
                };
                let lowered_stop_raw = if let Some(stop_expr) = $stop {
                    let Some(stop_lowered) = $emitter.lower_stmt_expr_for_ir(stop_expr)? else {
                        return Ok(None);
                    };
                    Some(stop_lowered)
                } else {
                    None
                };
                let start_bound = normalize_bound(
                    lowered_start_raw,
                    crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                );
                let stop_bound = normalize_bound(
                    lowered_stop_raw,
                    crate::RustExpr::Ident("__sifr_slice_len".to_string()),
                );
                let take_count = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__sifr_slice_stop".to_string())),
                    method: "saturating_sub".to_string(),
                    args: vec![crate::RustExpr::Ident("__sifr_slice_start".to_string())],
                };
                let iter = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__sifr_slice_src".to_string(),
                                )),
                                method: "iter".to_string(),
                                args: vec![],
                            }),
                            method: "skip".to_string(),
                            args: vec![crate::RustExpr::Ident("__sifr_slice_start".to_string())],
                        }),
                        method: "take".to_string(),
                        args: vec![take_count],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                };
                let slice_expr = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Vec".to_string(),
                        "from_iter".to_string(),
                    ])),
                    args: vec![iter],
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_src".to_string(),
                            ty: None,
                            value: crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new($lowered_object),
                            },
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_len".to_string(),
                            ty: None,
                            value: crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__sifr_slice_src".to_string(),
                                )),
                                method: "len".to_string(),
                                args: vec![],
                            },
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_start".to_string(),
                            ty: None,
                            value: start_bound,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_slice_stop".to_string(),
                            ty: None,
                            value: stop_bound,
                        },
                    ],
                    expr: Some(Box::new(slice_expr)),
                }));
            }
            _ => return Ok(None),
        }
    }};
}
