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

## Driver Consumption

Phase 35 m35.4b routes `check`, `build`, `run`, `emit`, project compilation, and test-runner frontend flows through `sifr_frontend` without preserving duplicate semantics-bearing driver frontend paths. The driver remains responsible for stdlib bootstrap/cache plumbing, build planning, codegen invocation, Cargo/rustc execution, and renderer/CLI presentation.

The deleted driver migration shims were:

- `crates/sifr_driver/src/frontend/module_lowering.rs`
- `crates/sifr_driver/src/frontend/parser_diagnostics.rs`
- `crates/sifr_driver/src/project/exports.rs`

`verification/performance/check_split_brain_guardrail.py` now has no driver/CLI migration allowlist. New parser/lowering/type-check/semantic diagnostic entrypoints outside `sifr_syntax`, `sifr_frontend`, and approved `sifr_hir` internals fail the local validation lane.

## Extension Boundary

Phase 36 editor analysis must consume `sifr_frontend` query results. LSP and editor adapters must not parse, lower, type-check, derive semantic diagnostics, or inspect HIR directly for semantic answers.
