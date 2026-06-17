# TypeScript-Go Architecture Transfer: Snapshot Reuse And Structural Replacement

status: merged via [#2251](https://github.com/sifr-lang/sifr/pull/2251)

snapshot-reuse surface introduces the first process-local reuse storage after snapshot-reuse surface locked cache
identity. `sifr_frontend` now keeps ref-counted cache entries for parse trees,
source-map file views, lowered HIR, module diagnostics, and module symbol
indexes. Cache entries are addressed by the snapshot-reuse surface typed key families and are held
by `Arc` so active module states can share immutable data by identity across
queries and edits. Cache maps prune entries whose only remaining strong
reference is the cache itself after source or query invalidation releases the
active module state.

`FrontendContext` still invalidates by source hash and semantic boundary. A
private one-module body replacement is accepted only when parsing succeeds and
the module import/export signature is unchanged. Export signatures include
public constants, class decorators, and all class-body methods because
`collect_module_exports` exposes the full class method table to dependents.
Signature shapes use Ruff AST comparable views so source ranges and node indexes
do not turn whitespace-only edits into public API changes. These replacements
preserve graph revision and unchanged module cache identities.
Source-map and module graph views are still rebuilt when source-hash metadata
changes, so callers cannot observe stale file hashes through diagnostics, HIR,
or LSP-facing source maps.

`WorkspaceSnapshot` now stores immutable snapshot payloads behind `Arc`:
overlays, dependency records, source maps, module graphs, compiler options, and
package/config identity. Unchanged snapshot payloads are reused by pointer
identity across repeated snapshots. Snapshots do not yet retain per-module parse,
HIR, diagnostics, or analysis entries directly; those entries are retained while
the owning `FrontendContext` module state or another query view holds them.

Current limitations:

- cache residency is process-local and bounded by active frontend context state
- source-file view cache keys include document version because the view carries
  editor metadata, even though snapshot-reuse surface content keys intentionally omit document
  identity inputs
- graph/source-map view reuse is conservative when metadata changes
- package graph, lint, format, and flow graph reuse remain future work even
  though snapshot-reuse surface defines their key identities

Validation so far:

- `cargo test -p sifr_frontend ref_counted_module_caches_reuse_identity_on_hits`
- `cargo test -p sifr_frontend structural_one_module_replacement_reuses_unchanged_cache_entries`
- `cargo test -p sifr_frontend document_version_only_update_recaches_source_file_view`
- `cargo test -p sifr_frontend reverse_dependent_invalidation_reuses_unchanged_parse_entry`
- `cargo test -p sifr_frontend dunder_method_signature_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend single_underscore_method_signature_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend class_decorator_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend leading_whitespace_edit_preserves_export_signature_scope`
- `cargo test -p sifr_frontend public_constant_value_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend` -> PASS, 38 tests
- `cargo check --workspace`
- `cargo fmt --check`
- `cargo clippy -p sifr_frontend -- -D warnings`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- Claude reviewer pass 1 -> CHANGES_REQUESTED
- Claude reviewer pass 2 -> SATISFIED with residual recommendations
- Claude reviewer pass 3 -> SATISFIED
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time 330.61s, advisories: warm wall-time budget exceeded; group skew is high
