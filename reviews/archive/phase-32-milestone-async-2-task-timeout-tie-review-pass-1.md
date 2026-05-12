

---

## Review: Phase 32 / milestone_async_2 — task-handle timeout same-tick completion priority

### Verdict: **SATISFIED**

### Findings

**No blockers.** The slice is correct, focused, and properly validates the intended behavior.

**1. Codegen test assertion is sound** (`lib_codegen_tests.rs:3790`)
- `assert!(result.rust_source.contains("biased;"))` correctly verifies the Tokio select uses biased polling order, giving inner completion priority over the sleep timer when both are ready in the same tick.
- The assertion is placed in `test_task_timeout_handle_lowers_to_private_timeout_result`, which exercises `task.timeout(handle, 1.0)` with a spawned worker — the same semantic path as the e2e fixture.

**2. E2E fixture is meaningful and non-misleading** (`task_timeout_completion_wins_tie.sifr`)
- `scope.spawn(worker())` creates a task that completes in zero ticks (no await points in `worker`).
- `task.timeout(handle, 0.0)` races a same-tick sleep against the inner completion.
- Without `biased;`, the Tokio scheduler could return either branch nondeterministically on same-tick readiness. With `biased;`, the generated code always takes inner completion first. The fixture runs to completion without assertion failure, confirming correct behavior.
- The fixture exercises the correct public surface: `task.timeout(handle, duration)` from `task.scope()`.

**3. Phase doc update is accurate** (`32_async_ecosystem.md:401`)
- "same-tick timeout validation coverage with `task_timeout_completion_wins_tie.sifr`; the generated timeout race uses biased completion-first selection" correctly describes what the slice adds.

**4. Design contract is correctly locked**
- `async_concurrency_model.md:381` and `545`: "If the deadline and inner completion become ready in the same scheduler tick, inner completion wins."
- The generated `tokio::select! { biased; result = &mut receiver => {...}, _ = tokio::time::sleep(duration) => {...} }` in `preamble.rs:316-317` is the deterministic implementation of this contract.

### Required fixes

None.

### Test/validation gaps

None identified. The slice has both:
- A unit codegen assertion that directly inspects the generated Rust source for `biased;`.
- An e2e pass fixture that exercises the public API and runs to successful completion, confirming no panics or runtime errors.

### Notes on phase/design alignment

- The milestone scope at `32_async_ecosystem.md:319-324` defines "same scheduler tick gives inner completion priority" as a required behavior. This slice adds the validation that locks it.
- The slice correctly does **not** change runtime semantics — the `biased;` directive was already present in the timeout codegen from its initial implementation. This slice's purpose is validation coverage, not behavior change.
