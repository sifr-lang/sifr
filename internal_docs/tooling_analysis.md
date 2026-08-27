# Sifr Tooling Analysis Architecture

status: tooling-rules-locked

## Implementation Status

- tooling lock locked the analysis, formatter, lint, LSP, editor, and VS Code rules.
- formatter/linter foundation added `sifr_format` and `sifr_lint` as concrete workspace crates.
- analysis-host foundation adds `sifr_analysis` as the concrete editor-query crate.
- editor-query layer fills the first complete editor query layer through `sifr_analysis`.

## Ownership

developer tooling surface locks the editor analysis boundary to these Sifr-owned crates:

- `sifr_format`: formatting over `sifr_syntax` tokens, trivia, comments, and source maps.
- `sifr_lint`: suppressible policy-rule metadata, severity resolution, suppression parsing, and exclusion policy.
- `sifr_analysis`: editor-oriented semantic queries over `sifr_frontend`, `sifr_diagnostics`, `sifr_format`, `sifr_lint`, and approved read-only compiler views.
- `sifr_lsp`: LSP 3.17 protocol adapter over `sifr_analysis`.

`sifr_analysis` is the only editor-query crate. LSP handlers, VS Code commands, and other editor adapters must not traverse HIR or raw syntax directly to answer semantic questions.

## Analysis Host Rules

`sifr_analysis::AnalysisHost` owns the editor session model for:

- project and single-file opening
- open-document overrides
- document versions
- coherent source snapshots
- invalidation reports from `sifr_frontend`
- stale-version rejection
- current-workspace symbol identity
- editor query results and metadata

The public query surface is the developer tooling surface editor query rules: diagnostics, workspace diagnostics, completion, hover, signature help, definition, declaration, type definition, references, prepare rename, rename, document symbols, workspace symbols, semantic tokens, inlay hints, document highlights, folding ranges, selection ranges, type hierarchy, code actions, formatting, generated Rust preview, explain diagnostic, test discovery, and test command metadata.

Every query result must carry enough revision metadata to prove it was produced from the snapshot captured for that request.

analysis-host foundation implementation:

- `AnalysisHost::open_single_file` and `AnalysisHost::open_project` wrap `sifr_frontend::FrontendContext`.
- `AnalysisHost::update_document` enforces monotonic document versions before updating the canonical frontend context.
- `AnalysisSnapshot` captures graph/source revisions and rejects stale queries after document invalidation.
- The current-workspace `SymbolIndex` is built from `sifr_frontend::ProjectAnalysisView` and `ModuleGraphView`; symbol ids include module, file, kind, name, and ordinal so dirty-bucket refresh and cold rebuild paths preserve identity for unchanged symbols.
- TypeScript-Go bucketed index adds bucket readiness over workspace/package/stdlib symbol and import entries. Package and stdlib buckets are explicit unavailable states until frontend graph views carry those identities. When a document update invalidates known modules and an index already exists, `AnalysisHost` refreshes only those dirty buckets before completion, workspace-symbol, and import-symbol queries reuse the clean buckets.
- All developer tooling surface editor query methods compile through `sifr_analysis`. analysis-host foundation implements session/query plumbing, diagnostics, workspace diagnostics, document/workspace symbols, formatter handoff, lint handoff, and completion ranking infrastructure; the full feature logic for hover, references, rename edits, semantic tokens, code actions, generated Rust preview, explain diagnostic enrichment, and test metadata lands in editor-query layer.

editor-query layer implementation:

- Editor token facts are derived through `FrontendContext::parse_module`, not a raw parser path.
- Hover, definition/declaration/type-definition, references, prepare-rename, rename, document highlights, folding ranges, selection ranges, semantic tokens, and inlay hints are token-backed and snapshot-gated.
- Diagnostics combine canonical frontend hard diagnostics with `sifr_lint` policy diagnostics.
- Code actions offer explicit Sifr policy suppression edits for lint diagnostics.
- Generated Rust preview calls the read-only
  `sifr_compiler_services::compile_source_preview` handoff and returns
  structured unavailability when compilation fails.
- Parity coverage lives in `verification/areas/developer_tooling/parity_manifest.json`, `verification/areas/developer_tooling/editor_query_snapshots/`, and `verification/areas/developer_tooling/completion_quality/`.

TypeScript-Go architecture transfer process runtime implementation:

- `AnalysisHost` owns a serialized `sifr_frontend::WorkspaceSession` instead of a bare `FrontendContext`.
- `AnalysisSnapshot` carries the captured `WorkspaceSnapshot` and the graph/source analysis revision.
- Snapshot query methods cover the full developer tooling surface editor query surface and stamp `QueryMetadata::workspace_snapshot_id`.
- Direct `AnalysisHost` query methods remain for internal tests and serialized callers.

TypeScript-Go architecture transfer task context and shutdown implementation:

- `sifr_lsp::Session` owns the persistent LSP analysis workspace instead of storing analysis hosts in `DocumentStore`.
- `DocumentStore` now tracks protocol document state only: URI, path, text, version, and settings.
- Open/change/save notifications feed the latest document state into session-owned analysis handles through `WorkspaceSession` overlays.
- Analysis-backed LSP requests capture snapshots through `Session::with_document_analysis` and reject publication if the workspace snapshot or document version is stale.

## Required Frontend Exports

frontend query architecture exports are sufficient for tooling lock to proceed:

- `sifr_syntax` exposes parse, token, text-position, and syntax range primitives.
- `sifr_frontend` exposes `FrontendContext`, module/source graph identity, source maps, query metadata, invalidation reports, diagnostics, source-map lookup stubs, symbol/type-display/editor-query view structs, and selection-range view shape.
- `sifr_diagnostics` owns canonical diagnostic rendering and schema data.
- `sifr_codegen` remains the generated-Rust authority and will be called through compiler-owned handoff APIs when preview support lands.

Missing implementation detail inside a placeholder view is not a blocker for formatter/linter foundation, but adding a second parser, lowerer, checker, diagnostic engine, formatter, linter, or codegen path is blocked by the tooling guardrails.

## Query Boundaries

Allowed dependencies:

- `sifr_analysis` may call `sifr_frontend`, `sifr_diagnostics`, `sifr_format`,
  `sifr_lint`, and `sifr_compiler_services`. It must not depend on the build
  orchestration crate `sifr_driver`.
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

formatter/linter foundation lint foundation:

- defines Sifr-owned policy metadata
- implements `# sifr: ignore[rule-id]`
- rejects blanket suppressions
- reports unknown and unused suppressions
- implements the first suppressible policy rule, `trailing-whitespace`
- supports diagnostics modes and include/exclude discovery options without applying excludes to explicit file targets

Production-grade linter work:

- source plan: `production-grade-sifr-linter record`
- execution tracker: `production-grade-sifr-linter-execution record`
- Ruff rule-family/config audit manifest: `verification/areas/developer_tooling/linter_manifests/ruff_rule_config_audit.json`
- lint CLI parity manifest: `verification/areas/developer_tooling/linter_manifests/lint_cli_parity.json`
- rule metadata manifest: `verification/areas/developer_tooling/linter_manifests/lint_rule_metadata.json`
- suppression-gate manifest: `verification/areas/developer_tooling/linter_manifests/suppression_gate.json`
- enforcement: `python3 verification/areas/developer_tooling/check_linter_reuse_rules.py`

This capability keeps Ruff Python rule families, Ruff's Python rule registry,
Ruff Server diagnostic behavior, and Python project/module semantics out of
Sifr lint authority. compiler feature locks those decisions in machine-readable
manifests before config, discovery, parser-aware suppressions, stage-gated
orchestration, fix support, or editor actions are expanded.

The linter CLI implements the non-fix `sifr lint [OPTIONS] [FILES]...` surface
through `crates/sifr/src/lint_cli.rs` and keeps command execution separate from
package/build/check orchestration. `sifr_lint` now owns `[lint]`,
`[lint.rules]`, and `[lint.per-file-ignores]` config loading from `sifr.toml`,
path-relative `extend`, CLI overrides, selector validation, per-file ignores,
glob-based include/exclude matching, and `ignore`-crate backed gitignore
discovery. The suppression gate remains `physical_line_only`; non-line rules
are still blocked until parser-aware suppression work.

Suppression attachment replaces line-only suppression attachment with
`sifr_lint::suppression::ParserAwareSuppressions`. The API parses Sifr
suppression directives once per source file, attaches them to physical-line or
statement ranges depending on rule suppression complexity, supports
`--ignore-suppressions`, reports unknown/unused/blanket suppression diagnostics
deterministically, and transitions the suppression gate manifest to
`parser_aware` for future syntax, HIR, and workspace policy rules.

Lint execution routes source and path linting through
`sifr_lint::LintRunner`. The runner exposes explicit execution-step state for
file discovery, token/trivia, physical-line, syntax-node, statement-range, HIR,
workspace, suppression filtering, per-file ignore filtering, fix filtering, and
deterministic sorting. Disabled rule families skip their execution steps, current
physical-line policy diagnostics remain preserved, invalid source still runs
source-independent policy steps, and path linting records file-discovery
execution before per-file source checks.

Policy linting adds representative Sifr-owned policy rule families without
porting Python lint semantics: token/trivia TODO/FIXME comment detection,
syntax-node positional boolean argument detection, HIR-backed large parameter
list policy, and duplicate import declaration policy. These rules use Sifr rule
IDs, `sifr_diagnostics`, parser-aware suppressions for non-physical rules, and
the stage-gated runner. The CLI also exposes `sifr lint --statistics` for
deterministic per-rule diagnostic counts.

Analysis diagnostics pass the frontend's canonical HIR view to HIR-backed lint
rules. The editor path does not create a second frontend context for the same
source revision.

The safe-fix engine adds the first Sifr-owned safe fix engine and policy-only editor
actions. `trailing-whitespace` now carries a machine-applicable safe suggestion,
`sifr_lint` applies non-overlapping fix groups deterministically, `sifr lint`
implements the fix-related Ruff-compatible surfaces through Sifr rule metadata,
and LSP code actions use typed `Hard`/`Policy` diagnostic class payloads instead
of diagnostic-code prefixes. Safe per-diagnostic fixes are synchronous; fix-all
is deferred through `codeAction/resolve` and rejects stale document versions.

Production runtime audit closes the production-grade linter work by making the public and
internal docs match the shipped command/editor rules. `docs/linter.md`
documents config, rule IDs, parser-aware suppressions, safe fix behavior, exit
status, and editor behavior. `internal_docs/lsp_server.md`,
`internal_docs/editor_integrations.md`, `internal_docs/vscode_extension.md`, and
`internal_docs/tooling_verification.md` lock the LSP/editor rules that only
typed policy diagnostics receive suppression or fix actions and that editor
adapters must not implement lint semantics locally.

## Generated Rust And Test Metadata

Generated Rust preview uses compiler/codegen APIs and source maps. It must be cancellable, revision-checked, budgeted under `lsp-generated-rust-preview`, and must not return partial generated code after a document change invalidates the request.

Test discovery and editor test commands use Sifr CLI/test-runner metadata. Projects with no tests return an empty test tree rather than guessed Python-style tests.
