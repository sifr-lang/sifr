# Frontend Cache Invalidation

status: active

Phase 35 v1 frontend caching is process-local and module-granular.

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
3. Invalidates parse, lower, type-check, module diagnostics, project diagnostics, module analysis, and project analysis entries for changed source.
4. Advances the source revision on every update.
5. Advances the graph revision when text changes and rebuilds deterministic import edges.
6. Records the changed file, document version transition, invalidated modules, and invalidated query kinds in `InvalidationReport`.

## Verification

`verification/performance/check_frontend_cache_contract.py` runs focused Rust tests proving deterministic cached diagnostics and source-update invalidation. The quick validation lane invokes this script before the broader compiler tests.
