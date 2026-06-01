# Frontend Cache Invalidation

status: active

Phase 35 v1 frontend caching is process-local and module-granular.

TypeScript-Go architecture transfer M1 note: this document describes the
pre-session cache behavior. Dependency-sensitive invalidation, dirty-scope
classification, module signatures, structural replacement, and snapshot-scoped
cache reuse are planned across M6-M10. M6 adds the first session-level
dirty-scope vocabulary and event compaction; dependency-sensitive module
replacement remains M7-M10 work and is not implemented by the current
`FrontendContext::update_module_source` path.

TypeScript-Go architecture transfer M2 note: project loading can now record
source-provider dependency reads before snapshots exist. The tracked records
include successful file reads, directory reads, file and directory probes,
canonicalization, and failed lookups. These records are not yet consumed for
dependency-sensitive invalidation; M3-M6 wire them into session snapshots and
M7 consumes them for dependency-sensitive invalidation.

TypeScript-Go architecture transfer M3 note: `WorkspaceSession` now owns the
tracked dependency records and freezes them into `WorkspaceSnapshot` alongside
source maps, module graphs, overlay records, compiler options, package/config
identity, and cache-registry handles. M6 adds the session-level dirty-scope
vocabulary and event compaction; dependency-sensitive invalidation remains M7
work.

TypeScript-Go architecture transfer M4 note: `WorkspaceSnapshot` now carries a
dirty-scope report slot consumed by `AnalysisSnapshot`. The report is
conservative in M4: reloads, overlay changes, and analysis document updates mark
workspace scope, while precise event compaction and dependency-sensitive dirty
scope remain later work. M6 replaces that placeholder with explicit
session-level dirty-scope reports and event compaction; dependency-sensitive
module invalidation remains M7.

## Cache State

Each `FrontendContext` module owns cached parse, lower, diagnostics, and analysis entries. Query results include metadata with:

- query kind
- cache hit or miss
- graph revision
- source revision

## Update Rules

`FrontendContext::update_module_source`:

1. Computes the new source hash.
2. Preserves query cache entries when the source hash is unchanged.
3. Invalidates the changed module's parse entry when source text changes.
4. Resets lowered HIR, diagnostics, analysis, and exported definition state for all modules in the context so downstream importers cannot observe stale dependency exports.
5. Invalidates parse, lower, type-check, module diagnostics, project diagnostics, module analysis, and project analysis query kinds in the returned report.
6. Advances the source revision on every update.
7. Advances the graph revision when text changes and rebuilds deterministic import edges.
8. Records the changed file, document version transition, invalidated modules, and invalidated query kinds in `InvalidationReport`.

Dependency-sensitive query recomputation lowers local imports before the importing module and repopulates external definitions through the same `sifr_frontend` export collector used by driver project compilation.

## Verification

`verification/performance/check_frontend_cache_contract.py` runs focused Rust tests proving deterministic cached diagnostics and source-update invalidation. `cargo test -p sifr_frontend` also covers project import-edge identity and dependency-export diagnostics through the canonical frontend. The quick validation lane invokes the cache contract script before the broader compiler tests.
