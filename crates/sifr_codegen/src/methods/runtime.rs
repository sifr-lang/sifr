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
mod tests {
    use super::*;
    use sifr_type_system::{task_group_type, task_scope_type};

    #[test]
    fn runtime_registry_requires_exact_owner_and_arity() {
        let receiver = RustExpr::Ident("receiver".to_string());
        let arg = RustExpr::Ident("argument".to_string());
        for owner in [task_scope_type(), task_group_type()] {
            for (method, arity) in [
                ("__sifr_spawn_infallible", 1),
                ("__sifr_spawn_result", 1),
                ("__sifr_spawn_infallible_with_context", 2),
                ("__sifr_spawn_result_with_context", 2),
                ("__sifr_scope_spawn_blocking_infallible", 1),
                ("__sifr_scope_spawn_blocking_result", 1),
                ("__sifr_scope_spawn_cpu_infallible", 1),
                ("__sifr_scope_spawn_cpu_result", 1),
                ("__sifr_scope_spawn_process", 1),
            ] {
                assert!(
                    lower_runtime_receiver_method(
                        &owner,
                        method,
                        &receiver,
                        &vec![arg.clone(); arity]
                    )
                    .is_some()
                );
                assert!(
                    lower_runtime_receiver_method(
                        &owner,
                        method,
                        &receiver,
                        &vec![arg.clone(); arity - 1]
                    )
                    .is_none()
                );
                assert!(
                    lower_runtime_receiver_method(
                        &owner,
                        method,
                        &receiver,
                        &vec![arg.clone(); arity + 1]
                    )
                    .is_none()
                );
                let mut collision = owner.clone();
                if let Type::Class { identity, .. } = &mut collision {
                    *identity = None;
                }
                assert!(
                    lower_runtime_receiver_method(
                        &collision,
                        method,
                        &receiver,
                        &vec![arg.clone(); arity]
                    )
                    .is_none()
                );
            }
        }
        let join_set = Type::JoinSet(Box::new(Type::Int), Box::new(Type::Never));
        for method in [
            "__sifr_add_task",
            "__sifr_add_blocking_task",
            "__sifr_spawn_blocking",
            "__sifr_spawn_cpu",
        ] {
            assert!(
                lower_runtime_receiver_method(
                    &join_set,
                    method,
                    &receiver,
                    std::slice::from_ref(&arg)
                )
                .is_some()
            );
            assert!(lower_runtime_receiver_method(&join_set, method, &receiver, &[]).is_none());
            assert!(
                lower_runtime_receiver_method(
                    &join_set,
                    method,
                    &receiver,
                    &[arg.clone(), arg.clone()]
                )
                .is_none()
            );
            assert!(
                lower_runtime_receiver_method(
                    &Type::Int,
                    method,
                    &receiver,
                    std::slice::from_ref(&arg)
                )
                .is_none()
            );
        }
        let generator = Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Never));
        assert!(lower_runtime_receiver_method(&generator, "aclose", &receiver, &[]).is_some());
        assert!(lower_runtime_receiver_method(&generator, "aclose", &receiver, &[arg]).is_none());
        assert!(lower_runtime_receiver_method(&Type::Int, "aclose", &receiver, &[]).is_none());
        assert!(lower_runtime_receiver_method(&generator, "invented", &receiver, &[]).is_none());
    }
}
