## TypeScript-Go M14 Bucketed Indexes — Review Pass 2

Verdict: **Pass-1 fixes all in place.** Remaining items are residual cleanup, not blockers.

### Pass-1 items verified

- **H1 fixed** — `bucket_kind_for_file` is gone. `build_buckets` (`crates/sifr_analysis/src/symbols.rs:206-264`) assigns `SymbolBucketKind::Workspace` to every per-module bucket; the entrypoint is no longer mis-bucketed.
- **H2 fixed** — `SymbolId` is now `format!("m{}:f{}:{kind}:{}:{ordinal}", …)` (`crates/sifr_analysis/src/symbols.rs:228-233`); no revision encoding. New unit test `dirty_refresh_matches_cold_rebuild_symbol_entries` (`crates/sifr_analysis/src/symbols.rs:426-439`) asserts `refreshed.entries() == rebuilt.entries()`.
- **H3 fixed** — `AnalysisHost::completion` now calls `completion_symbols("")` (`crates/sifr_analysis/src/host/implementation.rs:172`), and `workspace_import_symbols` is exposed as a host method at `crates/sifr_analysis/src/host/implementation.rs:307-313` with the test asserting its `WorkspaceSymbols` query metadata at `crates/sifr_analysis/src/host/tests.rs:266-274`.
- **H4 fixed** — `SymbolBucketReadinessState` (`crates/sifr_analysis/src/symbols.rs:46-59`) is `Exact | StaleButUsable | NeedsBackgroundRefresh | Unavailable`; `symbols_from_available_buckets` filters on `is_available()`.
- **H5 fixed (deferred with explicit Unavailable)** — `aggregate_readiness` (`crates/sifr_analysis/src/symbols.rs:283-290`) maps Package/Stdlib aggregates to `Unavailable`; doc states the limitation at `internal_docs/typescript_go_architecture_transfer_m14_bucketed_indexes.md:12-16`, `internal_docs/frontend_query_architecture.md:70-71`, and `internal_docs/architecture.md:282`.
- **M1 fixed** — `approved_lanes_exclude_single_owner_compiler_state` now asserts `lane_names.is_disjoint(&single_owner_names)` (`crates/sifr_analysis/src/worker_lanes.rs:55-63`).
- **M6 fixed** — M14 doc explicitly states the declarative-only policy (`internal_docs/typescript_go_architecture_transfer_m14_bucketed_indexes.md:35-37`).

### Remaining issues

**Stale doc claim about SymbolId encoding** (`internal_docs/tooling_analysis.md:45`). The line "symbol ids include graph/source revision, module, file, kind, name, and ordinal" now contradicts the H2 fix — `SymbolId` no longer encodes revision. Update to match the format produced at `crates/sifr_analysis/src/symbols.rs:228-233`.

**Host-level dirty-refresh test does not actually prove bucket reuse** (`crates/sifr_analysis/src/host/tests.rs:298`). `assert_eq!(helper_before, helper_after)` only compares the projected `WorkspaceSymbol { name, kind, file, container_name }`. Those values are deterministic functions of module/file/name/kind, so the assertion would still pass under a full cold rebuild. The cold-vs-refresh equivalence is covered by the symbols.rs unit test at line 426-439, but no host-level test verifies "the helper bucket was not re-flattened" — making the test name a bit aspirational. A stronger check would compare the `SymbolIndexEntry` slice (id+ordinal) for the helper module across the edit.

**Empty `dirty_modules` early-return is untested** (`crates/sifr_analysis/src/symbols.rs:99-103`). When `dirty.is_empty()`, `refresh_modules` bumps `self.revision` and returns without re-flattening — fine when nothing changed, but `refresh_existing_symbol_index` (`crates/sifr_analysis/src/host/implementation.rs:643-660`) calls it regardless and there is no test exercising "invalidated_modules empty but revision advanced". This was Pass-1 M3 and remains open.

**Refresh path does not handle modules added without being marked dirty** (`crates/sifr_analysis/src/symbols.rs:104-111`). The `for (id, bucket) in refreshed` loop only inserts buckets whose module is in `dirty`. If a new module appears in the analysis view but isn't flagged dirty by the frontend, no bucket is created for it and `flatten_buckets` will not surface its symbols until the next cold build. This depends on the `InvalidationReport::invalidated_modules` contract — worth either a test pinning that contract or a defensive insertion of any module present in `refreshed` but absent from `self.buckets`.

**`ModuleId::new` is still not `#[doc(hidden)]`** (`crates/sifr_frontend/src/graph_cache_and_queries.rs:31-35`). The other three test-only constructors flagged in Pass-1 L2 got `#[doc(hidden)]` (`SourceRevision::new` at `crates/sifr_frontend/src/source_maps.rs:24`, `SourceHash::new` at `:40`, `GraphRevision::new` at `crates/sifr_frontend/src/graph_cache_and_queries.rs:47`); `ModuleId::new` was missed.

**`completion_symbols` is byte-identical to `workspace_symbols`** (`crates/sifr_analysis/src/symbols.rs:158-165`) — both call `symbols_from_available_buckets(query, |_| true)`. Acceptable as a wiring placeholder, but worth noting in the M14 doc that completion currently does no completion-specific filtering beyond bucket readiness.

**M14 guardrail is still string-presence only** (`verification/tooling/check_typescript_go_m1_guardrails.py:347-387`). It now checks more identifier names (the four readiness states are not asserted, only `SymbolBucketReadiness`, plus `completion_symbols`, `workspace_import_symbols`, `import_entry_count`, and two test function names), which usefully tightens the contract, but it still does not invoke `cargo test` the way M5/M11/M13 validators do. Consider at least asserting that `SymbolBucketReadinessState::Exact`/`StaleButUsable`/`NeedsBackgroundRefresh`/`Unavailable` are all referenced — today only `SymbolBucketReadiness` (without `State`) is checked, so a regression that drops three of the four states and reverts to a boolean would not trip this guardrail.

### Bottom line

Functional fixes for H1–H5 and M1 are in place and tested. The residual items above are cleanup (stale doc claim, weak host-level test, untested edge cases in `refresh_modules`, guardrail breadth). None block the milestone closeout; recommend folding the stale `tooling_analysis.md:45` line and the `SymbolBucketReadinessState` guardrail tightening into this branch before merge.
