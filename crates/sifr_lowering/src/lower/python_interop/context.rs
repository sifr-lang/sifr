use super::{
    is_direct_type, parameter_metadata, parse_method_target_path, receiver_is_owned, LowerCtx,
};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirExpr, HirFStringPart, HirFunction, HirParam, HirWithItemKind, PythonCleanupPolicy,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
};
use sifr_python_ast::{Expr, ExprCall, Parameters};
use sifr_type_system::Type;

pub(in crate::lower) fn python_context_borrow_in_owned_expr(
    expr: &HirExpr,
    ctx: &LowerCtx,
) -> Option<String> {
    match expr {
        HirExpr::Name { name, .. } if ctx.python_context_borrows.contains_key(name) => {
            Some(name.clone())
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. }
        | HirExpr::ConstructorCall { args: elements, .. } => elements
            .iter()
            .find_map(|element| python_context_borrow_in_owned_expr(element, ctx)),
        HirExpr::DictLiteral { keys, values, .. } => keys
            .iter()
            .chain(values)
            .find_map(|element| python_context_borrow_in_owned_expr(element, ctx)),
        HirExpr::QuestionMark { expr, .. }
        | HirExpr::OkWrap { value: expr, .. }
        | HirExpr::ErrWrap { value: expr, .. } => python_context_borrow_in_owned_expr(expr, ctx),
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => python_context_borrow_in_owned_expr(then_expr, ctx)
            .or_else(|| python_context_borrow_in_owned_expr(else_expr, ctx)),
        HirExpr::WalrusExpr { value, .. } => python_context_borrow_in_owned_expr(value, ctx),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => python_context_borrow_in_owned_expr(expr, ctx).or_else(|| {
            generators.iter().find_map(|(_, iter, filter)| {
                python_context_borrow_reference(iter, ctx).or_else(|| {
                    filter
                        .as_ref()
                        .and_then(|filter| python_context_borrow_reference(filter, ctx))
                })
            })
        }),
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => python_context_borrow_in_owned_expr(key_expr, ctx)
            .or_else(|| python_context_borrow_in_owned_expr(val_expr, ctx))
            .or_else(|| {
                generators.iter().find_map(|(_, iter, filter)| {
                    python_context_borrow_reference(iter, ctx).or_else(|| {
                        filter
                            .as_ref()
                            .and_then(|filter| python_context_borrow_reference(filter, ctx))
                    })
                })
            }),
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => python_context_borrow_reference(expr, ctx)
            .or_else(|| python_context_borrow_reference(iter, ctx))
            .or_else(|| {
                filter
                    .as_deref()
                    .and_then(|filter| python_context_borrow_reference(filter, ctx))
            }),
        HirExpr::Lambda { body, .. } => python_context_borrow_reference(body, ctx),
        HirExpr::Call { args, ty, .. }
        | HirExpr::PythonCall { args, ty, .. }
        | HirExpr::IntrinsicCall { args, ty, .. }
        | HirExpr::IteratorCall { args, ty, .. }
        | HirExpr::SuperCall { args, ty, .. } => {
            python_context_borrow_in_call_args(args, type_can_hold_python_opaque(ty, ctx), ctx)
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ty,
        } => python_context_borrow_in_call_args(
            args,
            type_can_hold_python_opaque(ty, ctx) || method_stores_arguments(method),
            ctx,
        )
        .or_else(|| {
            (!matches!(object.as_ref(), HirExpr::Name { .. })
                || type_can_hold_python_opaque(ty, ctx))
            .then(|| python_context_borrow_in_owned_expr(object, ctx))
            .flatten()
        }),
        HirExpr::Index { object, ty, .. } | HirExpr::FieldAccess { object, ty, .. }
            if type_can_hold_python_opaque(ty, ctx) =>
        {
            python_context_borrow_in_owned_expr(object, ctx)
        }
        HirExpr::Await { value, ty } if type_can_hold_python_opaque(ty, ctx) => {
            python_context_borrow_in_owned_expr(value, ctx)
        }
        _ => None,
    }
}

fn python_context_borrow_in_call_args(
    args: &[HirExpr],
    inspect_direct_names: bool,
    ctx: &LowerCtx,
) -> Option<String> {
    args.iter().find_map(|argument| {
        (inspect_direct_names || !matches!(argument, HirExpr::Name { .. }))
            .then(|| python_context_borrow_in_owned_expr(argument, ctx))
            .flatten()
    })
}

fn method_stores_arguments(method: &str) -> bool {
    matches!(
        method,
        "append" | "insert" | "extend" | "add" | "update" | "setdefault"
    )
}

pub(in crate::lower) fn reject_python_context_borrow_created_value(
    value: &HirExpr,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    let creates_owned_value = match value {
        HirExpr::ListLiteral { .. }
        | HirExpr::SetLiteral { .. }
        | HirExpr::DictLiteral { .. }
        | HirExpr::TupleLiteral { .. }
        | HirExpr::ConstructorCall { .. }
        | HirExpr::WalrusExpr { .. }
        | HirExpr::Lambda { .. }
        | HirExpr::ListComp { .. }
        | HirExpr::SetComp { .. }
        | HirExpr::DictComp { .. }
        | HirExpr::GeneratorExpr { .. } => true,
        HirExpr::Call { ty, .. }
        | HirExpr::PythonCall { ty, .. }
        | HirExpr::IntrinsicCall { ty, .. }
        | HirExpr::IteratorCall { ty, .. }
        | HirExpr::SuperCall { ty, .. }
        | HirExpr::Index { ty, .. }
        | HirExpr::FieldAccess { ty, .. }
        | HirExpr::Await { ty, .. } => type_can_hold_python_opaque(ty, ctx),
        HirExpr::MethodCall { method, ty, .. } => {
            method_stores_arguments(method) || type_can_hold_python_opaque(ty, ctx)
        }
        _ => false,
    };
    if creates_owned_value {
        reject_python_context_borrow_storage(value, range, ctx);
    }
}

pub(in crate::lower) fn reject_python_context_borrow_storage(
    value: &HirExpr,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if let Some(borrowed) = python_context_borrow_in_owned_expr(value, ctx) {
        ctx.error_with_code_at(
            DiagnosticCode::PYCTX_INVALID_DECLARATION,
            format!(
                "invalid Python context declaration: entered binding '{borrowed}' is a context-scoped borrow and cannot be stored outside its context binding"
            ),
            range,
        );
    }
}

pub(in crate::lower) fn lower_python_context_owned_expr(
    expr: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let value = super::super::expressions::lower_expr(expr, ctx)?;
    reject_python_context_borrow_storage(&value, expr.range(), ctx);
    Some(value)
}

pub(in crate::lower) fn reject_python_context_borrow_discard(
    value: &HirExpr,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if let Some(borrowed) = python_context_borrow_in_owned_expr(value, ctx) {
        ctx.error_with_code_at(
            DiagnosticCode::PYCTX_INVALID_DECLARATION,
            format!(
                "invalid Python context declaration: entered binding '{borrowed}' is a context-scoped borrow and cannot be discarded"
            ),
            range,
        );
    }
}

fn python_context_borrow_reference(expr: &HirExpr, ctx: &LowerCtx) -> Option<String> {
    if let HirExpr::Name { name, .. } = expr {
        return ctx
            .python_context_borrows
            .contains_key(name)
            .then(|| name.clone());
    }
    let in_exprs = |expressions: &[HirExpr]| {
        expressions
            .iter()
            .find_map(|expression| python_context_borrow_reference(expression, ctx))
    };
    match expr {
        HirExpr::BinOp { left, right, .. } => python_context_borrow_reference(left, ctx)
            .or_else(|| python_context_borrow_reference(right, ctx)),
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => python_context_borrow_reference(operand, ctx),
        HirExpr::Compare {
            left, comparators, ..
        } => python_context_borrow_reference(left, ctx).or_else(|| in_exprs(comparators)),
        HirExpr::BoolOp { values, .. }
        | HirExpr::Call { args: values, .. }
        | HirExpr::PythonCall { args: values, .. }
        | HirExpr::IntrinsicCall { args: values, .. }
        | HirExpr::IteratorCall { args: values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::ConstructorCall { args: values, .. }
        | HirExpr::SuperCall { args: values, .. } => in_exprs(values),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => python_context_borrow_reference(condition, ctx)
            .or_else(|| python_context_borrow_reference(then_expr, ctx))
            .or_else(|| python_context_borrow_reference(else_expr, ctx)),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => python_context_borrow_reference(start, ctx)
            .or_else(|| python_context_borrow_reference(end, ctx))
            .or_else(|| {
                step.as_deref()
                    .and_then(|step| python_context_borrow_reference(step, ctx))
            }),
        HirExpr::DictLiteral { keys, values, .. } => in_exprs(keys).or_else(|| in_exprs(values)),
        HirExpr::Index { object, index, .. } => python_context_borrow_reference(object, ctx)
            .or_else(|| python_context_borrow_reference(index, ctx)),
        HirExpr::MethodCall { object, args, .. } => {
            python_context_borrow_reference(object, ctx).or_else(|| in_exprs(args))
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => python_context_borrow_reference(element, ctx)
            .or_else(|| python_context_borrow_reference(collection, ctx)),
        HirExpr::FString { parts, .. } => parts.iter().find_map(|part| match part {
            HirFStringPart::Literal(_) => None,
            HirFStringPart::Expr(expression) => python_context_borrow_reference(expression, ctx),
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => python_context_borrow_reference(object, ctx)
            .or_else(|| {
                start
                    .as_deref()
                    .and_then(|value| python_context_borrow_reference(value, ctx))
            })
            .or_else(|| {
                stop.as_deref()
                    .and_then(|value| python_context_borrow_reference(value, ctx))
            })
            .or_else(|| {
                step.as_deref()
                    .and_then(|value| python_context_borrow_reference(value, ctx))
            }),
        HirExpr::Lambda { body, .. } => python_context_borrow_reference(body, ctx),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => python_context_borrow_reference(expr, ctx).or_else(|| {
            generators.iter().find_map(|(_, iter, filter)| {
                python_context_borrow_reference(iter, ctx).or_else(|| {
                    filter
                        .as_ref()
                        .and_then(|filter| python_context_borrow_reference(filter, ctx))
                })
            })
        }),
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => python_context_borrow_reference(key_expr, ctx)
            .or_else(|| python_context_borrow_reference(val_expr, ctx))
            .or_else(|| {
                generators.iter().find_map(|(_, iter, filter)| {
                    python_context_borrow_reference(iter, ctx).or_else(|| {
                        filter
                            .as_ref()
                            .and_then(|filter| python_context_borrow_reference(filter, ctx))
                    })
                })
            }),
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => python_context_borrow_reference(expr, ctx)
            .or_else(|| python_context_borrow_reference(iter, ctx))
            .or_else(|| {
                filter
                    .as_deref()
                    .and_then(|filter| python_context_borrow_reference(filter, ctx))
            }),
        _ => None,
    }
}

fn type_can_hold_python_opaque(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, .. } => ctx.python_opaque_classes.contains_key(name),
        Type::List(item)
        | Type::Set(item)
        | Type::Iterable(item)
        | Type::Iterator(item)
        | Type::Awaitable(item)
        | Type::Result(item, _) => type_can_hold_python_opaque(item, ctx),
        Type::Coroutine(ok, error)
        | Type::Task(ok, error)
        | Type::TaskResult(ok, error)
        | Type::AsyncIterator(ok, error)
        | Type::AsyncGenerator(ok, error) => {
            type_can_hold_python_opaque(ok, ctx) || type_can_hold_python_opaque(error, ctx)
        }
        Type::Tuple(items) | Type::Union(items) => items
            .iter()
            .any(|item| type_can_hold_python_opaque(item, ctx)),
        Type::Dict(key, value) => {
            type_can_hold_python_opaque(key, ctx) || type_can_hold_python_opaque(value, ctx)
        }
        _ => false,
    }
}

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
