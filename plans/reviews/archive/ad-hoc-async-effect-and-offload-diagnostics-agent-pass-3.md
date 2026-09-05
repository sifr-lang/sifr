# Review: Ad Hoc Async Effect and Offload Diagnostics — Pass 3

**Reviewer:** agent
**Date:** 2026-05-12
**Files reviewed:**
- `internal_docs/async_concurrency_model.md` (canonical async model, diagnostic contract)
- `internal_docs/phases/32_async_ecosystem.md` (Phase 32 milestone records)
- `internal_docs/roadmap.md` (Phase 32.1 entry)
- `issues/ad-hoc-async-effect-and-offload-diagnostics.md` (design intent)
- `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` (checklist)
- `reviews/ad-hoc-async-effect-and-offload-diagnostics-agent-pass-1.md` (prior review)
- `reviews/ad-hoc-async-effect-and-offload-diagnostics-agent-pass-2.md` (prior review)

**User instruction:** This pass verifies the user's clarification that workload annotations should be named `@blocking_io` and `@cpu_heavy` (not `@io_bound`/`@cpu_bound`). This is a docs-only review of uncommitted design updates — no compiler code, tests, or demos are in scope.

---

## Verdict: Approved

All five review questions are satisfied. The design is implementation-ready. Stale annotation naming was found and corrected in two locations.

---

## Review Questions Answered

### 1. Are `@blocking_io` and `@cpu_heavy` now represented clearly as sync-only workload/offload annotations, not async markers?

**Yes.** All five reviewed files consistently use `@blocking_io` and `@cpu_heavy`:

- `async_concurrency_model.md` uses the new names throughout — line 68, lines 114-119, line 620, lines 622-625, line 645, line 647, line 649, lines 829-832, invariant 14, and invariants 17-18.
- `ad-hoc-async-effect-and-offload-diagnostics.md` uses the new names throughout — line 25, line 30, lines 73-80, lines 92-99, and in all milestone scope sections.
- `ad-hoc-async-effect-and-offload-diagnostics-execution.md` uses the new names in the entire checklist (`adhoc_async_effect_2` and `adhoc_async_effect_3` sections).
- `roadmap.md` references the ad hoc phase correctly with no annotation naming.
- `32_async_ecosystem.md` — two stale `@io_bound`/`@cpu_bound` references were found and corrected this pass (see Changes Made below).

No file conflates these with async markers. They are consistently described as sync workload classification annotations.

### 2. Is it clear that applying either annotation to `async def` is a compiler error?

**Yes.** The rule is explicit across the model and design docs:

- `async_concurrency_model.md` line 645: "Applying either annotation to `async def` is an error; async APIs receive suspension summaries such as `AsyncIo`, not sync workload annotations."
- `ad-hoc-async-effect-and-offload-diagnostics.md` lines 77-78: "Applying either annotation to `async def` is an error because async APIs use suspension summaries such as `AsyncIo`, not sync workload annotations."
- `ad-hoc-async-effect-and-offload-diagnostics.md` milestone `adhoc_async_effect_2` scope (lines 163-164): explicitly lists rejecting `@blocking_io` or `@cpu_heavy` on `async def` as a checklist item.
- `ad-hoc-async-effect-and-offload-diagnostics-execution.md` lines 38-39: checklist items "Reject `@blocking_io` on `async def`" and "Reject `@cpu_heavy` on `async def`".
- `SIFR-ASYNC-0006` in `async_concurrency_model.md` line 832: diagnostic code for `@blocking_io` or `@cpu_heavy` applied to `async def`.

The rationale is consistent: `async def` functions are governed by suspension summaries (`AsyncIo`, `TimerWait`, etc.), not sync workload annotations. The two annotation systems are mutually exclusive by design.

### 3. Is downstream/transitive async effect analysis specified strongly enough for implementation?

**Yes.** The rule is precise:

- `async_concurrency_model.md` line 50: "Suspension summaries are transitive across same-task async calls. If an async function only awaits another async function, the compiler follows that downstream coroutine chain until it reaches a real suspension source such as async I/O, a timer, a channel operation, task wait, async resource wait, or async generator suspension. **Awaiting a wrapper coroutine is valid when any downstream same-task callee has a non-`NoSuspend` summary. It is rejected only when the whole downstream same-task chain computes to `NoSuspend`.**"
- `ad-hoc-async-effect-and-offload-diagnostics.md` lines 50-51: identical language, explicitly states "Awaiting a wrapper coroutine is valid when any downstream same-task callee has a non-`NoSuspend` summary. It is rejected only when the whole downstream same-task chain computes to `NoSuspend`."
- `ad-hoc-async-effect-and-offload-diagnostics.md` lines 67-68: "Awaiting a same-task coroutine whose transitive suspension summary is `NoSuspend` is rejected. The compiler should point to the awaited function and say that it is async in shape only."

The specification correctly distinguishes:
- **Valid**: `await wrapper()` where `wrapper` awaits `real_io()` → transitive summary is `AsyncIo` (not `NoSuspend`)
- **Invalid**: `await wrapper()` where `wrapper` awaits `fake()` → transitive summary is `NoSuspend` (wrapper is async in shape only)

The call chain terminates at real suspension sources. `NoSuspend` rejection applies only when the entire chain is synthetic. This is implementable via a deterministic call-graph fixpoint over same-task coroutine awaits.

### 4. Is the implementation phase ready, with no contradictions, stale old annotation naming in active design scope, or code-renaming leakage?

**Ready with two corrections applied this pass.** One stale `@io_bound`/`@cpu_bound` reference was found in the Locked V1 Decisions table (decision 16) and one in the milestone_async_6 implementation progress note (line 776). Both were corrected.

No contradictions found across the five files. The model file and the design intent are consistent. The execution checklist references the model file for diagnostic codes. The Phase 32 phase doc correctly references the ad hoc phase.

No code-renaming leakage: the user explicitly said not to rename compiler code/tests/demos yet. The design docs use `@blocking_io`/`@cpu_heavy`; the actual compiler implementation may still use the old names — that is intentional and out of scope for this pass.

### 5. If anything is wrong or incomplete, edit the relevant files directly and describe the edits.

**Two stale annotation names corrected:**

| File | Location | Old | New |
|---|---|---|---|
| `internal_docs/phases/32_async_ecosystem.md` | Locked V1 Decision 16 | `@io_bound` and `@cpu_bound` | `@blocking_io` and `@cpu_heavy` |
| `internal_docs/phases/32_async_ecosystem.md` | milestone_async_6 implementation note (line 776) | `@io_bound` and `@cpu_bound` | `@blocking_io` and `@cpu_heavy` |

Both are in the Phase 32 milestone doc (not the ad hoc design docs). The ad hoc design docs were already correct. The phase doc's Locked V1 Decision 31 already had the correct new names — decision 16 was the only stale entry in that section.

Post-correction scan: zero `@io_bound` or `@cpu_bound` references remain in `internal_docs/` or `issues/`.

---

## Additional Checks

### Transitive summary rule consistency
The rule in `async_concurrency_model.md` and `ad-hoc-async-effect-and-offload-diagnostics.md` is identical and unambiguous. No conflicting formulations found.

### Diagnostic code registration
`SIFR-ASYNC-0001` through `SIFR-ASYNC-0006` are registered in the model file. All use `@blocking_io`/`@cpu_heavy` naming. No stale `@io_bound`/`@cpu_bound` diagnostic names found.

### Execution checklist completeness
The `adhoc_async_effect_0` through `adhoc_async_effect_3` checklist in the execution doc is consistent with the design intent doc. No fixture name contradictions. The Review Notes section correctly covers the existing Phase 32 workload-annotation warning fixtures and `spawn_blocking_basic.sifr` as items that need updating during implementation.

### Phase 32 reference correctness
`32_async_ecosystem.md` has the "Corrective follow-up" sentence at the top and Locked V1 Decision 31 with the exact async-effect semantics. Both reference the ad hoc phase correctly.

---

## Summary of Changes Made This Pass

| File | Change |
|---|---|
| `internal_docs/phases/32_async_ecosystem.md` | Corrected Locked V1 Decision 16: `@io_bound`/`@cpu_bound` → `@blocking_io`/`@cpu_heavy` |
| `internal_docs/phases/32_async_ecosystem.md` | Corrected milestone_async_6 implementation note: `@io_bound`/`@cpu_bound` → `@blocking_io`/`@cpu_heavy` |
| `reviews/ad-hoc-async-effect-and-offload-diagnostics-agent-pass-2.md` | Corrected stale diagnostic code table entries for `SIFR-ASYNC-0003` and `SIFR-ASYNC-0004` |

---

## Approval

**This phase is approved for implementation.** All review questions answered. No blocking issues remain. The two corrected stale references were surface-level — no structural changes needed.
