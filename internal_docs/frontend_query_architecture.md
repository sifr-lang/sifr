# Frontend Query Architecture

status: active

`crates/sifr_frontend/` is the canonical process-local frontend query crate. It owns session loading, module identities, graph views, source-map views, parse/lower/type-check/diagnostic/analysis query methods, cache metadata, and update invalidation reports. Source text, line maps, source-file metadata, and UTF-8/UTF-16/UTF-32 position conversions are provided by the lower-level `sifr_source` crate so frontend, diagnostics, syntax, and LSP do not keep separate source-position authorities.

## Current Boundary

`FrontendContext` supports:

- single-file context loading with optional external definitions for driver integration
- project loading from an entrypoint directory with the entrypoint pinned to `ModuleId(0)`
- deterministic `FileId` and `ModuleId` assignment within a context revision
- module graph and source map inspection, including real source-map position/range round trips for registered source files
- parse, lower, type-check, module diagnostics, project diagnostics, module analysis, and project analysis queries
- query metadata with cache hit/miss status plus graph/source revisions
- dependency-first project lowering so imported module exports are available before importers are checked
- canonical HIR compile helpers used by the driver for CLI, project, and test-runner frontend flows

`sifr_frontend` consumes `sifr_syntax` for parsing and `sifr_hir` for lowering and semantic diagnostics. It does not invoke codegen, rustc, cargo, CLI policy, or build artifact creation.

TypeScript-Go architecture transfer M1 note: the `FrontendContext` API described
here is the pre-session, process-local query facade. M3/M4 own re-expressing
this surface around `WorkspaceSession` and immutable `WorkspaceSnapshot`
handles; M2 owns moving semantic file reads behind `SourceProvider` first.

TypeScript-Go architecture transfer M2 note: `sifr_frontend` now exposes the
source-provider boundary. `FrontendContext::load_project_with_provider` accepts
any `SourceProvider`, `load_project_tracked` returns dependency reads captured
by `TrackingSourceProvider`, and `OverlaySourceProvider` can substitute unsaved
buffer text for disk files without mutating disk state. The overlay record owns
URI, path, document version, source hash, source text/line map, and disk-match
state; M3 will move overlay lifecycle ownership into `WorkspaceSession`.

TypeScript-Go architecture transfer M3 note: `WorkspaceSession` now owns the
overlay table, last loaded `FrontendContext`, tracked provider dependencies,
workspace revision, snapshot ids, compiler options, package/config identity, and
cache registry generation handles. `WorkspaceSnapshot` freezes inspectable
source-map and module-graph views, but analysis queries still use the existing
`AnalysisHost` revision-token snapshot until M4 migrates them.

TypeScript-Go architecture transfer M4 note: `sifr_analysis::AnalysisHost` now
owns a `WorkspaceSession`, and `AnalysisSnapshot` carries a frozen
`WorkspaceSnapshot` handle plus the analysis graph/source revision. LSP request
handlers capture an `AnalysisSnapshot` at the document-store boundary and route
diagnostics, symbols, formatting, generated Rust preview, and editor/navigation
queries through snapshot methods while execution remains serialized.

TypeScript-Go architecture transfer M9 note: `sifr_frontend::cache_keys`
defines the deterministic identity layer for future snapshot reuse. Query/cache
families now have typed key structures that include source content, compiler,
workspace, package/config, and policy fingerprints before M10 adds reusable
cache storage.

TypeScript-Go architecture transfer M10 note: `FrontendContext` now owns
ref-counted cache storage for parse trees, source-map file views, lowered HIR,
module diagnostics, and module symbol indexes. The storage uses the M9 typed key
families plus a semantic graph fingerprint for HIR and downstream entries, so
unchanged dependents do not observe stale imported signatures. Changed modules
drop their parse entries, while unchanged reverse dependents retain content-valid
parse entries across semantic invalidation. `WorkspaceSnapshot` freezes reusable
`Arc` payloads for overlays, dependency records, source maps, module graphs,
compiler options, and package/config identity; per-module parse/HIR/diagnostic
entries remain retained by active frontend query state in M10.

## Driver Consumption

Phase 35 m35.4b routes `check`, `build`, `run`, `emit`, project compilation, and test-runner frontend flows through `sifr_frontend` without preserving duplicate semantics-bearing driver frontend paths. The driver remains responsible for stdlib bootstrap/cache plumbing, build planning, codegen invocation, Cargo/rustc execution, and renderer/CLI presentation.

The deleted driver migration shims were:

- `crates/sifr_driver/src/frontend/module_lowering.rs`
- `crates/sifr_driver/src/frontend/parser_diagnostics.rs`
- `crates/sifr_driver/src/project/exports.rs`

`verification/performance/check_split_brain_guardrail.py` now has no driver/CLI migration allowlist. New parser/lowering/type-check/semantic diagnostic entrypoints outside `sifr_syntax`, `sifr_frontend`, and approved `sifr_hir` internals fail the local validation lane.

## Extension Boundary

Phase 36 editor analysis must consume `sifr_frontend` query results. LSP and editor adapters must not parse, lower, type-check, derive semantic diagnostics, or inspect HIR directly for semantic answers.
