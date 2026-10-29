//! Typed admission for compiler-provided runtime receiver methods.
use crate::RustExpr;
use sifr_type_system::{Type, is_task_owner_type};

pub(super) fn lower_runtime_receiver_method(
    receiver_ty: &Type,
    method: &str,
    receiver: &RustExpr,
    args: &[RustExpr],
) -> Option<RustExpr> {
    let (arity, borrow_context) = match receiver_ty.resolve_alias() {
        Type::Class { .. } if is_task_owner_type(receiver_ty) => match method {
            "__sifr_spawn_infallible_with_context" | "__sifr_spawn_result_with_context" => {
                (2, true)
            }
            "__sifr_spawn_infallible"
            | "__sifr_spawn_result"
            | "__sifr_scope_spawn_blocking_infallible"
            | "__sifr_scope_spawn_blocking_result"
            | "__sifr_scope_spawn_cpu_infallible"
            | "__sifr_scope_spawn_cpu_result"
            | "__sifr_scope_spawn_process" => (1, false),
            _ => return None,
        },
        Type::JoinSet(_, _) => match method {
            "__sifr_add_task"
            | "__sifr_add_blocking_task"
            | "__sifr_spawn_blocking"
            | "__sifr_spawn_cpu" => (1, false),
            _ => return None,
        },
        Type::AsyncGenerator(_, _) if method == "aclose" => (0, false),
        _ => return None,
    };
    if args.len() != arity {
        return None;
    }
    let mut args = args.to_vec();
    if borrow_context && let Some(context) = args.first_mut() {
        // Runtime context helpers format the label synchronously. They accept
        // Display by value, so borrowing retains the source context for reuse.
        *context = RustExpr::Ref {
            mutable: false,
            expr: Box::new(context.clone()),
        };
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(receiver.clone()),
        method: method.to_string(),
        args,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/methods/runtime/tests.rs"]
mod tests;
