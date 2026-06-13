### Findings

All six items from Pass 2's "Remaining issues" are addressed in the working tree:

1. **Stale `SymbolId` doc claim** — `internal_docs/tooling_analysis.md:45` now reads "symbol ids include module, file, kind, name, and ordinal" — revision removed, matches `crates/sifr_analysis/src/symbols.rs:229-235`.

2. **`ModuleId::new` doc(hidden)** — `crates/sifr_frontend/src/graph_cache_and_queries.rs:32` now carries `#[doc(hidden)]`, matching the sibling constructors at `source_maps.rs:24,40` and `graph_cache_and_queries.rs:48`.

3. **All `SymbolBucketReadinessState` variants asserted in guardrail** — `verification/tooling/check_typescript_go_m1_guardrails.py:362-369` explicitly checks `Exact`, `StaleButUsable`, `NeedsBackgroundRefresh`, and `Unavailable`, so a regression collapsing the enum to a bool would now trip.

4. **Empty `dirty_modules` test** — `empty_dirty_refresh_advances_revision_without_changing_entries` at `crates/sifr_analysis/src/symbols.rs:451-461` covers the early-return path: revision bumped, entries unchanged.

5. **New-module defensive insertion** — `crates/sifr_analysis/src/symbols.rs:107-113` adds the `id.module.is_some() && !self.buckets.contains_key(&id)` branch, and `refresh_inserts_new_module_bucket_even_when_not_marked_dirty` at `:463-503` proves a module present in the analysis view but absent from `dirty` still surfaces. (Note: the empty-`dirty` early return at `:100-103` still skips this branch, but the contract that an empty `invalidated_modules` means "no module set delta" is reasonable and now testable.)

6. **`completion_symbols` placeholder documented** — `internal_docs/typescript_go_architecture_transfer_m14_bucketed_indexes.md:24-26` explicitly states completion currently applies the same filter as workspace_symbols plus bucket readiness, with ranking deferred to `sifr_analysis::completion`.

No new correctness bugs or scope blockers turned up on re-read of `symbols.rs`, the guardrail, or the M14 doc. The aggregate (module=`None`) Workspace bucket stays `Exact` with zero entries and is correctly retained across refreshes, which is harmless.

SATISFIED.
