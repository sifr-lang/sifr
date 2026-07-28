use super::task_scope_calls::{non_send_reason, non_share_safe_reason};
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Decorator, Expr};
use sifr_type_system::Type;

pub(in crate::lower) fn has_threadsafe_callback_decorator(decorators: &[Decorator]) -> bool {
    decorators.iter().any(|decorator| {
        let Expr::Call(call) = &decorator.expression else {
            return false;
        };
        super::python_interop::decorator_path(&call.func)
            .is_some_and(|path| path.as_slice() == ["rust", "callback"])
    })
}

pub(in crate::lower) fn validate_threadsafe_callback_captures(
    callable: &str,
    args: &[HirExpr],
    argument_ranges: &[Option<TextRange>],
    fallback_range: TextRange,
    ctx: &mut LowerCtx,
) {
    if !ctx.rust_threadsafe_callback_targets.contains(callable) {
        return;
    }
    let Some(function) = ctx.functions.get(callable).cloned() else {
        return;
    };
    let callback_indices = function
        .params
        .iter()
        .enumerate()
        .filter_map(|(index, (_, ty, _))| {
            matches!(ty.resolve_alias(), Type::Callable(..)).then_some(index)
        })
        .collect::<Vec<_>>();
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
        for (capture_name, capture_ty) in captures {
            let reason = non_send_reason(&capture_ty)
                .map(|reason| format!("is not sendable: {reason}"))
                .or_else(|| {
                    non_share_safe_reason(&capture_ty)
                        .map(|reason| format!("is not share-safe: {reason}"))
                });
            if let Some(reason) = reason {
                ctx.error_with_code_at(
                    DiagnosticCode::RUST_CALLBACK_CONTRACT,
                    format!(
                        "invalid Rust callback attachment: handler `{name}` capture `{capture_name}` of type `{}` {reason}",
                        capture_ty.display_name()
                    ),
                    range,
                );
            }
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
