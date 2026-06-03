# TypeScript-Go M16 Trace And Status Surfaces — Review Pass 2

Verdict: SATISFIED (with low-priority residual recommendations)

Scope reviewed: the M16 trace/status milestone working tree, focused on whether
the pass 1 blockers listed in
`reviews/typescript-go-m16-trace-status-review-pass-1.md` are resolved.

## Pass 1 blocker status

### 1. Dead workspace latency/scheduler/cancellation recorders or misleading doc/guardrail — RESOLVED
- `WorkspaceSession::record_scheduler_trace` and
  `WorkspaceSession::record_cancellation_trace` no longer exist. The workspace
  surface only retains `record_update_latency_ms` and `record_stale_rejection`
  (`crates/sifr_frontend/src/workspace_session.rs:440-447`).
- `record_update_latency_ms` is now wired from real production call sites:
  `crates/sifr_lsp/src/analysis_workspace.rs:104` on document open and
  `crates/sifr_lsp/src/analysis_workspace.rs:130` on document update, both
  measuring `Instant::now()`-derived elapsed milliseconds.
- LSP scheduler / cancellation / stale-rejection / LSP-timing events stay on
  `Session.traces` (`crates/sifr_lsp/src/session.rs:213-266`,
  `:178-211`, `:130-136`), and the M16 doc explicitly says so:
  "LSP request scheduling, cancellation, stale diagnostic rejection, and timing
  markers use the same trace phase vocabulary in the LSP session trace buffer,
  which is also bounded to the newest 256 events and exposed through the custom
  `sifr/debugTrace` request"
  (`internal_docs/typescript_go_architecture_transfer_m16_trace_status.md:20-24`).
- `record_stale_rejection` is reached from
  `AnalysisHost::ensure_snapshot_current`
  (`crates/sifr_analysis/src/host/implementation.rs:827-835`), exercised by
  `stale_snapshot_is_rejected_after_update`.

### 2. Missing production accessor for the LSP trace buffer — RESOLVED
- `Session::trace_snapshot` is now `pub(crate)` instead of `#[cfg(test)]`
  (`crates/sifr_lsp/src/session.rs:327-331`).
- The dispatcher exposes it as a real LSP request:
  `"sifr/debugTrace" => Ok(Value::String(session.trace_snapshot().render_text()))`
  in `crates/sifr_lsp/src/requests/mod.rs:49`.
- Coverage: `debug_trace_request_exposes_lsp_trace_events`
  (`crates/sifr_lsp/src/session.rs:484-504`) drives an enqueue + cancel cycle
  and asserts the request body contains `phase=scheduler` and
  `phase=cancellation`.

### 3. Unbounded trace growth + per-snapshot clone — RESOLVED
- Workspace trace is capped at the newest 256 events:
  `MAX_TRACE_EVENTS: usize = 256` and `prune_before_push` evicts the oldest
  event before each push (`crates/sifr_frontend/src/workspace_trace.rs:7`,
  `:252-256`). `next_sequence` continues to monotonically increase, so eviction
  doesn't reset event ordering.
- LSP trace is capped symmetrically: `MAX_LSP_TRACE_EVENTS: usize = 256` and
  `Session::trace` evicts on overflow
  (`crates/sifr_lsp/src/session.rs:18`, `:314-325`).
- Per-snapshot clone work is now bounded at 256 events × event-string size,
  which is the correct mitigation for the long-lived editor session case. The
  `WorkspaceSnapshot.debug` field stays an `Arc<WorkspaceDebugSnapshot>` so
  downstream readers share the snapshot's trace clone instead of recloning.

### 4. AnalysisHost::debug_snapshot side effect — RESOLVED
- `crates/sifr_analysis/src/host/debug_status.rs:9` now reads
  `self.symbol_index.as_ref()` (an `Option<&SymbolIndex>`) and only enriches
  readiness when the index already exists. When it does not, the helper
  `unavailable_readiness` returns `Unavailable` rows for Workspace / Package /
  Stdlib buckets without constructing or refreshing the index. Repeated debug
  queries no longer bump `SymbolIndex.revision` or trigger workspace-symbol
  build work.
- The guardrail pins the absence of the side-effecting form: it requires both
  `symbol_index.as_ref()` to be present and `symbol_index()?` to be absent in
  `debug_status.rs`
  (`verification/tooling/check_typescript_go_m1_guardrails.py:499-505`).

### 5. Missing dependency-sensitive invalidation trace regression — RESOLVED
- `dependency_sensitive_invalidation_is_explained_in_trace`
  (`crates/sifr_analysis/src/host/tests.rs:121-165`) writes a small two-module
  project, edits the exported `value()` return type from `int` to `str`,
  asserts `report.dirty_scope_report.scope` is
  `WorkspaceDirtyScope::ReverseDependencies { .. }`, and asserts the debug
  snapshot's trace contains an `Invalidation` event whose detail mentions both
  `ReverseDependencies` and `ExportSignatureChanged`. This is the missing
  half of the M16 closeout from pass 1.
- The stale-rejection regression remains green
  (`crates/sifr_analysis/src/host/tests.rs:91-119`) and also asserts the trace
  detail contains `captured_workspace`, which proves the workspace/graph/source
  revision detail from `ensure_snapshot_current` is reachable.

### 6. Guardrail/doc tautology — RESOLVED
The `validate_m16_trace_status_state` block
(`verification/tooling/check_typescript_go_m1_guardrails.py:443-511`) now
checks real call sites and structural constants, not just symbol presence:

- Pins `record_update_latency_ms` *plus* `elapsed_ms` in the LSP analysis file
  (i.e., proves the latency recorder is fed from real wall-clock measurement,
  not just defined).
- Pins the literal `"sifr/debugTrace"` and `trace_snapshot().render_text()`
  call in the LSP requests dispatcher (proves the LSP trace buffer is
  externally reachable).
- Pins `MAX_TRACE_EVENTS` in `workspace_trace.rs` and `MAX_LSP_TRACE_EVENTS`
  in `session.rs` (proves both buffers carry a bound constant).
- Pins `symbol_index.as_ref()` and the absence of `symbol_index()?` in
  `debug_status.rs` (proves the readiness query stays side-effect-free).
- Pins `cmd_trace`, `trace_entrypoint`, and `render_text` in
  `trace_cli.rs` (proves the CLI snapshot command exists).
- The doc snippet checks now require `M16 updated`, `WorkspaceTracePhase`,
  `WorkspaceStatusSnapshot`, `WorkspaceDebugSnapshot`, and `sifr trace` in
  `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md`.

`python3 verification/tooling/check_typescript_go_m1_guardrails.py` passes
locally on this working tree.

## Closeout fit

The M16 issue closeout
(`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:1104-1107`)
requires:

> A stale rejection and a dependency-sensitive invalidation can be explained
> from trace output. Status output is useful for editor bug reports without
> exposing private internals.

Both halves are now demonstrated by tests:
- Stale rejection: `stale_snapshot_is_rejected_after_update` asserts the
  workspace trace contains a `StaleRejection` event with captured/current
  workspace + graph/source revisions.
- Dependency-sensitive invalidation: the new
  `dependency_sensitive_invalidation_is_explained_in_trace` asserts the
  invalidation event names `ReverseDependencies` and `ExportSignatureChanged`.

Status output (`WorkspaceDebugSnapshot.status.render_text()`) emits open files,
project count, source/module/dependency counts, cache entry counts, memory
counters, last-update latency, and index readiness — all derived from local
state, no telemetry / heap introspection. AC-25 is reachable through two
complementary surfaces:
- `sifr trace <file>` renders a one-shot CLI snapshot for project or
  single-file inputs.
- `sifr/debugTrace` returns the running LSP's session trace.

## Residual recommendations (non-blocking)

These remain from pass 1 and are not required for SATISFIED:

1. `record_compiler_phase_trace` still writes static
   `Parse/Lower/TypeCheck/Ownership/Flow modules=N` events on every reload
   (`crates/sifr_frontend/src/workspace_session.rs:560-587`). They are
   evidence-free placeholders; the only real phase signal is the `Cache`
   event emitted at snapshot time. Consider either dropping them or
   wiring them to actual phase entrypoints in a follow-up.
2. `WorkspaceSession::debug_snapshot` still emits a placeholder
   `frontend: exact|unavailable` bucket in `index_readiness`
   (`crates/sifr_frontend/src/workspace_session.rs:621-628`).
   `AnalysisHost::debug_snapshot` overwrites it with real per-bucket readiness,
   but the `sifr trace` CLI path goes through `WorkspaceSession::snapshot()`
   directly and therefore shows the placeholder. The doc should either call
   this out or the CLI should route through `AnalysisHost::debug_snapshot`.
3. `WorkspaceTraceState::prune_before_push` and `Session::trace` evict via
   `Vec::remove(0)` (O(n) memmove). With cap 256 this is negligible; if either
   trace grows hotter, `VecDeque` would amortize.
4. `WorkspaceDebugSnapshot.render_text` lacks a trailing newline between the
   `[status]` block and `[trace]` header (pass 1 finding #9); cosmetic.
5. `sifr/debugTrace` exposes only `Session.traces`. Per-document workspace
   traces (carrying the dependency-sensitive `Invalidation` events) remain
   reachable through `AnalysisHost::debug_snapshot` / `sifr trace` but not via
   the LSP request. If editor bug reports should also bundle workspace trace,
   a follow-up could fold per-host debug snapshots into the LSP response.

## Verdict

SATISFIED. All six pass 1 blockers are resolved, the closeout test surface
covers both stale rejection and dependency-sensitive invalidation, trace
buffers are bounded, the LSP trace is editor-reachable, the debug snapshot
no longer has compute side effects, and the M16 guardrail now pins
production call sites and structural constants rather than symbol presence.
The residual items above can be folded into M17 or a small follow-up; none
block marking M16 closed.
