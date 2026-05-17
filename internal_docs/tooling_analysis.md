# Sifr Tooling Analysis Architecture

status: phase36-contract-locked

## Ownership

Phase 36 locks the editor analysis boundary to these Sifr-owned crates:

- `sifr_format`: formatting over `sifr_syntax` tokens, trivia, comments, and source maps.
- `sifr_lint`: suppressible policy-rule metadata, severity resolution, suppression parsing, and exclusion policy.
- `sifr_analysis`: editor-oriented semantic queries over `sifr_frontend`, `sifr_diagnostics`, `sifr_format`, `sifr_lint`, and approved read-only compiler views.
- `sifr_lsp`: LSP 3.17 protocol adapter over `sifr_analysis`.

`sifr_analysis` is the only editor-query crate. LSP handlers, VS Code commands, and other editor adapters must not traverse HIR or raw syntax directly to answer semantic questions.

## Analysis Host Contract

`sifr_analysis::AnalysisHost` owns the editor session model for:

- project and single-file opening
- open-document overrides
- document versions
- coherent source snapshots
- invalidation reports from `sifr_frontend`
- stale-version rejection
- current-workspace symbol identity
- editor query results and metadata

The public query surface is the Phase 36 editor query contract: diagnostics, workspace diagnostics, completion, hover, signature help, definition, declaration, type definition, references, prepare rename, rename, document symbols, workspace symbols, semantic tokens, inlay hints, document highlights, folding ranges, selection ranges, type hierarchy, code actions, formatting, generated Rust preview, explain diagnostic, test discovery, and test command metadata.

Every query result must carry enough revision metadata to prove it was produced from the snapshot captured for that request.

## Required Frontend Exports

Phase 35 exports are sufficient for m36.1 to proceed:

- `sifr_syntax` exposes parse, token, text-position, and syntax range primitives.
- `sifr_frontend` exposes `FrontendContext`, module/source graph identity, source maps, query metadata, invalidation reports, diagnostics, source-map lookup stubs, symbol/type-display/editor-query view structs, and selection-range view shape.
- `sifr_diagnostics` owns canonical diagnostic rendering and schema data.
- `sifr_codegen` remains the generated-Rust authority and will be called through compiler-owned handoff APIs when preview support lands.

Missing implementation detail inside a placeholder view is not a blocker for m36.2, but adding a second parser, lowerer, checker, diagnostic engine, formatter, linter, or codegen path is blocked by the tooling guardrails.

## Query Boundaries

Allowed dependencies:

- `sifr_analysis` may call `sifr_frontend`, `sifr_diagnostics`, `sifr_format`, `sifr_lint`, and approved read-only compiler/codegen views.
- `sifr_format` may call `sifr_syntax` and `sifr_diagnostics`.
- `sifr_lint` may call `sifr_frontend`, `sifr_diagnostics`, and approved read-only syntax/HIR views.
- `sifr_lsp` may call `sifr_analysis` and protocol conversion helpers only.

Forbidden dependencies and behavior:

- no production dependency on `ty_python_semantic`, Python project semantics from `ty_project`, Python environment discovery, Ruff Server semantic behavior, Pyright, or Python language servers
- no raw parser entrypoint outside `sifr_syntax` and approved low-level compiler crates
- no editor-owned type checker, diagnostics derivation, formatter, linter, or codegen logic
- no direct HIR traversal from LSP handlers for semantic answers

## Diagnostics, Rules, And Suppressions

Hard correctness diagnostics remain compiler diagnostics and are not suppressible or downgradeable:

- parse errors
- soundness-critical type errors
- ownership, move, and borrow errors
- `Result` and `Option` safety errors
- runtime-panic-prevention errors
- workspace/import errors that would make compilation ambiguous or unsound

Policy rules are the only suppressible diagnostics. Rule metadata is Sifr-owned and includes rule id, summary, docs URL, default level, status, source location, and configured level. Suppression syntax is:

```sifr
value = legacy_call()  # sifr: ignore[rule-id]
```

Blanket `sifr: ignore`, unknown rule ids, and unused suppressions produce deterministic policy diagnostics. Python `type: ignore` comments do not suppress Sifr diagnostics.

## Formatting And Linting

`sifr_format` formats source text through `sifr_syntax` only. It must preserve comments, meaningful blank lines, string contents, source spans needed by diagnostics, and Sifr parameter-convention syntax. Formatting must be deterministic, idempotent, parser-round-tripped, and equivalent between `sifr fmt`, `sifr fmt --check`, analysis formatting queries, and LSP formatting edits.

`sifr_lint` evaluates policy diagnostics without changing compiler hard diagnostics. It owns severity resolution, suppression handling, and discovery exclusions.

## Generated Rust And Test Metadata

Generated Rust preview uses compiler/codegen APIs and source maps. It must be cancellable, revision-checked, budgeted under `lsp-generated-rust-preview`, and must not return partial generated code after a document change invalidates the request.

Test discovery and editor test commands use Sifr CLI/test-runner metadata. Projects with no tests return an empty test tree rather than guessed Python-style tests.
