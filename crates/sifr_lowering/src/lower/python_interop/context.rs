use super::{
    is_direct_type, parameter_metadata, parse_method_target_path, receiver_is_owned, LowerCtx,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirFunction, HirParam, PythonInteropDeclaration, PythonInteropDecoratorKind,
    PythonInteropEffect,
};
use sifr_python_ast::{ExprCall, Parameters};
use sifr_type_system::Type;

pub(super) fn parse_context_method(
    kind: PythonInteropDecoratorKind,
    call: &ExprCall,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    let (label, member, consumes_receiver) = match kind {
        PythonInteropDecoratorKind::ContextEnter => ("python.context.enter", "__enter__", false),
        PythonInteropDecoratorKind::ContextExit => ("python.context.exit", "__exit__", true),
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
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver,
        parameters: parameter_metadata(parameters).into_iter().skip(1).collect(),
        required_import_root: None,
    })
}

pub(super) fn validate_context_method_signature(
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
        PythonInteropDecoratorKind::ContextEnter => {
            if !params.is_empty() {
                invalid_context(
                    ctx,
                    "`@python.context.enter` takes no parameters after the receiver",
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
        PythonInteropDecoratorKind::ContextExit => {
            if params.len() != 1
                || !matches!(params[0].ty.resolve_alias(), Type::Class { name, .. } if name == "ExitCause")
            {
                invalid_context(
                    ctx,
                    "`@python.context.exit` requires exactly one `ExitCause` parameter after `own self`",
                    declaration.span,
                );
            }
            if !matches!(ok_type.resolve_alias(), Type::Class { name, .. } if name == "ExitDecision")
            {
                invalid_context(
                    ctx,
                    "`@python.context.exit` must return `Result[ExitDecision, PythonError]`",
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
    let enter_methods = methods
        .iter()
        .filter(|method| {
            method.python_interop.first().is_some_and(|declaration| {
                declaration.kind == PythonInteropDecoratorKind::ContextEnter
            })
        })
        .collect::<Vec<_>>();
    let exit_methods = methods
        .iter()
        .filter(|method| {
            method.python_interop.first().is_some_and(|declaration| {
                declaration.kind == PythonInteropDecoratorKind::ContextExit
            })
        })
        .collect::<Vec<_>>();
    let has_context_method = !enter_methods.is_empty() || !exit_methods.is_empty();
    if cleanup != Some(sifr_ir::PythonCleanupPolicy::Context) {
        if has_context_method {
            invalid_context(
                ctx,
                "context protocol decorators require an enclosing `@python.opaque(cleanup=context)` class",
                range,
            );
        }
        return;
    }
    if enter_methods.len() != 1 || exit_methods.len() != 1 {
        invalid_context(
            ctx,
            "`cleanup=context` requires exactly one context enter and one consuming context exit declaration",
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

fn invalid_context(ctx: &mut LowerCtx, reason: &str, span: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCTX_INVALID_DECLARATION,
        format!("invalid Python context declaration: {reason}"),
        span,
    );
}
