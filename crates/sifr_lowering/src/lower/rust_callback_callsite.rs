use super::task_scope_calls::{non_send_reason, non_share_safe_reason};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Decorator, Expr};
use sifr_type_system::{ParamConvention, Type};
use std::collections::{HashMap, HashSet};

pub(in crate::lower) fn has_threadsafe_callback_decorator(decorators: &[Decorator]) -> bool {
    decorators.iter().any(|decorator| {
        let Expr::Call(call) = &decorator.expression else {
            return false;
        };
        super::python_interop::decorator_path(&call.func)
            .is_some_and(|path| path.as_slice() == ["rust", "callback"])
    })
}

pub(in crate::lower) fn record_threadsafe_callback_target(
    callable: String,
    params: &[(String, Type, ParamConvention)],
    decorators: &[Decorator],
    ctx: &mut LowerCtx,
) {
    if !has_threadsafe_callback_decorator(decorators) {
        return;
    }
    let callback_indices = params
        .iter()
        .enumerate()
        .filter_map(|(index, (_, ty, _))| {
            matches!(ty.resolve_alias(), Type::Callable(..)).then_some(index)
        })
        .collect();
    ctx.rust_threadsafe_callback_targets
        .insert(callable, callback_indices);
}

pub(in crate::lower) fn validate_threadsafe_callback_captures(
    callable: &str,
    args: &[HirExpr],
    argument_ranges: &[Option<TextRange>],
    fallback_range: TextRange,
    ctx: &mut LowerCtx,
) {
    let Some(callback_indices) = ctx.rust_threadsafe_callback_targets.get(callable).cloned() else {
        return;
    };
    for index in callback_indices {
        let range = argument_ranges
            .get(index)
            .copied()
            .flatten()
            .unwrap_or(fallback_range);
        let Some(HirExpr::Name { name, .. }) = args.get(index) else {
            reject_unverifiable_handler(range, ctx);
            continue;
        };
        let Some(captures) = ctx.nested_function_captures.get(name).cloned() else {
            let top_level = ctx.functions.contains_key(name)
                && ctx.lookup_current_function_binding(name).is_none()
                && ctx.lookup_outer_function_binding(name).is_none();
            if !ctx.scope.resolves_to_module_binding(name) && !top_level {
                ctx.error_with_code_at(
                    DiagnosticCode::RUST_CALLBACK_CONTRACT,
                    format!(
                        "invalid Rust callback attachment: handler `{name}` is a callable value whose captures cannot be proven thread-safe; use a top-level function or a directly declared nested function"
                    ),
                    range,
                );
            }
            continue;
        };
        let mut valid = true;
        for (capture_name, capture_ty) in &captures {
            let mut visited = HashSet::from([name.clone()]);
            if let Some(violation) =
                retained_capture_violation(capture_name, capture_ty, ctx, &mut visited)
            {
                ctx.error_with_code_at(
                    DiagnosticCode::RUST_CALLBACK_CONTRACT,
                    format!(
                        "invalid Rust callback attachment: handler `{name}` capture `{}` of type `{}` {}",
                        violation.path,
                        violation.ty.display_name(),
                        violation.reason
                    ),
                    range,
                );
                valid = false;
            }
        }
        if valid {
            let mut capture_plans = HashMap::new();
            collect_retained_handler_capture_plans(
                name,
                ctx,
                &mut HashSet::new(),
                &mut capture_plans,
            );
            ctx.rust_threadsafe_callback_move_handlers
                .extend(capture_plans);
            ctx.mark_binding_moved_with_flow(name);
        }
    }
}

struct CaptureViolation {
    path: String,
    ty: Type,
    reason: String,
}

fn retained_capture_violation(
    capture_name: &str,
    capture_ty: &Type,
    ctx: &LowerCtx,
    visited: &mut HashSet<String>,
) -> Option<CaptureViolation> {
    if matches!(
        capture_ty.resolve_alias(),
        Type::Callable(..) | Type::AsyncCallable(..)
    ) {
        return Some(CaptureViolation {
            path: capture_name.to_string(),
            ty: capture_ty.clone(),
            reason: "is a callable value whose captures cannot be proven thread-safe".to_string(),
        });
    }
    if let Some(reason) = non_send_reason(capture_ty) {
        return Some(CaptureViolation {
            path: capture_name.to_string(),
            ty: capture_ty.clone(),
            reason: format!("is not sendable: {reason}"),
        });
    }
    if let Some(reason) = non_share_safe_reason(capture_ty) {
        return Some(CaptureViolation {
            path: capture_name.to_string(),
            ty: capture_ty.clone(),
            reason: format!("is not share-safe: {reason}"),
        });
    }
    if let Some(nested_captures) = ctx.nested_function_captures.get(capture_name) {
        if !visited.insert(capture_name.to_string()) {
            return Some(CaptureViolation {
                path: capture_name.to_string(),
                ty: capture_ty.clone(),
                reason: "has a recursive callable capture cycle that cannot be proven thread-safe"
                    .to_string(),
            });
        }
        for (nested_name, nested_ty) in nested_captures {
            if let Some(mut violation) =
                retained_capture_violation(nested_name, nested_ty, ctx, visited)
            {
                violation.path = format!("{capture_name}.{}", violation.path);
                return Some(violation);
            }
        }
        visited.remove(capture_name);
        return None;
    }
    if capture_ty.ownership() != sifr_type_system::OwnershipKind::Copy
        && !capture_ty.supports_derived_clone()
    {
        return Some(CaptureViolation {
            path: capture_name.to_string(),
            ty: capture_ty.clone(),
            reason: "is not clone-capable for retained callback ownership".to_string(),
        });
    }
    None
}

fn collect_retained_handler_capture_plans(
    handler: &str,
    ctx: &LowerCtx,
    visited: &mut HashSet<String>,
    plans: &mut HashMap<String, Vec<String>>,
) {
    if !visited.insert(handler.to_string()) {
        return;
    }
    let Some(captures) = ctx.nested_function_captures.get(handler) else {
        return;
    };
    plans.insert(
        handler.to_string(),
        captures
            .iter()
            .filter(|(name, ty)| {
                ctx.nested_function_captures.contains_key(name)
                    || ty.ownership() != sifr_type_system::OwnershipKind::Copy
            })
            .map(|(name, _)| name.clone())
            .collect(),
    );
    for (capture, _) in captures {
        if ctx.nested_function_captures.contains_key(capture) {
            collect_retained_handler_capture_plans(capture, ctx, visited, plans);
        }
    }
}

fn reject_unverifiable_handler(range: TextRange, ctx: &mut LowerCtx) {
    ctx.error_with_code_at(
        DiagnosticCode::RUST_CALLBACK_CONTRACT,
        "invalid Rust callback attachment: thread-safe handlers must be named functions so captures can be proven sendable and share-safe"
            .to_string(),
        range,
    );
}
