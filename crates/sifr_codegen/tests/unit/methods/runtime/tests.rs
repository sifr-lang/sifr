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
