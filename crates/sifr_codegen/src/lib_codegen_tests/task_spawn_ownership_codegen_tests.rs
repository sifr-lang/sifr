use super::*;

#[test]
fn test_thread_pool_executor_submit_reuses_blocking_task_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "class ThreadPoolExecutor:\n    pass\n\n\n@cpu_heavy\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(compute_value)\n    result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(
        result
            .rust_source
            .contains("struct __SifrBlockingTask<T, E>")
    );
    assert!(
        result
            .rust_source
            .contains("fn __sifr_spawn_blocking_infallible<")
    );
    assert!(
        result
            .rust_source
            .contains("__sifr_spawn_blocking_infallible(compute_value);")
    );
    assert!(
        result
            .required_features
            .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio)
    );
}

#[test]
fn test_scope_spawn_lowers_owned_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(value: int) -> int:\n    await task.sleep(0.0)\n    return value\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        value: int = 41\n        handle = scope.spawn(worker(value))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(
        result
            .rust_source
            .contains("scope.__sifr_spawn_infallible(worker(value));")
    );
}

#[test]
fn test_scope_spawn_lowers_owned_move_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(own items: list[int]) -> int:\n    await task.sleep(0.0)\n    return len(items)\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker([1, 2]))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(
        result
            .rust_source
            .contains("scope.__sifr_spawn_infallible(worker(vec![1_i64, 2_i64]));")
    );
}
