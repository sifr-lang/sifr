use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::StmtReturn;
use sifr_type_system::{FunctionType, OwnershipKind, Type};

use crate::hir_nodes::{HirExpr, HirStmt};

use super::LowerCtx;
use super::expressions::lower_expr;
use super::ownership_diagnostics;
use super::task_scope_calls::sync_guard_type_label;

pub(in crate::lower) fn lower_return(
    ret: &StmtReturn,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> HirStmt {
    if ctx.current_function_is_generator {
        if let Some(val) = &ret.value {
            let Some(expr) = lower_expr(val, ctx) else {
                return HirStmt::Return { value: None };
            };
            let expr_ty = expr.ty().clone();
            if matches!(expr_ty.resolve_alias(), Type::None) {
                if matches!(expr, HirExpr::NoneLiteral) {
                    return HirStmt::Return { value: None };
                }
                let generator_kind = if ctx.current_function_is_async_generator {
                    "async generator"
                } else {
                    "generator"
                };
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "{generator_kind} exhaustion accepts only bare 'return' or 'return None'; None-typed return expressions are rejected because generators cannot expose a return value"
                    ),
                    val.range(),
                );
                return HirStmt::Return { value: None };
            }
            let generator_kind = if ctx.current_function_is_async_generator {
                "async generator"
            } else {
                "generator"
            };
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "non-None {generator_kind} return values are rejected; yielded items are exposed through the iterator protocol"
                ),
                val.range(),
            );
        }
        return HirStmt::Return { value: None };
    }

    let value = if let Some(val) = &ret.value {
        let expected_expr_type = match func_type.return_type.resolve_alias() {
            Type::Result(ok_type, _error_type) => ok_type.as_ref().clone(),
            return_type => return_type.clone(),
        };
        ctx.push_contextual_expr_type(val.range(), expected_expr_type);
        let lowered = lower_expr(val, ctx);
        ctx.pop_contextual_expr_type();
        let Some(expr) = lowered else {
            // Keep control-flow shape intact after expression diagnostics so
            // return-completeness analysis does not emit a cascade error.
            return HirStmt::Return {
                value: Some(HirExpr::NoneLiteral),
            };
        };
        let expr_ty = expr.ty().clone();

        if let HirExpr::Name { name, ty, .. } = &expr {
            if ctx.borrowed_params.contains(name.as_str())
                && ty.ownership() == OwnershipKind::Move
                && !ty.contains_affine_resource()
            {
                ownership_diagnostics::borrowed_parameter_return_escape(ctx, name, val.range());
            }
        }
        if !ctx.is_stdlib_lowering() {
            if let Some(label) = sync_guard_type_label(&expr_ty) {
                ownership_diagnostics::sync_guard_return_escape(ctx, label, val.range());
            }
        }
        transfer_return_ownership(&expr, val.range(), ctx);

        if let Type::Result(ref ok_ty, _) = *func_type.return_type {
            if expr_ty.is_assignable_to(ok_ty) && !matches!(expr_ty, Type::Result(_, _)) {
                return HirStmt::Return {
                    value: Some(HirExpr::OkWrap {
                        ty: func_type.return_type.as_ref().clone(),
                        value: Box::new(expr),
                    }),
                };
            }
        }

        if !expr_ty.is_assignable_to(&func_type.return_type) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "return type mismatch: expected '{}', got '{}'",
                    func_type.return_type.display_name(),
                    expr_ty.display_name()
                ),
                val.range(),
            );
        }
        Some(expr)
    } else {
        if *func_type.return_type != Type::None {
            if let Type::Result(ref ok_ty, _) = *func_type.return_type {
                if **ok_ty == Type::None {
                    return HirStmt::Return {
                        value: Some(HirExpr::OkWrap {
                            ty: func_type.return_type.as_ref().clone(),
                            value: Box::new(HirExpr::NoneLiteral),
                        }),
                    };
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: expected '{}', got 'None'",
                    func_type.return_type.display_name()
                ),
                ret.range(),
            );
        }
        None
    };

    HirStmt::Return { value }
}

fn transfer_return_ownership(expr: &HirExpr, range: TextRange, ctx: &mut LowerCtx) {
    use super::must_use_obligations::MustUseObligationKind;

    match expr {
        HirExpr::Name { name, .. } if ctx.python_context_borrows.contains_key(name) => {
            ctx.error_with_code_at(
                DiagnosticCode::PYCTX_INVALID_DECLARATION,
                format!(
                    "invalid Python context declaration: entered binding '{name}' is a context-scoped borrow and cannot escape its with block"
                ),
                range,
            );
        }
        HirExpr::Name { name, ty, .. }
            if ctx.borrowed_params.contains(name)
                && ty.ownership() == OwnershipKind::Move
                && ty.contains_affine_resource() =>
        {
            ownership_diagnostics::borrowed_affine_parameter_escape(ctx, name, "return", range);
        }
        HirExpr::Name { name, ty, .. }
            if ty.ownership() == OwnershipKind::Move
                && ctx.live_must_use_bindings.contains_key(name) =>
        {
            let Some(obligation) = ctx.live_must_use_bindings.get(name).cloned() else {
                return;
            };
            if obligation.kind == MustUseObligationKind::CloseLike {
                ctx.mark_moved_with_flow(name);
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::PYCTX_INVALID_DECLARATION,
                    format!(
                        "invalid Python context declaration: binding '{name}' owns {obligation} and must be consumed by its dedicated context statement rather than returned or aggregated"
                    ),
                    range,
                );
            }
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. } => {
            for element in elements {
                transfer_return_ownership(element, range, ctx);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for element in keys.iter().chain(values) {
                transfer_return_ownership(element, range, ctx);
            }
        }
        HirExpr::ConstructorCall { args, .. } => {
            for argument in args {
                transfer_return_ownership(argument, range, ctx);
            }
        }
        HirExpr::IteratorCall { args, .. } => {
            for argument in args {
                transfer_return_ownership(argument, range, ctx);
            }
        }
        HirExpr::OkWrap { value, .. } => transfer_return_ownership(value, range, ctx),
        HirExpr::QuestionMark { expr, .. } | HirExpr::ErrWrap { value: expr, .. } => {
            transfer_return_ownership(expr, range, ctx);
        }
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            transfer_return_ownership(then_expr, range, ctx);
            transfer_return_ownership(else_expr, range, ctx);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            transfer_return_ownership(expr, range, ctx);
            for (_, iter, filter) in generators {
                transfer_return_ownership(iter, range, ctx);
                if let Some(filter) = filter {
                    transfer_return_ownership(filter, range, ctx);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            transfer_return_ownership(key_expr, range, ctx);
            transfer_return_ownership(val_expr, range, ctx);
            for (_, iter, filter) in generators {
                transfer_return_ownership(iter, range, ctx);
                if let Some(filter) = filter {
                    transfer_return_ownership(filter, range, ctx);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            transfer_return_ownership(expr, range, ctx);
            transfer_return_ownership(iter, range, ctx);
            if let Some(filter) = filter {
                transfer_return_ownership(filter, range, ctx);
            }
        }
        _ => {}
    }
}
