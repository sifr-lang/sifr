# Concurrency/Runtime Provider Cancellation Closure — Fable Review Pass 2

Branch: `codex/concurrency-cancellation-closure`
Scope: diff from `main` (5 files) plus consistency sweep of issue, inventory, workload database, structured work model, and dependency policy.

## Verdict: FAIL

One blocking finding remains. Both pass-1 blockers are fixed at their cited locations, but the B2 contradiction (unconditional `tokio-util`) survives in one additional line of the same issue document.

## Pass-1 blocker verification

### B1 — public `cancel_scope`/`CancelScope` surface removal: FIXED

All records now consistently state the closed model (task-handle cancel plus compiler-recognized `async with task.timeout(duration)` same-task scope; no public `cancel_scope`, `CancelScope`, or token surface):

- `internal_docs/structured_runtime_work_model.md:157` — closure paragraph replaces the open "settled stable API named `CancelScope`" language.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:96` — observed-state note records the closed decision and points to the closure paragraph.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:331` — cancellation contract closure decision recorded.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:481` — `sifr.task` scope list replaces `cancel_scope` with the `task.timeout` scope form.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:491` — timeout/deadline bullet records the closed scope surface and the negative public-API statement.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:901` — Stable task APIs resolved-decision row updated; explicitly states no separate public `cancel_scope`/`CancelScope` exists.
- `verification/stdlib/concurrency_runtime_substrate_inventory.json:179,240,244,11073` and `verification/stdlib/concurrency_runtime_substrate_inventory.md:23,66` — acceptance, surface, and workload entries updated.
- `verification/stdlib/concurrency_runtime_workload_database.md:9` — `sifr.task.cancel_scope` row replaced with `async with task.timeout(duration)`.

A repo-wide grep finds no remaining affirmative `cancel_scope`/`CancelScope` public-API claims; surviving mentions are negations ("no public `CancelScope`…"), historical execution-ledger records, or unrelated LSP `CancellationToken` material.

### B2 — Resolved Decisions Rust ecosystem row: FIXED at the cited row, but the same contradiction survives elsewhere (see blocking finding)

- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:910` — Resolved Decisions Rust ecosystem row now says "conditional `tokio-util 0.7.18`".
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:272` — Locked Dependencies Ring 2 row now conditions the addition on implementation proof and records that the closed model added no unconditional `tokio-util` cancellation dependency.

## Blocking findings

1. **`issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:261` — dependency-ring summary still implies unconditional `tokio-util`.**
   The line reads: "Ring 2 generated-runtime core: `tokio`, `tokio-util`, conditional `futures-util`, and `tracing` …". Because `futures-util` is explicitly tagged `conditional` in the same list, an unqualified `tokio-util` reads as accepted/unconditional — contradicting the now-conditional Ring 2 table row (line 272) and the Resolved Decisions row (line 910) in the same document. This is the B2 defect surviving in a third location.
   **Required fix:** change the list to "… `tokio`, conditional `tokio-util`, conditional `futures-util`, and `tracing` …".

## Non-blocking notes

- `internal_docs/dependency_policy.md:32` — "`tokio-util` when a Sifr-owned cancellation scope or Tokio I/O helper requires it" is already conditional ("when … requires it"), so not blocking, but "Sifr-owned cancellation scope" now only exists as internal `task.timeout` lowering machinery, not a public type. Consider rewording to "internal cancellation machinery" to avoid resurrecting the public-scope reading.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:272` — in "features `rt`, `io-util`, and `time` only if implementation proves it is needed", the "only if" placement can be read as qualifying only the feature selection rather than the dependency addition itself. The binding notes disambiguate; optional wording polish.
- Working-tree state: the issue file changes are split across staged and unstaged hunks, and the two inventory files plus the workload database are entirely unstaged. Stage everything before committing the closure.
