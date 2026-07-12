use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::StmtReturn;
use sifr_type_system::{FunctionType, OwnershipKind, Type};

use crate::hir_nodes::{HirExpr, HirStmt};

use super::expressions::lower_expr;
use super::ownership_diagnostics;
use super::task_scope_calls::sync_guard_type_label;
use super::LowerCtx;

pub(in crate::lower) fn lower_return(
    ret: &StmtReturn,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> HirStmt {
    if ctx.current_function_is_async_generator {
        if let Some(val) = &ret.value {
            let Some(expr) = lower_expr(val, ctx) else {
                return HirStmt::Return {
                    value: Some(HirExpr::NoneLiteral),
                };
            };
            let expr_ty = expr.ty().clone();
            if matches!(expr_ty.resolve_alias(), Type::None) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "return with a value inside async generator bodies requires async generator state-machine lowering and is not supported yet"
                        .to_string(),
                    val.range(),
                );
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "non-None async generator return values are rejected in v1; async generators expose yielded items through AsyncGenerator[T, E]"
                        .to_string(),
                    val.range(),
                );
            }
            return HirStmt::Return { value: Some(expr) };
        }

        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "return inside async generator bodies requires async generator state-machine lowering and is not supported yet"
                .to_string(),
            ret.range(),
        );
        return HirStmt::Return { value: None };
    }

    let value = if let Some(val) = &ret.value {
        let Some(expr) = lower_expr(val, ctx) else {
            // Keep control-flow shape intact after expression diagnostics so
            // return-completeness analysis does not emit a cascade error.
            return HirStmt::Return {
                value: Some(HirExpr::NoneLiteral),
            };
        };
        let expr_ty = expr.ty().clone();

        if let HirExpr::Name { name, ty } = &expr {
            if ctx.borrowed_params.contains(name.as_str()) && ty.ownership() == OwnershipKind::Move
            {
                ownership_diagnostics::borrowed_parameter_return_escape(ctx, name, val.range());
            } else if ty.ownership() == OwnershipKind::Move
                && ctx.live_must_use_bindings.contains_key(name)
            {
                ctx.mark_moved_with_flow(name);
            }
        }
        if !ctx.is_stdlib_lowering() {
            if let Some(label) = sync_guard_type_label(&expr_ty) {
                ownership_diagnostics::sync_guard_return_escape(ctx, label, val.range());
            }
        }
        transfer_return_ownership(&expr, ctx);

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

fn transfer_return_ownership(expr: &HirExpr, ctx: &mut LowerCtx) {
    match expr {
        HirExpr::Name { name, ty }
            if ty.ownership() == OwnershipKind::Move
                && ctx.live_must_use_bindings.contains_key(name) =>
        {
            ctx.mark_moved_with_flow(name);
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. } => {
            for element in elements {
                transfer_return_ownership(element, ctx);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for element in keys.iter().chain(values) {
                transfer_return_ownership(element, ctx);
            }
        }
        HirExpr::ConstructorCall { args, .. } => {
            for argument in args {
                transfer_return_ownership(argument, ctx);
            }
        }
        HirExpr::IteratorCall { args, .. } => {
            for argument in args {
                transfer_return_ownership(argument, ctx);
            }
        }
        HirExpr::OkWrap { value, .. } => transfer_return_ownership(value, ctx),
        HirExpr::QuestionMark { expr, .. } | HirExpr::ErrWrap { value: expr, .. } => {
            transfer_return_ownership(expr, ctx);
        }
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            transfer_return_ownership(then_expr, ctx);
            transfer_return_ownership(else_expr, ctx);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            transfer_return_ownership(expr, ctx);
            for (_, iter, filter) in generators {
                transfer_return_ownership(iter, ctx);
                if let Some(filter) = filter {
                    transfer_return_ownership(filter, ctx);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            transfer_return_ownership(key_expr, ctx);
            transfer_return_ownership(val_expr, ctx);
            for (_, iter, filter) in generators {
                transfer_return_ownership(iter, ctx);
                if let Some(filter) = filter {
                    transfer_return_ownership(filter, ctx);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            transfer_return_ownership(expr, ctx);
            transfer_return_ownership(iter, ctx);
            if let Some(filter) = filter {
                transfer_return_ownership(filter, ctx);
            }
        }
        _ => {}
    }
}
