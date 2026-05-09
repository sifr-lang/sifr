use super::expressions::lower_expr;
use super::statement_diagnostics;
use super::statements::lower_stmts;
use super::LowerCtx;
use crate::hir_nodes::{HirAsyncWithKind, HirStmt};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, StmtWith};
use sifr_type_system::{FunctionType, Type};

fn task_scope_type() -> Type {
    Type::Class {
        name: "TaskScope".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

fn simple_with_target_name(optional_vars: Option<&Expr>, ctx: &mut LowerCtx) -> Option<String> {
    let vars = optional_vars?;
    if let Expr::Name(n) = vars {
        Some(n.id.to_string())
    } else {
        statement_diagnostics::unsupported_form(
            ctx,
            "with target must be a simple name",
            vars.range(),
        );
        None
    }
}

fn async_task_call_name(expr: &Expr) -> Option<(&str, &sifr_python_ast::ExprCall)> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Attribute(attr) = call.func.as_ref() else {
        return None;
    };
    let Expr::Name(module_name) = attr.value.as_ref() else {
        return None;
    };
    if module_name.id.as_str() == "task" {
        Some((attr.attr.as_str(), call))
    } else {
        None
    }
}

pub(super) fn lower_async_with(
    with_stmt: &StmtWith,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async with is only valid inside async functions".to_string(),
            with_stmt.range(),
        );
        return None;
    }
    if with_stmt.items.len() != 1 {
        statement_diagnostics::unsupported_form(
            ctx,
            "async with supports exactly one built-in context item in v1",
            with_stmt.range(),
        );
        return None;
    }

    let item = &with_stmt.items[0];
    let Some((task_fn, call)) = async_task_call_name(&item.context_expr) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async with only supports task.scope() and task.timeout(duration) in v1".to_string(),
            item.context_expr.range(),
        );
        return None;
    };

    if !call.arguments.keywords.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
            format!("task.{task_fn}() does not accept keyword arguments"),
            call.arguments.keywords[0].range(),
        );
        return None;
    }

    ctx.with_pushed_scope(|ctx| {
        let (kind, target) = match task_fn {
            "scope" => {
                if !call.arguments.args.is_empty() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                        "task.scope() takes no arguments".to_string(),
                        item.context_expr.range(),
                    );
                    return None;
                }
                let target = simple_with_target_name(item.optional_vars.as_deref(), ctx);
                if let Some(name) = &target {
                    ctx.scope.define(name.clone(), task_scope_type());
                }
                (HirAsyncWithKind::TaskScope, target)
            }
            "timeout" => {
                if call.arguments.args.len() != 1 {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                        "task.timeout() takes exactly one duration argument".to_string(),
                        item.context_expr.range(),
                    );
                    return None;
                }
                if item.optional_vars.is_some() {
                    statement_diagnostics::unsupported_form(
                        ctx,
                        "async with task.timeout(duration) does not bind a target in v1",
                        item.optional_vars
                            .as_ref()
                            .map_or(item.context_expr.range(), |vars| vars.range()),
                    );
                    return None;
                }
                let duration = lower_expr(&call.arguments.args[0], ctx)?;
                if !matches!(duration.ty().resolve_alias(), Type::Int | Type::Float) {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_MISMATCH,
                        format!(
                            "task.timeout() duration must be int or float, got '{}'",
                            duration.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                    return None;
                }
                (HirAsyncWithKind::TaskTimeout { duration }, None)
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "async with only supports task.scope() and task.timeout(duration) in v1"
                        .to_string(),
                    item.context_expr.range(),
                );
                return None;
            }
        };
        let body = lower_stmts(&with_stmt.body, func_type, ctx);
        Some(HirStmt::AsyncWith { kind, target, body })
    })
}
