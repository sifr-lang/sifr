Review summary below.

**RESULT: PASS**

## Verification Walkthrough

**1. Capture summary scoping (`capture_collection.rs`, `state_collection.rs`, `statement_dispatch.rs`):**
Captures are computed inside `infer_nested_function_types` against the post-convergence environment (`state_collection.rs:145-150`), then push/pop balanced via `push_nested_function_captures` / `restore_nested_function_captures` around `lower_stmts` (`statement_dispatch.rs:110-163`). Save/restore is per-name with `Option<Vec<…>>`, so shadowing across sibling/nested blocks is correctly unwound. `collect_referenced_names_in_stmts` and `collect_nonlocal_names` both stop at `Stmt::FunctionDef`, so deeper nested-function references aren't merged into the outer function's captures. Module-level lowering goes through `lower_function` → `lower_stmts`, so each function body has an isolated push/pop frame; no leak between siblings.

**2. Diagnostic honesty (`offload_worker_captures.rs:23-43`):**
Non-send captures → `OWN_NON_SEND_TASK_CAPTURE` (SIFR-OWN-0010). Sendable nested captures → `expression_diagnostics::type_mismatch` (SIFR-TYPE-0002) with the explicit "does not accept nested worker functions with captures yet" wording. Matches the phase doc rule and the new fail-fixture expectations. The `captures.is_empty()` guard is effectively dead because `collect_nested_function_captures` only inserts non-empty entries, but it's harmless defensive code.

**3. Top-level named workers unaffected:**
Validator early-returns `Some(())` when (a) the worker arg is not `HirExpr::Name` or (b) the name is not in `nested_function_captures`. Top-level `def`s are never inserted into that map (only nested defs are walked), so the existing top-level pass fixtures (e.g. `spawn_blocking_basic`, `parallel_map_basic`, scoped/JoinSet pass fixtures) keep their behavior. I spot-checked the wiring at `parallel_calls.rs:160`, `task_calls.rs:128`, `task_join_set_calls.rs:294`, `task_scope_offload_calls.rs:132`, and `blocking_executor_calls.rs:46`; all invoke the validator before the `Type::Function` extraction.

**4. Coverage of M3 worker boundaries (review contract #4):**
- ✅ `parallel.map`/`try_map`, `Pool.map`/`try_map` (shared `validate_parallel_map_like_call`)
- ✅ `ThreadPoolExecutor.submit`
- ✅ `task.spawn_cpu`
- ✅ `JoinSet.spawn_blocking`/`spawn_cpu`
- ✅ scoped `scope`/`group.spawn_blocking`/`spawn_cpu`
- ⚠️ **`task.spawn_blocking` is *not* wired through the shared validator** (`task_calls.rs:223-322`). I reproduced: passing a nested worker (sendable or non-send capture) yields `SIFR-TYPE-0002: task.spawn_blocking() requires a sync function argument, got 'Callable[[], int]'`. Safety is preserved (the boundary still rejects), but the diagnostic does not match the phase-doc's "compile-time ownership diagnostics" framing for non-send captures, and there's no `task_spawn_blocking_non_send_capture_rejected` fixture. **Non-blocking** since rejection still happens at compile time and `task.spawn_blocking` is not explicitly enumerated in the M3 traceability surface table or in this wave's stated wiring scope.

**5. Docs/traceability honesty:**
- Phase doc edit (line 606) precisely restates the v1 rule and the explicit deferral until function-value capture ABI exists — no overclaim.
- Traceability surface rows for `parallel.map`/`try_map` and `task.spawn_cpu` add the two/one new capture fixtures and the explanatory wording in Notes. Fail-suite row adds all four new fixtures. The "Closure capture sendability diagnostics beyond item/output/error sendability" follow-up is appropriately removed since the v1 rule it described is now realized.
- Other surfaces (`TaskScope`/`TaskGroup` scoped offload, `task.JoinSet`) intentionally don't claim capture-rule fixtures because none were added; the shared validator code path makes the rule structurally uniform, which is acceptable evidence.

## Non-blocking polish (not required for merge)

1. `task.spawn_blocking()` in `task_calls.rs:249-262` should call `validate_offload_worker_captures` before the `Type::Function` extraction so the user-visible diagnostic for non-send nested captures is SIFR-OWN-0010 instead of a generic "got 'Callable[[], int]'". A symmetric fixture (`spawn_blocking_non_send_capture_rejected.sifr`) would complete the M3 v1 capture-rule coverage matrix.
2. Optional: drop the unreachable `if captures.is_empty()` arm in `offload_worker_captures.rs:20-22`, since `collect_nested_function_captures` already filters empty capture vectors.

Branch is otherwise clean: untouched network-phase docs/reviews are correctly out of scope.
