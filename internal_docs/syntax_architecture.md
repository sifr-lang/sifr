# Sifr Syntax Architecture

status: active

`crates/sifr_syntax/` is the Sifr-owned wrapper around the checked-in Sifr Ruff fork parser, AST token stream, source positions, and parser diagnostic categories.

## Ownership

`sifr_syntax` owns:

- module parse entrypoints over Sifr source text
- stable parser diagnostic mapping before semantic lowering
- token views consumed by Phase 35 fixtures and Phase 36 syntax asset drift checks
- source text byte/line/column conversion helpers

`sifr_syntax` does not own name resolution, lowering, type checking, ownership analysis, semantic diagnostics, formatting policy, editor queries, or code generation.

## Current API

- `parse_module(source, context)` returns a `ParsedModule` with a Sifr-facing suite and token view.
- `parse_module_raw(source, context)` preserves the raw Ruff parsed module for migration-source driver paths that still need raw AST access before m35.4b removes duplicate parser ownership.
- `SourceText` converts UTF-8 text positions and byte offsets for frontend/tooling source-map use.

## Fork Update Contract

The current Ruff fork revision is recorded in `verification/performance/ruff_fork_revalidation.json`. Token fixtures in `verification/performance/sifr_syntax_token_fixtures/` record the same revision. `verification/performance/check_ruff_fork_update_contract.py` fails when the submodule revision changes without fixture revalidation evidence.

## Migration State

`sifr_driver::frontend::parser_diagnostics` delegates to `sifr_syntax::parse_module_raw`. Existing raw parser use in CLI mode detection, stdlib bootstrap, tests, and codegen tests remains an explicit m35.4b migration target.
