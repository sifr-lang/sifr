# Review: TypeScript-Go architecture transfer M10 — Snapshot Reuse and Structural Replacement (pass 2)

Branch: `wave_tsgo_m10_snapshot_reuse`
Files re-reviewed (dirty tree):

- `crates/sifr_frontend/src/cache_keys.rs`
- `crates/sifr_frontend/src/frontend_reuse.rs`
- `crates/sifr_frontend/src/graph_cache_and_queries.rs`
- `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs`
- `crates/sifr_frontend/src/lib.rs`
- `crates/sifr_frontend/src/module_signatures.rs`
- `crates/sifr_frontend/src/query_diagnostics.rs`
- `crates/sifr_frontend/src/query_diagnostics_m10_tests.rs` (new)
- `crates/sifr_frontend/src/workspace_session.rs`
- `internal_docs/architecture.md`
- `internal_docs/frontend_cache_invalidation.md`
- `internal_docs/frontend_query_architecture.md`
- `internal_docs/typescript_go_architecture_transfer_m10_snapshot_reuse.md`
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`

I re-ran `cargo test -p sifr_frontend` (35 passed), `cargo clippy -p sifr_frontend -- -D warnings`, `cargo fmt --check`, and `python3 scripts/check_file_size_guardrails.py`. All pass.

## Status of pass 1 blockers

### HIGH #1 — Dunder / `__init__` / class decorator gap → **resolved**

`crates/sifr_frontend/src/module_signatures.rs:140` now selects class-body methods via `is_class_api_member` (public OR dunder, where dunder is `len() > 4 && starts_with("__") && ends_with("__")`), and `module_signatures.rs:168` includes `class.decorator_list` in the class shape. New regression coverage in `crates/sifr_frontend/src/query_diagnostics_m10_tests.rs:44-85` exercises an `__init__` parameter-type change and asserts `WorkspaceDirtyScope::ReverseDependencies` + `ExportSignatureChanged`. The class shape also covers `class.arguments` and `class.type_params`, so base-class and type-parameter changes propagate.

### HIGH #2 — `clear_module_caches` + immediate `prune_unshared` evicting unchanged parse entries → **resolved**

`crates/sifr_frontend/src/graph_cache_and_queries.rs:825-845` now takes a second `modules_with_source_changes` slice and only nulls `module_state.parsed` for that subset. `update_module_source` calls `clear_module_caches(&invalidated_modules, &[module])` at `graph_cache_and_queries.rs:516`, so reverse dependents keep their parse `Arc` and survive `prune_unshared`. New regression `reverse_dependent_invalidation_reuses_unchanged_parse_entry` (`graph_cache_and_queries/reuse.rs:600-638`) asserts both `CacheStatus::Hit` and a stable `parse_cache_identity` for `main` after a public-signature change in `helper`.

### MEDIUM — 900-line cap pressure on `query_diagnostics.rs` → **resolved**

`query_diagnostics.rs` is now 861 lines; the M10-specific tests were moved into a sibling `query_diagnostics_m10_tests.rs` (106 lines) wired in `lib.rs:18-19` under `#[cfg(test)]`. `cache_keys.rs` remains at 899 (one-line headroom — worth keeping in mind, but not a blocker).

### MEDIUM — docs/tracker overstating snapshot-pinned retention → **resolved**

The locked-decision wording in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:569` and the adoption-table row at `:413` now say cache entries are retained by "active frontend context state and retained snapshot payloads, then released when no live owner can observe them." `internal_docs/typescript_go_architecture_transfer_m10_snapshot_reuse.md:22-26` is explicit that per-module parse/HIR/diagnostics/analysis Arcs are not pinned by snapshots in M10. `frontend_query_architecture.md` and `frontend_cache_invalidation.md` updates are coherent with that scope.

## New findings (ordered by severity)

### MEDIUM — `ExportSignature` still excludes non-dunder underscore-prefixed methods that HIR exports into `Type::Class.methods`

`crates/sifr_frontend/src/module_signatures.rs:214-216` defines `is_class_api_member = is_public(name) || is_dunder_name(name)`, so a method like `_helper` is filtered out of the export shape.

However, the HIR class lowering at `crates/sifr_hir/src/lower/classes/class_body_lowering.rs:262-383` puts every `FunctionDef` in the class body into either `class.methods` or `class.operator_impls` — there is no visibility filter on `_helper`. The frontend then exports the full `class.methods` (plus `class.operator_impls`) into the dependent-visible `Type::Class.methods` at `crates/sifr_frontend/src/query_diagnostics.rs:362-393`, again without filtering by leading underscore.

Consequence: if `helper.sifr` contains `class Box: def _helper(self) -> int: ...` and `main.sifr` calls `Box()._helper()`, then changing `_helper`'s return type from `int` to `str` does not alter `ExportSignature`. `signatures_can_replace_module_in_project` returns `true`, `update_module_source` only invalidates `[helper]`, and `main`'s lowered HIR keeps the stale `Box` class type — type-check against `Box()._helper()` will not flag the mismatch until something else invalidates `main`.

This is a narrower version of pass 1's HIGH #1. Pass 1 explicitly suggested "include all class-body members … or split a separate 'class type identity' signature that mirrors what `collect_module_exports` actually exports." The current fix opted for "public + dunder" only, but Sifr's Python-derived semantics do not restrict `_helper` calls from external modules at the type-check layer, so the gap remains exercisable.

Two cheap closures:
- Drop the visibility filter on class-body `FunctionDef` members entirely (any class-body method is part of the class type identity).
- Or add a regression test that proves Sifr rejects `Box()._helper()` from a dependent (which would prove the gap is unobservable); if it does not reject, fold the leading-underscore methods into `is_class_api_member`.

### MEDIUM — `ExportSignature` shape uses Debug format on AST nodes that contain `TextRange` / `AtomicNodeIndex`

`crates/sifr_frontend/src/module_signatures.rs:127-191` builds the shape with `format!("{:?}", function.parameters)` (and similar for `function.returns`, `function.decorator_list`, `function.type_params`, `class.arguments`, `class.decorator_list`, `class.type_params`, `assign.annotation`, `assign.value`). `third_party/ruff/crates/ruff_python_ast/src/nodes.rs:3101-3110` shows `Parameters` derives `Debug` over the full struct including `range: TextRange` and `node_index: AtomicNodeIndex`. Equivalent ranges are embedded throughout the AST node types reached by these `Debug` calls.

That means an edit that inserts a blank line above a class or function — purely cosmetic — shifts the `TextRange` of every node below it. `ExportSignature` then differs, `can_replace_module_in_project` returns `false`, and the reverse dependency closure is invalidated. The current correctness criteria are met (no under-invalidation), but the M10 reuse promise ("private body edits invalidate only the changed module when public/import signatures are unchanged") leaks into over-invalidation on whitespace-only or comment-only edits.

This is pre-existing from M7 (pass 1 LOW #3) but M10 is the first milestone where reverse-dep reuse is load-bearing. Recommendation: normalize away `range` / `node_index` before fingerprinting (or define a small visitor that stringifies semantic-only fields), and ideally before merge land a test that proves a leading-whitespace-only edit lands as `OneModule`/`None`.

### LOW — `update_module_source` parses replacement source but does not seed `reuse_caches.parse`

`crates/sifr_frontend/src/graph_cache_and_queries.rs:483-488` calls `sifr_syntax::parse_module(source.as_str(), …)` solely to derive `new_signature`, then discards the `ParsedModule`. When `ensure_parsed` runs for the same module next, it re-parses from source (`reuse.rs` … `graph_cache_and_queries.rs:629-646`). Pass 1 flagged this; the fix is straightforward (insert into `reuse_caches.parse` under the new key and stash the `Arc` on `module_state`) but was deferred. Not a correctness issue.

### LOW — `prune_unshared` runs twice per `update_module_source`

`clear_module_caches` calls `self.reuse_caches.prune_unshared()` at `graph_cache_and_queries.rs:844`, and `update_module_source` then calls it again at `:557` on every code path. The second prune is necessary for the `WorkspaceDirtyScope::None` (document-version-only) path where `clear_module_caches` is not called, but the redundancy on the text-changed path is a small cost. Refactor (call once at the bottom of `update_module_source`, drop the call inside `clear_module_caches`) would tighten this.

### LOW — `rebuild_edges` still bypasses `reuse_caches.parse`

`graph_cache_and_queries.rs:775-808` calls `sifr_syntax::parse_module` directly for every module. After a non-replaceable update, every module is re-parsed even though only the changed module's parse-cache key changed. Pass 1 LOW; still applicable. Plumbing this through `reuse_caches` would salvage parse reuse on the import-graph-restructure path.

### LOW — `source_map_arc_for_reuse` constructs a `SourceMapView` by cloning `SourceFileView` contents rather than sharing leaf `Arc<SourceFileView>`s

`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:164-166` collects `self.cached_source_file_view(index).as_ref().clone()` into a `Vec<SourceFileView>`. The `Arc<SourceFileView>` produced by the ref-counted cache is therefore not shared with the snapshot-facing source map. Identity reuse is verified through `FrontendCacheEntryIdentity`, not via `Arc::ptr_eq`. Pass 1 LOW; deferring is fine, but `SourceMapView.files` could become `Vec<Arc<SourceFileView>>` or `Arc<[SourceFileView]>` for real cross-snapshot leaf sharing.

## Residual risks and missing tests

- **Class decorator change has implementation but no regression test.** `module_signatures.rs:168` includes `class.decorator_list`, but no test exercises adding/removing a class decorator and asserting reverse-dependency invalidation. Worth one targeted test (e.g., `@dataclass`-style annotation toggled on/off) before closeout, since the implementation is otherwise inferred from the dunder test.
- **Single-underscore class-method changes are unguarded.** See MEDIUM #1 above; even if the team decides this is out of scope, the decision should be recorded in `typescript_go_architecture_transfer_m10_snapshot_reuse.md` so a future reader knows the gap is deliberate.
- **No test that whitespace-only edits stay `OneModule`/`None`.** The over-invalidation risk in MEDIUM #2 is invisible until exercised. A test like "insert a leading newline; assert `WorkspaceDirtyScope::OneModule`" would lock the contract.
- **`can_replace_module_in_project` is still only exercised through `FrontendContext`**, not the `WorkspaceSession` snapshot consumer surface. Pass 1 noted this; M10 still does not cover the LSP/analysis path end-to-end. Out-of-scope for closeout if explicitly deferred.
- **`reload()` discards the entire `FrontendContext`** including all reuse caches (`workspace_session.rs:280-293`). Editor batching scenarios will throw away every reuse entry on each reload. M10 deliberately scopes this to "future work," but it remains the largest gap between the milestone's intent and its observable behavior.

## Cross-checks

- `signatures_can_replace_module_in_project` (`reuse.rs:141-149`) treats parse failure as "not replaceable" and reuses the existing import/export equality check. Reverse-dependency scope selection (`graph_cache_and_queries.rs:543-549`) routes parse-failure through `GraphStructure`, which is conservatively correct.
- `clear_module_caches` correctly preserves `module_state.source_file_view` (pass 1 LOW #5 was about the symmetry comment; the diff leaves the asymmetry in place and the runtime behavior is correct because source-file views are reset directly in `update_module_source`).
- The new `query_diagnostics_m10_tests.rs` mirrors the helper-project / `temp_project_dir` pattern from the existing `query_diagnostics::tests` module — no new test infrastructure introduced.
- `WorkspaceSnapshot` `Arc::ptr_eq` assertions (`workspace_session.rs:720-746`) verify cross-snapshot identity reuse for overlays, source dependencies, source map, module graph, compiler options, and package-config identity. The semantic-graph fingerprint test contract is unchanged from M9.
- Tracker row (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:19`) is in `in progress` / `pending PR` state, consistent with the M10 doc.

## Verdict

SATISFIED

The four pass 1 blockers are addressed: dunder/init/decorator coverage, parse-entry preservation for unchanged reverse dependents, file-size headroom on `query_diagnostics.rs`, and the corrected snapshot-retention wording in docs and tracker. New regression tests (`dunder_method_signature_update_invalidates_reverse_dependents`, `reverse_dependent_invalidation_reuses_unchanged_parse_entry`, `public_constant_value_update_invalidates_reverse_dependents`) lock those behaviors in.

The remaining MEDIUM findings (single-underscore class methods exempt from the export signature, and Debug-format sensitivity to source-position changes) are real but limited: the first is an arguable scope decision pending a language-visibility statement, and the second causes over-invalidation rather than stale reads. Recommend recording the underscore-visibility decision in the M10 doc and adding the missing decorator-change and whitespace-only-edit regression tests before milestone close, but neither blocks merge of the current diff.
