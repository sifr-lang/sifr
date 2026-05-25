# Ad Hoc Phase: Production-Grade Sifr Formatter

Status: planned on 2026-05-25

## Purpose

Replace the conservative Phase 36 formatter foundation with a production-grade Sifr formatter that reuses the maintained Ruff formatter architecture instead of rebuilding formatting logic in Sifr.

Sifr already depends on the Sifr-maintained Ruff fork for parsing, AST, tokens, trivia, and source ranges. The formatter should use that same fork as the formatting source of truth, so Sifr formatting has Ruff's breadth, stability model, configuration behavior, and ecosystem-hardening path while still supporting Sifr-specific syntax extensions.

## Source Inputs

This phase is based on the current Sifr and Ruff contracts:

- Sifr `sifr_format` foundation in `crates/sifr_format`
- Phase 36 formatter and LSP handoff contracts in `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
- Tooling architecture notes in `internal_docs/tooling_analysis.md` and `internal_docs/tooling_verification.md`
- Ruff formatter docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/formatter.md`
- Ruff configuration docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/configuration.md`
- Ruff preview docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/preview.md`
- Ruff formatter crate docs in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_formatter/README.md`
- Ruff formatter contributor docs in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_formatter/CONTRIBUTING.md`
- Ruff editor setup docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/integrations.md` and <https://docs.astral.sh/ruff/editors/setup/>
- Sifr Ruff fork maintenance branch: <https://github.com/sifr-lang/ruff/tree/sifr/0.15.12-maintenance>
- Sifr Ruff formatter seed PR: <https://github.com/sifr-lang/ruff/pull/1>, merged as `b251656613629e054308951a4df1928b3f749b1b`

## Quality Contract

- Entry criteria: the Sifr Ruff fork can parse current Sifr syntax extensions, including Sifr parameter conventions, and Sifr's full local validation is green before implementation begins.
- Ruff fork baseline: this Sifr branch must point `third_party/ruff` at `sifr-lang/ruff` branch `sifr/0.15.12-maintenance` at or after merge commit `b251656613629e054308951a4df1928b3f749b1b`, which includes the parameter-convention formatter work from `sifr-lang/ruff#1`.
- Exit criteria: `sifr fmt` is a production formatter with Ruff-backed document formatting, check mode, config semantics, editor formatting parity, Sifr syntax extension coverage, and regression gates that prevent parser or AST extensions from landing without formatter support.

### Required quality controls

- Do not reimplement a parallel formatter in Sifr when the Ruff formatter can own the behavior.
- Do not add fallback formatting paths for unsupported syntax; unsupported Sifr AST nodes must be explicit implementation blockers with tests.
- Keep one syntax source of truth: `sifr_format` formats through `sifr_syntax`, which wraps the Sifr Ruff fork parser/AST/trivia/source map substrate.
- Formatting must not lower, type-check, run ownership analysis, or depend on semantic diagnostics.
- Formatting must be deterministic, idempotent, parser-round-tripped, and stable under repeated runs.
- Formatting must preserve comments, pragmas, shebang-like leading trivia if supported, line endings according to configured policy, string contents, and source ranges needed by diagnostics and editor edits.
- CLI, analysis, and LSP formatting must share the same formatter API and options model.
- Every Sifr syntax extension that reaches the AST must have formatter coverage before the extension is considered complete.
- Local validation evidence must be recorded in the execution issue before implementation PRs merge.
- Full local suite passes before phase closure:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`

## Problem Statement

The current `sifr_format` crate is intentionally conservative. It validates syntax, preserves string tokens, trims trailing whitespace, normalizes line endings, and guarantees a final newline. That is useful as a Phase 36 foundation, but it is not a production formatter.

Production Sifr formatting needs the full behavior users expect from Ruff:

- stable whole-document formatting across all supported syntax
- check mode and write mode with predictable exit status
- config discovery and formatter-specific options
- comment and pragma handling
- docstring code formatting where applicable
- range formatting for editor integrations
- idempotence and ecosystem-scale regression checks
- preview behavior that can evolve style without silently changing stable formatting

The root issue is not a missing cleanup rule in `sifr_format`; it is that Sifr should be using the maintained Ruff formatter path for AST-aware formatting. Since Sifr's frontend syntax substrate is already a Ruff fork, the formatter should be made Sifr-aware in that fork and exposed through Sifr-owned wrappers.

## Product Decision

Sifr will provide `sifr fmt` as a Ruff-backed formatter for Sifr source.

Sifr editor formatting is LSP-first, following Ruff's production editor model: editor integrations launch the language server and request document or range formatting through LSP. The production editor path is `sifr lsp --stdio` handling `textDocument/formatting` and `textDocument/rangeFormatting`; `sifr fmt` is the CLI, CI, hook, and manual-file surface. Editor integrations must not invoke a separate formatter implementation, a Python/Ruff fallback, or direct parser/AST formatting logic.

The architecture is:

```text
.sifr source
  -> sifr_syntax
  -> Sifr Ruff fork parser / AST / comments / trivia
  -> Sifr-aware Ruff formatter implementation
  -> sifr_format wrapper API
  -> sifr fmt, sifr_analysis, sifr_lsp
```

The formatter must support every Ruff formatter capability that is meaningful for Sifr source. When Ruff has a formatter feature and Sifr's language model can express the same concept, this phase either supports it or records an explicit non-applicability decision below. Implementation PRs must not defer, reopen, or reinterpret these planning decisions without a new reviewed planning update.

Ruff stable style is the default. Ruff preview style is allowed only behind a Sifr formatter preview setting or flag that maps to Ruff's preview model.

## Scope

In scope:

1. Replace the whitespace-only formatter foundation with an AST-aware Ruff-backed formatter path.
2. Keep `crates/sifr_format` as the Sifr-owned public wrapper for CLI, analysis, and LSP formatting.
3. Support whole-file formatting for files and recursively discovered project paths.
4. Support `--check` behavior with deterministic diagnostics and Ruff-compatible drift semantics.
5. Add Ruff-compatible formatter CLI behavior for write, check, diff, stdin/stdout, file-name-aware stdin, single-file range formatting, preview toggles, line-length overrides, path discovery controls, and deterministic exit codes.
6. Implement Sifr formatter configuration with Ruff-compatible defaults where applicable:
   - line length
   - indent width
   - indent style
   - quote style
   - line ending
   - magic trailing comma behavior
   - docstring code formatting
   - docstring code line length
   - formatter preview mode
   - formatter-specific exclude/include behavior
7. Define config discovery and precedence for Sifr projects, including how `sifr.toml`, Ruff config files, explicit path arguments, excludes, extends, and VCS ignores interact.
8. Support Ruff formatting pragmas where meaningful:
   - `# fmt: off`
   - `# fmt: on`
   - `# fmt: skip`
   - `# yapf: disable`
   - `# yapf: enable`
9. Support Sifr-specific syntax extensions:
   - parameter conventions: `mut`, `own`, and canonical `own mut`
   - tolerant parse form `mut own` formatted canonically as `own mut`
   - Sifr type syntax, generics, option/result-shaped annotations, match/case, ownership-aware collection syntax, and all AST extensions current at implementation time
10. Preserve Ruff's formatter guarantees:
   - no formatting panics for valid source
   - formatted output remains parseable
   - second formatting pass is byte-identical
   - comments attach to the same intended syntax
   - invalid source reports diagnostics rather than producing partial output
11. Add range-formatting support through `sifr_analysis` and `sifr_lsp` using the same formatter core.
12. Add validation that prevents Sifr Ruff fork parser/AST extensions from landing without formatter rules.
13. Prefer direct reuse of Ruff formatter, workspace, resolver, source-kind, diff, range, cache, and option-conversion crates/APIs whenever their semantics are language-neutral or cleanly Sifr-adaptable.
14. Make all checked-in editor integrations expose formatter support through the Sifr LSP server:
   - Neovim: `editor_integrations/neovim/ftdetect/sifr.lua` and `editor_integrations/neovim/lsp/sifr.lua`
   - Zed: `editor_integrations/zed/extension.toml` and `editor_integrations/zed/languages/sifr/config.toml`
   - Helix: `editor_integrations/helix/languages.toml`
   - Emacs: `editor_integrations/emacs/sifr-mode.el`
   - VS Code: `editor_integrations/vscode/` and the `sifr-lang/sifr-vscode` contract
   - shared docs: `editor_integrations/README.md`
15. Update public and internal docs for formatter commands, config, editor behavior, LSP format-on-save/manual formatting setup, and known limitations.

Out of scope:

- Import sorting as part of formatting. Ruff treats import sorting as lint/fix behavior, so Sifr formatting must not silently reorder imports unless a reviewed Sifr lint/fix phase adds that policy.
- Python lint rules, Python semantic analysis, or Python project resolution.
- Notebook formatting unless a later reviewed Sifr product phase makes notebooks part of Sifr's language surface.
- Style knobs that Ruff intentionally does not expose.
- A formatter that depends on HIR, type checking, ownership analysis, or generated Rust.

## Ruff Capability Parity Contract

This phase locks the Sifr formatter capability matrix before implementation. Each Ruff formatter capability is classified as one of:

- `supported`: Sifr implements the same behavior.
- `adapted`: Sifr implements the same user-facing capability with Sifr-specific naming or project integration.
- `not-applicable`: Ruff behavior has no Sifr language or product equivalent, with a written rationale and review approval.
- `not-exposed`: Ruff exposes a Python/product-specific control that Sifr intentionally does not expose in this phase.

The initial required matrix rows are:

| Capability | Required Sifr outcome |
| --- | --- |
| Whole-file formatting | Supported for `.sifr` files through the Ruff formatter core |
| Recursive path formatting | Supported for files and directories, respecting Sifr project discovery |
| Check mode | Supported with stable nonzero drift result and diagnostics |
| Diff mode | Supported through Sifr CLI wrapper code that reuses Ruff unified-diff/source utilities; no Ruff CLI subprocess |
| Stdin formatting | Supported; `--stdin-filename` supplies config/path context and stdin without a filename uses current-directory config context |
| Single-file CLI range formatting | Supported with Ruff-compatible `start_line:start_column-end_line:end_column` semantics adapted to Sifr source positions |
| Formatter cache | Adapted from Ruff cache behavior with cache keys covering source metadata, formatter options, Sifr version, and Sifr Ruff fork revision |
| Gitignore and force-exclude controls | Adapted to Sifr file discovery |
| CLI line-length override | Supported |
| CLI preview toggles | Supported |
| CLI target/source-version option | Not exposed; Python `--target-version` semantics do not apply to Sifr, and Sifr has no formatter syntax-version policy in this phase |
| Extension/language mapping | Not applicable; `.sifr` is the only formatter source kind in this phase |
| Config discovery | Adapted to Sifr project rules with Ruff-compatible formatter option semantics |
| Exclude/include and VCS ignores | Adapted to Sifr project discovery and explicit-target behavior |
| Line length | Supported |
| Indent width | Supported |
| Indent style | Supported |
| Quote style | Supported where Sifr string syntax matches Ruff/Python string syntax |
| Line ending | Supported |
| Magic trailing comma behavior | Supported where Sifr AST/list/call shapes match Ruff behavior |
| Docstring code formatting | Supported for Sifr docstrings using Sifr snippet parsing/formatting in Ruff-recognized docstring code forms; disabled by default |
| Preview style | Supported behind explicit preview flag/config |
| `fmt` pragmas | Supported for Sifr comments |
| Editor document formatting | Supported through LSP `textDocument/formatting` over `sifr_lsp -> sifr_analysis -> sifr_format`; advertised only when `sifr.format.enable` is true |
| Editor range formatting | Supported through LSP `textDocument/rangeFormatting` over the same formatter path, with stable edit ranges and no split-brain formatter path |
| Editor setup and format-on-save | Adapted for Neovim, Zed, Helix, Emacs, and VS Code through checked-in LSP integration assets and docs |
| VS Code document formatting provider | Adapted through the extension's native LSP client; no extension-owned formatter implementation |
| Formatter ecosystem checks | Adapted to Sifr fixtures/corpus plus Ruff fork formatter tests |

## Architecture Requirements

### Ruff fork

- The baseline fork dependency for this phase is `sifr-lang/ruff` branch `sifr/0.15.12-maintenance` at or after `b251656613629e054308951a4df1928b3f749b1b`.
- The previously open formatter seed PR, `sifr-lang/ruff#1` from `codex/format-sifr-param-conventions`, is merged and must remain present in the branch consumed by `third_party/ruff`.
- Sifr implementation branches must consume the merged maintenance branch commit through the `third_party/ruff` submodule. They must not depend on an unmerged feature branch, local-only patch, or post-processing workaround for parameter conventions.
- The Sifr Ruff fork owns AST formatting rules for Sifr syntax extensions.
- Any AST enum, node, token, or comment attachment extension for Sifr must either use an existing Ruff formatter implementation correctly or add a Sifr-specific formatter implementation in the fork.
- Formatter tests in the fork must cover each Sifr extension with snapshot-style expected output.
- Formatter behavior for `mut own` must canonicalize to `own mut` because Sifr's source, docs, diagnostics, and snapshots treat `own mut` as canonical.
- The fork must expose a stable integration point for Sifr crates rather than requiring Sifr to shell out to a CLI for in-process formatting.

### Sifr crates

- `sifr_syntax` remains the only parser/token/trivia/source-map boundary.
- `sifr_format` owns Sifr-facing option parsing, config conversion, diagnostics, file discovery, and text edit creation.
- `sifr_analysis` calls `sifr_format` for document and range formatting.
- `sifr_lsp` only converts LSP requests and options into `sifr_analysis` calls, advertises document and range formatting capabilities when `sifr.format.enable` is true, and handles cancellation/stale-version behavior through the Phase 36 scheduling model.
- `crates/sifr` owns CLI flags, exit codes, and user-facing rendering.
- Editor integrations launch `sifr lsp --stdio` and rely on standard editor LSP formatting commands, save hooks, or document formatting providers. Direct editor calls to `sifr fmt` are allowed only as explicit manual CLI commands outside the primary editor formatting provider path.

### Config ownership

Sifr must not silently reinterpret Python-only Ruff settings as Sifr behavior. The config layer must:

- define the canonical Sifr formatter config source and precedence
- document any Ruff config files that Sifr reads for migration convenience
- keep formatter options separate from lint options
- reject unknown or unsupported formatter settings with deterministic diagnostics
- include tests for closest-config precedence, explicit target behavior, excludes, extends, and default values

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | `sifr fmt <file.sifr>` formats AST-aware Sifr code through the Ruff formatter core, not the whitespace-only foundation |
| AC-2 | `sifr fmt --check <path>` exits cleanly when formatted and nonzero with deterministic diagnostics when drift exists |
| AC-3 | formatting valid Sifr source is idempotent across at least two formatter passes |
| AC-4 | formatted output re-parses through `sifr_syntax` and preserves parser/source-map invariants needed by diagnostics |
| AC-5 | all current Sifr syntax extensions have formatter snapshot coverage in the Sifr Ruff fork and Sifr wrapper tests |
| AC-6 | `mut own` parameters format to canonical `own mut` while `own`, `mut`, and default parameters remain stable |
| AC-7 | comments, blank lines, and formatter pragmas match Ruff behavior where applicable |
| AC-8 | formatter configuration covers supported Ruff formatter options with tested defaults, precedence, and diagnostics |
| AC-9 | CLI formatting, analysis formatting, and LSP formatting produce equivalent output or edits for the same source/options |
| AC-10 | range formatting produces minimal, stable text edits and never routes around `sifr_format` |
| AC-11 | invalid source reports diagnostics without writing partial output |
| AC-12 | fork-level formatter tests, Sifr formatter tests, tooling contract checks, and full Sifr validation all pass locally |
| AC-13 | docs explain formatter command usage, configuration, editor behavior, preview behavior, and explicit non-applicable Ruff capabilities |
| AC-14 | a guardrail fails when a new Sifr AST syntax extension has no formatter coverage |
| AC-15 | Neovim, Zed, Helix, Emacs, and VS Code integrations document and validate LSP document formatting, range formatting where supported by the editor, and format-on-save/manual formatting setup without editor-owned formatter logic |

## Milestone Breakdown

Each milestone is independently reviewable and must close with validation evidence in the execution tracker before the next milestone starts. Milestones are sequential because later work depends on earlier contracts being executable, not merely documented.

### Milestone 1: `formatter_contract_manifests_and_ast_inventory`

Goal: turn this planning contract into checked manifests that implementation cannot drift from.

Entry criteria:

- this phase plan is reviewed and approved
- the Sifr Ruff fork revision used by the superproject is recorded and is at or after `b251656613629e054308951a4df1928b3f749b1b`
- current Sifr parser/AST extension inventory can be generated or inspected deterministically

Scope:

- create machine-readable capability and CLI parity manifests that exactly mirror the locked tables in this document
- create the first formatter AST coverage manifest for all current Sifr syntax extensions
- add manifest consistency checks that fail when docs, manifests, and Ruff formatter CLI/options drift
- add a fork-baseline guardrail that fails if `third_party/ruff` points to a commit that does not contain the merged `sifr-lang/ruff#1` parameter-convention formatter change
- record not-applicable and not-exposed rows exactly as planned; do not add new undecided rows

Outputs:

- formatter capability manifest
- formatter CLI parity manifest
- formatter AST coverage manifest
- recorded Ruff fork baseline commit and submodule branch
- guardrail script stubs or active checks wired to local validation as appropriate for manifest consistency

Validation:

- manifest self-tests
- `git diff --check`
- the smallest local validation lane that exercises tooling guardrails

Review gate:

- external review confirms the manifests faithfully encode the approved plan and contain no deferred decisions

### Milestone 2: `ruff_fork_sifr_formatter_ast_completion`

Goal: make the maintained Sifr Ruff fork format every current Sifr AST extension.

Entry criteria:

- Milestone 1 manifests are merged
- Sifr AST coverage manifest lists every current parser/AST extension
- `third_party/ruff` is pinned to `sifr-lang/ruff` branch `sifr/0.15.12-maintenance` at or after merged PR `sifr-lang/ruff#1`

Scope:

- treat `sifr-lang/ruff#1` as the merged parameter-convention formatter seed, not as future work
- implement formatter rules for all current Sifr AST extensions in the Ruff fork
- add a public Sifr formatter wrapper in the fork for Sifr crates to call
- cover parameter conventions: default, `mut`, `own`, `own mut`, and tolerant `mut own` formatting to canonical `own mut`
- support Sifr type syntax, generics, match/case, ownership-aware collection syntax, docstring snippet formatting hooks, and current Sifr-specific trivia/comment behavior
- add fork-level formatter snapshots, idempotence tests, parser-roundtrip tests, and invalid-source tests

Outputs:

- Sifr Ruff fork commit or PR containing the remaining formatter support beyond the merged parameter-convention seed
- fork-level fixture corpus for all Sifr AST extensions
- updated superproject reference plan or submodule update PR if this milestone owns consumption

Validation:

- `cargo test -p ruff_python_formatter --lib`
- Sifr-specific formatter tests in the Ruff fork
- formatter idempotence and parser-roundtrip tests for every coverage-manifest row

Review gate:

- external review confirms the Ruff fork prints Sifr syntax directly and no Sifr wrapper source post-processing is used

### Milestone 3: `sifr_format_ruff_backed_core`

Goal: replace the conservative Sifr formatter foundation with the Ruff-backed in-process formatter API.

Entry criteria:

- Milestone 2 formatter support is available to the superproject
- `sifr_format` can depend on the required Sifr Ruff fork formatter crates through the established dependency strategy

Scope:

- route `sifr_format::format_source` and `format_range` through the Sifr-aware Ruff formatter wrapper
- convert Sifr `FormatOptions` into Ruff `PyFormatOptions` through the Ruff option conversion path where possible
- preserve Sifr-owned diagnostics, file IO, text edits, and API boundaries
- return deterministic Sifr diagnostics for parse, format, print, unsupported-option, and invalid-source failures
- preserve parser roundtrip, idempotence, comments, pragmas, source-map invariants, and string contents

Outputs:

- Ruff-backed `sifr_format` core
- Sifr wrapper tests for whole-document formatting, range formatting, idempotence, parser roundtrip, invalid source, and all Sifr syntax extensions

Validation:

- `cargo test -p sifr_format`
- `python3 verification/tooling/check_formatter_contract.py`
- `python3 verification/tooling/check_formatter_contract.py --self-test`

Review gate:

- external review confirms there is one formatter core and no fallback whitespace formatter path remains

### Milestone 4: `formatter_cli_and_config_parity`

Goal: make `sifr fmt` match the locked Ruff-compatible CLI and config contract.

Entry criteria:

- Milestone 3 `sifr_format` core is merged
- CLI and config manifests from Milestone 1 are green

Scope:

- implement `sifr fmt [OPTIONS] [FILES]...`
- support default target `.`, write mode, `--check`, `--diff`, `--stdin-filename`, stdin without filename, `--range`, `--line-length`, `--preview`, `--no-preview`, `--exclude`, `--respect-gitignore`, `--no-respect-gitignore`, `--force-exclude`, `--no-force-exclude`, `--no-cache`, `--cache-dir`, global `--config`, and global `--isolated`
- intentionally do not expose Python `--target-version` or `--extension`
- implement `sifr.toml` `[format]`, explicit `extend`, Ruff migration config reading, CLI override precedence, unknown-key diagnostics, unsupported Python-only option diagnostics, VCS ignore behavior, explicit-target behavior, and formatter cache behavior
- implement output summaries, changed-file listing, unified diff output, and abnormal-error exit status exactly as documented

Outputs:

- production `sifr fmt` CLI surface
- formatter config loader and tests
- CLI fixture coverage for every CLI parity manifest row

Validation:

- `cargo test -p sifr`
- `cargo test -p sifr_format`
- CLI fixture suite for all formatter options and config precedence

Review gate:

- external review confirms CLI/config behavior follows the locked manifests and reuses Ruff crates/APIs wherever practical

### Milestone 5: `analysis_lsp_editor_formatter_parity`

Goal: make production editor formatting use the same LSP/analysis formatter path as the CLI, following Ruff's LSP-first editor setup model.

Entry criteria:

- Milestone 4 CLI/config behavior is merged
- `sifr_analysis` and `sifr_lsp` build against the Ruff-backed formatter API

Scope:

- route document formatting and range formatting through `sifr_analysis` into `sifr_format`
- keep `sifr_lsp` as a protocol adapter only
- advertise `documentFormattingProvider` and `documentRangeFormattingProvider` only when `sifr.format.enable` is true
- convert LSP formatter options, workspace settings, initialization options, and editor-provided formatting options into the same Sifr `FormatOptions` model and config precedence used by the CLI
- preserve Phase 36 LSP scheduling requirements: separate formatting lane, request cancellation, stale document-version rejection, invalid-range diagnostics, UTF-8/UTF-16/UTF-32 conversion correctness, and no starvation of latency-sensitive editor queries
- update `verification/tooling/lsp_protocol_matrix.json` with production formatter coverage for `textDocument/formatting`, `textDocument/rangeFormatting`, formatter settings changes, capability-disable behavior, and the `lsp-formatting` performance budget
- extend `verification/tooling/lsp_protocol_smoke.py` and `verification/tooling/lsp_protocol_stress.py` to cover initialize, open/change, document formatting, range formatting, cancellation, stale versions, invalid ranges, settings changes, and clean shutdown
- add parity snapshots proving CLI, analysis, and LSP produce equivalent output or edits for the same source/options
- update `verification/tooling/parity_manifest.json` and editor query snapshots for document and range formatting
- update all checked-in editor integrations so their documented formatter path is LSP-based:
  - Neovim uses `vim.lsp.buf.format` or equivalent client LSP formatting against `sifr lsp --stdio`
  - Zed uses the Sifr language server as the formatter provider and documents format-on-save behavior
  - Helix uses its language-server formatting integration and documents `auto-format = true`
  - Emacs uses Eglot or lsp-mode document formatting, including a save hook example
  - VS Code uses the standard document formatting provider from the Sifr LSP client and format-on-save settings
- update `editor_integrations/README.md`, `internal_docs/editor_integrations.md`, `internal_docs/lsp_server.md`, `internal_docs/tooling_analysis.md`, `internal_docs/vscode_extension.md`, and the VS Code extension contract so editor setup mirrors the LSP-first Ruff setup pattern
- extend `verification/tooling/check_editor_assets.py`, `verification/tooling/check_vscode_extension_contract.py`, and VS Code extension tests to reject editor-owned formatter logic, Python/Ruff fallbacks, missing `sifr lsp --stdio` launch configuration, and missing formatter setup docs

Outputs:

- analysis formatting API parity
- LSP document/range formatting protocol support
- protocol matrix, parity manifest, smoke, stress, and performance-budget coverage for formatting
- editor snapshot fixtures
- updated editor integration assets and docs for Neovim, Zed, Helix, Emacs, and VS Code
- VS Code extension contract updates for document formatting provider and format-on-save support

Validation:

- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- tooling parity checks covering formatter paths
- `python3 verification/tooling/lsp_protocol_smoke.py`
- `python3 verification/tooling/lsp_protocol_stress.py`
- `python3 verification/tooling/check_editor_assets.py`
- `python3 verification/tooling/check_vscode_extension_contract.py`
- VS Code extension unit/smoke tests from the configured extension checkout

Review gate:

- external review confirms no formatter split-brain exists between CLI, analysis, LSP, and editor integrations, and every checked-in editor integration exposes formatting through `sifr lsp --stdio`

### Milestone 6: `formatter_corpus_guardrails_and_performance`

Goal: harden formatter correctness and prevent future syntax extensions from bypassing formatter support.

Entry criteria:

- Milestones 1 through 5 are merged
- formatter manifests and CLI/config fixtures are green

Scope:

- implement `verification/tooling/check_formatter_ast_coverage.py`
- wire formatter coverage guardrails into `scripts/run_all_tests.sh`
- add Sifr formatter corpus checks inspired by Ruff formatter ecosystem checks
- cover idempotence, parser roundtrip, panic-free formatting, invalid-source diagnostics, comments, pragmas, docstring snippets, config matrix, CLI matrix, editor parity, and all syntax extension coverage rows
- add the checked-in formatter showcase input at `demos/formatter_showcase/main.sifr.input` to the formatter corpus without treating it as a normal `.sifr` demo fixture
- add large-file and project formatting performance budgets
- add editor-integration guardrail seeds that fail when formatting is wired through a non-LSP formatter, a direct Python/Ruff fallback, or extension-owned formatter code
- ensure any new Sifr parser/AST extension fails validation unless formatter coverage is added

Outputs:

- formatter AST coverage guardrail
- formatter corpus and performance budget evidence
- formatter showcase corpus entry based on `demos/formatter_showcase/main.sifr.input`
- validation lane wiring and self-tests

Validation:

- formatter guardrail and self-test
- performance budget checks
- `scripts/run_all_tests.sh --profile quick`

Review gate:

- external review confirms future syntax extensions cannot land without formatter coverage

### Milestone 7: `formatter_docs_closeout_and_release_readiness`

Goal: close the phase with docs, validation evidence, and release-ready formatter behavior.

Entry criteria:

- Milestones 1 through 6 are merged
- quick validation is green

Scope:

- update internal architecture docs and public formatter docs
- document command usage, configuration, LSP/editor formatting behavior, format-on-save setup for all checked-in editor integrations, preview behavior, docstring snippets, cache behavior, non-exposed Python options, and non-applicable Ruff capabilities
- update the execution issue with validation evidence and merged PR links for every milestone
- prove every locked matrix row is implemented or explicitly not exposed as planned
- run the formatter showcase demo by copying `demos/formatter_showcase/main.sifr.input` to a temporary `.sifr` file, running `sifr fmt`, checking the formatted result, and recording the before/after output or diff in the execution tracker
- run final external production-readiness review

Outputs:

- updated docs
- completed execution tracker
- recorded formatter showcase before/after evidence
- phase closeout review artifact

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- targeted docs/contract checks
- formatter showcase copy-and-format smoke run

Review gate:

- final review confirms the formatter is production-grade, Ruff-backed, gap-free, and ready for release documentation

## Validation Plan

Every implementation PR must run the smallest relevant local gate plus targeted formatter tests. Phase closure requires:

```bash
cargo test -p ruff_python_formatter --lib
cargo test -p sifr_format
cargo test -p sifr_analysis
cargo test -p sifr_lsp
cargo test -p sifr
python3 verification/tooling/check_formatter_contract.py
python3 verification/tooling/check_formatter_contract.py --self-test
scripts/run_all_tests.sh --profile quick
scripts/run_all_tests.sh
```

If the Ruff fork adds or changes formatter ecosystem tooling for Sifr syntax, the equivalent fork-level command must be added to this list before phase closure.

## Review Requirements

- This planning phase must be externally reviewed before implementation starts.
- Each implementation milestone must include a focused review that checks root-cause closure, Ruff capability parity, Sifr architecture boundaries, validation coverage, and documentation updates.
- Phase closure requires a final review that explicitly confirms every locked capability row is satisfied, no formatter split-brain paths exist, and no unsupported Sifr AST extensions remain.

## Execution Log

- `2026-05-25`: phase drafted after reading Sifr formatter contracts and Ruff formatter/configuration/preview docs.
