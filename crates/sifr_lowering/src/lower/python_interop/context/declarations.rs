use super::{
    invalid_context, is_direct_type, parameter_metadata, parse_method_target_path,
    receiver_is_owned, ExprCall, HirFunction, HirParam, HirWithItemKind, LowerCtx, Parameters,
    PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    Type,
};

pub(in crate::lower) fn python_context_item_kind(
    manager_type: &Type,
    ctx: &mut LowerCtx,
    range: ruff_text_size::TextRange,
) -> Option<HirWithItemKind> {
    let Type::Class { name, methods, .. } = manager_type.resolve_alias() else {
        return None;
    };
    let cleanup = ctx
        .python_opaque_classes
        .get(name)
        .and_then(|declaration| declaration.cleanup);
    if cleanup != Some(PythonCleanupPolicy::Context) {
        return None;
    }
    let Some(enter) = methods
        .iter()
        .find(|(method_name, _)| method_name == "__enter__")
        .map(|(_, function)| function)
    else {
        invalid_context(ctx, "context manager has no declared enter method", range);
        return None;
    };
    let Some(exit) = methods
        .iter()
        .find(|(method_name, _)| method_name == "__exit__")
        .map(|(_, function)| function)
    else {
        invalid_context(ctx, "context manager has no declared exit method", range);
        return None;
    };
    let Type::Result(entered_type, enter_error_type) = enter.return_type.resolve_alias() else {
        invalid_context(ctx, "context enter declaration has no Result return", range);
        return None;
    };
    let Type::Result(_, exit_error_type) = exit.return_type.resolve_alias() else {
        invalid_context(ctx, "context exit declaration has no Result return", range);
        return None;
    };
    let entered_type = if let Type::Class {
        name: entered_name, ..
    } = entered_type.resolve_alias()
    {
        ctx.class_types
            .get(entered_name)
            .cloned()
            .unwrap_or_else(|| entered_type.as_ref().clone())
    } else {
        entered_type.as_ref().clone()
    };
    let entered_is_opaque_borrow = matches!(entered_type.resolve_alias(), Type::Class { name, .. } if ctx.python_opaque_classes.contains_key(name));
    Some(HirWithItemKind::Python {
        entered_type,
        enter_error_type: enter_error_type.as_ref().clone(),
        exit_error_type: exit_error_type.as_ref().clone(),
        entered_is_opaque_borrow,
    })
}

pub(in crate::lower) fn parse_context_method(
    kind: PythonInteropDecoratorKind,
    call: &ExprCall,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    let (label, member, consumes_receiver, effect) = match kind {
        PythonInteropDecoratorKind::ContextEnter => (
            "python.context.enter",
            "__enter__",
            false,
            PythonInteropEffect::BlockingIo,
        ),
        PythonInteropDecoratorKind::ContextExit => (
            "python.context.exit",
            "__exit__",
            true,
            PythonInteropEffect::BlockingIo,
        ),
        PythonInteropDecoratorKind::ContextAsyncEnter => (
            "python.context.aenter",
            "__aenter__",
            false,
            PythonInteropEffect::Async,
        ),
        PythonInteropDecoratorKind::ContextAsyncExit => (
            "python.context.aexit",
            "__aexit__",
            true,
            PythonInteropEffect::Async,
        ),
        _ => return None,
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        invalid_context(
            ctx,
            &format!("`@{label}(Self.{member})` requires exactly one target"),
            call.range,
        );
        return None;
    }
    let target = parse_method_target_path(&call.arguments.args[0], ctx)?;
    if target.segments.as_slice() != ["Self", member] {
        invalid_context(
            ctx,
            &format!("`@{label}` target must be `Self.{member}`"),
            target.span,
        );
        return None;
    }
    let receiver_owned = receiver_is_owned(parameters);
    if receiver_owned != consumes_receiver {
        let expected = if consumes_receiver {
            "own self"
        } else {
            "self"
        };
        invalid_context(
            ctx,
            &format!("`@{label}` requires receiver `{expected}`"),
            call.range,
        );
        return None;
    }
    Some(PythonInteropDeclaration {
        kind,
        target: Some(target),
        span: call.range,
        effect,
        cleanup: None,
        consumes_receiver,
        parameters: parameter_metadata(parameters).into_iter().skip(1).collect(),
        required_import_root: None,
    })
}

pub(in crate::lower) fn validate_context_method_signature(
    declaration: &PythonInteropDeclaration,
    params: &[HirParam],
    return_type: &Type,
    ctx: &mut LowerCtx,
) {
    let Type::Result(ok_type, error_type) = return_type.resolve_alias() else {
        invalid_context(
            ctx,
            "context protocol declarations must return `Result[..., PythonError]`",
            declaration.span,
        );
        return;
    };
    if !matches!(error_type.resolve_alias(), Type::Class { name, .. } if name == "PythonError") {
        invalid_context(
            ctx,
            "context protocol declarations must use `PythonError` as their error type",
            declaration.span,
        );
    }
    match declaration.kind {
        PythonInteropDecoratorKind::ContextEnter
        | PythonInteropDecoratorKind::ContextAsyncEnter => {
            let label = if declaration.kind == PythonInteropDecoratorKind::ContextAsyncEnter {
                "python.context.aenter"
            } else {
                "python.context.enter"
            };
            if !params.is_empty() {
                invalid_context(
                    ctx,
                    &format!("`@{label}` takes no parameters after the receiver"),
                    declaration.span,
                );
            }
            if !is_direct_type(ok_type, true, ctx) {
                invalid_context(
                    ctx,
                    &format!(
                        "context entry result `{}` has no direct Python conversion",
                        ok_type.display_name()
                    ),
                    declaration.span,
                );
            }
        }
        PythonInteropDecoratorKind::ContextExit | PythonInteropDecoratorKind::ContextAsyncExit => {
            let label = if declaration.kind == PythonInteropDecoratorKind::ContextAsyncExit {
                "python.context.aexit"
            } else {
                "python.context.exit"
            };
            if params.len() != 1
                || !matches!(params[0].ty.resolve_alias(), Type::Class { name, .. } if name == "ExitCause")
            {
                invalid_context(
                    ctx,
                    &format!(
                        "`@{label}` requires exactly one `ExitCause` parameter after `own self`"
                    ),
                    declaration.span,
                );
            }
            if !matches!(ok_type.resolve_alias(), Type::Class { name, .. } if name == "ExitDecision")
            {
                invalid_context(
                    ctx,
                    &format!("`@{label}` must return `Result[ExitDecision, PythonError]`"),
                    declaration.span,
                );
            }
        }
        _ => {}
    }
}

pub(in crate::lower) fn validate_context_class_methods(
    class_name: &str,
    methods: &[HirFunction],
    cleanup: Option<sifr_ir::PythonCleanupPolicy>,
    ctx: &mut LowerCtx,
    range: ruff_text_size::TextRange,
) {
    let expected_cleanup = match cleanup {
        Some(PythonCleanupPolicy::Context) => Some((
            PythonInteropDecoratorKind::ContextEnter,
            PythonInteropDecoratorKind::ContextExit,
            "context",
        )),
        Some(PythonCleanupPolicy::AsyncContext) => Some((
            PythonInteropDecoratorKind::ContextAsyncEnter,
            PythonInteropDecoratorKind::ContextAsyncExit,
            "async_context",
        )),
        _ => None,
    };
    let protocol_methods = methods
        .iter()
        .filter(|method| {
            method.python_interop.first().is_some_and(|declaration| {
                matches!(
                    declaration.kind,
                    PythonInteropDecoratorKind::ContextEnter
                        | PythonInteropDecoratorKind::ContextExit
                        | PythonInteropDecoratorKind::ContextAsyncEnter
                        | PythonInteropDecoratorKind::ContextAsyncExit
                )
            })
        })
        .collect::<Vec<_>>();
    let Some((enter_kind, exit_kind, cleanup_label)) = expected_cleanup else {
        if !protocol_methods.is_empty() {
            invalid_context(
                ctx,
                "context protocol decorators require a matching `cleanup=context` or `cleanup=async_context` opaque class",
                range,
            );
        }
        return;
    };
    let enter_methods = protocol_methods
        .iter()
        .copied()
        .filter(|method| {
            method
                .python_interop
                .first()
                .is_some_and(|declaration| declaration.kind == enter_kind)
        })
        .collect::<Vec<_>>();
    let exit_methods = methods
        .iter()
        .filter(|method| {
            method
                .python_interop
                .first()
                .is_some_and(|declaration| declaration.kind == exit_kind)
        })
        .collect::<Vec<_>>();
    if protocol_methods.len() != enter_methods.len() + exit_methods.len() {
        invalid_context(
            ctx,
            &format!(
                "`cleanup={cleanup_label}` does not accept the other context protocol's decorators"
            ),
            range,
        );
        return;
    }
    if enter_methods.len() != 1 || exit_methods.len() != 1 {
        invalid_context(
            ctx,
            &format!("`cleanup={cleanup_label}` requires exactly one matching context enter and one consuming context exit declaration"),
            range,
        );
        return;
    }
    let Type::Result(entered_type, _) = enter_methods[0].return_type.resolve_alias() else {
        return;
    };
    let Type::Class {
        name: entered_name, ..
    } = entered_type.resolve_alias()
    else {
        if let Some(obligation) = ctx.must_use_obligation_for_type(entered_type) {
            invalid_context(
                ctx,
                &format!(
                    "context entry result `{}` contains {obligation}; entered aggregates cannot hide semantic cleanup obligations",
                    entered_type.display_name()
                ),
                enter_methods[0]
                    .python_interop
                    .first()
                    .map_or(range, |declaration| declaration.span),
            );
        }
        return;
    };
    if entered_name == class_name {
        return;
    }
    let entered_cleanup = ctx
        .python_opaque_classes
        .get(entered_name)
        .and_then(|declaration| declaration.cleanup);
    if entered_cleanup.is_some() && entered_cleanup != Some(sifr_ir::PythonCleanupPolicy::Drop) {
        invalid_context(
            ctx,
            &format!(
                "context entry cannot return distinct opaque `{entered_name}` with cleanup policy `{entered_cleanup:?}`; only the manager identity or `cleanup=drop` is allowed"
            ),
            enter_methods[0]
                .python_interop
                .first()
                .map_or(range, |declaration| declaration.span),
        );
    }
}
