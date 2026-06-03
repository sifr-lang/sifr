# Claude Review: TypeScript-Go Milestone Slicing

Date: 2026-05-29

Scope: iterative Claude review of milestone sizing, ordering, dependencies, and independent reviewability for `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`.

## Rounds

1. First milestone review approved the overall layering but found M3 too broad, M8 too broad, and the M2/M3 overlay lifecycle boundary ambiguous.
2. Second review confirmed the M3 split and overlay ownership fix, then requested splitting cache keys from cache/COW/structural replacement, making M1 an explicit pre-flight gate, and tightening M2 wording.
3. Final review found no real blockers after the split, but recommended explicit dependencies, making M4's `AnalysisHost` snapshot migration clearer, and moving flow graph work earlier next to invalidation.
4. Post-refinement check found no cycles or logical blockers. The only concern was that trace/status is necessarily late; the phase now states earlier milestones add hooks incrementally and M16 normalizes them.

## Incorporated Changes

- Split workspace session data model from analysis snapshot migration.
- Split scheduler queues from cancellation/progress/watchdog work.
- Split cache-key/fingerprint work from ref-counted caches, copy-on-write maps, and structural replacement.
- Added explicit `Depends on:` lines to every milestone.
- Made M1 a pre-flight gate for M2-M5.
- Clarified M2 owns overlay record/provider behavior and M3 owns overlay lifecycle in `WorkspaceSession`.
- Moved flow graph work earlier after dependency invalidation.
- Moved per-request budgets earlier than cancellation/progress so latency evidence guides later work.
- Clarified M16 as a trace/status normalization milestone, not the first trace implementation point.
