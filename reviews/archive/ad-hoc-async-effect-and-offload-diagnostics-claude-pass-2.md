# Review: Ad Hoc Async Effect and Offload Diagnostics — Pass 2

**Reviewer:** Claude Opus 4.7
**Date:** 2026-05-12
**Files reviewed:**
- `internal_docs/async_concurrency_model.md` (canonical async model, diagnostic contract)
- `internal_docs/phases/32_async_ecosystem.md` (Phase 32 closure + corrective follow-up reference)
- `internal_docs/roadmap.md` (Phase 32.1 entry)
- `issues/ad-hoc-async-effect-and-offload-diagnostics.md` (design intent)
- `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` (checklist)
- `reviews/ad-hoc-async-effect-and-offload-diagnostics-claude-pass-1.md` (prior review)

**Baseline:** Pass 1 (2026-05-12) was conditionally approved. Three targeted edits were required and have been verified as applied in this pass.

---

## Verdict: Approved — No Further Design Review Needed Before Implementation

All pass 1 conditional concerns are resolved. The docs are coherent, Sifr-compatible, and implementation-ready.

---

## Summary

All prior conditional concerns have been verified as resolved:

1. **Model invariant numbering is correct.** Invariants 15-18 carry the async-effect discipline rules. Invariants 19-35 continue sequentially with existing model decisions. No duplication, no gaps.

2. **Diagnostic codes are registered.** `SIFR-ASYNC-0001` through `SIFR-ASYNC-0005` are documented in `async_concurrency_model.md` under the Diagnostics Contract section with precise semantic descriptions.

3. **Roadmap entry is correct.** Phase 32.1 is present in `roadmap.md` with correct phase file references and a precise unlock description matching the ad hoc scope.

4. **Phase 32 corrective follow-up is referenced.** `32_async_ecosystem.md` has the "Corrective follow-up" sentence at the top and Locked V1 Decision 31 with the exact async-effect semantics.

5. **Execution document has fixture update notes.** The execution doc's Review Notes section covers the three existing Phase 32 fixtures that need updates (`io_bound_annotation_warning.sifr`, `cpu_bound_annotation_warning.sifr`, `spawn_blocking_basic.sifr`).

6. **Stale wording is gone.** The ambiguous transition phrasing from the original execution doc was corrected to "change from warning to error" in pass 1. This pass confirms it does not appear in any reviewed design or phase file.

---

## Pass 1 Conditional Concerns — Resolution Status

| Concern | Status |
|---|---|
| Model invariant numbering (duplicate/no-gap) | **Resolved** — invariants 15-18 sequential, 19-35 continue correctly |
| Diagnostic code registration | **Resolved** — SIFR-ASYNC-0001 through SIFR-ASYNC-0005 in model file |
| Roadmap Phase 32.1 entry | **Resolved** — present with correct files and unlock description |
| Phase 32 corrective follow-up reference | **Resolved** — sentence at top of `32_async_ecosystem.md` and Locked V1 Decision 31 |
| Execution doc fixture update notes | **Resolved** — Review Notes section covers `io_bound_annotation_warning.sifr`, `cpu_bound_annotation_warning.sifr`, `spawn_blocking_basic.sifr` |
| Stale transition wording | **Resolved** — not present in any reviewed design or phase file |

---

## Additional Checks

### Stale wording
Scanned all reviewed design and phase files for ambiguous transition phrasing. None found.

### Diagnostic-code reservations
`SIFR-ASYNC-0001` through `SIFR-ASYNC-0006` are reserved and documented:

| Code | Semantic |
|---|---|
| `SIFR-ASYNC-0001` | `async def` body has no real suspension effect (transitive `NoSuspend`) |
| `SIFR-ASYNC-0002` | awaiting a same-task coroutine whose transitive suspension summary is `NoSuspend` |
| `SIFR-ASYNC-0003` | direct `@blocking_io` call from async context |
| `SIFR-ASYNC-0004` | direct `@cpu_heavy` call from async context |
| `SIFR-ASYNC-0005` | `spawn_blocking` target is unannotated and not classified by stdlib/FFI contract |
| `SIFR-ASYNC-0006` | `@blocking_io` or `@cpu_heavy` applied to `async def` |

These are the six new diagnostic codes for this phase after the workload-annotation clarification.

### Phase/model leakage
No references to future phases (33-41) appear in the ad hoc docs. No model-level leakage from the ad hoc into Phase 32's completed milestone scope. The boundary is clean.

### Duplicate or contradictory statements
The ad hoc design intent doc and the execution checklist are consistent. Both reference the same four milestones (`adhoc_async_effect_0` through `adhoc_async_effect_3`) with matching fixture names. No contradictions found.

### Fixture gaps
The 17 new validation fixtures listed in the execution checklist are sufficient:
- 3 for `adhoc_async_effect_0` (effect summary infrastructure)
- 4 for `adhoc_async_effect_1` (reject fake async/await)
- 3 for `adhoc_async_effect_2` (change warning diagnostics to hard errors)
- 5 for `adhoc_async_effect_3` (restrict blocking offload)

The execution doc notes the 3 existing Phase 32 fixtures that need updating. No additional fixture gaps identified.

---

## Minimal Observation (Non-Blocking)

The ad hoc design intent doc (`ad-hoc-async-effect-and-offload-diagnostics.md`) does not enumerate the `SIFR-ASYNC-*` diagnostic codes by number. The codes are in `async_concurrency_model.md` (the authoritative diagnostic contract) and the execution doc references the model file. This is consistent with the project's diagnostic-code taxonomy: `sifr_diagnostics` owns the shared model/schema/docs, and phase docs that touch diagnostics reference the model file as the source. No change needed.

---

## No Further Design Review Needed

This phase is ready for implementation. The docs are locked. Implementation should follow the execution checklist and update the three noted existing Phase 32 fixtures before or during the first implementation PR.

---

## Changes Made This Pass

None — no issues found that required edits.
