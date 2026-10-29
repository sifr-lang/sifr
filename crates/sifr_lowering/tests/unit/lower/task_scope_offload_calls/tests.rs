use super::*;

#[test]
fn fallback_worker_errors_keep_canonical_parallel_identity() {
    let ctx = LowerCtx::new();
    for name in ["WorkerError", "WorkerRuntimeError"] {
        assert!(matches!(
            worker_error_type(name, &ctx),
            Type::Class { identity: Some(identity), .. }
                if identity == format!("sifr.parallel.{name}")
        ));
    }
}
