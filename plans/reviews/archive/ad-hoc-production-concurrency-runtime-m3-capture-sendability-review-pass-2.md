Verified pass-1 polish addressed. Walkthrough below.

**RESULT: PASS**

## Pass-2 verification

**1. Polish item #1 (shared validator for `task.spawn_blocking()`):** Now wired at `crates/sifr_lowering/src/lower/task_calls.rs:253-262`, sitting between `lower_expr` and the `Type::Function` extraction — same call shape as `task.spawn_cpu()` at line 128 and as the four other worker boundaries. Confirmed via grep: all six boundaries (`parallel_calls.rs:160`, `task_calls.rs:128`, `task_calls.rs:253`, `task_join_set_calls.rs:294`, `task_scope_offload_calls.rs:132`, `blocking_executor_calls.rs:46`) route through `validate_offload_worker_captures`. No M3 worker boundary that accepts a named sync worker is missing the check.

**2. New fail fixture (`spawn_blocking_non_send_capture_rejected.sifr`):** Mirrors `spawn_cpu_non_send_capture_rejected.sifr` exactly — same `LocalCell(NonSend)` shape, same nested-function pattern with `assert cell is not None`, swapping `@cpu_heavy` → `@blocking_io` and `task.spawn_cpu` → `task.spawn_blocking`. `@blocking_io` is allowed for `task.spawn_blocking` (per the existing `spawn_blocking_blocking_io_allowed` pass fixture), so classification passes and we land on the SIFR-OWN-0010 path. Fixture header `# expect-error: SIFR-OWN-0010` matches the diagnostic emitted by the validator at `offload_worker_captures.rs:25-31`.

**3. Capture summary scoping (unchanged from pass 1):** `infer_nested_function_types` still computes captures against the post-convergence environment and `push_nested_function_captures` / `restore_nested_function_captures` are still balanced around `lower_stmts` (`statement_dispatch.rs:110-194`). `Option<Vec<…>>` save/restore preserves shadowing. No regressions; the only change in `task_calls.rs` is the validator insertion — control flow for top-level named workers is preserved by the validator's early-return when the name is absent from `nested_function_captures`.

**4. No new issue introduced:**
- Order of checks for `task.spawn_blocking()`: arity/keyword guards → `lower_expr` → capture validator → `Type::Function` → arity-of-params → workload annotation → send/non-send return-type checks. This is symmetric with `task.spawn_cpu()` and won't accidentally suppress the existing `spawn_blocking_non_send_rejected` (top-level `build_cell` returning `LocalCell`), since that fixture's worker is a top-level def that doesn't appear in `nested_function_captures` — the validator returns `Some(())` and the existing return-type non-send check still fires SIFR-TYPE-0002. Verified by the 415 fail tests still passing.
- `captures.is_empty()` arm at `offload_worker_captures.rs:20-22` is still present (polish item #2 was explicitly optional and remains harmless).

**5. Docs/traceability honesty:**
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:14` adds `spawn_blocking_non_send_capture_rejected` to the `task.spawn_blocking` row and appends "and rejects non-send nested worker captures at the offload boundary" — accurate, matches the new code path. Fail-suite row at line 34 includes the fixture in correct lexicographic position between `spawn_blocking_non_send_rejected` and `spawn_blocking_unannotated_rejected`.
- No claim was added about supporting *sendable* nested captures; the existing `parallel_map_send_capture_deferred_rejected` row and the v1 deferral wording on phase doc line 606 still correctly say nested workers with captures aren't accepted yet. No overclaim.

Branch validation (per user): fmt, lowering check, file-size + HIR guardrails, fixture-specific check (SIFR-OWN-0010), and full fail suite (415 fixtures) all green. Ready to merge.
