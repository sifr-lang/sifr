use super::invalid_context;
use crate::lower::{statement_diagnostics, LowerCtx};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{HirAsyncWithKind, HirExpr, HirStmt, PythonCleanupPolicy};
use sifr_python_ast::{Expr, StmtWith};
use sifr_type_system::{FunctionType, Type};

pub(in crate::lower) struct PythonAsyncContextMetadata {
    pub manager_class: String,
    pub entered_type: Type,
    pub enter_error_type: Type,
    pub exit_error_type: Type,
    pub entered_is_opaque_borrow: bool,
}

pub(in crate::lower) fn python_async_context_metadata(
    manager_type: &Type,
    ctx: &mut LowerCtx,
    range: ruff_text_size::TextRange,
) -> Option<PythonAsyncContextMetadata> {
    let Type::Class {
        name: manager_class,
        methods,
        ..
    } = manager_type.resolve_alias()
    else {
        return None;
    };
    let cleanup = ctx
        .python_opaque_classes
        .get(manager_class)
        .and_then(|declaration| declaration.cleanup);
    if cleanup != Some(PythonCleanupPolicy::AsyncContext) {
        return None;
    }
    let Some(enter) = method(methods, "__aenter__") else {
        invalid_context(
            ctx,
            "async context manager has no declared aenter method",
            range,
        );
        return None;
    };
    let Some(exit) = method(methods, "__aexit__") else {
        invalid_context(
            ctx,
            "async context manager has no declared aexit method",
            range,
        );
        return None;
    };
    let Some((entered_type, enter_error_type)) = coroutine_result(&enter.return_type) else {
        invalid_context(
            ctx,
            "async context aenter has no coroutine Result return",
            range,
        );
        return None;
    };
    let Some((_, exit_error_type)) = coroutine_result(&exit.return_type) else {
        invalid_context(
            ctx,
            "async context aexit has no coroutine Result return",
            range,
        );
        return None;
    };
    let entered_type = match entered_type.resolve_alias() {
        Type::Class { name, .. } => ctx.class_types.get(name).cloned().unwrap_or(entered_type),
        _ => entered_type,
    };
    let entered_is_opaque_borrow = matches!(
        entered_type.resolve_alias(),
        Type::Class { name, .. } if name == manager_class
    );
    Some(PythonAsyncContextMetadata {
        manager_class: manager_class.clone(),
        entered_type,
        enter_error_type,
        exit_error_type,
        entered_is_opaque_borrow,
    })
}

fn method<'a>(methods: &'a [(String, FunctionType)], name: &str) -> Option<&'a FunctionType> {
    methods
        .iter()
        .find_map(|(candidate, function)| (candidate == name).then_some(function))
}

fn coroutine_result(ty: &Type) -> Option<(Type, Type)> {
    let Type::Coroutine(ok, error) = ty.resolve_alias() else {
        return None;
    };
    Some((ok.as_ref().clone(), error.as_ref().clone()))
}

pub(in crate::lower) fn try_lower_python_async_with(
    with_stmt: &StmtWith,
    item: &sifr_python_ast::WithItem,
    func_type: &FunctionType,
    context: &HirExpr,
    ctx: &mut LowerCtx,
) -> Option<Option<HirStmt>> {
    let context_owner = match context {
        HirExpr::Name { name, .. } => Some(name.as_str()),
        _ => None,
    };
    let direct_metadata =
        python_async_context_metadata(context.ty(), ctx, item.context_expr.range());
    let result_metadata = if direct_metadata.is_none() {
        let Type::Result(ok_type, error_type) = context.ty().resolve_alias() else {
            return None;
        };
        python_async_context_metadata(ok_type, ctx, item.context_expr.range()).map(|metadata| {
            (
                metadata,
                ok_type.as_ref().clone(),
                error_type.as_ref().clone(),
            )
        })
    } else {
        None
    };
    let (metadata, context) = if let Some(metadata) = direct_metadata {
        (metadata, context.clone())
    } else if let Some((metadata, manager_type, construction_error)) = result_metadata {
        if !ctx.in_try_block {
            ctx.error_with_code_at(
                DiagnosticCode::PYCTX_INVALID_DECLARATION,
                "invalid Python context declaration: fallible Python async context construction requires an enclosing try block".to_string(),
                item.context_expr.range(),
            );
            return Some(None);
        }
        if !return_type_accepts_error(&func_type.return_type, &construction_error) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "fallible Python async context construction requires a compatible enclosing Result error type".to_string(),
                item.context_expr.range(),
            );
            return Some(None);
        }
        (
            metadata,
            HirExpr::QuestionMark {
                expr: Box::new(context.clone()),
                ty: manager_type,
            },
        )
    } else {
        return None;
    };
    Some(lower_python_async_with(
        with_stmt,
        item,
        func_type,
        context,
        context_owner,
        metadata,
        ctx,
    ))
}

#[allow(clippy::too_many_arguments)]
fn lower_python_async_with(
    with_stmt: &StmtWith,
    item: &sifr_python_ast::WithItem,
    func_type: &FunctionType,
    context: HirExpr,
    context_owner: Option<&str>,
    metadata: PythonAsyncContextMetadata,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    if !return_type_accepts_error(&func_type.return_type, &metadata.enter_error_type)
        || !return_type_accepts_error(&func_type.return_type, &metadata.exit_error_type)
    {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "fallible Python async context enter/exit requires a compatible enclosing Result error type".to_string(),
            item.context_expr.range(),
        );
        return None;
    }
    let Type::Result(_, active_error_type) = func_type.return_type.resolve_alias() else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "Python async context requires an enclosing Result return type".to_string(),
            item.context_expr.range(),
        );
        return None;
    };
    let target = simple_target(item.optional_vars.as_deref(), ctx);
    if let Some(owner) = context_owner {
        ctx.mark_moved_with_flow(owner);
    }
    let mut previous_borrow = None;
    if let Some(name) = &target {
        ctx.scope
            .define(name.clone(), metadata.entered_type.clone());
        if metadata.entered_is_opaque_borrow {
            previous_borrow = Some((
                name.clone(),
                ctx.python_context_borrows
                    .insert(name.clone(), item.context_expr.range()),
            ));
        }
    }
    let body = crate::lower::statements::lower_stmts(&with_stmt.body, func_type, ctx);
    if let Some((name, previous)) = previous_borrow {
        if let Some(range) = previous {
            ctx.python_context_borrows.insert(name, range);
        } else {
            ctx.python_context_borrows.remove(&name);
        }
    }
    Some(HirStmt::AsyncWith {
        kind: HirAsyncWithKind::Python {
            context,
            manager_class: metadata.manager_class,
            entered_type: metadata.entered_type,
            enter_error_type: metadata.enter_error_type,
            exit_error_type: metadata.exit_error_type,
            entered_is_opaque_borrow: metadata.entered_is_opaque_borrow,
            active_error_type: active_error_type.as_ref().clone(),
        },
        target,
        body,
    })
}

fn simple_target(optional_vars: Option<&Expr>, ctx: &mut LowerCtx) -> Option<String> {
    let vars = optional_vars?;
    if let Expr::Name(name) = vars {
        Some(name.id.to_string())
    } else {
        statement_diagnostics::unsupported_form(
            ctx,
            "with target must be a simple name",
            vars.range(),
        );
        None
    }
}

fn return_type_accepts_error(return_type: &Type, error_type: &Type) -> bool {
    let Type::Result(_, active_error) = return_type.resolve_alias() else {
        return false;
    };
    error_type.is_assignable_to(active_error)
}
