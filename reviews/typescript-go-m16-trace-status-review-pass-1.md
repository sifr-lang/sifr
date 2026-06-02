# TypeScript-Go M16 Trace And Status Surfaces — Review Pass 1

Verdict: CHANGES_REQUESTED

Scope reviewed: the M16 trace/status milestone closeout. Specifically the
working-tree changes to `crates/sifr_frontend/src/workspace_trace.rs`,
`crates/sifr_frontend/src/workspace_session.rs`,
`crates/sifr_analysis/src/host/debug_status.rs`,
`crates/sifr_analysis/src/host/implementation.rs`,
`crates/sifr_lsp/src/{session,diagnostics,notifications/mod,server}.rs`,
`crates/sifr/src/trace_cli.rs`,
`internal_docs/typescript_go_architecture_transfer_m16_trace_status.md`, and
`verification/tooling/check_typescript_go_m1_guardrails.py`.

## Summary

M16 lands the right vocabulary — a normalized `WorkspaceTracePhase` enum, a
`WorkspaceStatusSnapshot`, a `WorkspaceDebugSnapshot` carried by every
`WorkspaceSnapshot`, an `AnalysisHost::debug_snapshot` that adds symbol-bucket
readiness, and a `sifr trace` CLI that renders both surfaces. Stale-snapshot
rejection now records the captured/current workspace and graph/source
revisions before the analysis host returns `StaleSnapshot`, and the new test
`stale_snapshot_is_rejected_after_update` proves the trace explanation exists
for that path.

However the closeout criterion in `issues/...m16-trace-status` is broader than
what the working tree actually achieves:

> A stale rejection **and a dependency-sensitive invalidation** can be
> explained from trace output. Status output is useful for editor bug reports
> without exposing private internals.

Several pieces of the surface are either dead, split-brained, or
unbounded-growth, which makes the editor-bug-report claim and the
dependency-sensitive-invalidation closeout only partially demonstrable.

## Findings

### 1. WorkspaceSession trace recorders are dead in production
`crates/sifr_frontend/src/workspace_session.rs:440-455` adds public
`record_update_latency_ms`, `record_scheduler_trace`, and
`record_cancellation_trace` recorders, but nothing in the workspace ever calls
them outside their own definitions. The guardrail only checks the symbols
exist in the file (`verification/tooling/check_typescript_go_m1_guardrails.py:442-454`),
not that any production caller invokes them.

Consequences:
- `WorkspaceStatusSnapshot.last_update_latency_ms` is permanently `None`. The
  M16 doc (`internal_docs/.../m16_trace_status.md:25-29`) advertises
  "last-update latency" as part of status output; the renderer at
  `crates/sifr_frontend/src/workspace_trace.rs:127-129` only emits that line
  when `Some(_)`, so today it is unreachable.
- The scheduler and cancellation events the LSP records live in
  `crates/sifr_lsp/src/session.rs:313-321` (`Session.traces:
  Vec<WorkspaceTraceEvent>`) but are never propagated into the
  `WorkspaceSession.trace`. The `sifr trace` CLI in
  `crates/sifr/src/trace_cli.rs:35-51` therefore cannot ever show scheduler /
  cancellation / LSP-timing data — only the source-update / cache /
  invalidation / stale-rejection events that the workspace session itself
  emits. The M16 doc's claim that "LSP request scheduling, cancellation,
  stale diagnostic rejection, and timing markers use the same trace phase
  vocabulary" is technically true (same enum) but practically misleading
  (two separate buffers, only one is reachable from any external surface).

### 2. No production accessor for the LSP session trace buffer
`Session::trace_snapshot()` is gated `#[cfg(test)]`
(`crates/sifr_lsp/src/session.rs:323-328`). The LSP `Session.traces` is the
only place scheduler / cancellation / stale-diagnostic-publication / LSP-timing
events live, and nothing in production reads it. Combined with finding (1),
this means an editor bug report cannot extract the LSP trace at all today —
neither via `sifr trace`, nor via a `$/sifr.debug` request, nor via a flush at
shutdown. AC-25 ("status/debug surface reports … for editor bug reports") is
not actually reachable from outside the LSP process.

### 3. Unbounded trace growth + per-snapshot clone
Both trace buffers grow without a cap:
- `WorkspaceTraceState.events` (`workspace_trace.rs:198`).
- `Session.traces` in the LSP (`session.rs:28`).

Every `WorkspaceSession::snapshot()` call additionally:
1. Records a new `Cache` event (`workspace_session.rs:495-502`).
2. Clones the entire trace into a fresh `Arc<WorkspaceDebugSnapshot>`
   (`workspace_session.rs:504-509`, plus `trace: self.trace.snapshot()` at
   `workspace_session.rs:650`).

For an LSP that snapshots per editor query and per overlay change, this is
O(snapshots × trace_len) memory and CPU growth over a long session, with the
debug snapshot kept alive on the workspace snapshot itself. There is no cap,
ring buffer, or downsampling. For a "deterministic compiler-service trace"
intended to survive long-lived sessions, this needs a bounded retention
policy (size, age, or per-phase quota) and almost certainly a way to elide
debug payload construction when no consumer is asking for it.

### 4. AnalysisHost::debug_snapshot has compute side effects
`crates/sifr_analysis/src/host/debug_status.rs:6-25` calls
`self.symbol_index()?`, which in `host/implementation.rs:620-641` builds or
refreshes the bucketed symbol index when stale. A debug-status query thus
silently does workspace symbol-index work and bumps `SymbolIndex.revision`.
For editor bug-report scenarios this changes the observed state — calling
"debug snapshot" twice can yield different bucket readiness on the second
call. It should read the existing readiness lazily (`Unavailable` if not yet
built) rather than building on demand.

### 5. Compiler-phase trace events are static placeholders, not phase
evidence
`record_compiler_phase_trace` (`workspace_session.rs:568-595`) emits
`Parse`, `Lower`, `TypeCheck`, `Ownership`, and `Flow` events on every
reload, each carrying nothing but `modules=N`. They do not reflect actual
phase execution, timing, cache hit/miss, or per-module work; they are
constant tags repeated every reload. The doc copy ("WorkspaceSession records
… compiler phase summaries") oversells what is recorded. This makes the
trace noisier without making any phase actually explainable. The `Cache`
event at snapshot time (`cache=FrontendReuseStats { ... }`) is the only
real phase-level signal today.

This also amplifies finding (3): every reload writes 6 events the trace
does not need.

### 6. Closeout test gap for dependency-sensitive invalidation
The closeout for M16 requires that "a stale rejection **and a
dependency-sensitive invalidation** can be explained from trace output."
- Stale rejection: covered by
  `stale_snapshot_is_rejected_after_update`
  (`crates/sifr_analysis/src/host/tests.rs:79-106`). ✓
- Dependency-sensitive invalidation: there is no test that asserts a
  `WorkspaceDirtyScope::ReverseDependencies { path }` (or
  `ImportSignatureChanged` / `ExportSignatureChanged` reason) appears in the
  workspace trace. The available tests only assert `SourceTextChanged`
  (`workspace_session_tests.rs:147-189`) for a OneModule case, or merge
  semantics for synthetic reports (`workspace_session_tests.rs:411-537`).
  The path that actually produces `ReverseDependencies` lives in
  `graph_cache_and_queries.rs:555-562`, and the test
  `dunder_method_signature_update_invalidates_reverse_dependents` (M10
  era) does not check trace output. Without a regression that exercises a
  real export-signature edit and asserts the trace surfaces
  `scope=ReverseDependencies` and the export/import reason, the closeout
  cannot be considered demonstrated.

### 7. `sifr trace` CLI cannot reflect a running LSP server
`trace_cli.rs:35-51` opens a one-shot `WorkspaceSession` and renders its
debug snapshot. There is no way to attach to a running `sifr lsp` and ask
for its trace/status, nor a notification the server emits on shutdown.
Combined with (1) and (2), the CLI is only useful as a sanity check on a
freshly loaded session — not as the "editor bug report" surface the
milestone scopes claim.

### 8. Doc drift and guardrail tautology
- `internal_docs/typescript_go_architecture_transfer_m16_trace_status.md`
  claims the status surface includes "last-update latency" and that LSP
  scheduler/cancellation/timing share the trace vocabulary; both are true
  on paper but unreachable from any external surface (see findings 1, 2).
  Either wire up the producers or rewrite the doc to admit the LSP
  scheduler trace is held separately and is currently test-only.
- `verification/tooling/check_typescript_go_m1_guardrails.py:442-493`
  validates that the new symbols (`record_update_latency_ms`,
  `record_stale_rejection`, `WorkspaceDebugSnapshot`, `cmd_trace`, etc.) are
  present in their respective files. It does not check that any production
  call site exists. The dead recorders identified in finding (1) pass the
  guardrail trivially. The guardrail should additionally pin at least one
  call site for the latency / scheduler / cancellation recorders, or the
  recorders should be removed if they are not part of M16 scope.

### 9. Minor / stylistic
- `WorkspaceDebugSnapshot.render_text` puts `[status]` and `[trace]` headers
  but no terminator between them; renderable, but a trailing newline
  before `[trace]` would make multi-snapshot capture (e.g., piped log
  parsing) easier.
- `record_compiler_phase_trace` records an `Invalidation` event with the
  current `dirty_scope_report` on every reload — even when the scope is
  `None` and reasons are empty. That is `scope=None reasons=[]` noise on the
  initial load.
- `WorkspaceIndexReadinessStatus` is replaced wholesale by
  `AnalysisHost::debug_snapshot` (overwrites the frontend's `"frontend"`
  placeholder bucket). That works, but the placeholder in
  `workspace_session.rs:629-636` becomes confusing because it is what
  callers see when they use `WorkspaceSession::snapshot()` directly (e.g.
  the CLI `sifr trace`) — they get a single `frontend: exact|unavailable`
  bucket rather than the real per-bucket readiness. Either drop the
  placeholder or call out in the doc that real readiness only appears via
  `AnalysisHost::debug_snapshot`.

## Suggested follow-ups before SATISFIED

Required:
1. Wire `WorkspaceSession::record_update_latency_ms`,
   `record_scheduler_trace`, and `record_cancellation_trace` from real call
   sites (LSP request lifecycle, scheduler dispatch, cancellation), OR
   delete them and rewrite the M16 doc to reflect that LSP scheduler /
   cancellation / timing live only on `Session.traces`. Either decision is
   acceptable, but pick one and align doc + guardrail to it.
2. Add an editor-reachable accessor for the LSP `Session.traces` (a
   `$/sifr.debug` request, a structured shutdown log, or a periodic
   notification). Or, if that is M17 territory, explicitly defer it in the
   M16 doc and the closeout language.
3. Add a regression that opens a small project, edits an exported
   signature, and asserts the workspace trace events include a
   `phase=invalidation` event whose detail mentions
   `ReverseDependencies` and `ExportSignatureChanged`. This is the missing
   half of the M16 closeout.
4. Stop refreshing the symbol index from
   `AnalysisHost::debug_snapshot`. Read the current readiness (or report
   `Unavailable`) without forcing a rebuild.

Recommended:
5. Bound `WorkspaceTraceState.events` and `Session.traces` (ring buffer or
   size cap) and either lazily defer the debug-snapshot clone or only
   build it on demand. Long-lived LSP sessions otherwise grow without
   bound.
6. Drop the no-evidence `Parse/Lower/TypeCheck/Ownership/Flow` placeholder
   events, or replace them with real cache hit/miss / module-count deltas
   emitted by the actual phase entrypoints.
7. Strengthen the M16 guardrail to pin at least one production call site
   per new recorder (mirrors how the existing M11/M13/M14 guardrails check
   real call shapes, not just symbol presence).

## Verdict

CHANGES_REQUESTED. The trace/status vocabulary and CLI scaffolding are in
place, and stale-snapshot rejection is now genuinely explainable from
trace output. But (a) the LSP scheduler / cancellation / latency
recorders are dead-code as of this working tree, (b) there is no
editor-reachable surface for the LSP trace buffer, (c) the
dependency-sensitive-invalidation half of the closeout has no regression
yet, and (d) the trace surfaces have unbounded growth and at least one
side-effecting debug query. Items (a)-(c) should be resolved (or
explicitly descoped in the doc + tracker) before M16 is marked closed.
