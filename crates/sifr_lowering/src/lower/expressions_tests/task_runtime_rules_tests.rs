use super::*;
use crate::HirAsyncWithKind;

#[test]
fn test_task_group_accepts_reserved_none_context() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup(ctx=None) as group:\n        handle = group.spawn(worker())\n        result = await handle\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "reserved TaskGroup ctx=None should lower: {result:?}"
    );
}

#[test]
fn test_task_group_accepts_sifr_context() {
    let source = "class Context:\n    name: str\n\n    def __init__(self, name: str):\n        self.name = name\n\nasync def main() -> Result[None, ScopeFailure]:\n    ctx: Context = Context(\"request\")\n    async with task.TaskGroup(ctx=ctx) as group:\n        pass\n    return None\n";
    let module = lower_source(source).expect("TaskGroup ctx=Context should lower");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    let HirStmt::AsyncWith { kind, .. } = &main.body[1] else {
        panic!("expected task group async with");
    };
    assert!(matches!(
        kind,
        HirAsyncWithKind::TaskGroup {
            context: Some(HirExpr::Name { name, .. })
        } if name == "ctx"
    ));
}

#[test]
fn test_task_group_rejects_invalid_context_type() {
    let source = "class Context:\n    pass\n\nasync def main() -> Result[None, ScopeFailure]:\n    ctx: Context = Context()\n    async with task.TaskGroup(ctx=ctx) as group:\n        return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.TaskGroup() ctx must be sifr.task.Context or None")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
    }));
}

#[test]
fn test_task_spawn_scoped_lowers_through_named_owner_with_reserved_none_context() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup(ctx=None) as group:\n        handle = task.spawn_scoped(worker(), ctx=None)\n        result = await handle\n    return None\n";
    let module = lower_source(source).expect("spawn_scoped with ctx=None should lower");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    let HirStmt::AsyncWith { body, .. } = &main.body[0] else {
        panic!("expected task group async with");
    };
    let handle_assignment = body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "handle"))
        .expect("handle assignment should exist");
    let HirStmt::Let { value, .. } = handle_assignment else {
        panic!("expected handle let statement");
    };
    let HirExpr::MethodCall {
        object, method, ty, ..
    } = value
    else {
        panic!("expected spawn_scoped to lower as owner method call");
    };
    assert_eq!(method, "__sifr_spawn_infallible");
    assert!(matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "group"));
    assert!(
        matches!(ty, Type::Task(ok, err) if matches!(ok.as_ref(), Type::Int) && matches!(err.as_ref(), Type::Never))
    );
}

#[test]
fn test_task_spawn_scoped_rejects_without_active_owner() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_scoped(worker())\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.spawn_scoped() requires an active structured task owner")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
    }));
}

#[test]
fn test_task_spawn_scoped_lowers_with_sifr_context() {
    let source = "class Context:\n    name: str\n\n    def __init__(self, name: str):\n        self.name = name\n\nasync def worker() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    ctx: Context = Context(\"request\")\n    async with task.TaskGroup() as group:\n        handle = task.spawn_scoped(worker(), ctx=ctx)\n    return None\n";
    let module = lower_source(source).expect("spawn_scoped ctx=Context should lower");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    let HirStmt::AsyncWith { body, .. } = &main.body[1] else {
        panic!("expected task group async with");
    };
    let HirStmt::Let { value, .. } = &body[0] else {
        panic!("expected handle assignment");
    };
    let HirExpr::MethodCall {
        method,
        args,
        source: Some(call_source),
        ..
    } = value
    else {
        panic!("expected spawn_scoped to lower as owner method call");
    };
    assert_eq!(method, "__sifr_spawn_infallible_with_context");
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0], HirExpr::Name { name, .. } if name == "ctx"));
    assert_eq!(u32::from(call_source.arg_ranges[0].len()), 3);
    assert_eq!(u32::from(call_source.arg_ranges[1].len()), 8);
}

#[test]
fn test_task_spawn_scoped_requires_named_owner() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup():\n        handle = task.spawn_scoped(worker())\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.spawn_scoped() requires a named active task owner")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
    }));
}

#[test]
fn test_sequential_same_name_task_groups_do_not_share_error_type_state() {
    let source = "async def value_child() -> Result[int, ValueError]:\n    await task.sleep(0.0)\n    raise ValueError(\"value\")\n\nasync def io_child() -> Result[int, IOError]:\n    await task.sleep(0.0)\n    raise IOError(\"io\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup() as group:\n        first = group.spawn(value_child())\n    async with task.TaskGroup() as group:\n        second = group.spawn(io_child())\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "sequential same-name task groups should not inherit error type state: {result:?}"
    );
}

#[test]
fn test_sequential_same_name_task_groups_do_not_share_open_state() {
    let source = "async def worker() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup() as group:\n        first = group.spawn(worker())\n        result = await first\n    async with task.TaskGroup() as group:\n        second = group.spawn(worker())\n    return None\n";
    let result = lower_source(source);
    assert!(
        result.is_ok(),
        "sequential same-name task groups should not inherit open-state taint: {result:?}"
    );
}

#[test]
fn test_task_select_rejects_positional_branches() {
    let source = "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def second() -> int:\n    await task.sleep(0.0)\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        one = scope.spawn(first())\n        two = scope.spawn(second())\n        selected = await task.select(one, two)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.select() takes named task branches")
            && e.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
    }));
}

#[test]
fn test_task_select_rejects_single_named_branch() {
    let source = "async def first() -> int:\n    await task.sleep(0.0)\n    return 1\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        one = scope.spawn(first())\n        selected = await task.select(first=one)\n    return None\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("task.select() takes exactly two named task branches")
            && e.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
    }));
}
