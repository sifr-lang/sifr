# Review: TypeScript-Go architecture transfer M10 — Snapshot Reuse and Structural Replacement (pass 3)

Branch: `wave_tsgo_m10_snapshot_reuse`
Files re-reviewed (dirty tree):

- `crates/sifr_frontend/src/cache_keys.rs`
- `crates/sifr_frontend/src/frontend_reuse.rs`
- `crates/sifr_frontend/src/graph_cache_and_queries.rs`
- `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs`
- `crates/sifr_frontend/src/lib.rs`
- `crates/sifr_frontend/src/module_signatures.rs`
- `crates/sifr_frontend/src/query_diagnostics.rs`
- `crates/sifr_frontend/src/query_diagnostics_m10_tests.rs`
- `crates/sifr_frontend/src/workspace_session.rs`
- `internal_docs/architecture.md`
- `internal_docs/frontend_cache_invalidation.md`
- `internal_docs/frontend_query_architecture.md`
- `internal_docs/typescript_go_architecture_transfer_m10_snapshot_reuse.md`
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`

I re-ran `cargo test -p sifr_frontend` (38 passed), `cargo clippy -p sifr_frontend -- -D warnings`, `cargo fmt --check`, and `python3 scripts/check_file_size_guardrails.py`. All pass.

## Status of pass 2 residuals

### MEDIUM #1 — Single-underscore class methods → **resolved**

`crates/sifr_frontend/src/module_signatures.rs:144` now matches every `Stmt::FunctionDef` in the class body without a visibility filter, so `_helper` (and any other underscore-prefixed method) participates in the class shape. This matches `collect_module_exports` at `crates/sifr_frontend/src/query_diagnostics.rs:362-393`, which already places every method/operator into `Type::Class.methods`. The new regression `single_underscore_method_signature_update_invalidates_reverse_dependents` (`query_diagnostics_m10_tests.rs:88-128`) exercises a `_helper` return-type change and asserts `WorkspaceDirtyScope::ReverseDependencies` plus `ExportSignatureChanged`. The previously-noted under-invalidation path is now closed.

### MEDIUM #2 — Debug-format source-range sensitivity → **resolved**

Signature shapes are built from `ComparableParameters`/`ComparableExpr`/`ComparableDecorator`/`ComparableArguments`/`ComparableTypeParams` wrappers (`module_signatures.rs:1-5,127-191,225-251`). The Ruff comparable types deliberately strip `range: TextRange` and `node_index: AtomicNodeIndex` (`third_party/ruff/crates/ruff_python_ast/src/comparable.rs:1392-1433`, plus the docstring at `:1-16`), and the per-`ComparableExpr` variants all destructure `range: _, node_index: _`. The new regression `leading_whitespace_edit_preserves_export_signature_scope` (`query_diagnostics_m10_tests.rs:172-208`) inserts a leading newline and asserts `WorkspaceDirtyScope::OneModule` with `invalidated_modules == vec![helper]`, locking the contract.

### Class decorator regression coverage → **resolved**

`class_decorator_update_invalidates_reverse_dependents` (`query_diagnostics_m10_tests.rs:131-169`) flips `@old_decorator` to `@new_decorator` on a `class Box: pass` and asserts `ReverseDependencies` + `ExportSignatureChanged`. Together with the existing `dunder_method_signature_update_invalidates_reverse_dependents` and the new `single_underscore_method_signature_update_invalidates_reverse_dependents`, every branch of the class-body-shape code is exercised.

## New findings (ordered by severity)

### LOW — `update_module_source` parses replacement source but does not seed `reuse_caches.parse`

`graph_cache_and_queries.rs:483-488` parses the replacement source solely to derive `new_signature`, then discards the `ParsedModule`. `ensure_parsed` (`graph_cache_and_queries.rs:629-647`) will re-parse the same content on next access. Pass 1 and pass 2 LOW; still applicable. Not a correctness issue but a missed reuse opportunity.

### LOW — `prune_unshared` runs twice per text-changed `update_module_source`

`clear_module_caches` calls `self.reuse_caches.prune_unshared()` at `graph_cache_and_queries.rs:844`, and `update_module_source` calls it again at `:557` on every path. The second prune is required for the `WorkspaceDirtyScope::None` (version-only) path, but the text-changed path runs it twice. Pass 2 LOW; still applicable.

### LOW — `rebuild_edges` still bypasses `reuse_caches.parse`

`graph_cache_and_queries.rs:775-808` calls `sifr_syntax::parse_module` directly for every module. After a non-replaceable update, every module is re-parsed even though only the changed module's parse-cache key changed. Pass 1/2 LOW; still applicable.

### LOW — `source_map_arc_for_reuse` clones `SourceFileView` rather than sharing leaf `Arc`s

`graph_cache_and_queries/reuse.rs:164-166` does `self.cached_source_file_view(index).as_ref().clone()` into `Vec<SourceFileView>`, so the `Arc<SourceFileView>` from the ref-counted cache is not shared with the snapshot-facing `SourceMapView`. Identity reuse remains observable via `FrontendCacheEntryIdentity`. Pass 1/2 LOW; still applicable.

### LOW — class body member dedup is order-insensitive but cannot disambiguate same-name override sequences

`module_signatures.rs:170` sorts `member_shapes` before joining. Python and Sifr treat same-name redefinitions as last-wins, but if two class-body methods share a name with different shapes, sort+dedup will compare against alphabetical shape order rather than declaration order, so reordering the redefinitions in source would still be detected as a signature change (because both shapes are present) but the *which one is "the" method* identity is lost. This is an edge case (HIR lowering would warn or take the second), and it does not under-invalidate; flagging it for the record only.

## Residual risks and missing tests

- **`can_replace_module_in_project` is still only exercised through `FrontendContext`**, not the `WorkspaceSession` snapshot consumer surface. Pass 1/2 noted this; out-of-scope for closeout if explicitly deferred.
- **`reload()` discards the entire `FrontendContext`** including all reuse caches (`workspace_session.rs:276-312`). Editor batching scenarios throw away every reuse entry on each reload. M10 explicitly scopes this to future work, but it remains the largest gap between the milestone's intent and its observable behavior.
- The `prune_unshared` redundancy and the unseeded parse-cache entry in `update_module_source` and `rebuild_edges` are the only concrete optimization wins left on the M10 surface; they could be folded into a small follow-up without changing the public contract.

## Cross-checks

- `signatures_can_replace_module_in_project` (`graph_cache_and_queries/reuse.rs:141-149`) still treats parse failure as "not replaceable" and reuses the existing import/export equality check; the scope selection at `graph_cache_and_queries.rs:543-549` routes parse-failure through `GraphStructure`, conservatively correct.
- `clear_module_caches` (`graph_cache_and_queries.rs:825-845`) preserves `module_state.parsed` for any module not in `modules_with_source_changes` and clears `lowered`/`diagnostics`/`analysis` for every module in the invalidated set. The pre-existing `reverse_dependent_invalidation_reuses_unchanged_parse_entry` test (`graph_cache_and_queries/reuse.rs:600-638`) keeps locking this on the reverse-dependency path.
- `WorkspaceSnapshot` `Arc::ptr_eq` assertions (`workspace_session.rs:718-749`) confirm cross-snapshot identity reuse for overlays, source dependencies, source map, module graph, compiler options, and package/config identity.
- Comparable wrappers reach into expressions that still strip `range`/`node_index` per the Ruff source; spot-checked `ExprName` (`comparable.rs:982-984`, omits `ctx`), `ComparableParameter` (`:422-435`), `ComparableTypeParams` (`:1374-1433`), and `ComparableNumber::Float` uses `to_bits` for stable float identity.
- `query_diagnostics_m10_tests.rs` mirrors the helper-project / `temp_project_dir` pattern from `query_diagnostics::tests`; no new test infrastructure introduced. The split keeps `query_diagnostics.rs` at 860 lines (under the 900-line cap) and `query_diagnostics_m10_tests.rs` at 229 lines.
- `cache_keys.rs` remains at 899 lines — one-line headroom on the 900-line cap. Worth keeping in mind when adding fields or doc comments, but not a blocker for closeout.
- Tracker row (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:19`) and `internal_docs/typescript_go_architecture_transfer_m10_snapshot_reuse.md:1-61` describe the comparable-view shape construction and the resolved class-body-method coverage consistently with the diff.

## Verdict

SATISFIED

Both pass 2 MEDIUM findings are closed: the export signature now includes every class-body `FunctionDef` (matching `collect_module_exports`), and signature shapes are built from Ruff comparable wrappers that intentionally strip `TextRange`/`AtomicNodeIndex`. The three new regression tests (`single_underscore_method_signature_update_invalidates_reverse_dependents`, `class_decorator_update_invalidates_reverse_dependents`, `leading_whitespace_edit_preserves_export_signature_scope`) lock the contracts. The only remaining residuals are pre-existing LOW reuse optimizations (`update_module_source` parse-cache seeding, double `prune_unshared`, `rebuild_edges` bypass, `SourceFileView` clone) and the explicitly-deferred `reload()`/snapshot-consumer-surface gaps, none of which block merge.
