I'll deliver the review inline since the write needs approval. Here are the findings.

## TypeScript-Go M14 Bucketed Indexes And Safe Parallel Lanes — Review Pass 1

Verdict: **CHANGES_REQUESTED**. The bucket/refresh skeleton and guardrail/doc surface are present, but several correctness bugs and scope gaps would let the milestone close with the public surface in place and the underlying contract broken.

### High Severity

**H1. `bucket_kind_for_file` misclassifies the entrypoint as `Stdlib`**
`crates/sifr_analysis/src/symbols.rs:259-265` treats `file.as_u32() == 0` as Stdlib. But both `FrontendContext::load_single_file_with_external_defs` (`crates/sifr_frontend/src/graph_cache_and_queries.rs:321-323`) and `load_project_with_provider` (`crates/sifr_frontend/src/graph_cache_and_queries.rs:428-430`) always assign `FileId(0)` to the user entrypoint. There is no stdlib synthesis path in `sifr_frontend` today, so this branch always represents the user's primary source. Every real project will mis-bucket its entrypoint as `Stdlib`. The two unit tests at `crates/sifr_analysis/src/symbols.rs:298-307` mask this by using `FileId::new(1)`/`FileId::new(2)`, and the host-level test (`crates/sifr_analysis/src/host/tests.rs:208-282`) never asserts bucket kinds.

**H2. Cold-build and dirty-refresh paths emit different `SymbolId`s for the same content**
`SymbolId` encodes the build-time `(graph, source)` revision (`crates/sifr_analysis/src/symbols.rs:201-216`). `refresh_modules` (`crates/sifr_analysis/src/symbols.rs:77-99`) only rebuilds dirty buckets; clean buckets keep their original ids while `self.revision` bumps. The cold-load path in `AnalysisHost::symbol_index` (`crates/sifr_analysis/src/host/implementation.rs:612-633`) calls `SymbolIndex::build` again whenever the cached index is `None`/stale, re-encoding every id at the new revision. So the "same" symbol gets one id incrementally and a different id after any cold rebuild — any consumer that treats `SymbolId` as snapshot-stable will see id churn the milestone claims not to introduce.

**H3. `AnalysisHost::completion` and other host queries ignore the buckets**
`crates/sifr_analysis/src/host/implementation.rs:160-191` still calls `self.symbol_index()?.workspace_symbols("")` (flat entries list). M14 scope and AC-22 require completion/import suggestions to consult buckets and per-bucket readiness. The bucketed flow is currently write-only — the host builds/refreshes buckets but only exposes `symbol_bucket_readiness` for inspection. `workspace_import_symbols` (`crates/sifr_analysis/src/symbols.rs:157-169`) is `pub` on `SymbolIndex` but is not re-exported via `AnalysisHost`, has no host-level test, and exists primarily to satisfy the guardrail string match.

**H4. `SymbolBucketReadiness.ready` is hard-coded `true`**
`SymbolBucket.ready` is set to `true` for every per-module bucket (`crates/sifr_analysis/src/symbols.rs:218-225`) and every aggregate (`crates/sifr_analysis/src/symbols.rs:227-238`); nothing else mutates it. The locked decision (`issues/...:705-707`) calls for `Exact`/`StaleButUsable`/`NeedsBackgroundRefresh`/`Unavailable` readiness. None of those states are representable. The doc's "Empty package and stdlib aggregate buckets are still reported as ready so callers can distinguish unavailable data from no data" is misleading — no path can ever surface "unavailable" either.

**H5. `SymbolBucketKind::Package` is never populated**
No `PackageId` in `SymbolBucketId` (`crates/sifr_analysis/src/symbols.rs:33-37`), no package classification in `build_buckets`, no test that adds a package-scoped module. The only `Package` bucket that ever exists is the aggregate empty bucket inserted at `crates/sifr_analysis/src/symbols.rs:227-238`. AC-22 and locked shape `PackageBucket(PackageId)` are not met. Either explicitly defer to a follow-up milestone in the M14 doc and tracker, or thread package identity through `ProjectAnalysisView`.

### Medium Severity

**M1. `approved_lanes_exclude_single_owner_compiler_state` does not exclude anything** (`crates/sifr_analysis/src/worker_lanes.rs:35-54`). Asserts variant presence in each slice but never that the two sets are disjoint. The test name implies an exclusion invariant the assertions don't check.

**M2. Guardrail script enforces names, not contracts** (`verification/tooling/check_typescript_go_m1_guardrails.py:346-371`). String-presence only — none of H1–H5 would trip it. M5/M11/M13 validators at least run `cargo test` or load JSON manifests; M14 is grep-only. At minimum, run a targeted `cargo test` filter or assert that `AnalysisHost::completion` references the bucket APIs.

**M3. Test coverage gaps for the dirty-refresh path** (`crates/sifr_analysis/src/host/tests.rs:208-282`). Does not cover: editing a non-entrypoint module; a dirty list referencing a now-deleted module; an empty `dirty_modules` that nevertheless advances the revision (current code at `crates/sifr_analysis/src/symbols.rs:84-88` silently bumps `self.revision` without re-flattening); bucket-kind assertions (would fail per H1); cold-rebuild vs refresh equivalence (would fail per H2).

**M4. `refresh_modules` accumulates heterogeneous-revision entries by design**. Beyond the id issue in H2, `self.buckets` ends up holding clean per-module buckets keyed at the original build revision next to refreshed buckets at the new revision. Any M9 typed cache key that fingerprints `SymbolIndex` content will produce non-deterministic fingerprints depending on which path produced the index.

**M5. M14 doc undersells the limitations** (`internal_docs/typescript_go_architecture_transfer_m14_bucketed_indexes.md`, 34 lines). Does not mention: unpopulated `Package` buckets (H5), the `file_id == 0` heuristic and entrypoint misclassification (H1), boolean readiness vs the locked four-state model (H4), or that completion bypasses buckets (H3).

**M6. Worker lanes are declarative-only** (`crates/sifr_analysis/src/worker_lanes.rs`). Enums + `pub const` slices with no integration anywhere; no compile/runtime check prevents a hypothetical worker from picking up a single-owner phase. Defensible as scope, but the doc should make the "no execution surface uses these yet" status explicit.

### Low Severity

**L1.** `cargo test -p sifr_analysis symbol_index -- --nocapture` (validation block at `issues/...:131`) also hits the pre-existing `project_symbol_index_is_stable_for_workspace_queries`. Consider a more targeted filter.

**L2.** New `ModuleId::new`/`GraphRevision::new`/`SourceRevision::new`/`SourceHash::new` constructors (`crates/sifr_frontend/src/{graph_cache_and_queries.rs,source_maps.rs}`) exist solely for `sifr_analysis::symbols::tests`. Consider gating behind `cfg(any(test, feature = "test-helpers"))` or `#[doc(hidden)]`. Constructing `GraphRevision`/`SourceRevision` from arbitrary `u64`s without monotonicity is a footgun if anything else picks them up.

**L3.** Tracker row at `issues/...:23` is "pending" — fine for an in-progress branch, but closeout should also flip `Status: in progress` at the top of the M14 doc.

### Suggested Path Forward

1. Fix H1 so the entrypoint is not labelled `Stdlib`; add a regression test asserting `Workspace` for it. Either own real package/stdlib classification this milestone or explicitly defer in doc + tracker.
2. Resolve H2/M4 by removing the revision from `SymbolId` or patching ids in place during refresh; add a cold-vs-refresh equivalence test.
3. Wire `completion` (and a host-exposed `workspace_import_symbols`) through the bucketed view; add tests that prove the path is exercised.
4. Either land a real readiness state machine (H4) or trim the surface and document the deferral.
5. Strengthen the M1 guardrail to assert at least one behavioral check, not just identifier presence.
6. Update the M14 doc with the deferrals and limitations.

Want me to retry writing this to `reviews/typescript-go-m14-bucketed-indexes-review-pass-1.md` (the empty file already exists in the working tree) once you grant write permission?
