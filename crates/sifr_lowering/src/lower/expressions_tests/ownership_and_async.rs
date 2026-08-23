use super::*;
#[test]
pub(super) fn test_poisoned_initializer_binding_suppresses_followup_operator_cascade() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s = xs[0] + xs[0]\n    return s + 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let unsupported_operator_count = errors
        .iter()
        .filter(|error| error.message.contains("unsupported operand type(s) for +"))
        .count();
    assert_eq!(
        unsupported_operator_count, 1,
        "poisoned initializer binding should not trigger a second operator cascade: {errors:?}"
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.message == "undefined variable: 's'"),
        "poisoned initializer binding should suppress undefined-name cascades: {errors:?}"
    );
}

#[test]
pub(super) fn test_poisoned_initializer_binding_suppresses_followup_unary_cascade() {
    let result =
        lower_source("def main(xs: list[int]) -> int:\n    s = xs[0] + xs[0]\n    return -s\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("unsupported operand type(s)"))
            .count(),
        1,
        "poisoned initializer binding should not trigger unary operator cascades: {errors:?}"
    );
}

#[test]
pub(super) fn test_poisoned_initializer_binding_suppresses_formatting_cascades() {
    for use_site in [
        "print(value)",
        "text: str = str(value)",
        "text: str = f\"{value}\"",
        "total: int = value + 1",
        "total = value + 1",
    ] {
        let source = format!("def main():\n    value: MissingType[int] = 42\n    {use_site}\n");
        let errors = lower_source(&source).expect_err("unknown annotation should be rejected");
        assert_eq!(
            errors.len(),
            1,
            "poisoned binding should not trigger a formatting cascade at {use_site}: {errors:?}"
        );
        assert!(
            errors[0].message.contains("unknown type"),
            "the originating annotation error should be preserved: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_use_after_move() {
    let source = "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "print(", "s"))
    }));
}

#[test]
pub(super) fn test_borrowed_structural_upcast_requires_cloneable_source() {
    let result = lower_source(
        "class Root(NonSend):\n    value: int\n\nclass Child(Root):\n    extra: int\n\ndef consume(value: Root | int) -> int:\n    return 1\n\ndef main():\n    value: Child | int = Child(1, 2)\n    consume(value)\n",
    );
    let errors = result.expect_err("non-clone borrowed union conversion should be rejected");
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("requires a cloneable source representation")),
        "{errors:?}"
    );
}

#[test]
pub(super) fn test_await_task_handle_consumes_handle_binding() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        first = await handle\n        second = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "second = await ", "handle"))
    }));
}

#[test]
pub(super) fn test_task_handle_join_consumes_handle_binding() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        first = await handle.join()\n        second = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "second = await ", "handle"))
    }));
}

#[test]
pub(super) fn test_task_handle_cancel_does_not_consume_handle_binding() {
    let source = concat!(
        "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n        handle.",
        "cancel",
        "()\n        result = await handle\n    return None\n",
    );
    let result = lower_source(source);
    assert!(result.is_ok(), "cancel should borrow the task handle");
}

#[test]
pub(super) fn test_spawn_blocking_lowers_to_blocking_task_handle() {
    let source = "@cpu_heavy\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(compute_value)\n    result = await handle\n    return None\n";
    let module = lower_source(source).expect("lowering should succeed");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    let handle_assignment = main
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "handle"))
        .expect("handle assignment should exist");
    let HirStmt::Let { value, .. } = handle_assignment else {
        panic!("expected handle assignment");
    };
    let HirExpr::Call { func, ty, .. } = value else {
        panic!("expected spawn_blocking call");
    };
    assert_eq!(func, "__sifr_spawn_blocking_infallible");
    assert!(
        matches!(ty, Type::BlockingTask(ok, err) if matches!(ok.as_ref(), Type::Int) && matches!(err.as_ref(), Type::Never))
    );
}

#[test]
pub(super) fn test_thread_pool_executor_submit_lowers_to_blocking_task_handle() {
    let source = "class ThreadPoolExecutor:\n    pass\n\n\n@cpu_heavy\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(compute_value)\n    result = await handle\n    return None\n";
    let module = lower_source(source).expect("lowering should succeed");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    let handle_assignment = main
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "handle"))
        .expect("handle assignment should exist");
    let HirStmt::Let { value, .. } = handle_assignment else {
        panic!("expected handle assignment");
    };
    let HirExpr::Call { func, ty, .. } = value else {
        panic!("expected ThreadPoolExecutor.submit call");
    };
    assert_eq!(func, "__sifr_spawn_blocking_infallible");
    assert!(
        matches!(ty, Type::BlockingTask(ok, err) if matches!(ok.as_ref(), Type::Int) && matches!(err.as_ref(), Type::Never))
    );
}

#[test]
pub(super) fn test_spawn_blocking_rejects_unclassified_target() {
    let source = "def compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(compute_value)\n    result = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.spawn_blocking() target 'compute_value' is not classified")
            && e.code == Some(DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET)
    }));
}

#[test]
pub(super) fn test_thread_pool_executor_submit_rejects_unclassified_target() {
    let source = "class ThreadPoolExecutor:\n    pass\n\n\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(compute_value)\n    result = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("ThreadPoolExecutor.submit() target 'compute_value' is not classified")
            && e.code == Some(DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET)
    }));
}

#[test]
pub(super) fn test_thread_pool_executor_submit_rejects_non_send_return() {
    let source = "class ThreadPoolExecutor:\n    pass\n\nclass LocalCell(NonSend):\n    pass\n\n\n@cpu_heavy\ndef build_cell() -> LocalCell:\n    return LocalCell()\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(build_cell)\n    result = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("ThreadPoolExecutor.submit() cannot return non-send value type")
    }));
}

#[test]
pub(super) fn test_task_handle_cancel_after_await_rejects_moved_handle() {
    let source = concat!(
        "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n        result = await handle\n        handle.",
        "cancel",
        "()\n    return None\n",
    );
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "        handle.", "handle"))
    }));
}

#[test]
pub(super) fn test_scope_spawn_accepts_owned_coroutine_arguments() {
    let source = "async def worker(value: int) -> int:\n    await task.sleep(0.0)\n    return value\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        value: int = 41\n        handle = scope.spawn(worker(value))\n        result = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "owned sendable spawn arguments should lower: {result:?}"
    );
}

#[test]
pub(super) fn test_scope_spawn_rejects_borrowed_parameter_argument() {
    let source = "async def worker(own items: list[int]) -> int:\n    await task.sleep(0.0)\n    return len(items)\n\nasync def main(items: list[int]) -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker(items))\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("scope.spawn() cannot move borrowed parameter 'items' across a task boundary")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
    }));
}

#[test]
pub(super) fn test_scope_spawn_consumes_owned_move_argument() {
    let source = "async def worker(own items: list[int]) -> int:\n    await task.sleep(0.0)\n    return len(items)\n\nasync def main() -> Result[None, ScopeFailure]:\n    items: list[int] = [1]\n    async with task.scope() as scope:\n        handle = scope.spawn(worker(items))\n        items.append(2)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "handle = scope.spawn(worker(items))\n",
                    "items",
                ))
    }));
}

#[test]
pub(super) fn test_scope_spawn_rejects_non_send_field_argument() {
    let source = "class LocalCell(NonSend):\n    pass\n\nclass Job:\n    cell: LocalCell\n\nasync def worker(own job: Job) -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    cell: LocalCell = LocalCell()\n    job: Job = Job(cell)\n    async with task.scope() as scope:\n        handle = scope.spawn(worker(job))\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("scope.spawn() cannot move `job` of type `Job` across a task boundary")
            && e.message.contains("field `cell` is not sendable")
            && e.code == Some(DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE)
    }));
}

#[test]
pub(super) fn test_scope_spawn_rejects_self_with_non_send_field() {
    let source = "class LocalCell(NonSend):\n    pass\n\nclass Owner:\n    cell: LocalCell\n\nasync def worker(own owner: Owner) -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def launch(own self: Owner) -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker(self))\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("scope.spawn() cannot move `self` of type `Owner` across a task boundary")
            && e.message.contains("field `cell` is not sendable")
            && e.code == Some(DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE)
    }));
}

#[test]
pub(super) fn test_scope_spawn_rejects_lock_guard_argument() {
    let source = "class LockGuard[T]:\n    pass\n\nasync def worker(own guard: LockGuard[int]) -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    guard: LockGuard[int] = LockGuard()\n    async with task.scope() as scope:\n        handle = scope.spawn(worker(guard))\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("scope.spawn() cannot move `guard`")
            && e.message.contains("across a task boundary")
            && e.message.contains("`LockGuard` is a lock guard")
            && e.code == Some(DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE)
    }));
}

#[test]
pub(super) fn test_channel_send_rejects_non_send_element() {
    let source = "class ChannelSender[T]:\n    async def send(self, own value: T) -> None:\n        return None\n\nclass LocalCell(NonSend):\n    pass\n\nasync def main() -> None:\n    sender: ChannelSender[LocalCell] = ChannelSender()\n    cell: LocalCell = LocalCell()\n    await sender.send(cell)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("channel send cannot transfer `cell`")
            && e.message
                .contains("`LocalCell` inherits the `NonSend` marker")
            && e.code == Some(DiagnosticCode::OWN_NON_SEND_CHANNEL_ELEMENT)
    }));
}

#[test]
pub(super) fn test_shared_rejects_mutable_list_value() {
    let source = "class Shared[T]:\n    def __init__(self, own value: T):\n        pass\n\ndef main() -> None:\n    items: list[int] = [1]\n    shared: Shared[list[int]] = Shared(items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("Shared cannot publish `items`")
            && e.message.contains("list values are mutable")
            && e.code == Some(DiagnosticCode::OWN_NON_SHARE_SAFE_SHARED_VALUE)
    }));
}

#[test]
pub(super) fn test_mutable_borrow_parameter_across_await_rejected() {
    let source = "async def mutate_after_await(mut items: list[int]) -> int:\n    await task.sleep(0.0)\n    items.append(2)\n    return len(items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("mutable borrow `items` cannot cross await")
            && e.code == Some(DiagnosticCode::OWN_BORROW_ACROSS_AWAIT)
            && e.primary_range == Some(range_for(source, "await task.sleep(0.0)"))
    }));
}

#[test]
pub(super) fn test_mutable_borrow_parameter_across_async_generator_yield_rejected() {
    let source = "async def stream(mut items: list[int]) -> AsyncGenerator[int, GeneratorCloseError]:\n    yield len(items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("mutable borrow `items` cannot cross async generator yield")
            && e.code == Some(DiagnosticCode::OWN_BORROW_ACROSS_AWAIT)
            && e.primary_range == Some(range_for(source, "yield len(items)"))
    }));
}

#[test]
pub(super) fn test_async_generator_pending_anext_rejects_reentrant_advance() {
    let source = "async def numbers() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 1\n    yield 2\n\nasync def main() -> Result[None, GeneratorCloseError]:\n    agen = numbers()\n    first = anext(agen)\n    second = anext(agen)\n    observed: Result[Option[int], GeneratorCloseError] = await first\n    other: Result[Option[int], GeneratorCloseError] = await second\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("async generator `agen` already has a pending anext() advance")
            && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && matches!(
                e.args.get("binding"),
                Some(sifr_diagnostics::DiagnosticArg::String(binding)) if binding == "agen"
            )
            && e.primary_range == Some(range_for_after(source, "second = anext(", "agen"))
    }));
}

#[test]
pub(super) fn test_async_generator_anext_pending_state_clears_after_await() {
    let source = "async def numbers() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 1\n    yield 2\n\nasync def main() -> Result[None, GeneratorCloseError]:\n    agen = numbers()\n    first = anext(agen)\n    observed: Result[Option[int], GeneratorCloseError] = await first\n    second = anext(agen)\n    other: Result[Option[int], GeneratorCloseError] = await second\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "awaited anext handle should release the async generator advance state: {result:?}"
    );
}

#[test]
pub(super) fn test_await_after_completed_mutable_borrow_lowers() {
    let source = "def mutate_local(mut items: list[int]) -> None:\n    items.append(2)\n    return None\n\nasync def main() -> None:\n    items: list[int] = [1]\n    mutate_local(items)\n    await task.sleep(0.0)\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "completed same-task mutable borrow should not block a later await: {result:?}"
    );
}

#[test]
pub(super) fn test_lock_guard_across_await_rejected() {
    let source = "class LockGuard[T]:\n    pass\n\nasync def main() -> None:\n    guard: LockGuard[int] = LockGuard()\n    await task.sleep(0.0)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("lock guard `guard` cannot cross await")
            && e.code == Some(DiagnosticCode::OWN_BORROW_ACROSS_AWAIT)
            && e.primary_range == Some(range_for(source, "await task.sleep(0.0)"))
    }));
}

#[test]
pub(super) fn test_semaphore_permit_across_await_rejected() {
    let source = "class SemaphorePermit:\n    pass\n\nasync def main() -> None:\n    permit: SemaphorePermit = SemaphorePermit()\n    await task.sleep(0.0)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("semaphore permit `permit` cannot cross await")
            && e.code == Some(DiagnosticCode::OWN_BORROW_ACROSS_AWAIT)
            && e.primary_range == Some(range_for(source, "await task.sleep(0.0)"))
    }));
}

#[test]
pub(super) fn test_lock_guard_return_escape_rejected() {
    let source = "class LockGuard[T]:\n    pass\n\ndef make_guard() -> LockGuard[int]:\n    guard: LockGuard[int] = LockGuard()\n    return guard\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("cannot return lock guard")
            && e.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
            && e.primary_range == Some(range_for_after(source, "return ", "guard"))
    }));
}

#[test]
pub(super) fn test_semaphore_permit_return_escape_rejected() {
    let source = "class SemaphorePermit:\n    pass\n\ndef make_permit() -> SemaphorePermit:\n    permit: SemaphorePermit = SemaphorePermit()\n    return permit\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("cannot return semaphore permit")
            && e.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
            && e.primary_range == Some(range_for_after(source, "return ", "permit"))
    }));
}

#[test]
pub(super) fn test_task_timeout_consumes_handle_binding() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await task.timeout(handle, 1.0)\n        second = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "second = await ", "handle"))
    }));
}

#[test]
pub(super) fn test_task_race_consumes_handle_collection_binding() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handles = [scope.spawn(worker()), scope.spawn(worker())]\n        result = await task.race(handles)\n        second = await task.race(handles)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range
                == Some(range_for_after(
                    source,
                    "second = await task.race(",
                    "handles",
                ))
    }));
}

#[test]
pub(super) fn test_for_loop_consumes_task_handle_collection_binding() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handles = [scope.spawn(worker()), scope.spawn(worker())]\n        for handle in handles:\n            result = await handle\n        second = await task.gather(handles)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range
                == Some(range_for_after(
                    source,
                    "second = await task.gather(",
                    "handles",
                ))
    }));
}

#[test]
pub(super) fn test_failure_annotation_resolves_in_function_signature() {
    let source = "def observe(failure: Failure[ValueError]) -> None:\n    return None\n";
    let module = lower_source(source).expect("Failure annotation should lower");
    let param_ty = &module.functions[0].params[0].ty;

    assert_eq!(
        param_ty,
        &Type::Failure(Box::new(Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "ValueError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        }))
    );
    assert_eq!(param_ty.display_name(), "Failure[ValueError]");
}

#[test]
pub(super) fn test_task_select_consumes_handle_bindings() {
    let source = "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def second() -> str:\n    await task.sleep(0.0)\n    return \"two\"\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        one = scope.spawn(first())\n        two = scope.spawn(second())\n        selected = await task.select(one=one, two=two)\n        late = await one\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("moved value")
            && e.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
            && e.primary_range == Some(range_for_after(source, "late = await ", "one"))
    }));
}

#[test]
pub(super) fn test_task_timeout_context_manager_requires_timeout_error_result_for_awaits() {
    let source = "async def main() -> None:\n    async with task.timeout(1.0):\n        await task.sleep(0.0)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("must return Result[..., TimeoutError]")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
    }));
}

#[test]
pub(super) fn test_double_mutable_borrow_has_ownership_code() {
    let source = "def swap(mut a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    swap(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot borrow 'items' as mutable more than once")
            && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range == Some(range_for_after_anchor(source, "swap(items, ", "items"))
    }));
}

#[test]
pub(super) fn test_mutable_after_immutable_borrow_has_ownership_code() {
    let source = "def read_then_mutate(a: list[int], mut b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    read_then_mutate(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as mutable because it is already borrowed as immutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "read_then_mutate(items, ",
                    "items",
                ))
    }));
}

#[test]
pub(super) fn test_immutable_after_mutable_borrow_has_ownership_code() {
    let source = "def mutate_then_read(mut a: list[int], b: list[int]):\n    pass\n\ndef main():\n    items: list[int] = [1, 2, 3]\n    mutate_then_read(items, items)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains(
            "cannot borrow 'items' as immutable because it is already borrowed as mutable",
        ) && e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && e.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "mutate_then_read(items, ",
                    "items",
                ))
    }));
}

#[test]
pub(super) fn test_for_loop_move_has_ownership_code() {
    let result = lower_source(
        "def consume(own s: str) -> int:\n    return len(s)\n\ndef main():\n    s: str = \"hello\"\n    for i in range(3):\n        result: int = consume(s)\n        print(result)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("is moved inside loop body")
            && e.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)
    }));
}

#[test]
pub(super) fn test_while_loop_move_has_ownership_code() {
    let result = lower_source(
        "def consume(own s: str) -> int:\n    return len(s)\n\ndef main():\n    s: str = \"hello\"\n    i: int = 0\n    while i < 3:\n        result: int = consume(s)\n        i = i + 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("is moved inside loop body")
            && e.code == Some(DiagnosticCode::OWN_MOVED_ACROSS_LOOP)
    }));
}

#[test]
pub(super) fn test_borrow_by_default_no_move() {
    let result = lower_source(
        "def process(s: str) -> int:\n    return len(s)\ndef main():\n    s: str = \"hello\"\n    x: int = process(s)\n    print(s)\n",
    );
    assert!(
        result.is_ok(),
        "borrow-by-default should not cause use-after-move"
    );
}

#[test]
pub(super) fn test_user_defined_sum_shadows_builtin() {
    let result = lower_source(
        "def sum(num1: int, num2: int) -> int:\n    return num1 + num2\ndef main():\n    assert sum(12, 5) == 17\n",
    );
    assert!(
        result.is_ok(),
        "user-defined sum should shadow the builtin lowering path"
    );
}

#[test]
pub(super) fn test_builtin_set_constructor_accepts_list_iterable() {
    let result = lower_source("def main():\n    seen = set([1, 2, 2])\n    assert 2 in seen\n");
    assert!(
        result.is_ok(),
        "set(list[T]) should lower as a builtin constructor"
    );
}

#[test]
#[ignore = "depends on driver-loaded stdlib compat registry"]
pub(super) fn test_bare_deque_call_resolves_without_import() {
    let result = lower_source(
        "from sifr.collections import deque\n\ndef main():\n    q = deque([1])\n    q.append(2)\n    assert q.popleft() == 1\n",
    );
    assert!(
        result.is_ok(),
        "deque(...) should resolve through the compat stdlib surface: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_generic_constructor_infers_typevar_from_optional_union_param() {
    let result = lower_source(
        "class Bucket[T]:\n    items: list[T]\n\n    def __init__(self, items: list[T] | None = None):\n        if items is None:\n            self.items = []\n        else:\n            self.items = items\n\n    def first(self) -> T | None:\n        if len(self.items) == 0:\n            return None\n        return self.items[0]\n\ndef main() -> int:\n    bucket = Bucket([1])\n    value = bucket.first()\n    if value is None:\n        return 0\n    return value + 1\n",
    );
    assert!(
        result.is_ok(),
        "constructor call should infer T from list[T] | None parameter when called with list[int]: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_defaultdict_list_call_requires_explicit_import() {
    let result = lower_source("def main():\n    groups = defaultdict(list)\n");
    assert!(result.is_err(), "bare defaultdict(list) must be rejected");
}

#[test]
pub(super) fn test_defaultdict_list_call_resolves_with_explicit_import() {
    let result = lower_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef main():\n    groups = defaultdict(list)\n    groups[\"a\"].append(\"x\")\n    assert len(groups[\"a\"]) == 1\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict(list) should resolve through explicit sifr.collections import: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_defaultdict_alias_call_resolves_with_explicit_import() {
    let result = lower_source_with_stdlib_collections(
        "from sifr.collections import defaultdict as dd\n\ndef main():\n    groups = dd(set)\n    groups[\"a\"].add(1)\n    assert 1 in groups[\"a\"]\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict alias should resolve through explicit sifr.collections import: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_defaultdict_keyword_constructor_unsupported_has_stdlib_code() {
    let source = "from sifr.collections import defaultdict\n\ndef main():\n    groups = defaultdict(default_factory=list)\n    _ = groups\n";
    let result = lower_source_with_stdlib_collections(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "defaultdict() does not support keyword arguments"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for(source, "default_factory"))
    }));
}

#[test]
pub(super) fn test_defaultdict_unpacked_keyword_constructor_unsupported_has_stdlib_code() {
    let source = "from sifr.collections import defaultdict\n\ndef main():\n    groups = defaultdict(**{\"default_factory\": list})\n    _ = groups\n";
    let result = lower_source_with_stdlib_collections(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "defaultdict() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for(source, "**{\"default_factory\": list}"))
    }));
}

#[test]
pub(super) fn test_builtin_sum_wrong_arity_has_call_code() {
    let source = "def main():\n    data: list[int] = [1, 2, 3]\n    print(sum(data, data))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sum() takes exactly 1 argument(s), got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "sum(data, ", "data"))
    }));
}

#[test]
pub(super) fn test_sorted_unexpected_keyword_has_call_code() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, bogus=True)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sorted() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "sorted(nums, ", "bogus"))
    }));
}

#[test]
pub(super) fn test_sorted_and_range_missing_required_argument_have_call_code() {
    let sorted_source = "def main():\n    values: list[int] = sorted()\n";
    let sorted_result = lower_source(sorted_source);
    assert!(sorted_result.is_err());
    let sorted_errors = sorted_result.unwrap_err();
    assert!(sorted_errors.iter().any(|error| {
        error.message == "sorted() missing required argument 'iterable'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(sorted_source, "sorted"))
    }));

    let range_source = "def main():\n    values: list[int] = list(range())\n";
    let range_result = lower_source(range_source);
    assert!(range_result.is_err());
    let range_errors = range_result.unwrap_err();
    assert!(range_errors.iter().any(|error| {
        error.message == "range() missing required argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(range_source, "range"))
    }));
}

#[test]
pub(super) fn test_function_unexpected_keyword_has_call_code() {
    let source = "def greet(name: str) -> str:\n    return \"hello\"\n\ndef main():\n    print(greet(\"Alice\", punctuation=\"!\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got an unexpected keyword argument 'punctuation'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "greet(\"Alice\", ",
                    "punctuation",
                ))
    }));
}

#[test]
pub(super) fn test_keyword_after_positional_has_call_code() {
    let source = "def greet(name: str, greeting: str) -> str:\n    return greeting\n\ndef main():\n    print(greet(\"Alice\", name=\"Bob\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got multiple values for argument 'name'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(source, "greet(\"Alice\", ", "name"))
    }));
}

#[test]
pub(super) fn test_range_duplicate_stop_keyword_has_call_code() {
    let source = "def main():\n    print(list(range(10, stop=20)))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "range() got multiple values for argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "range(10, ", "stop"))
    }));
}

#[test]
pub(super) fn test_map_callable_arity_mismatch_has_call_code() {
    let source = "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    values: list[int] = map(inc, [1, 2], [3, 4])\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "map() callable expects 1 argument(s), got 2 iterable(s)"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "map(inc, [1, 2], ",
                    "[3, 4]",
                ))
    }));
}

#[test]
pub(super) fn test_non_simple_call_target_has_call_code() {
    let source = "def make() -> int:\n    return 1\n\ndef main():\n    value: int = make()(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "only simple function calls are supported"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(source, "value: int = ", "make()"))
    }));
}

#[test]
pub(super) fn test_open_missing_path_has_call_code() {
    let source = "def main():\n    _file = open()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "open() requires at least 1 argument: open(path) or open(path, mode)"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(source, "open"))
    }));
}
