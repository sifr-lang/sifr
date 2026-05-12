# Review: Ad Hoc Async Effect and Offload Diagnostics

**Reviewer:** Claude Opus 4.7
**Date:** 2026-05-12
**Files reviewed:**
- `internal_docs/async_concurrency_model.md` (async suspension model, diagnostic contract)
- `internal_docs/phases/32_async_ecosystem.md` (Phase 32 closure + ad hoc reference)
- `internal_docs/roadmap.md` (Phase 32.1 entry)
- `issues/ad-hoc-async-effect-and-offload-diagnostics.md` (design intent)
- `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` (checklist)

**Implementation baseline:** Phase 32 (completed 2026-05-11), current HIR has `@io_bound`/`@cpu_bound` warnings via `LoweringWarningDiagnostic::BlockingWorkInAsync` + `SIFR-TYPE-0903`, `spawn_blocking` rejects non-send returns but does not validate workload classification.

---

## Verdict: Conditionally Approved — Three Targeted Edits Required

The ad hoc phase design is sound, Sifr-compatible, and implementation-ready. The model semantics are correctly layered across `async_concurrency_model.md` and the issue docs. Three inconsistencies between the current implementation baseline and the ad hoc requirements must be resolved before implementation begins. All three are surface-level — no structural redesign needed.

---

## Summary of Changes Made

1. **`internal_docs/roadmap.md`**: Added Phase 32.1 entry to the execution table with correct phase files and a precise unlock description. Removed the stale incorrect entry.

2. **`internal_docs/phases/32_async_ecosystem.md`**: Added the "Corrective follow-up" reference sentence and Phase 32.1 entry to the "Locked V1 Decisions" table (decision 31: async effect discipline rules).

3. **`internal_docs/async_concurrency_model.md`**: Added "Async Effect Discipline" as a top-level section in the canonical async model, with explicit `SIFR-ASYNC-dddd` diagnostic codes for the new ad hoc diagnostics. Added model invariants 15-18.

4. **`issues/ad-hoc-async-effect-and-offload-diagnostics.md`**: Corrected execution document wording from ambiguous transition wording to "change from warning to error" (matching the current HIR baseline). Added clarification that stdlib/FFI classification is checked against the compiler's stdlib annotation database and FFI contract registry.

5. **`issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md`**: Added note about existing fixture updates to the Review Notes section.

---

## Specific Questions Answered

### 1. Is the internal suspension summary concept sufficient?

**Yes.** The seven categories (`NoSuspend`, `AsyncIo`, `TimerWait`, `ChannelWait`, `TaskWait`, `AsyncResourceWait`, `GeneratorSuspend`) cover the full surface. Key design decisions that make it work:

- `TaskWait` covers awaiting task handles, blocking task handles, composition APIs (`gather`/`select`/`race`), scope/group cleanup, and same-task coroutines with a non-empty summary. The spec explicitly says same-task coroutines with transitive `NoSuspend` are rejected, keeping the two concerns (suspension fact vs. diagnostic rule) separate.
- `AsyncResourceWait` covers async context-manager enter/exit, async iterator `anext`, and async cleanup — a clean bucket for resource-lifetime operations.
- `GeneratorSuspend` correctly covers the async generator case without needing a separate bucket.
- The exact enum names are internal implementation details; only the semantic rule (async code must suspend for a real reason) is public.

**Remaining concern:** The spec says `TaskWait` includes "another async function with a non-empty suspension summary." There is no "call graph" definition for same-task coroutines vs. spawned coroutines. Implementation must track whether a coroutine is awaited in the same task (`await foo()`) vs. spawned into a child task (`scope.spawn(foo())`). Same-task awaiting is the only place where `NoSuspend` rejection matters. This is probably already in the HIR through the existing `Coroutine` type handling, but the spec does not explicitly call out the same-task distinction. **Recommendation:** Implementation should reuse the existing same-task coroutine handling in the HIR (`current_function_is_async` + coroutine await lowering) and not introduce a new "call graph" concept.

### 2. Is rejecting `async def` with `NoSuspend` too strict for protocol implementations, tests, stubs, or compatibility veneers?

**The explicit escape hatch is correct and sufficient.** The spec requires "an explicit reviewed escape hatch with a reason-bearing annotation." This is intentionally stricter than a warning-by-default path — the goal is to prevent silent fake async habits.

The spec correctly handles:
- **Protocol implementations**: async protocol methods may have no current suspension (they implement the shape, not the body). The escape hatch is the right path.
- **Test stubs**: `@test_marker` or similar reasons allow fake async in test-only code.
- **Compatibility veneers**: `sifr.asyncio` wrappers that delegate to canonical APIs but preserve async shape.

The only risk is implementation burden: the escape hatch requires a reason-bearing annotation and a reviewed opt-in mechanism. This is more work than a warning, which is the correct tradeoff — users who need fake async for protocol reasons must do so intentionally.

**No change needed.**

### 3. Should direct `@io_bound`/`@cpu_bound` calls in async code be errors?

**Yes — the upgrade from warning to error is correct.** The rationale:

- Warnings on deliberately annotated code are insufficient. `@io_bound` and `@cpu_bound` are explicit user declarations. A warning after an explicit annotation sends a confusing signal.
- Direct blocking I/O calls in async code can block the runtime worker and degrade concurrent throughput. This is not a style issue — it is a correctness issue.
- The upgrade requires an explicit migration path: users must choose `spawn_blocking` or `task.spawn_blocking` for offload. This is the intended migration.
- "Cheap sync helpers" remain allowed because they are unannotated. The diagnostic targets only explicitly classified blocking/CPU-heavy work.

**Implementation concern to flag:** The current Phase 32 positive fixtures `io_bound_annotation_warning.sifr` and `cpu_bound_annotation_warning.sifr` pass today because the annotation produces a warning, not an error. After this phase lands, these fixtures will become errors unless updated. The spec says "existing Phase 32 positive async fixtures remain valid unless they intentionally covered now-rejected fake async/offload behavior." Both fixtures intentionally covered the exact behavior being upgraded from warning to error — they are negative fixtures by design and must be updated. The execution document's Review Notes now reflects this.

### 4. Should `spawn_blocking` accept both `@io_bound` and `@cpu_bound`?

**Yes.** Both annotations are explicit workload classification markers, and both are valid targets for blocking offload:
- `@io_bound` for I/O: the canonical case for `spawn_blocking`. No info diagnostic by default.
- `@cpu_bound` for CPU-heavy: valid, though users are encouraged to prefer `ThreadPoolExecutor` for sustained CPU work.

The "no info diagnostic for valid `@io_bound` offload" default is the right choice. A later optional info diagnostic may suggest a native async replacement only when the compiler knows a specific one. This avoids noisy diagnostics for intentional, correct offload.

**One clarification needed:** The spec says `spawn_blocking` is accepted for "external/FFI contracts as blocking or CPU-heavy." This means the compiler must have a registry of FFI/external contracts that classify functions. This registry does not exist in the current Phase 32 implementation. Implementation should document the registry interface and note it as a deferred infrastructure item. For now, unannotated functions are rejected. **Added clarification to the ad hoc issue.**

### 5. Should unannotated local sync functions be rejected by `spawn_blocking`?

**Yes — and this is the strongest new invariant.** The current Phase 32 implementation (`lower_task_spawn_blocking_call` and `lower_thread_pool_submit_call`) only rejects non-send return types. It does not validate workload classification.

The ad hoc requirement closes this gap:
- Unannotated local sync functions: rejected with a diagnostic suggesting either calling the helper directly (if cheap) or adding `@io_bound`/`@cpu_bound` (if genuinely blocking/CPU-heavy).
- Annotated functions (`@io_bound`, `@cpu_bound`): allowed.
- Stdlib-known functions (in the compiler's stdlib annotation database): allowed.
- External-contract-classified functions: allowed.

The rationale is compelling: if a sync helper is cheap, call it directly — offloading cheap work adds thread-switch overhead and defeats the purpose of structured concurrency. If it is expensive, classify it so the compiler can help.

### 6. Are roadmap/phase links and validation fixture names sufficient?

**Roadmap and phase links: correct.** The `roadmap.md` execution table now correctly references Phase 32.1 with the two issue files. Phase 32's "Corrective follow-up" sentence and the new "Locked V1 Decision 31" in `32_async_ecosystem.md` are consistent.

**Validation fixture names: sufficient, but existing fixtures need updates.** The 17 new fixture names are descriptive and follow the existing naming convention. However, the existing Phase 32 fixtures `io_bound_annotation_warning.sifr` and `cpu_bound_annotation_warning.sifr` (currently in the quick lane) will need to be updated or removed, because they test the exact behavior being upgraded from warning to error. The execution document now notes this.

The existing fixture `spawn_blocking_basic.sifr` passes an unannotated sync function to `task.spawn_blocking`. Under the new rules, this is rejected — the fixture needs an update (add `@io_bound` to the helper function, or move the test to a negative fixture).

---

## Implementation Traps and Edge Cases

### Trap 1: Async comprehensions and suspension summary

Async comprehensions `[item async for item in source]` are protocol sugar over `async for`. The suspension summary should be inherited from the `async for` desugaring. Implementation must ensure async comprehensions are not analyzed as a separate syntactic construct with a separate summary computation — they must flow through the same summary logic as manual `async for`.

### Trap 2: `async with` and `async for` in the summary

`async with` blocks and `async for` loops introduce await points at their protocol boundaries. Implementation must include these in the suspension summary computation:
- `async with resource as x` awaits `__aenter__` and `__aexit__`.
- `async for item in source` awaits `anext()` repeatedly.

These are covered by `AsyncResourceWait` (enter/exit) and `ChannelWait` (iteration) respectively. Implementation should verify the HIR lowering captures these await points when computing summaries.

### Trap 3: `await anext()` and `await agen.aclose()`

These are explicit async calls on async generators and iterators. They should contribute to the suspension summary through the same mechanism as any other await expression. Implementation must ensure `anext()` and `aclose()` are in the stdlib annotation database as having async suspension effects.

### Trap 4: Nested async functions and transitive summary

When an `async def` A calls `await` on a same-task coroutine from `async def` B, the summary of A depends on the summary of B. Implementation must track the call graph of same-task coroutine awaits and compute the transitive summary. This is not a graph traversal — it is a simple recursive check through the existing coroutine type system.

### Trap 5: Mixing sync and async work in one function

The model says "cheap sync helper calls inside async code remain allowed." But if an async function calls a mix of cheap helpers and annotated `@io_bound` work, the annotated call is the one that triggers the error. The cheap helpers are not flagged. This is correct — the diagnostic targets only explicitly annotated blocking/CPU-heavy work.

### Trap 6: `async def` that calls `spawn_blocking` only

Consider:
```sifr
async def offload_only() -> int:
    result = await task.spawn_blocking(compute)
    return result
```

This function's suspension summary is `TaskWait` (it awaits a `BlockingTask`). It is valid. The spec correctly handles this because awaiting a `BlockingTask` counts as an awaitable with a non-empty suspension effect.

### Trap 7: Protocol escape hatch interaction with type checking

The escape hatch for async protocol conformance must not break the existing protocol conformance checking. If a user adds the escape hatch annotation to an async method with `NoSuspend`, the type checker should still verify protocol shape requirements. Implementation must separate the suspension summary check from the protocol conformance check.

---

## Remaining Concerns

### Concern 1 (Informational): No FFI/contract registry in current implementation

The ad hoc spec says `spawn_blocking` accepts functions "known by an external/FFI contract as blocking or CPU-heavy." The current Phase 32 implementation has no FFI contract registry. Implementation should add the registry interface to `sifr_hir` with a deferred positive validation fixture, rather than blocking the whole phase. Unannotated functions remain rejected until the registry is populated. **Clarification added to the ad hoc issue.**

### Concern 2 (Informational): Exit-code behavior for warnings vs. errors

The current HIR emits `LoweringWarningDiagnostic::BlockingWorkInAsync` for `@io_bound`/`@cpu_bound` calls, which maps to `SIFR-TYPE-0903` (Severity::Warning). After this phase lands, these should become `SIFR-ASYNC-0001` (Severity::Error). Implementation must ensure the exit-code handling in `sifr_driver` correctly propagates errors and does not let the new errors become non-fatal. This is likely already the case (HIR errors are errors), but the diagnostic code migration needs to be explicit in the implementation PR.

### Concern 3 (Low priority): Async generator reentrant `anext()` is runtime-deferred

The ad hoc spec defers the reentrant `anext()` case (multiple concurrent `anext()` calls on the same generator) to runtime. The current Phase 32 implementation has `SIFR-OWN-0002` for HIR-level rejection but defers the runtime `GeneratorBusyError` case. This is consistent with the spec. No change needed.

---

## More Review Needed?

**No.** This phase is ready for implementation subject to the edits above. A follow-up implementation review should be conducted after the first PR lands to verify:
1. The suspension summary computation correctly handles all seven categories.
2. The diagnostic codes `SIFR-ASYNC-0001` through `SIFR-ASYNC-0004` are registered and documented.
3. The existing Phase 32 fixtures that now test rejected behavior are updated.
4. The exit-code behavior for the new error diagnostics is correct.

---

## Changes Made to Docs

| File | Change |
|------|--------|
| `internal_docs/roadmap.md` | Added Phase 32.1 entry to execution table (corrected from incorrect prior entry) |
| `internal_docs/phases/32_async_ecosystem.md` | Added "Corrective follow-up" sentence and Phase 32.1 entry to "Locked V1 Decisions" table |
| `internal_docs/async_concurrency_model.md` | Promoted "Async Effect Discipline" to a top-level section; added explicit `SIFR-ASYNC-dddd` diagnostic codes; added model invariants 15-18 |
| `issues/ad-hoc-async-effect-and-offload-diagnostics.md` | Corrected ambiguous transition wording to "change from warning to error"; added FFI/stdlib classification note |
| `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` | Added Review Notes entry about existing fixture updates |
