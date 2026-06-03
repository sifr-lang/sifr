# Review: TypeScript-Go architecture transfer M10 — Snapshot Reuse and Structural Replacement (pass 1)

Branch: `wave_tsgo_m10_snapshot_reuse`
Files reviewed (dirty tree):

- `crates/sifr_frontend/src/cache_keys.rs`
- `crates/sifr_frontend/src/frontend_reuse.rs` (new)
- `crates/sifr_frontend/src/graph_cache_and_queries.rs`
- `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs` (new)
- `crates/sifr_frontend/src/lib.rs`
- `crates/sifr_frontend/src/module_signatures.rs`
- `crates/sifr_frontend/src/query_diagnostics.rs`
- `crates/sifr_frontend/src/workspace_session.rs`
- `internal_docs/architecture.md`
- `internal_docs/frontend_cache_invalidation.md`
- `internal_docs/frontend_query_architecture.md`
- `internal_docs/typescript_go_architecture_transfer_m10_snapshot_reuse.md` (new)
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`

I ran `cargo test -p sifr_frontend -- ref_counted_module_caches_reuse_identity_on_hits structural_one_module_replacement_reuses_unchanged_cache_entries document_version_only_update_recaches_source_file_view public_constant_value_update_invalidates_reverse_dependents` — all four targeted M10 tests pass. `python3 scripts/check_file_size_guardrails.py` passes.

## Findings (ordered by severity)

### HIGH — `ExportSignature` ignores dunder methods (`__init__`, `__add__`, …), class decorators, and operator impls; `can_replace_module_in_project` will treat dependent‑visible changes as safe reuse

`crates/sifr_frontend/src/module_signatures.rs:122-174`, `crates/sifr_frontend/src/module_signatures.rs:209-211` (`is_public`).

`export_signature` only captures `Stmt::FunctionDef` and `Stmt::ClassDef` members whose name does not start with `_`. That excludes:

- `__init__` — the public constructor signature.
- `__add__`, `__lt__`, `__eq__`, `__iter__`, and every other operator dunder.

But `crates/sifr_frontend/src/query_diagnostics.rs:380-393` actively exports `class.operator_impls` into the dependent‑visible class type (`Type::Class.methods`), and HIR lowering specially handles `__init__` for constructor lowering. The class shape in `export_signature` (`crates/sifr_frontend/src/module_signatures.rs:166-174`) is also missing `class.decorator_list`, while function shape correctly includes `function.decorator_list`.

Consequence under M10's new structural‑replacement gate:

1. A change to `__add__`'s return type, `__init__`'s parameter list, or a class decorator does not change `ModuleSignature`.
2. `signatures_can_replace_module_in_project` returns `true` (`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:141-149`).
3. `update_module_source` only invalidates `[module]`, never the reverse closure (`crates/sifr_frontend/src/graph_cache_and_queries.rs:506-523`).
4. Reverse dependents keep their cached HIR and `external_defs`, which still carry the old dunder/init/decorator metadata. Type checks against `a + b`, `Vec(x=…)`, and decorator‑gated behavior see stale class types.

The M10 closeout criterion is explicit: "stale cache entries cannot be observed through diagnostics, HIR, or LSP queries." This gap silently violates that.

Pre‑M10 the gap was latent — `module_signature` existed (M7) but no code branched on signature equality to short‑circuit reverse‑dep invalidation. M10 turned it into a correctness gate, so the gap is now load‑bearing.

Suggested fix:

- Either include all class‑body members (capturing names, parameter shapes, decorators, and operator impls) in the class shape, or split a separate "class type identity" signature that mirrors what `collect_module_exports` actually exports.
- Add at least one regression test that asserts an `__init__`/`__add__` signature change either degrades to `ReverseDependencies` or re‑lowers the dependent's HIR.

### HIGH — `clear_module_caches` + immediate `prune_unshared` evicts content‑valid parse/source‑map entries; the ref‑counted cache provides no cross‑edit reuse beyond what `module_state` already pins

`crates/sifr_frontend/src/graph_cache_and_queries.rs:825-835`, `crates/sifr_frontend/src/graph_cache_and_queries.rs:557` (also in `update_module_source`), `crates/sifr_frontend/src/frontend_reuse.rs:158-160`.

The reuse cache prunes any entry whose only remaining strong reference is the cache itself (`strong_count == 1`). The two strong references it expects are (a) the cache and (b) `ModuleState.parsed/lowered/diagnostics/analysis`. Snapshots do not retain these Arcs — `WorkspaceSnapshot` only holds `source_map` and `module_graph`. So in practice the only external holder is `ModuleState`.

After a public‑export change with reverse‑dep invalidation:

- `clear_module_caches([reverse-deps…])` drops `module_state.parsed/lowered/diagnostics/analysis` for every dependent.
- `prune_unshared()` runs immediately. Every dependent's cache entries are now `strong_count == 1` and get evicted.
- Next query for those dependents re‑parses from source even though source content is unchanged and the parse key is unchanged.

So the M10 promise that "parse and HIR results are reused across snapshots when content and compiler options are unchanged" only holds for snapshots that observe the *same* `FrontendContext` state. As soon as one public signature changes anywhere in the project, parse trees for every unchanged dependent are rebuilt from scratch.

This is a missed‑feature, not an observable correctness bug, but it makes the M10 reuse story essentially decorative outside the single‑module structural‑replacement path. The M10 doc admits "cache residency is process-local and bounded by active frontend context state" — but the closeout criterion was reuse "across snapshots." The implementation does not deliver that.

Suggested fix:

- Do not clear `module_state.parsed` when only HIR/diagnostics/analysis need invalidation — parse is content‑addressed and unchanged.
- Either drop `prune_unshared` after clearing, or move it out of the synchronous edit path (e.g., snapshot drop).
- Add a test that drives two public‑signature edits and asserts the parse cache hits on reverse dependents whose source did not change.

### MEDIUM — `query_diagnostics.rs` is at the exact 900‑line guardrail cap

`crates/sifr_frontend/src/query_diagnostics.rs` is 900 lines. `scripts/check_file_size_guardrails.py:142` compares with `>`, so 900 passes — but any future addition fails. `cache_keys.rs` is 899 lines (one line headroom). M10's added test (`public_constant_value_update_invalidates_reverse_dependents`) is what pushed `query_diagnostics.rs` to the cap.

Suggested fix: split the test module out (similar to the `graph_cache_and_queries/reuse.rs` pattern used in this PR), or move test fixtures into a sibling module before M11 work lands. Maintenance‑readiness, not correctness.

### MEDIUM — `source_map_arc_for_reuse` clones inner `SourceFileView` content rather than sharing the cached `Arc<SourceFileView>`

`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:160-173`:

```rust
let files = (0..self.modules.len())
    .map(|index| self.cached_source_file_view(index).as_ref().clone())
    .collect();
```

The cached `Arc<SourceFileView>` is held only by `ModuleState`. The `Arc<SourceMapView>` exposed to snapshots contains a `Vec<SourceFileView>` of clones. So the source‑file ref‑counted cache and the snapshot‑held source map are decoupled: identical source files don't share storage at the leaf level between snapshots.

`document_version_only_update_recaches_source_file_view` hides this because it inspects identities via `FrontendCacheEntryIdentity` strings, not via `Arc::ptr_eq`. Not a correctness issue, but it means snapshot memory cost scales linearly with module count × snapshot count even when files are unchanged.

Suggested fix: make `SourceMapView.files` either `Vec<Arc<SourceFileView>>` or `Arc<[SourceFileView]>` so cross‑snapshot sharing is real. Defer if scope creep, but track it.

### MEDIUM — `source_file_key_fingerprint` includes `document_version`, churning the source‑map cache on every editor version bump

`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:294-317`.

This is intentional (the test `document_version_only_update_recaches_source_file_view` verifies it), and `prune_unshared` keeps the entry count bounded. But it means every keystroke (in an LSP scenario where each edit increments document version) churns the source‑map‑view cache despite identical source text and `source_hash`. The M9 locked decision explicitly excludes "document identity inputs" from content keys (test `document_identity_inputs_are_intentionally_omitted_from_content_keys` in `cache_keys.rs`). Including version in *the view key* is defensible because the view tracks editor metadata, but it should be a documented choice, not an accidental one.

Suggested action: either keep version inside `SourceFileView` but key the cache only by source hash + identity, or document why M10 deliberately diverges from M9's "no document identity in content keys" rule for the view key family.

### MEDIUM — `WorkspaceSnapshot` does not retain parse / HIR / diagnostics / analysis Arcs; the closeout wording about snapshot‑pinned reuse overstates what the implementation delivers

`crates/sifr_frontend/src/workspace_session.rs:183-195`. Snapshots carry `overlays`, `source_dependencies`, `source_map`, `module_graph`, `compiler_options`, `package_config_identity`. They do *not* carry `Arc<ParsedModule>`, `Arc<LoweringResult>`, `Arc<Vec<RenderedDiagnostic>>`, or `Arc<ModuleAnalysisView>`.

Combined with the prune behavior (HIGH #2), the practical lifetime of every reuse cache entry equals the lifetime of the active `ModuleState` entry — no extra retention from snapshots. The locked decision phrasing "cache entries are reference-counted by snapshots and are released when no retained snapshot can observe them" (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:567-569`) is not met. The M10 doc text is more honest, but the milestone tracker should be aligned.

Either narrow the milestone doc to admit the actual scope or extend `WorkspaceSnapshot` to retain the per‑module reuse entries used during analysis.

### LOW — `can_replace_module_in_project` re‑parses the replacement source twice

`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:119-139` calls `sifr_syntax::parse_module` to compute the new signature; `update_module_source` parses the same source again `crates/sifr_frontend/src/graph_cache_and_queries.rs:483-488`. The LSP fast‑path will pay double parse cost. Optionally accept a pre‑parsed module or return the parse result for reuse.

### LOW — `rebuild_edges` re‑parses every module on the slow path and bypasses `reuse_caches.parse`

`crates/sifr_frontend/src/graph_cache_and_queries.rs:775-808` calls `sifr_syntax::parse_module` directly for every module. After a non‑replaceable update, all modules are re‑parsed even though most still have valid parse cache entries. Routing rebuild_edges through `reuse_caches` (or at least through `parse_key_fingerprint` lookup) would salvage reuse on the slow path.

### LOW — `ExportSignature` shape for `Stmt::Assign` and constants uses `format!("{:?}", assign.value)`

`crates/sifr_frontend/src/module_signatures.rs:179-191`. Debug format of arbitrary expression values is locale‑independent but not stable across formatter changes inside the `sifr_python_ast` AST representation. Not a runtime risk, but if AST Debug output evolves, fingerprints shift unexpectedly. Consider an explicit normalization once the M9 fingerprint design is reused here.

### LOW — `module_graph_view` / `source_map_view` and their Arc counterparts duplicate construction logic

`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:151-173` and `244-278`. Two near‑identical builders for the same two views, one Arc‑caching and one not. Acceptable today, fragile under change.

### LOW — `clear_module_caches` does not reset `module_state.source_file_view`

`crates/sifr_frontend/src/graph_cache_and_queries.rs:825-835` clears parsed/lowered/diagnostics/analysis but not `source_file_view`. After a public‑export change with reverse‑dep invalidation, dependents keep their old `source_file_view` (which is still correct because content is unchanged). Behavior is fine, just worth a comment so a future reader doesn't try to "complete the symmetry."

## Residual risks and missing tests

- **No test confirms snapshot pinning prevents pruning.** `prune_unshared` retains entries while any external Arc is alive; given that snapshots don't actually pin parse/HIR/diagnostics, a future change that *does* pin them needs a contract test today.
- **No test for the dunder/`__init__`/class‑decorator signature gap (HIGH #1).** A targeted test (helper changes `__init__` parameter count or `__add__` return type → main's HIR / diagnostics observe the new signature) would catch any future regression.
- **No test for parse reuse across reverse‑dep invalidation (HIGH #2).** A test that updates a helper's exported signature, then asserts an unchanged sibling module's `parse_module` returns `CacheStatus::Hit`, would prove ref‑counted reuse delivers across edits.
- **No test for `WorkspaceDirtyScope::OneModule` while a long‑lived snapshot is alive.** The existing test drops snapshots before mutation, so we never observe how an Arc‑pinned old snapshot coexists with a replaced module.
- **`can_replace_module_in_project` is not exercised through the `WorkspaceSession` layer.** All M10 tests drive `FrontendContext` directly. Snapshot consumers (`AnalysisHost`, LSP) never observe structural replacement in tests.
- **`reload()` discards the entire `FrontendContext`** (`crates/sifr_frontend/src/workspace_session.rs:276-312`), and therefore the entire `FrontendReuseCaches`, on every overlay change that requires reload. Out of M10 scope, but worth recording as a residual: editor sessions that batch overlay updates will throw away all reuse work on every reload until a session‑scoped cache survives across reloads.

## Cross‑checks

- Public `WorkspaceSnapshot` field type changes (`Vec<…>` → `Arc<Vec<…>>`, `Option<X>` → `Option<Arc<X>>`) are absorbed by downstream consumers via auto‑deref (`crates/sifr_analysis/src/host/implementation.rs:806-811` reads `snapshot.module_graph.as_ref()?.revision` unchanged). I grepped `sifr_analysis` and `sifr_lsp` and saw no consumers that assume owning move of these payloads.
- `cache_keys.rs` already had test coverage (`document_identity_inputs_are_intentionally_omitted_from_content_keys`) protecting M9's "no document identity in content keys" rule. M10 honors that for `ParseCacheKey`/`DiagnosticsCacheKey`. The new `frontend-source-file-view` fingerprint is a separate domain that *does* include `document_version`, consistent with intent but undocumented.
- `semantic_graph_fingerprint` (`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:360-383`) embeds every module's `ModuleSignature::cache_key_input()` plus edges. This is conservative enough that any signature change anywhere invalidates every HIR/diagnostics/index cache entry. Good for correctness; explains why prune evicts everything after a public change.
- Module‑scoped fingerprints (`module_scoped_fingerprint`, reuse.rs:385-401) include module id, name, path. `ParseCacheKey` does *not* — two modules with identical source text in the same workspace will share parse cache entries, acceptable since `ParsedModule` itself does not embed module name.
- Diff for `architecture.md`, `frontend_cache_invalidation.md`, `frontend_query_architecture.md`, and the new M10 doc are coherent with each other; tracker M10 row is marked `in progress` with `pending` PR.

## Verdict

CHANGES_REQUESTED

Primary blocker: the `ExportSignature` gap on dunders (`__init__`, `__add__`, …) and class decorators (HIGH #1) lets `can_replace_module_in_project` skip reverse‑dependency invalidation when dependent‑visible class behavior actually changed. That directly violates the M10 closeout criterion "stale cache entries cannot be observed through diagnostics, HIR, or LSP queries."

Secondary concern: the `clear_module_caches` + immediate `prune_unshared` pattern means the new ref‑counted caches deliver no cross‑edit reuse beyond what `ModuleState` already pinned (HIGH #2). M10's scope explicitly promised "ref-counted parse cache" reuse "across snapshots when content … is unchanged" — the current implementation does not deliver that, and the milestone doc/tracker should either be tightened or the prune behavior corrected before closeout.

Also fix or refactor the 900‑line cap pressure on `query_diagnostics.rs` (MEDIUM #1) and tighten the doc/tracker claims about snapshot Arc retention (MEDIUM #4) before merge.
