macro_rules! stmt_expr_await_and_registry {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::Await { value, .. } = $expr {
            if let Some(duration) = $emitter.active_timeout_durations.last().cloned() {
                let Some(future) = $emitter.lower_timeout_aware_await_future_for_ir(value)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::TimeoutAwait {
                    duration: Box::new(duration),
                    future: Box::new(future),
                    error: Box::new($emitter.timeout_error_for_ir()),
                }));
            }
            if let HirExpr::Call { func, args, .. } = value.as_ref() {
                if func == "__sifr_task_sleep" {
                    let [duration] = args.as_slice() else {
                        return Ok(None);
                    };
                    let Some(duration_expr) =
                        crate::try_lower_task_duration_expr(duration, "__sifr_task_sleep_seconds")
                    else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::Await(Box::new(
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "tokio".to_string(),
                                "time".to_string(),
                                "sleep".to_string(),
                            ])),
                            args: vec![duration_expr],
                        },
                    ))));
                }
            }
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            let awaited_value = match crate::resolve_alias_type_for_plain_call(value.ty()) {
                Type::Task(_, _) | Type::BlockingTask(_, _) => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_value),
                    method: "join".to_string(),
                    args: vec![],
                },
                _ => lowered_value,
            };
            return Ok(Some(crate::RustExpr::Await(Box::new(awaited_value))));
        }

        let skip_leaf_registry_lowering = matches!(
            $expr,
            HirExpr::Call { .. }
                | HirExpr::PythonCall { .. }
                | HirExpr::IteratorCall { .. }
                | HirExpr::ConstructorCall { .. }
                | HirExpr::MethodCall { .. }
                | HirExpr::BinOp { .. }
                | HirExpr::UnaryOp { .. }
                | HirExpr::Compare { .. }
                | HirExpr::BoolOp { .. }
                | HirExpr::Slice { .. }
                | HirExpr::ListLiteral { .. }
                | HirExpr::TupleLiteral { .. }
                | HirExpr::DictLiteral { .. }
                | HirExpr::SetLiteral { .. }
                | HirExpr::OkWrap { .. }
                | HirExpr::ErrWrap { .. }
        );
        if !skip_leaf_registry_lowering {
            if let Some(lowered) = $emitter.try_lower_registry_expr_result($expr)? {
                return Ok(Some(lowered));
            }
        }
        if let HirExpr::Call { func, args, .. } = $expr {
            if func == "print" {
                return $emitter.lower_print_call_expr_for_ir(args);
            }
        }
        if let HirExpr::FieldAccess { object, field, ty } = $expr {
            if let Some(lowered) =
                $emitter.try_lower_structured_field_access_expr(object, field, ty)?
            {
                return Ok(Some(lowered));
            }
        }
    }};
}
