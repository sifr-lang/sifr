# Final Phase Closure Review: Ad Hoc Async Effect And Offload Diagnostics

**Phase:** 32.1 — Ad Hoc Async Effect And Offload Diagnostics
**Date:** 2026-05-12
**Reviewer:** agent (final phase closure)
**Verdict:** APPROVED

## Review Scope

This review covers the closure branch (`adhoc-async-diagnostics-closure`) which updates only tracking documentation. Implementation was delivered through milestone PRs #2096–#2100 and validated during each PR.

## Source Documents

- Issue: `issues/ad-hoc-async-effect-and-offload-diagnostics.md`
- Execution tracker: `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md`
- Design: `internal_docs/async_concurrency_model.md`
- Phase: `internal_docs/phases/32_async_ecosystem.md`
- Roadmap: `internal_docs/roadmap.md`

## Milestone Evidence

Each milestone PR was independently reviewed and approved:

| Milestone | PR | Review |
|---|---|---|
| Annotation rename (`@blocking_io`/`@cpu_heavy`) | #2096 | `reviews/adhoc_async_workload_annotation_rename_review.md` |
| Effect summary infrastructure | #2097 | `reviews/adhoc_async_effect_summary_review.md` |
| Reject fake async/await | #2098 | `reviews/adhoc_reject_fake_async_review.md` |
| Enforce workload annotations | #2099 | `reviews/adhoc_enforce_workload_annotations_review.md` |
| Restrict blocking offload targets | #2100 | `reviews/adhoc_restrict_blocking_offload_targets_review.md` |

## Design Completeness Check

### Suspension Effect Model
- Two-state internal summaries (`NoSuspend` / non-empty) computed to fixpoint ✓
- Transitive summaries through same-task coroutine awaits ✓
- Direct effects for known primitives: `task.sleep`, channels, task handles, async CM, anext ✓

### Fake Async Rejection
- `async def` with `NoSuspend` rejected with `SIFR-ASYNC-0001` ✓
- Awaiting same-task coroutine with transitive `NoSuspend` rejected with `SIFR-ASYNC-0002` ✓
- Protocol-conformance escape hatch via `@__sifr_async_protocol_no_suspend__` ✓

### Workload Annotation Enforcement
- `@blocking_io` / `@cpu_heavy` on `async def` rejected with `SIFR-ASYNC-0003` / `SIFR-ASYNC-0004` ✓
- Direct annotated calls in async code rejected with `SIFR-ASYNC-0006` ✓
- Cheap sync helpers in async code remain allowed ✓

### Blocking Offload Target Restriction
- `task.spawn_blocking` / `ThreadPoolExecutor.submit` require classified targets ✓
- Classified targets: `@blocking_io`, `@cpu_heavy`, stdlib-known ✓
- Unannotated local sync functions rejected ✓
- Existing sendability, arity, result diagnostics preserved ✓

### Non-Goals Verification
- No public effect type system added ✓
- `@blocking_io` / `@cpu_heavy` remain sync-only ✓
- No silent rewrite to async tasks or blocking offload ✓
- Cheap sync helpers remain allowed in async code ✓

## Validation Coverage Check

All 16 validation fixtures from the issue exist and are covered:

### adhoc_async_effect_0 (positive — e2e pass)
- `async_effect_summary_sleep.sifr` ✓
- `async_effect_summary_channel_receive.sifr` ✓
- `async_effect_summary_transitive_await.sifr` ✓

### adhoc_async_effect_1 (negative — e2e fail)
- `async_no_suspend_rejected.sifr` ✓
- `async_transitive_no_suspend_await_rejected.sifr` ✓
- `await_sync_function_rejected.sifr` ✓
- `async_protocol_no_suspend_requires_escape_hatch.sifr` ✓

### adhoc_async_effect_2 (mixed)
- `blocking_io_on_async_def_rejected.sifr` ✓
- `cpu_heavy_on_async_def_rejected.sifr` ✓
- `blocking_io_direct_call_in_async_rejected.sifr` ✓
- `cpu_heavy_direct_call_in_async_rejected.sifr` ✓
- `cheap_sync_helper_in_async_allowed.sifr` ✓ (in quick lane)

### adhoc_async_effect_3 (mixed)
- `spawn_blocking_blocking_io_allowed.sifr` ✓
- `spawn_blocking_cpu_heavy_allowed.sifr` ✓
- `spawn_blocking_unannotated_rejected.sifr` ✓
- `thread_pool_submit_unannotated_rejected.sifr` ✓
- `spawn_blocking_known_stdlib_blocking_allowed.sifr` ✓

## Local Validation

- `scripts/run_all_tests.sh --profile quick`: passed during each milestone PR ✓
- `scripts/run_all_tests.sh`: passed on closure branch with `profile=pr`, 73 pass fixtures, 0 failures, 28 hardening variants, and the existing group-skew advisory ✓

## Closure Branch Doc Updates

Checked against the pre-closure state:

| File | Change | Correct |
|---|---|---|
| `issues/ad-hoc-async-effect-and-offload-diagnostics.md` | Status `proposed` → `completed`, DoD checked off, completion section added | ✓ |
| `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` | Status `proposed` → `completed`, validation checkboxes marked, PR review notes added, forward-looking TODOs removed | ✓ |
| `internal_docs/phases/32_async_ecosystem.md` | Header updated to "Corrective follow-up completed on 2026-05-12" | ✓ |
| `internal_docs/roadmap.md` | Phase 32.1 status `proposed` → `completed`, note updated | ✓ |

No implementation code was changed in the closure branch. All doc updates are accurate.

## Definition of Done Verification

From `issues/ad-hoc-async-effect-and-offload-diagnostics.md`:

- [x] Async effect summaries are computed deterministically — fixpoint through same-task coroutine awaits ✓
- [x] Fake async functions and fake awaits are rejected with Sifr diagnostics — `SIFR-ASYNC-0001/0002` ✓
- [x] Direct annotated blocking/CPU-heavy calls in async code are errors — `SIFR-ASYNC-0003/0004/0006` ✓
- [x] Blocking offload requires workload classification — `SIFR-ASYNC-0005` ✓
- [x] Existing Phase 32 positive async fixtures remain valid unless they intentionally covered now-rejected fake async/offload behavior ✓
- [x] The quick validation lane includes representative positive and negative fixtures — 4 fixtures in quick lane, 16 total ✓

## Known Advisory (Non-Blocking)

- The group-skew advisory observed during full validation is pre-existing and not introduced by this phase.

## Verdict

**APPROVED.** This phase is complete. The compiler now enforces async effect discipline: `async def` bodies must have a real suspension source, `await` must target awaitables with real async effects, direct `@blocking_io`/`@cpu_heavy` calls in async code are rejected, and `spawn_blocking`/`ThreadPoolExecutor.submit` require classified offload targets. All five milestone PRs are merged, all validation fixtures exist, full local validation passes, all tracking docs are accurately updated, and four independent milestone reviews are on record.
