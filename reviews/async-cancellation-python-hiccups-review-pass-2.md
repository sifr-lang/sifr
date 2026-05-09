# Async Cancellation Review — Pass 2

**Reviewer:** Claude Code (coherence pass)
**Date:** 2026-05-09
**Scope:** Cancellation semantics only; Python asyncio hiccup surface area; coherence of B1/B2/B3 resolutions from pass 1.

---

**Verdict: Blockers found**

The model is substantially improved. All three pass-1 blockers have been addressed at the **intent** level, and most resolution text is precise. However, one resolution introduces a mechanical gap that must be closed before implementation: the exclusion of `CancellationError` from `except Error` matching is asserted but its type-system mechanics are underspecified.

---

## B1: CancellationError exclusion from broad except Error — mechanical gap

**File:** `internal_docs/async_concurrency_model.md`
- "Typed Failure and Cancellation" (lines 145–153)
- "Lock task result semantics" (lines 376–385)
- "Define cancellation policy" (lines 390–399)

**File:** `internal_docs/architecture.md`
- Error type table (lines 523–525)

**What pass 1 asked:** Clarify that cancellation is scope-exit semantics and is not routed to `except Error as e` handlers. Define the mechanism.

**What the revision says:** "`CancellationError` is excluded from broad `except Error` matching... it is the only built-in exception to the ordinary error-subclass catch-all rule" (async_concurrency_model.md, line 152).

**What is missing:** The document never says HOW this exclusion works in the type system. `CancellationError` is still listed in the error type table as a subclass of `Error`:

> `CancellationError | Error | message: str | Materialized task-boundary evidence for a cancelled child task; active cancellation of the current task bypasses ordinary except Error`

Listing `CancellationError` as inheriting from `Error` while also asserting it is excluded from `except Error` matching creates a logical contradiction. If `CancellationError` subclasses `Error`, then `except Error as e` must catch it under the standard subclass-matching rules unless there is a special-casing mechanism. The mechanism is never described.

The resolution has two moving parts that are conflated:

1. **Active cancellation of the current task** bypasses the task's own `try`/`except` blocks and runs cleanup directly. This is the clean Rust/cancellation-token model. It is correct and well-specified.

2. **Materialized `CancellationError` from awaiting a child** — the `Result[T, E | CancellationError]` produced by `await Task[T, E]` — is described as "excluded from broad `except Error`." But if it enters a `try` block as a regular `Result` value and `CancellationError` subclasses `Error`, the standard exhaustiveness check should treat it as covered by `except Error as e`. The exclusion needs a mechanical rule in the type system: either `CancellationError` does NOT inherit from `Error` (change the error type table), or the auto-unwrap logic has a hardcoded special case, or the exhaustiveness checker treats `CancellationError` as an additional error branch that is never covered by `except Error`.

**Required fix — choose one and document it:**

- **Option A (cleanest):** `CancellationError` does not inherit from `Error`. Remove it from the error type table's parent column. Give it no parent or a structural parent that is not the `Error` root. This makes `except Error as e` never catch `CancellationError` naturally, and no special-case is needed in the type system or codegen. The type table line becomes: `CancellationError | — | message: str | ...`

- **Option B:** The auto-unwrap / exhaustiveness checker has a hardcoded rule: `CancellationError` is never considered "covered" by `except Error as e`. This must be stated explicitly in the "Lock task result semantics" section and implemented in the type checker milestone (milestone_async_1).

- **Option C:** Cancellation is handled via a separate control-flow path that never enters `try`/`except` at all. The await of a cancelled child produces a value that is NOT routed through the `Result` auto-unwrap path but instead propagates as a distinct scope-level signal. This requires more architectural specification.

**Option A is recommended.** It makes the type system self-consistent: `CancellationError` is a sibling of `Error`, not a child. It has its own separate catch path (`except task.CancellationError as e`), which matches the document's own intent. The error type table must be updated accordingly.

The fix must be applied in both documents:

1. `internal_docs/async_concurrency_model.md` — in "Lock task result semantics" (around line 380): change "CancellationError is excluded from broad except Error" to "CancellationError is not a subclass of Error and is therefore never matched by except Error as e."

2. `internal_docs/architecture.md` — error type table: remove `CancellationError` from the `Error` subclass column. Add a note: "CancellationError is not a subclass of Error; it has no parent and is never matched by except Error as e."

---

## B2: task.timeout race semantics — satisfied

**File:** `internal_docs/async_concurrency_model.md`, milestone_async_2 work items (lines 508–513)

All four race cases are defined:
- inner completes before duration → returns `Ok(result)`, inner not cancelled
- duration expires first → inner cancelled, returns `Err(TimeoutError)`
- same scheduler tick → inner completion wins (deterministic tie-breaking)
- outer scope cancelled → inner cancelled unconditionally

The `task_timeout_completion_wins_tie.sifr` fixture name in the positive validation confirms the tie-breaking is testable. No blockers.

---

## B3: Orphaned task handle semantics — satisfied with one clarification needed

**File:** `internal_docs/async_concurrency_model.md`, milestone_async_3 work items (lines 564–569)

The orphaned task handle policy is correctly specified:
- handle must be awaited, joined, cancelled, or moved into a tracked collection
- unconsumed handle at end-of-scope is a compile-time diagnostic
- `TaskScope.__aexit__` cancels and awaits remaining children as runtime safety backstop

The "moved into a tracked collection" exception for edge cases (handle stored in a collection, branching on `None`) is sound. However, the spec says "moved into a tracked collection that is consumed before scope exit" — it does not define what "tracked" means or how the compiler verifies the collection is consumed.

**Non-blocking clarification (R1):** Add one sentence to milestone_async_3 "orphaned task-handle rules": "A handle moved into a collection is considered tracked if the collection itself is consumed (drained, dropped, or its containing value goes out of scope) before the scope exits; the compiler verifies this through lifetime analysis."

This is refinement, not a blocker — the compile-time diagnostic and runtime backstop cover the common case. The edge case of tracked collections is an implementation detail for milestone_async_3 to resolve.

---

## Other cancellation-adjacent gaps (non-blocking)

**R2: SecondaryError must be documented in architecture.md Error type table — satisfied**

`SecondaryError` is already in the table at line 525 with the description "Cleanup or sibling failure evidence attached to a primary cancellation/failure; never masks the primary result." No action needed.

**R3: spawn_blocking cancellation behavior — satisfied**

Milestone_async_6 correctly defines cancellation of blocking tasks as non-forcible (lines 750–753). "v1 does not forcibly abort a running OS thread" is explicit. No action needed.

**R4: async cleanup secondary error path — satisfied**

Milestone_async_7 defines secondary error wrapping at lines 815–816 and 825–827. "SecondaryError never masks the primary cancellation/failure result" is explicit. No action needed.

**R5: gather secondary error types — satisfied**

Milestone_async_3 (lines 577–579) defines the fail-fast behavior: primary error is earliest, secondary errors are dropped unless a future `collect_all` API is added. The language "structured errors" from pass 1 has been replaced with "primary error" and "secondary structured errors" which maps to `SecondaryError`. No action needed.

**R6: uncancel / cancellation suppression future direction — satisfied**

The spec correctly defers this and notes the design direction. No action needed.

---

## Summary of required fixes

| ID | Location | Issue | Fix |
|----|----------|-------|-----|
| **B1** | `async_concurrency_model.md` lines 145–153, 376–385; `architecture.md` error type table | `CancellationError` listed as `Error` subclass while also excluded from `except Error` — mechanical mechanism undefined | Choose option A/B/C (recommended: Option A — remove `Error` parent from `CancellationError`); update both documents accordingly |

---

**Recommendation: iterate again**

The exclusion of `CancellationError` from `except Error` matching is the right intent, but the type-system mechanics are underspecified. Without knowing whether `CancellationError` is a non-`Error` type (Option A) or whether there is a special-case in the type checker/codegen (Option B/C), the implementation milestones cannot implement the behavior correctly. Close the mechanical gap first, then the model is ready.

After B1 is resolved, the only remaining item is the non-blocking clarification on tracked collections for orphaned task handles (R1), which is an implementation detail for milestone_async_3 rather than a design gap.
