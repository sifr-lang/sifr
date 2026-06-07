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
            }
        }
        if !ctx.allow_intrinsic_imports {
            if let Some(label) = sync_guard_type_label(&expr_ty) {
                ownership_diagnostics::sync_guard_return_escape(ctx, label, val.range());
            }
        }

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
