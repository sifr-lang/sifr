# Sifr Syntax Architecture

status: active

`crates/sifr_syntax/` is the Sifr-owned wrapper around the checked-in Sifr Ruff fork parser, AST token stream, source positions, and parser diagnostic categories.

## Ownership

`sifr_syntax` owns:

- module parse entrypoints over Sifr source text
- stable parser diagnostic mapping before semantic lowering
- token views consumed by frontend query architecture fixtures and developer tooling surface syntax asset drift checks
- source text byte/line/column conversion helpers

`sifr_syntax` does not own name resolution, lowering, type checking, ownership analysis, semantic diagnostics, formatting policy, editor queries, or code generation.

## Current API

- `parse_module(source, context)` returns a `ParsedModule` with a Sifr-facing suite and token view.
- `parse_module_suite(source, context)` returns only the Sifr-facing AST suite for compiler paths that do not need token/trivia data.
- `parse_module_raw(source, context)` preserves the raw Ruff parsed module for the few low-level compiler internals that need raw parser metadata while still going through the Sifr-owned syntax wrapper.
- `SourceText` converts UTF-8 text positions and byte offsets for frontend/tooling source-map use.

## Fork Update Rules

The current Ruff fork revision is recorded in `verification/areas/performance/ruff_fork_revalidation.json`. Token fixtures in `verification/areas/performance/sifr_syntax_token_fixtures/` record the same revision. `verification/areas/performance/check_ruff_fork_update_rules.py` fails when the submodule revision changes without fixture revalidation evidence.

## Migration State

frontend query architecture frontend query routing removed direct raw parser use from `sifr_driver` and the CLI. CLI mode detection, project discovery, stdlib bootstrap, and driver tests now parse through `sifr_syntax`/`sifr_frontend`; the split-brain guardrail rejects new `sifr_python_parser`, `ruff_python_parser`, or raw parse entrypoints outside approved syntax/frontend/HIR boundaries.
