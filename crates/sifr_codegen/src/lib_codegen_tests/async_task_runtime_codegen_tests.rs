use super::*;
#[test]
fn test_task_group_basic_lowers_to_scope_runtime_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup() as group:\n        handle = group.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTaskScope"));
    assert!(result
        .rust_source
        .contains("let mut group = __SifrTaskScope::new_task_group();"));
    assert!(result.rust_source.contains("fail_fast"));
    assert!(result
        .rust_source
        .contains("group.__sifr_spawn_infallible(worker());"));
    assert!(result
        .rust_source
        .contains("if let Err(__sifr_scope_failure) = group.__sifr_join_all().await"));
    assert!(result
        .rust_source
        .contains("else if let Some(cancellation) = child.cancellation.take()"));
    assert!(result
        .rust_source
        .contains("let _ = cancellation.request_cancel();"));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio));
}

#[test]
fn test_task_gather_lowers_to_private_gather_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def second() -> int:\n    await task.sleep(0.0)\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__sifr_task_gather(vec!["));
    assert!(result
        .rust_source
        .contains("let _ = cancellation.request_cancel();"));
    assert!(result.rust_source.contains("failure_results"));
    assert!(result.rust_source.contains("push_secondary_message"));
    assert!(result.rust_source.contains("ordered_values.push(value);"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, ::std::convert::Infallible>"));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio));
}

#[test]
fn test_scope_spawn_fallible_coroutine_lowers_to_result_spawn_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    await task.sleep(0.0)\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("tokio::sync::oneshot::Receiver<__SifrTaskResult<T, E>>"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_result(worker());"));
    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("let handle: __SifrTask<i64, ValueError>"));
    assert!(!result.rust_source.contains("let handle: Task<"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio));
}

#[test]
fn test_task_gather_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    await task.sleep(0.0)\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__SifrTaskResult<Vec<T>, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("sibling task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("sibling task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(!result.rust_source.contains("let result: TaskResult<"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, ValueError>"));
}

#[test]
fn test_task_race_lowers_to_private_race_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def second() -> int:\n    await task.sleep(1.0)\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__sifr_task_race(vec!["));
    assert!(result.rust_source.contains("let Some(mut first)"));
    assert!(result
        .rust_source
        .contains("let _ = cancellation.request_cancel();"));
    assert!(result
        .rust_source
        .contains("race loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ::std::convert::Infallible>"));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio));
}

#[test]
fn test_task_race_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    await task.sleep(0.0)\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("race loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
}

#[test]
fn test_task_select_lowers_to_private_select_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def second() -> str:\n    await task.sleep(1.0)\n    return \"two\"\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first=first_handle, second=second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrSelect2<A, B>"));
    assert!(result.rust_source.contains("async fn __sifr_task_select"));
    assert!(result
        .rust_source
        .contains("__sifr_task_select(first_handle, second_handle)"));
    assert!(result
        .rust_source
        .contains("select loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("select loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("let _ = second_cancellation.request_cancel();"));
    assert!(result
        .rust_source
        .contains("let _ = first_cancellation.request_cancel();"));
    assert!(result
        .rust_source
        .contains("second_observed.store(false, ::std::sync::atomic::Ordering::SeqCst)"));
    assert!(result
        .rust_source
        .contains("first_observed.store(false, ::std::sync::atomic::Ordering::SeqCst)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, ::std::convert::Infallible>, __SifrTaskResult<String, ::std::convert::Infallible>>"
    ));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::Tokio));
}

#[test]
fn test_task_select_fallible_tasks_preserves_distinct_error_parameters() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> Result[int, ValueError]:\n    await task.sleep(0.0)\n    raise ValueError(\"first\")\n\nasync def second() -> Result[str, IOError]:\n    await task.sleep(0.0)\n    raise IOError(\"second\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first=first_handle, second=second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("__SifrTaskResult<A, EA>"));
    assert!(result.rust_source.contains("__SifrTaskResult<B, EB>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, ValueError>, __SifrTaskResult<String, IOError>>"
    ));
}

#[test]
fn test_task_handle_join_lowers_to_task_result_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("async fn join(self)"));
    assert!(result
        .rust_source
        .contains("Cancelled(__SifrFailure<CancellationError>)"));
    assert!(result.rust_source.contains("fn cancelled() -> Self"));
    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ::std::convert::Infallible>"));
}

#[test]
fn test_await_task_handle_desugars_to_join_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ::std::convert::Infallible>"));
}

#[test]
fn test_task_handle_cancel_uses_cooperative_carrier_with_abort_fallback() {
    let source = concat!(
        "async def worker() -> int:\n    await task.sleep(10.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n        handle.",
        "cancel",
        "()\n        result = await handle\n    return None\n",
    );
    let result = generate_rust_with_metadata(
        &lower_module(parse_module(source).expect("parse failed").suite())
            .expect("lowering failed")
            .module,
    );

    assert!(result
        .rust_source
        .contains("cancellation: __SifrCancellationCarrier"));
    assert!(!result
        .rust_source
        .contains("cancellation: tokio::task::AbortHandle"));
    assert!(result
        .rust_source
        .contains("let _ = self.cancellation.request_cancel();"));
    assert!(result
        .rust_source
        .contains("static __SIFR_TASK_CANCELLATION:"));
    assert!(!result
        .rust_source
        .contains("fn __sifr_current_task_cancellation"));
    assert!(result
        .rust_source
        .contains("__SIFR_TASK_CANCELLATION.scope(child_cancellation"));
    assert!(!result
        .rust_source
        .contains("__SIFR_COOPERATIVE_SUPERVISORS_READY"));
    assert!(result
        .rust_source
        .contains(&format!("fn {}{}", "can", "cel(&self)")));
    assert!(result
        .rust_source
        .contains(&format!("handle.{}{}", "can", "cel();")));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result.rust_source.contains("__SifrTaskResult::cancelled()"));
    assert!(result.rust_source.contains("handle.join().await"));
}

#[test]
fn test_join_set_preserves_task_cancellation_carrier_until_terminal_drain() {
    let source = concat!(
        "async def worker() -> Result[int, ValueError]:\n",
        "    await task.sleep(0.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n",
        "    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n",
        "        joined = task.JoinSet[int, ValueError]()\n",
        "        entry_id = joined.add(handle)\n",
        "        outcomes = await joined.cancel_all()\n",
        "    return None\n",
    );
    let result = generate_rust_with_metadata(
        &lower_module(parse_module(source).expect("parse failed").suite())
            .expect("lowering failed")
            .module,
    );

    assert!(result
        .rust_source
        .contains("let __SifrTask { receiver, cancellation, observed, _error } = task;"));
    assert!(result
        .rust_source
        .contains("cancellation: Some(cancellation)"));
    assert!(result.rust_source.contains("blocking_abort: None"));
    assert!(result
        .rust_source
        .contains("if let Some(cancellation) = entry.cancellation"));
    assert!(result
        .rust_source
        .contains("let _ = cancellation.request_cancel();"));
    assert!(!result.rust_source.contains("cancellation.abort_handle()"));
}

#[test]
fn test_task_timeout_handle_lowers_to_private_timeout_result() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await task.timeout(handle, 1.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTimeoutResult<E>"));
    assert!(result.rust_source.contains("async fn __sifr_timeout"));
    assert!(result.rust_source.contains("biased;"));
    assert!(result.rust_source.contains("handle.__sifr_timeout"));
    assert!(result
        .rust_source
        .contains("failure.map_primary(__SifrTimeoutResult::Inner)"));
    assert!(result
        .rust_source
        .contains("__SifrFailure::new(__SifrTimeoutResult::Timeout)"));
    assert!(result
        .rust_source
        .contains("matches!(request, ::sifr_runtime::cancellation::CancellationRequest::Claimed)"));
    assert!(result
        .rust_source
        .contains("Ok(__SifrTaskResult::Ok(value)) => __SifrTaskResult::Ok(value)"));
    assert!(result.rust_source.contains(
        "let result: __SifrTaskResult<i64, __SifrTimeoutResult<::std::convert::Infallible>>"
    ));
}

#[test]
fn test_failure_cancellation_error_annotation_lowers_to_private_evidence_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "def observe(failure: Failure[CancellationError]) -> None:\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<CancellationError>)"));
}

#[test]
fn test_failure_annotation_lowers_to_private_failure_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("def observe(failure: Failure[ValueError]) -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("primary: E"));
    assert!(result
        .rust_source
        .contains("secondary: Vec<SecondaryError>"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<ValueError>)"));
}
