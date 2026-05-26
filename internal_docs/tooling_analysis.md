# Sifr Tooling Analysis Architecture

status: phase36-contract-locked

## Implementation Status

- m36.1 locked the analysis, formatter, lint, LSP, editor, and VS Code contracts.
- m36.2 added `sifr_format` and `sifr_lint` as concrete workspace crates.
- m36.3 adds `sifr_analysis` as the concrete editor-query crate.
- m36.4 fills the first complete editor query layer through `sifr_analysis`.

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

m36.3 implementation:

- `AnalysisHost::open_single_file` and `AnalysisHost::open_project` wrap `sifr_frontend::FrontendContext`.
- `AnalysisHost::update_document` enforces monotonic document versions before updating the canonical frontend context.
- `AnalysisSnapshot` captures graph/source revisions and rejects stale queries after document invalidation.
- The current-workspace `SymbolIndex` is built from `sifr_frontend::ProjectAnalysisView` and `ModuleGraphView`; symbol ids include graph/source revision, module, file, kind, name, and ordinal.
- All Phase 36 editor query methods compile through `sifr_analysis`. m36.3 implements session/query plumbing, diagnostics, workspace diagnostics, document/workspace symbols, formatter handoff, lint handoff, and completion ranking infrastructure; the full feature logic for hover, references, rename edits, semantic tokens, code actions, generated Rust preview, explain diagnostic enrichment, and test metadata lands in m36.4.

m36.4 implementation:

- Editor token facts are derived through `FrontendContext::parse_module`, not a raw parser path.
- Hover, definition/declaration/type-definition, references, prepare-rename, rename, document highlights, folding ranges, selection ranges, semantic tokens, and inlay hints are token-backed and snapshot-gated.
- Diagnostics combine canonical frontend hard diagnostics with `sifr_lint` policy diagnostics.
- Code actions offer explicit Sifr policy suppression edits for lint diagnostics.
- Generated Rust preview calls the canonical `sifr_driver::compile_with_metadata` handoff and returns structured unavailability when compilation fails.
- Parity coverage lives in `verification/tooling/parity_manifest.json`, `verification/tooling/editor_query_snapshots/`, and `verification/tooling/completion_quality/`.

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

`sifr_format` formats source text through `sifr_syntax` and the Sifr Ruff fork
formatter. It must preserve comments, meaningful blank lines, string contents,
source spans needed by diagnostics, and Sifr parameter-convention syntax.
Formatting must be deterministic, idempotent, parser-round-tripped, and
equivalent between `sifr fmt`, `sifr fmt --check`, analysis formatting queries,
and LSP formatting edits.

`sifr_lint` evaluates policy diagnostics without changing compiler hard diagnostics. It owns severity resolution, suppression handling, and discovery exclusions.

Production formatter path:

- parses through `sifr_syntax`, which wraps the Sifr Ruff fork parser/AST substrate
- formats through Sifr-aware Ruff formatter rules, not a Sifr source post-processing pass
- canonicalizes `mut own` parameters to `own mut` in the Ruff formatter rule
- preserves comments, pragmas, string contents, and source-map ranges needed by diagnostics and editor edits
- supports whole-file and range formatting through the same `sifr_format` API
- supports `sifr.toml` `[format]` discovery, CLI overrides, `--isolated`, excludes, gitignore behavior, and formatter cache controls
- optionally formats Sifr docstring snippets when `docstring-code-format` is enabled
- rejects invalid source and invalid ranges through stable Sifr formatter diagnostics

m36.2 lint foundation:

- defines Sifr-owned policy metadata
- implements `# sifr: ignore[rule-id]`
- rejects blanket suppressions
- reports unknown and unused suppressions
- implements the first suppressible policy rule, `trailing-whitespace`
- supports diagnostics modes and include/exclude discovery options without applying excludes to explicit file targets

Ad hoc production-grade linter phase:

- phase contract: `issues/ad-hoc-production-grade-sifr-linter.md`
- execution tracker: `issues/ad-hoc-production-grade-sifr-linter-execution.md`
- Ruff rule-family/config audit manifest: `verification/tooling/linter_manifests/ruff_rule_config_audit.json`
- lint CLI parity manifest: `verification/tooling/linter_manifests/lint_cli_parity.json`
- rule metadata manifest: `verification/tooling/linter_manifests/lint_rule_metadata.json`
- suppression-gate manifest: `verification/tooling/linter_manifests/suppression_gate.json`
- enforcement: `python3 verification/tooling/check_linter_reuse_contract.py`

The ad hoc phase keeps Ruff Python rule families, Ruff's Python rule registry,
Ruff Server diagnostic behavior, and Python project/module semantics out of
Sifr lint authority. Milestone 1 locks those decisions in machine-readable
manifests before config, discovery, parser-aware suppressions, phase-gated
orchestration, fix support, or editor actions are expanded.

## Generated Rust And Test Metadata

Generated Rust preview uses compiler/codegen APIs and source maps. It must be cancellable, revision-checked, budgeted under `lsp-generated-rust-preview`, and must not return partial generated code after a document change invalidates the request.

Test discovery and editor test commands use Sifr CLI/test-runner metadata. Projects with no tests return an empty test tree rather than guessed Python-style tests.
