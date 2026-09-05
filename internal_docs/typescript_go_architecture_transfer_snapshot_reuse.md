# TypeScript-Go Architecture Transfer: Snapshot Reuse And Structural Replacement

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
