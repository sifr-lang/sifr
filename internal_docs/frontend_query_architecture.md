# Frontend Query Architecture

status: active

`crates/sifr_frontend/` is the canonical process-local frontend query crate. It owns session loading, module identities, graph views, source-map views, parse/lower/type-check/diagnostic/analysis query methods, cache metadata, and update invalidation reports.

## Current Boundary

`FrontendContext` supports:

- single-file context loading with optional external definitions for driver integration
- project loading from an entrypoint directory
- deterministic `FileId` and `ModuleId` assignment within a context revision
- module graph and source map inspection
- parse, lower, type-check, module diagnostics, project diagnostics, module analysis, and project analysis queries
- query metadata with cache hit/miss status plus graph/source revisions

`sifr_frontend` consumes `sifr_syntax` for parsing and `sifr_hir` for lowering and semantic diagnostics. It does not invoke codegen, rustc, cargo, CLI policy, or build artifact creation.

## Driver Migration

Phase 35 m35.4a introduced the crate and routed parser diagnostics through `sifr_syntax`. m35.4b must route `check`, `build`, `run`, `emit`, project compilation, and test-runner frontend flows through `sifr_frontend` without preserving duplicate semantics-bearing paths.

## Extension Boundary

Phase 36 editor analysis must consume `sifr_frontend` query results. LSP and editor adapters must not parse, lower, type-check, derive semantic diagnostics, or inspect HIR directly for semantic answers.
