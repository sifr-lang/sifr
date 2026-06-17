# TypeScript-Go Architecture Transfer — Full Implementation Review (Pass 1)

Issue: `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`
Scope: phase-level closeout after M0-M17 are merged on `main`.
Reviewer date: 2026-06-02.

## Verdict

**SATISFIED.** The TypeScript-Go architecture transfer is functionally complete on `main`. All 18 milestones (M0-M17) are merged, all 31 acceptance criteria pass with file:line evidence, the locked architecture decisions have implementation referents, the M0/M1 guardrails are still in place, and the most recent authoritative local gate (`scripts/run_all_tests.sh --profile quick` for M16/M17) is PASS. No blocking findings.

## Acceptance Criteria Audit

| AC | Verdict | Primary evidence |
| --- | --- | --- |
| AC-1 Workspace/session snapshot API used by CLI and LSP | PASS | `crates/sifr_frontend/src/workspace_session.rs:187,470`; `crates/sifr/src/trace_cli.rs:39,50`; `crates/sifr_lsp/src/session.rs:12,157` |
| AC-2 Open buffers overlay disk via typed FS | PASS | `crates/sifr_frontend/src/source_provider.rs:61,174-255` |
| AC-3 Reads/probes/dirs/canon/failed lookups tracked | PASS | `crates/sifr_frontend/src/source_provider.rs:46-53,334-376,411` |
| AC-4 Cache keys carry content + parser/lower + compiler + workspace context | PASS | `crates/sifr_frontend/src/cache_keys.rs:24-39,69-103,237-241,302-307` |
| AC-5 Private body edits stay local | PASS | `crates/sifr_frontend/src/query_diagnostics.rs:577-619,621-` |
| AC-6 Public export changes invalidate reverse deps | PASS | `crates/sifr_frontend/src/query_diagnostics_m10_tests.rs:7-42,45,88,131` |
| AC-7 Snapshot+version identity for LSP, stale rejection | PASS | `crates/sifr_lsp/src/session.rs:367-373,602-640`; `crates/sifr_lsp/src/diagnostics.rs:64,75`; `crates/sifr_lsp/src/document_store.rs:89,162` |
| AC-8 Parallel work preserves deterministic order | PASS | `crates/sifr_analysis/src/worker_lanes.rs:12-32`; `crates/sifr_analysis/src/lib.rs:38-40` |
| AC-9 First-class flow graph nodes for narrowing/ownership | PASS | `crates/sifr_hir/src/lib.rs:10`; `crates/sifr_hir/src/flow_graph.rs:149`; `crates/sifr_hir/src/lower/mod_context.rs:419`; `crates/sifr_hir/src/lower/narrowing.rs:143-151` |
| AC-10 Existing tooling/narrowing/ownership tests green | PASS | M16 quick PASS 295.57s; M17 quick PASS 279.93s (per user gate evidence) |
| AC-11 UTF-8/UTF-16/UTF-32 round-trip support | PASS | `crates/sifr_source/src/lib.rs:13-17,173-208,301,376-382` |
| AC-12 Trace explains phases/cache/invalidation/snapshot/stale | PASS | `crates/sifr_frontend/src/workspace_trace.rs:9-23,49`; `crates/sifr/src/trace_cli.rs:12-50` |
| AC-13 Marker-based multi-file editor fixtures + stale-snapshot | PASS | `verification/tooling/editor_query_corpus/multi_file/main.sifr`; `internal_docs/typescript_go_architecture_transfer_m17_editor_corpus_snapshot_handles.md:9-19` |
| AC-14 Internal docs updated | PASS | `internal_docs/architecture.md:271-279+`; M0-M17 transfer docs present; `frontend_query_architecture.md`, `frontend_cache_invalidation.md`, `lsp_server.md`, `performance_budgets.md`, `tooling_verification.md` referenced |
| AC-15 No TS/JS semantic authority adopted | PASS | Only stylistic mentions: `crates/sifr_type_system/src/narrow.rs:3` (inspiration doc-comment); `crates/sifr_runtime/src/json.rs:318` (numeric-bound test, not semantic policy) |
| AC-16 `DirtyScope` enumerates all six classes | PASS | `crates/sifr_frontend/src/workspace_session.rs:112-124,127-144` |
| AC-17 `can_replace_module_in_project` gated by signatures | PASS | `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:119,141-149` |
| AC-18 Copy-on-write identity reuse for snapshot maps | PASS | `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:151-173,445,494` |
| AC-19 Editor/watch events compacted; storm → workspace invalidation | PASS | `crates/sifr_frontend/src/workspace_session.rs:339-358`; `crates/sifr_lsp/src/document_events.rs`; `crates/sifr_lsp/src/notifications/mod.rs` |
| AC-20 Real LSP scheduler (priority/cancel/debounce/progress/stale) | PASS | `crates/sifr_lsp/src/scheduler.rs:1-29`; `cancellation.rs`, `progress.rs`, `request_queue.rs`, `watchdog.rs`; `session.rs:563,602` |
| AC-21 Residency/config/watch retention | PASS | `crates/sifr_frontend/src/workspace_residency.rs:93-200` |
| AC-22 Workspace/Package/Stdlib bucketed indexes with readiness | PASS | `crates/sifr_analysis/src/symbols.rs:40-44,46-52,129` |
| AC-23 `.sifrbuildinfo` verified before acceleration | PASS | `crates/sifr_frontend/src/workspace_residency.rs:210-259` |
| AC-24 `sifr lsp --parent-pid` watchdog | PASS | `crates/sifr_lsp/src/watchdog.rs:14-36`; `crates/sifr/src/cli_model_and_entrypoint.rs` |
| AC-25 Status/debug surface | PASS | `crates/sifr_frontend/src/workspace_session.rs:610-650`; `crates/sifr_lsp/src/requests/mod.rs` (`sifr/debugTrace`); `crates/sifr/src/trace_cli.rs` |
| AC-26 Per-feature LSP budgets enforced separately | PASS | `verification/performance/manifest.json:67-84`; `verification/performance/budgets.json:857-1197` (18 `perf.lsp.*` scenarios) |
| AC-27 `perf.lsp.request_families` retained as smoke only | PASS | Aggregate case still in manifest/budgets alongside per-request cases |
| AC-28 Perf docs explain `perf.interactive.*` ↔ LSP budget relationship | PASS | `internal_docs/performance_budgets.md:100-108` |
| AC-29 `SIFR-IMPORT-0005` vs `SIFR-PACKAGE-*` separation | PASS | `crates/sifr_diagnostics/src/codes/registry.rs:32,525`; `crates/sifr_driver/src/project/package_discovery.rs:91,209-275`; `verification/tooling/check_diagnostic_source_canonicalization_rules.py:347,373` |
| AC-30 Package source map preserves ambiguous candidate sets | PASS | `crates/sifr_package/src/imports/source_map.rs:62-70,216,260` |
| AC-31 Five-state coverage + cross-prefix negative checks | PASS | `crates/sifr_package/src/milestone_adhoc_tsgo_m2_tests.rs:11,48-90`; `crates/sifr/tests/verification/package/package_ambiguous_import_canonical/`; `crates/sifr/tests/verification/package/package_fatal_source_map_no_import_ambiguity/`; canonicalization contract checker |

## Locked Architecture Decisions

Spot-checked against the locked decision sections of the issue:

- Source text/position authority: `sifr_source` exists with the locked public surface (`SourceText`, `LineMap`, `PositionEncoding`, `TextPosition`, `TextRangeUtf`, `SourceFile`) and proper byte/UTF-8/UTF-16/UTF-32 conversions. Dependency direction is guarded by `verification/tooling/check_typescript_go_m1_guardrails.py` (still on M16/M17 gate lists).
- Source provider/VFS boundary: `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, `TrackingSourceProvider` all present and used by frontend/driver/lint/format/package paths.
- Package source maps and diagnostic identity: ambiguous candidate sets preserved; `SIFR-IMPORT-0005` is the only emission site for source-import ambiguity; canonicalization contract checker forbids `SIFR-PACKAGE-*` co-emission.
- Workspace session/snapshot ownership: `WorkspaceSession` is the single mutable owner; `WorkspaceSnapshot` carries snapshot id, revisions, overlays, source maps, module graph, options, package/config identity, dirty-scope report, and Arc-backed cache payloads.
- Dirty scope/signatures/invalidation: `WorkspaceDirtyScope` and `WorkspaceDirtyReason` map cleanly to the locked enum design, with `WatcherStorm` and `Unknown` degrading to workspace scope.
- Cache, COW, and lifetime: `CompilerFingerprint`/`CacheKeyFingerprint` plus typed key identities; ref-counted reuse storage; identity-reuse tests present.
- LSP session/scheduling/cancellation: persistent `Session` owns `WorkspaceSession`; scheduler lanes (`LatencySensitive`, `Formatting`, `Workspace`, `Background`) present; phase-boundary cancellation, snapshot+document-version stale rejection, parent watchdog, and delayed progress all implemented.
- Event compaction/watchers/residency/build-info: all present per AC-19/21/23.
- Symbol/import indexes: bucketed by Workspace/Package/Stdlib with readiness states.
- Flow graph: first-class graph emitted in lowering, snapshot-scoped, debug/fingerprint observable.
- Tracing/status/editor corpus/budgets: `WorkspaceTracePhase` covers all locked phases; bounded `WorkspaceDebugSnapshot`; marker corpus exercises hover/completion/definition/references/rename/diagnostics/semantic-tokens/formatting + stale; per-request `perf.lsp.*` budgets cover the full Phase 36 default list while the aggregate case persists as smoke.

## Doc / Tracker State

- Phase-tracker rows for M0-M17 all read `merged` with PR references and recorded local validation (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:9-26`).
- M16 and M17 quick gates recorded as PASS at lines 187 and 205 (wall times 295.57s and 279.93s, advisory: group skew high).
- M0-M17 transfer design notes are all present under `internal_docs/typescript_go_architecture_transfer_m*.md`.
- `internal_docs/architecture.md` lists per-milestone transfer notes through M17.
- `internal_docs/performance_budgets.md` explains the Phase 35 ↔ M12 relationship.

## Non-blocking follow-ups

1. **Phase status line still says "in progress."** `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:3` still reads `Status: in progress`, but every milestone is merged and the M16/M17 closeout gates are recorded inline rather than via a separate tracker-only PR. The expected final touch is to flip this to a closed/completed state (and, if there is a phase-closure validation requirement, add a final `scripts/run_all_tests.sh` row distinct from the M17 quick gate). PR #2266 referenced by the user appears to be the closeout PR that handles this; once it merges this item resolves.
2. **Roadmap omission.** `internal_docs/roadmap.md` does not list this ad-hoc phase even though peer ad-hoc phases (31.6, 31.7, 32.1, 36.1, 36.2) are listed by name. The omission may be intentional (this phase is a private compiler-service overhaul rather than a user-visible track), but if it should appear, adding a row near the 36.x entries with a closure date would close the gap.
3. **Empty review file.** `reviews/typescript-go-architecture-transfer-full-implementation-review-pass-1.md` existed as a zero-byte placeholder before this review; this pass now fills it.
4. **Repeated `group skew is high` advisory** on every milestone quick-profile run including M16/M17. Not a blocker for this phase, but worth folding into the performance budgets or validation-lane follow-up backlog so it does not become permanent background noise.

## Blocking findings

None.
