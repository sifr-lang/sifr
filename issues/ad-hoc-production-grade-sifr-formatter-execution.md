# Ad Hoc Phase Execution: Production-Grade Sifr Formatter

Status: completed

Phase contract: `issues/ad-hoc-production-grade-sifr-formatter.md`

## Checklist

- [x] Phase plan reviewed and approved for implementation
- [x] Ruff fork parameter-convention formatter PR merged and submodule pinned
- [x] Ruff-to-Sifr formatter capability matrix created
- [x] Ruff fork formatter Sifr AST support completed
- [x] Sifr formatter core switched to Ruff-backed in-process formatting
- [x] CLI and config parity completed
- [x] Analysis, LSP, and editor integration formatting parity completed
- [x] Formatter corpus, guardrails, and performance checks completed
- [x] Formatter showcase demo copied to `.sifr`, formatted, and recorded
- [x] Internal and public docs updated
- [x] Full local validation recorded
- [x] Final production-readiness review approved

## Planning Lock Addendum

This addendum locks the formatter planning decisions before implementation starts. Implementation must follow these decisions as written; changing a capability classification, CLI surface, config rule, or non-applicability rationale requires a new reviewed planning update.

### Required Implementation Work

| ID | Work item | Required closeout |
| --- | --- | --- |
| W-1 | The Sifr formatter wrapper is still the conservative Phase 36 implementation. | Milestone 3 replaces it with the Ruff-backed in-process formatter core. |
| W-2 | The superproject must consume a Sifr Ruff fork revision whose formatter handles Sifr parameter conventions. | The seed formatter change is merged in `sifr-lang/ruff#1` as `b251656613629e054308951a4df1928b3f749b1b`; Milestone 1 must guard that `third_party/ruff` stays at or after that commit, and Milestone 2 expands coverage to all remaining Sifr AST extensions. |
| W-3 | Formatter config discovery does not exist in `sifr_format`. | Milestone 4 implements the config schema and precedence locked in this document. |
| W-4 | No automatic guardrail currently proves all Sifr AST extensions have formatter coverage. | Milestone 6 implements the guardrail design locked in this document. |

### Ruff Fork Baseline

This planning branch carries the required Ruff formatter seed change through the `third_party/ruff` submodule.

| Item | Locked value |
| --- | --- |
| Ruff fork repository | <https://github.com/sifr-lang/ruff> |
| Required maintenance branch | `sifr/0.15.12-maintenance` |
| Required seed PR | <https://github.com/sifr-lang/ruff/pull/1> |
| Seed PR branch | `codex/format-sifr-param-conventions` |
| Seed merge commit | `b251656613629e054308951a4df1928b3f749b1b` |
| Superproject submodule path | `third_party/ruff` |

Implementation requirements:

- `third_party/ruff` must point at `sifr-lang/ruff` branch `sifr/0.15.12-maintenance` at or after `b251656613629e054308951a4df1928b3f749b1b`.
- `.gitmodules` must keep `third_party/ruff.branch = sifr/0.15.12-maintenance`.
- Formatter implementation must consume the merged maintenance branch, not the deleted feature branch and not a local-only patch.
- If a future Ruff fork rebase rewrites this merge commit, Milestone 1 must record the replacement commit and prove it contains the same parameter-convention formatter tests and behavior before implementation continues.
- The phase must not reintroduce a Sifr wrapper source post-processing path for `mut`, `own`, `own mut`, or tolerant `mut own`; the merged Ruff fork formatter rule remains the source of truth.

### Ruff-to-Sifr Formatter Capability Matrix

Classifications:

- `supported`: required direct support with Ruff-compatible behavior.
- `adapted`: required support through Sifr-owned project/config/diagnostic wrappers.
- `not-applicable`: no Sifr product equivalent; must have review-approved rationale.
- `not-exposed`: Ruff exposes a Python/product-specific control that Sifr intentionally does not expose in this phase.

| Ruff formatter capability | Sifr classification | Implementation requirement |
| --- | --- | --- |
| Whole-file formatting | supported | `sifr_format` calls Ruff formatter library APIs over Sifr AST/source and returns complete formatted source. |
| Recursive path formatting | adapted | `sifr fmt <path>` discovers `.sifr` files through Sifr project/path rules while preserving Ruff explicit-target behavior. |
| Check mode | supported | `sifr fmt --check` reports drift without writes and exits nonzero on drift. |
| Diff mode | adapted | Ruff diff behavior is CLI-layer, not formatter-library-layer. Sifr must generate unified diffs from original/formatted text in its CLI wrapper without shelling out to Ruff. |
| Stdin formatting | adapted | Sifr CLI owns stdin/stdout and optional filename context, then calls the same in-process formatter API. |
| Single-file CLI range formatting | supported | Add a Ruff-compatible `--range` option for one resolved file only, using the same position grammar and formatter range API. |
| Formatter cache | adapted | Implement Ruff-style formatter cache behavior with cache keys covering source metadata, formatter options, Sifr version, and Sifr Ruff fork revision. |
| `--respect-gitignore` / `--no-respect-gitignore` | adapted | Match Ruff's file-selection behavior for Sifr paths. |
| `--force-exclude` / `--no-force-exclude` | adapted | Match Ruff's distinction between explicit roots and forced exclusions. |
| CLI `--exclude` | supported | Match Ruff comma-delimited file-pattern override semantics for format file selection. |
| CLI `--line-length` | supported | Match Ruff formatter override semantics and precedence. |
| CLI `--preview` / `--no-preview` | supported | Match Ruff preview toggle behavior while keeping stable style default. |
| CLI `--target-version` | not-exposed | Do not expose Python target-version semantics. Sifr has no formatter syntax-version flag in this phase; a future language-edition phase may add one explicitly. |
| CLI `--extension` | not-applicable | Sifr formatter source kind is `.sifr` only in this phase; multiple source-kind mapping requires a later product decision. |
| Config discovery | adapted | Sifr canonical config is `sifr.toml`; Ruff config files are migration inputs only under the precedence rules below. |
| Exclude/include and VCS ignores | adapted | Sifr file discovery must support formatter include/exclude settings, `.gitignore`, and explicit target overrides. |
| Line length | supported | Map directly to Ruff `PyFormatOptions` line width. |
| Indent width | supported | Map directly to Ruff indent width. |
| Indent style | supported | Map directly to Ruff indent style. |
| Quote style | supported | Map directly where Sifr string syntax uses Ruff-compatible string tokens. |
| Line ending | supported | Map directly to Ruff line-ending behavior. |
| Magic trailing comma behavior | supported | Map directly for Sifr calls, collections, signatures, and type constructs that use Ruff AST layouts. |
| Docstring code formatting | adapted | Support Ruff-recognized docstring code forms by formatting Sifr snippets with the Sifr parser/formatter. The option remains disabled by default. |
| Docstring code line length | adapted | Same disposition as docstring code formatting. |
| Formatter preview mode | supported | Expose explicit Sifr preview flag/config that maps to Ruff preview mode. Stable mode remains default. |
| `# fmt: off` and `# fmt: on` | supported | Apply at statement level, matching Ruff/Black semantics. |
| `# fmt: skip` | supported | Apply to preceding statement, case header, decorator, or other Ruff-supported syntactic boundary adapted to Sifr AST. |
| `# yapf: disable` and `# yapf: enable` | supported | Treat as aliases for `fmt: off/on` at the same statement-level boundaries Ruff documents. |
| Editor document formatting | supported | LSP `textDocument/formatting` routes through `sifr_lsp -> sifr_analysis -> sifr_format` and is advertised only when `sifr.format.enable` is true. |
| Editor range formatting | supported | LSP `textDocument/rangeFormatting` routes through the same formatter context; no independent LSP/editor formatter. |
| Editor setup and format-on-save | adapted | Neovim, Zed, Helix, Emacs, and VS Code assets/docs must expose formatting through `sifr lsp --stdio` and standard editor LSP formatting hooks. |
| VS Code document formatting provider | adapted | The extension uses the Sifr LSP client document formatting provider and format-on-save settings; extension TypeScript owns no formatter. |
| Formatter ecosystem checks | adapted | Add Sifr corpus checks inspired by Ruff ecosystem checks: idempotence, parser roundtrip, no panics, invalid-source diagnostics, comments/pragmas, config matrix, and performance budgets. |
| Notebook formatting | not-applicable | Sifr has no notebook product surface in this phase. A later product phase must add this explicitly if needed. |
| Import sorting | not-applicable | Ruff treats import sorting as lint/fix behavior, not formatting. Sifr format must not reorder imports. |

Implementation must not add new unclassified formatter capability rows. If implementation discovers a Ruff formatter capability missing from this table, work stops for a reviewed planning update before code proceeds.

### Ruff Formatter Integration API

The implementation target is an in-process library integration, not a CLI subprocess.

- `crates/sifr_format` depends on the Sifr Ruff fork formatter crates through the existing workspace/submodule dependency strategy.
- The minimum reusable Ruff APIs identified for implementation are:
  - `ruff_python_formatter::format_module_source`
  - `ruff_python_formatter::format_range`
  - `ruff_python_formatter::format_module_ast`
  - `ruff_python_formatter::PyFormatOptions`
  - `ruff_workspace::FormatterSettings`
  - `FormatterSettings::to_format_options`
  - `ruff_workspace` configuration option structs for shared formatter options
  - Ruff resolver/file-selection patterns for excludes, VCS ignores, explicit roots, package roots, and force-exclude behavior where they are language-neutral
  - Ruff source/diff utilities such as `SourceKind::diff` or a Sifr-owned equivalent over the same unified-diff semantics
  - option types such as `QuoteStyle`, `MagicTrailingComma`, `PreviewMode`, `DocstringCode`, and `DocstringCodeLineWidth`
- Milestone 2 adds a public Sifr formatter wrapper inside the fork rather than forcing Sifr crates to reach into private formatter modules.
- If a Ruff CLI/config feature is implemented outside a reusable library API today, Sifr must still reuse the closest underlying Ruff crate or algorithmic component before writing Sifr-specific code.
- `sifr_format` owns the public Sifr API:

```rust
pub struct FormatOptions {
    pub line_length: usize,
    pub indent_width: usize,
    pub indent_style: IndentStyle,
    pub quote_style: QuoteStyle,
    pub line_ending: LineEnding,
    pub magic_trailing_comma: MagicTrailingComma,
    pub preview: bool,
    pub docstring_code_format: bool,
    pub docstring_code_line_length: DocstringCodeLineLength,
}

pub fn format_source(
    source: &str,
    file: Option<&Path>,
    options: FormatOptions,
) -> Result<FormatResult, Vec<RenderedDiagnostic>>;

pub fn format_range(
    source: &str,
    file: Option<&Path>,
    range: TextRange,
    options: FormatOptions,
) -> Result<Vec<TextEdit>, Vec<RenderedDiagnostic>>;
```

- `sifr_format` converts `FormatOptions` into Ruff `PyFormatOptions`.
- `sifr_format` converts parse, format, print, IO, unsupported-option, and invalid-source failures into `RenderedDiagnostic` values with stable Sifr diagnostic codes.
- `sifr_format` must not expose raw Ruff diagnostics or require callers to know Ruff internals.
- CLI diff and stdin/stdout are Sifr wrapper behaviors around `format_source`; they are not blockers on Ruff exposing dedicated diff/stdin library APIs.

### Formatter CLI Parity Contract

The current `sifr fmt` surface only accepts `--check` and a required path. Milestone 4 must replace that placeholder surface with a Ruff-parity formatter command unless a row is reviewed as `not-applicable`.

Required command model:

```bash
sifr fmt [OPTIONS] [FILES]...
```

Required CLI behavior:

- default target is `.` when no file or directory is supplied
- write mode updates files in place
- `--check` performs no writes and exits nonzero when any file would be reformatted
- `--diff` performs no writes, prints unified diffs, and exits nonzero when any file would be reformatted
- `--stdin-filename <path>` reads from stdin, uses the filename for config/path/exclude/source-kind context, and writes formatted source to stdout in write mode
- stdin without a filename is supported; it reads from stdin, writes formatted source to stdout in write mode, treats the source kind as `.sifr`, and uses the current working directory for config discovery and relative path diagnostics
- `--range <range>` is accepted only when exactly one file resolves, uses Ruff's 1-based Unicode-codepoint line/column grammar, and routes to the same formatter range API used by LSP
- `--line-length <n>` overrides config line length
- `--preview` and `--no-preview` override config preview mode
- `--exclude <pattern[,pattern...]>` appends or overrides formatter file selection according to the reviewed Sifr/Ruff config model
- `--respect-gitignore` and `--no-respect-gitignore` mirror Ruff file selection semantics
- `--force-exclude` and `--no-force-exclude` mirror Ruff explicit-target exclusion semantics
- cache controls are implemented through adapted Ruff cache behavior
- Python-only `--extension` and Python `--target-version` semantics must not leak into Sifr; neither option is exposed by `sifr fmt` in this phase
- output summaries, changed-file listing, diff stream destination, and abnormal-error exit status must be documented and regression-tested against the chosen Ruff-compatible contract

This CLI parity manifest is locked. Milestone 1 turns it into a machine-readable test manifest and verifies it against the exact Ruff formatter command surface before Milestone 4 starts. If Ruff has added or removed formatter CLI options, implementation stops for a reviewed planning update rather than editing the manifest opportunistically.

| Ruff formatter CLI surface | Sifr spelling | Classification | Required fixture |
| --- | --- | --- | --- |
| `ruff format [FILES]...` | `sifr fmt [FILES]...` | adapted | `fmt_cli_default_dot_and_multi_path` |
| no positional files defaults to `.` | same | supported | `fmt_cli_default_dot_and_multi_path` |
| `--check` | `--check` | supported | `fmt_cli_check_exit_status_and_changed_listing` |
| `--diff` | `--diff` | supported | `fmt_cli_diff_exit_status_and_unified_diff` |
| `--no-cache` | `--no-cache` | adapted | `fmt_cli_cache_flags` |
| `--cache-dir <path>` | `--cache-dir <path>` | adapted | `fmt_cli_cache_flags` |
| `--respect-gitignore` | `--respect-gitignore` | adapted | `fmt_cli_gitignore_flags` |
| `--no-respect-gitignore` | `--no-respect-gitignore` | adapted | `fmt_cli_gitignore_flags` |
| `--exclude <pattern[,pattern...]>` | `--exclude <pattern[,pattern...]>` | supported | `fmt_cli_exclude_and_force_exclude` |
| `--force-exclude` | `--force-exclude` | adapted | `fmt_cli_exclude_and_force_exclude` |
| `--no-force-exclude` | `--no-force-exclude` | adapted | `fmt_cli_exclude_and_force_exclude` |
| `--line-length <n>` | `--line-length <n>` | supported | `fmt_cli_line_length_override` |
| `--stdin-filename <path>` | `--stdin-filename <path>` | adapted | `fmt_cli_stdin_filename` |
| stdin without files | stdin without files | adapted | `fmt_cli_stdin_default_context` |
| `--extension <ext:language>` | none | not-applicable | `fmt_cli_extension_rejected_or_absent` |
| `--target-version <version>` | none | not-exposed | `fmt_cli_target_version_absent` |
| `--preview` | `--preview` | supported | `fmt_cli_preview_flags` |
| `--no-preview` | `--no-preview` | supported | `fmt_cli_preview_flags` |
| `--range <range>` | `--range <range>` | supported | `fmt_cli_single_file_range` |
| global `--config <file-or-override>` | `--config <file-or-override>` or reviewed Sifr equivalent | adapted | `fmt_cli_config_override_precedence` |
| global `--isolated` | `--isolated` or reviewed Sifr equivalent | adapted | `fmt_cli_isolated_ignores_config` |
| global logging flags used by formatter summaries | Sifr diagnostic/logging equivalent | adapted | `fmt_cli_summary_and_error_streams` |

Formatter summary behavior, incompatible formatter-setting warnings, abnormal-error status, and changed-file listing are in scope and require fixtures even though they are not separate CLI flags.

### LSP and Editor Formatter Integration Contract

The production editor formatter path is LSP-first, matching Ruff's current editor setup pattern. Editors launch:

```bash
sifr lsp --stdio
```

and request formatting through standard LSP methods:

- `textDocument/formatting`
- `textDocument/rangeFormatting`

Required semantics:

- `sifr_lsp` advertises `documentFormattingProvider` and `documentRangeFormattingProvider` only when `sifr.format.enable` is true.
- `sifr_lsp` converts protocol requests, ranges, versions, workspace settings, initialization options, and editor-provided formatting options into `sifr_analysis` requests. It does not parse, format, or call Ruff formatter APIs directly.
- `sifr_analysis` calls `sifr_format` for document and range formatting, preserving the same `FormatOptions` and config precedence used by `sifr fmt`.
- Formatting work uses the Phase 36 separate formatting lane and preserves cancellation, stale document-version rejection, invalid-range diagnostics, line-index conversion, and UTF-8/UTF-16/UTF-32 position invariants.
- CLI, analysis, LSP, and editor-triggered formatting produce equivalent output or edits for the same source/options.
- `verification/tooling/lsp_protocol_matrix.json` must include positive and negative rows for document formatting, range formatting, capability-disable behavior, settings changes, and the `lsp-formatting` performance budget.
- `verification/tooling/lsp_protocol_smoke.py` and `verification/tooling/lsp_protocol_stress.py` must cover initialize, open/change, document formatting, range formatting, cancellation, stale versions, invalid ranges, formatter setting changes, and clean shutdown.

Required editor integration outcomes:

| Target | Required formatter support |
| --- | --- |
| Neovim | `editor_integrations/neovim/lsp/sifr.lua` launches `sifr lsp --stdio`; docs show manual formatting through `vim.lsp.buf.format` and format-on-save setup through the LSP client. |
| Zed | `editor_integrations/zed/extension.toml` and `editor_integrations/zed/languages/sifr/config.toml` register the Sifr LSP as the formatter provider; docs cover `format_on_save`. |
| Helix | `editor_integrations/helix/languages.toml` configures `sifr lsp --stdio`; docs cover `auto-format = true` and manual formatting through Helix's LSP format command. |
| Emacs | `editor_integrations/emacs/sifr-mode.el` documents Eglot/lsp-mode setup against `sifr lsp --stdio`; docs cover `eglot-format` or equivalent save hook. |
| VS Code | `editor_integrations/vscode` and the `sifr-lang/sifr-vscode` contract use the native LSP client document formatting provider, `editor.formatOnSave`, and Sifr formatter settings exposed through LSP configuration. |

Forbidden editor behavior:

- no editor-owned formatter implementation
- no direct Ruff or Python formatter fallback
- no direct parser/AST formatting logic in editor integration code
- no direct `sifr fmt` invocation as the primary document formatting provider
- no capability advertisement when `sifr.format.enable` is false

Direct `sifr fmt` usage remains the supported CLI, CI, hook, and manual-file workflow. It is not the production editor formatting provider.

### Config Layer Contract

Canonical Sifr formatter configuration lives in `sifr.toml`:

```toml
[format]
extend = []
line-length = 88
indent-width = 4
indent-style = "space"
quote-style = "double"
line-ending = "auto"
skip-magic-trailing-comma = false
docstring-code-format = false
docstring-code-line-length = "dynamic"
preview = false
exclude = []
extend-exclude = []
include = ["*.sifr"]
extend-include = []
respect-gitignore = true
force-exclude = false
cache = true
cache-dir = ".sifr_cache/format"
```

Precedence, highest to lowest:

1. CLI flags and explicit LSP/editor formatting options
2. nearest `sifr.toml` `[format]`
3. migration-only Ruff formatter config from `.ruff.toml`, `ruff.toml`, or `pyproject.toml` `[tool.ruff.format]`
4. built-in defaults matching Ruff defaults where the option is shared

Required semantics:

- `sifr.toml` is authoritative when present.
- Ruff config files are read only for shared formatter options and only when no nearer Sifr formatter config overrides them.
- Unknown Sifr formatter keys are deterministic configuration diagnostics.
- Python-only Ruff options are deterministic unsupported-option diagnostics, not ignored.
- Config files do not implicitly merge across directories. The only inheritance mechanism is explicit `[format].extend`, which is ordered, cycle-detected, path-relative to the declaring config file, and tested for deterministic override precedence.
- Explicit file targets are formatted even when they match excludes, matching Ruff's explicit-target behavior.
- Directory discovery respects formatter include/exclude settings and VCS ignore files.
- Shared formatter options must be represented through a Sifr equivalent of Ruff's `FormatterSettings` and converted into `PyFormatOptions` through Ruff's option conversion path where possible.
- CLI `--config <file-or-override>` supports one explicit config file path plus any number of TOML override expressions. Overrides use Sifr formatter keys and shared Ruff formatter key aliases; override expressions take precedence over discovered and explicit config files.
- CLI `--isolated` ignores discovered config files and migration Ruff configs but still accepts inline `--config key=value` overrides, matching Ruff's global option split.
- The config contract must include tests for CLI overrides for `line-length`, preview, excludes, gitignore behavior, forced excludes, cache behavior, `extend`, explicit config paths, inline config overrides, and isolated mode.

### Formatter Coverage Guardrail

Milestone 6 must add an automated guardrail, tentatively `verification/tooling/check_formatter_ast_coverage.py`, and wire it into `scripts/run_all_tests.sh`.

The guardrail must:

1. enumerate Sifr-enabled AST node kinds and Sifr-specific enum variants from the Sifr Ruff fork, including parameter convention variants
2. map each Sifr-only syntax extension to at least one formatter snapshot fixture and one Sifr wrapper fixture
3. fail if an extension is present in the parser/AST inventory but absent from the formatter coverage manifest
4. fail if a coverage manifest row is missing an implementation fixture or approved non-applicability rationale
5. run a positive idempotence and parse-roundtrip sample for every covered extension
6. require reviewer approval for any `not-applicable` formatter coverage row

The first coverage manifest must include `mut`, `own`, `own mut`, tolerant `mut own`, current Sifr type annotation extensions, generics, match/case, collection syntax, and every AST extension present in this planning lock.

### Sifr Parameter Formatter Requirement

Milestone 2 must verify or implement formatter support in the Sifr Ruff fork for Sifr parameter conventions.

Required behavior:

| Source convention | Formatted output |
| --- | --- |
| default borrow | no modifier before parameter name |
| `mut` | `mut name: T` |
| `own` | `own name: T` |
| `own mut` | `own mut name: T` |
| `mut own` | `own mut name: T` |

The implementation target is the Ruff formatter's parameter formatting rule or a reviewed Sifr-specific formatter rule in the fork. A Sifr wrapper may not post-process formatted source strings to repair parameter conventions.

### Docstring Code Formatting Decision

Sifr supports docstring code formatting in this phase, disabled by default to match Ruff's default.

Required behavior:

- `docstring-code-format = true` formats Sifr snippets embedded in docstrings through the same Sifr parser and Ruff-backed formatter core used for files.
- Ruff-recognized docstring code forms are in scope: doctest-style prompts, Markdown fenced code blocks, reStructuredText literal blocks, and `code-block` / `sourcecode` directives.
- Language-tagged fenced/directive blocks are formatted when the language tag is absent, `sifr`, `python`, or `py`, because Sifr's source language is Python syntax with Sifr extensions. Unknown language tags are left unchanged.
- Snippets that do not parse as Sifr are left unchanged and covered by fixtures; the formatter must not invent partial output for invalid embedded snippets.
- `docstring-code-line-length = "dynamic"` follows Ruff's dynamic behavior, and numeric values are mapped to Ruff-compatible line widths.
- Docstring formatting must preserve docstring indentation, quote selection, and surrounding prose.

### Pragma Scope Decision

Sifr will match Ruff formatter pragma semantics where the Sifr AST has equivalent syntactic boundaries:

- `# fmt: off` and `# fmt: on` suppress formatting for statement-level ranges.
- `# yapf: disable` and `# yapf: enable` are aliases for `fmt: off/on`.
- `# fmt: skip` suppresses the preceding statement-level formatting target, including case headers and decorators when the Sifr AST exposes those shapes.
- Pragmas inside expressions do not suppress formatting unless Ruff's range/comment attachment model already treats the containing syntactic boundary as suppressible.

Milestone 6 must add fixtures for every supported pragma form and for expression-level comments that must not suppress formatting.

## Review Log

- `2026-05-25`: planning review branch opened.
- `2026-05-25`: Claude Opus review pass 1 found the phase direction sound but not implementation-ready; blockers were recorded in this addendum.
- `2026-05-25`: Claude Opus review pass 2 confirmed the pass-1 gaps are resolved and the phase is ready for implementation.
- `2026-05-25`: Claude Opus review pass 3 checked formatter CLI/config parity and Ruff crate reuse. The plan was confirmed ready, with the CLI parity manifest added directly afterward so Milestone 4 has a locked option-by-option audit.
- `2026-05-25`: Claude Opus review pass 4 confirmed the CLI parity manifest closes the pass-3 finding and there are no remaining CLI/config/reuse blockers in the planning artifact.
- `2026-05-25`: Claude Opus review pass 5 checked for deferred planning decisions and found one remaining stdin-without-filename deferral.
- `2026-05-25`: Claude Opus review pass 6 confirmed the stdin behavior is explicit and the phase is ready with no deferred planning decisions.
- `2026-05-25`: Claude Opus review pass 7 confirmed the milestone plan is implementation-ready with no gaps.
- `2026-05-25`: Claude Opus review pass 8 confirmed the checked-in formatter showcase demo and milestone evidence requirements are implementation-ready with no gaps.
- `2026-05-25`: Claude Opus review pass 9 confirmed the LSP-first editor formatter plan covers Neovim, Zed, Helix, Emacs, and VS Code, extends completed Phase 36 without modifying it, and is implementation-ready with no blockers.
- `2026-05-26`: Claude Opus final Ruff-docs audit reviewed the local Ruff checkout under `/Users/yaseralnajjar/work/sifr/ruff`, including formatter docs, configuration docs, preview/versioning docs, integrations docs, formatter crate docs, formatter options, range formatting, and CLI implementation references. The review confirmed every Ruff formatter capability, CLI option, config setting, preview/versioning behavior, docstring/pragma behavior, range behavior, cache/file-selection behavior, and editor/LSP integration behavior is planned, adapted, not applicable, or not exposed with no remaining blockers.
- `2026-05-26`: `sifr-lang/ruff#1` was marked ready and merged into `sifr/0.15.12-maintenance` as `b251656613629e054308951a4df1928b3f749b1b`; this execution tracker now locks that merged fork baseline.
- `2026-05-26`: Claude Opus review pass 11 confirmed the fork-baseline contract is explicit and correctly treats `sifr-lang/ruff#1` as merged seed work, with one process blocker: the checked baseline checklist item must only remain checked once `.gitmodules` and the `third_party/ruff` submodule pointer are committed on this branch.
- `2026-05-26`: Claude Opus review pass 12 confirmed the pass-11 submodule commit blocker was resolved and the fork-baseline contract is explicit, with one remaining process blocker: the local branch had not yet been pushed to origin after the amended commit.
- `2026-05-26`: Claude Opus review pass 13 confirmed the branch is synchronized with origin, `third_party/ruff` is committed at `b251656613629e054308951a4df1928b3f749b1b`, `.gitmodules` tracks `sifr/0.15.12-maintenance`, the phase forbids feature-branch/local-only/wrapper-post-processing dependencies, and the phase is implementation-ready with no remaining blockers.
- `2026-05-26`: Milestone 1 implementation review requested for checked formatter capability, CLI parity, AST coverage, and Ruff baseline manifests.
- `2026-05-26`: Claude Opus Milestone 1 review pass 1 approved the manifests and baseline checks, with one blocking pre-existing validation issue: `verification/tooling/check_phase36_closeout.py` still referenced the pre-archive Phase 36 execution issue path.
- `2026-05-26`: Claude Opus Milestone 1 review pass 2 confirmed the archive-path fix resolves the pass-1 blocker and explicitly approved Milestone 1 to close so Milestone 2 may begin.
- `2026-05-26`: Claude Opus Milestone 2 review approved the Ruff fork changes with no blockers and confirmed the current Sifr AST extension surface is covered by the public Sifr wrappers plus formatter fixture corpus.
- `2026-05-26`: Claude Opus Milestone 2 superproject consumption review approved the Sifr PR with no blockers and explicitly approved Milestone 2 consumption to merge so Milestone 3 may begin.
- `2026-05-26`: Claude Opus Milestone 3 review approved the Ruff-backed `sifr_format` core with no blockers and explicitly approved Milestone 3 to merge so Milestone 4 may begin.
- `2026-05-26`: Claude Opus Milestone 4 wave 1 review approved the expanded formatter CLI surface and direct CLI behaviors with no blockers, leaving config discovery, excludes, gitignore, and cache behavior for wave 2.
- `2026-05-26`: Claude Opus Milestone 4 wave 2 review approved config discovery, explicit `--config`, `--isolated`, excludes, gitignore, force-exclude behavior, and cache creation with no blockers; Milestone 4 is approved to merge so Milestone 5 may begin.
- `2026-05-26`: Claude Opus Milestone 5 review approved the analysis, LSP, and editor formatting parity implementation with no blockers; Milestone 5 is approved to merge so Milestone 6 may begin after PR links are recorded.
- `2026-05-26`: Claude Opus Milestone 6 review approved the formatter corpus, AST coverage guardrail, docstring snippet coverage, editor guardrail seeds, validation wiring, and formatter performance budgets with no blockers; Milestone 6 is approved to close after PR merge.
- `2026-05-26`: Claude Opus Milestone 7 review approved the formatter public docs, internal architecture/tooling/LSP/editor docs, formatter showcase evidence, editor integration docs submodule updates, and execution tracker evidence with no blockers; Milestone 7 is approved to close.
- `2026-05-26`: Claude Opus final production-readiness review approved the full formatter phase for final local validation and closure with no blockers. The review confirmed AC-1 through AC-15, all 40 locked capability matrix rows, the single formatter core invariant across CLI/analysis/LSP/editors, AST-extension coverage blocking, docs, PR links, submodule pointers, and showcase evidence.

## Validation Log

- `2026-05-25`: `git diff --check` passed.
- `2026-05-25`: formatter showcase sanity check passed by copying `demos/formatter_showcase/main.sifr.input` to `target/formatter_showcase_check/main.sifr` and running `cargo run -q -p sifr -- check target/formatter_showcase_check/main.sifr`.
- `2026-05-26`: Ruff PR `sifr-lang/ruff#1` was validated locally from `third_party/ruff` branch `codex/format-sifr-param-conventions` before merge with `cargo fmt -p ruff_python_formatter --check`, `cargo check -p ruff_python_formatter --quiet`, `cargo test -p ruff_python_formatter sifr_ --quiet`, `cargo test -p ruff_python_formatter --lib --quiet`, and `cargo clippy -p ruff_python_formatter --all-targets -- -D warnings`.
- `2026-05-26`: after `sifr-lang/ruff#1` merged, `third_party/ruff` was fast-forwarded to `b251656613629e054308951a4df1928b3f749b1b` on `sifr/0.15.12-maintenance` and `.gitmodules` was updated to track that maintenance branch.
- `2026-05-25`: `scripts/run_all_tests.sh --profile quick` was run for this planning PR and reached the existing Phase 36 closeout guardrail before failing on `required closeout doc missing: issues/phase36-developer-tooling-execution.md`. That failure is outside this formatter-planning scope because Phase 36 is already complete and its execution issue has been archived on the current branch. No Phase 36 files or guardrails are changed by this phase PR.
- `2026-05-25`: after adding the LSP-first editor formatter plan, `scripts/run_all_tests.sh --profile quick` was rerun. It passed HIR/file-size/driver/package-manager guardrails, diagnostic checks, Phase 35 split-brain checks, developer tooling checks, VS Code extension contract/package checks, formatter contract checks, rule/suppression checks, analysis snapshot/split-brain checks, tooling parity, completion quality self-test, LSP protocol smoke/stress checks, and editor asset checks, then reached the same existing Phase 36 closeout guardrail failure: `required closeout doc missing: issues/phase36-developer-tooling-execution.md`.
- Validation evidence will be recorded per implementation milestone before merge.
- `2026-05-26`: Milestone 1 added machine-readable manifests under `verification/tooling/formatter_manifests/` for the Ruff baseline, Ruff-to-Sifr formatter capability matrix, formatter CLI parity, and current Sifr AST formatter coverage inventory.
- `2026-05-26`: Milestone 1 added `verification/tooling/check_formatter_phase_manifests.py` and wired it into `scripts/run_all_tests.sh` after the Phase 36 formatter contract check.
- `2026-05-26`: Milestone 1 restored `third_party/ruff` to `b251656613629e054308951a4df1928b3f749b1b` and revalidated the Phase 35 syntax token fixture revision metadata because the seed commit changes Ruff formatter files, not parser tokenization.
- `2026-05-26`: Milestone 1 targeted validation passed: `python3 verification/tooling/check_formatter_phase_manifests.py`, `python3 verification/tooling/check_formatter_phase_manifests.py --self-test`, `python3 verification/performance/check_ruff_fork_update_contract.py`, `cargo test -p sifr_syntax`, `python3 verification/tooling/check_formatter_contract.py`, `python3 verification/tooling/check_formatter_contract.py --self-test`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check`.
- `2026-05-26`: `scripts/run_all_tests.sh --profile quick` passed the new formatter manifest gate and all developer tooling gates through editor assets, then stopped at the existing archived Phase 36 closeout guardrail: `required closeout doc missing: issues/phase36-developer-tooling-execution.md`. The lane report was written to `target/validation_lane_reports/quick.latest.json`.
- `2026-05-26`: Milestone 1 fixed the Phase 36 closeout guardrail to reference `issues/archive/phase36-developer-tooling-execution.md`, matching the archived issue layout.
- `2026-05-26`: `python3 verification/tooling/check_phase36_closeout.py`, `python3 verification/tooling/check_phase36_closeout.py --self-test`, and `git diff --check` passed after the archive-path fix.
- `2026-05-26`: `scripts/run_all_tests.sh --profile quick` passed end to end after the archive-path fix. The lane report was written to `target/validation_lane_reports/quick.latest.json`; it recorded a warm wall-time advisory but exit status was 0.
- `2026-05-26`: Milestone 2 Ruff fork validation passed from `third_party/ruff`: `cargo fmt -p ruff_python_formatter --check`, `cargo test -p ruff_python_formatter sifr_ --quiet`, `cargo test -p ruff_python_formatter --test fixtures sifr_extensions --quiet`, `cargo test -p ruff_python_formatter --lib --quiet`, and `git -C third_party/ruff diff --check`.
- `2026-05-26`: Ruff fork PR <https://github.com/sifr-lang/ruff/pull/2> merged into `sifr/0.15.12-maintenance` as `f9da46641894c8a2380b1e0f17bed68f09c46643`, adding public Sifr formatter wrapper entrypoints and Sifr formatter fixture coverage.
- `2026-05-26`: Milestone 2 superproject consumption validation passed: `python3 verification/performance/check_ruff_fork_update_contract.py`, `python3 verification/tooling/check_formatter_phase_manifests.py`, `cargo test -p sifr_syntax`, `git diff --check`, and `scripts/run_all_tests.sh --profile quick`. The quick lane exited 0 and wrote `target/validation_lane_reports/quick.latest.json`; it recorded warm wall-time and group-skew advisories.
- `2026-05-26`: Milestone 3 targeted validation passed: `cargo fmt -p sifr_format --check`, `cargo test -p sifr_format`, `CARGO_TARGET_DIR=target/codex-m3 python3 verification/tooling/check_formatter_contract.py`, `CARGO_TARGET_DIR=target/codex-m3 python3 verification/tooling/check_formatter_contract.py --self-test`, `python3 verification/tooling/check_formatter_phase_manifests.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check`.
- `2026-05-26`: Milestone 3 quick validation passed with `scripts/run_all_tests.sh --profile quick`. The lane exited 0 and wrote `target/validation_lane_reports/quick.latest.json`; it recorded a warm wall-time advisory and group-skew advisory after compiling the new Ruff-backed formatter dependency graph.
- `2026-05-26`: Milestone 4 wave 1 targeted validation passed: formatter CLI smoke coverage for `--check`, `--diff`, `--stdin-filename`, stdin formatting, `--range`, `--line-length`, `--preview`, cache flag parsing, and multi-path/default directory behavior; `cargo fmt -p sifr -p sifr_format --check`; `cargo test -p sifr_format`; `cargo test -p sifr -- --skip test_e2e_pass`; `python3 verification/tooling/check_formatter_contract.py`; `python3 verification/tooling/check_formatter_contract.py --self-test`; `python3 scripts/check_file_size_guardrails.py`; and `git diff --check`.
- `2026-05-26`: Milestone 4 wave 2 targeted validation passed: formatter CLI smoke coverage for discovered `sifr.toml` `[format]`, explicit `--config` override, `--isolated`, excludes, `.gitignore`, explicit target force-exclude behavior, and cache directory creation; `cargo fmt -p sifr -p sifr_format --check`; `cargo test -p sifr_format`; `cargo test -p sifr -- --skip test_e2e_pass`; `python3 verification/tooling/check_formatter_contract.py`; `python3 verification/tooling/check_formatter_contract.py --self-test`; `python3 scripts/check_file_size_guardrails.py`; and `git diff --check`.
- `2026-05-26`: Milestone 5 targeted validation passed: `cargo fmt -p sifr -p sifr_format -p sifr_lsp -p sifr_analysis --check`; `cargo test -p sifr_lsp -p sifr_analysis -p sifr_format`; `cargo build -p sifr`; `python3 verification/tooling/lsp_protocol_smoke.py`; `python3 verification/tooling/lsp_protocol_stress.py`; `python3 verification/tooling/check_editor_assets.py`; `python3 verification/tooling/check_tooling_contract_lock.py`; self-tests for the LSP protocol, editor assets, and tooling contract checks; `cargo test -p sifr -- --skip test_e2e_pass`; `python3 scripts/check_file_size_guardrails.py`; and `git diff --check --ignore-submodules=none`.
- `2026-05-26`: Milestone 5 quick validation passed with `scripts/run_all_tests.sh --profile quick`. The lane exited 0 and wrote `target/validation_lane_reports/quick.latest.json`; it recorded warm wall-time and group-skew advisories.
- `2026-05-26`: Milestone 6 Ruff fork validation passed from `third_party/ruff`: `cargo fmt -p ruff_python_formatter --check` and `cargo test -p ruff_python_formatter --test fixtures sifr_extensions --quiet`.
- `2026-05-26`: Ruff fork PR <https://github.com/sifr-lang/ruff/pull/3> merged into `sifr/0.15.12-maintenance` as `8b95ca3d888910aa39632cd9873c05de5121ab67`, adding Sifr-tagged docstring code snippet formatting support and fixtures.
- `2026-05-26`: Milestone 6 updated the Ruff fork revalidation metadata and representative syntax token fixtures to record `8b95ca3d888910aa39632cd9873c05de5121ab67`; `python3 verification/performance/check_ruff_fork_update_contract.py` passed.
- `2026-05-26`: Milestone 6 targeted superproject validation passed: `cargo fmt -p sifr_format -p sifr --check`, `cargo clippy -p sifr_format -- -D warnings`, `cargo test -p sifr_format`, `cargo test -p sifr -- --skip test_e2e_pass`, `python3 verification/tooling/check_formatter_ast_coverage.py`, `CARGO_TARGET_DIR=target/codex-m6-ast-env python3 verification/tooling/check_formatter_ast_coverage.py`, `python3 verification/tooling/check_formatter_ast_coverage.py --self-test`, `python3 verification/tooling/check_formatter_phase_manifests.py`, `python3 verification/tooling/check_formatter_phase_manifests.py --self-test`, `python3 verification/tooling/check_editor_assets.py`, `python3 verification/tooling/check_editor_assets.py --self-test`, `python3 verification/performance/run_benchmarks.py --validate-only`, `python3 verification/performance/run_benchmarks.py --self-test`, `python3 verification/performance/check_budgets.py`, `python3 verification/performance/check_budgets.py --self-test`, formatter performance smoke and budget checks, `python3 scripts/check_file_size_guardrails.py`, `git diff --check --ignore-submodules=none`, and `git -C third_party/ruff diff --check`.
- `2026-05-26`: Milestone 6 formatter performance baselines were captured in `target/performance/formatter-m6-baseline.json` and added to `verification/performance/baselines.json` and `verification/performance/budgets.json` for `perf.formatter.corpus.project_check` and `perf.formatter.large_file.check`.
- `2026-05-26`: Milestone 6 quick validation passed from committed state with `scripts/run_all_tests.sh --profile quick`. The lane exited 0 and wrote `target/validation_lane_reports/quick.latest.json`; it recorded `wall_time=1804.37s`, `cache_hits=12/12`, no swaps, and warm wall-time/group-skew advisories.
- `2026-05-26`: Milestone 7 formatter showcase smoke passed by copying `demos/formatter_showcase/main.sifr.input` to `target/formatter_showcase_m7/main.sifr`, running `cargo run -q -p sifr -- fmt --no-cache target/formatter_showcase_m7/main.sifr`, and checking the formatted result with `cargo run -q -p sifr -- check target/formatter_showcase_m7/main.sifr`.
- `2026-05-26`: Milestone 7 targeted docs/contract validation passed: `cargo fmt -p sifr --check`, `python3 verification/tooling/check_editor_assets.py`, `python3 verification/tooling/check_editor_assets.py --self-test`, `python3 verification/tooling/check_formatter_ast_coverage.py`, `python3 verification/tooling/check_formatter_ast_coverage.py --self-test`, `python3 verification/tooling/check_formatter_phase_manifests.py`, `python3 verification/tooling/check_formatter_phase_manifests.py --self-test`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check --ignore-submodules=none`.
- `2026-05-26`: final quick validation passed from committed state with `scripts/run_all_tests.sh --profile quick`. The lane exited 0 and wrote `target/validation_lane_reports/quick.latest.json`; it recorded `wall_time=1148.21s`, `max_rss=672.6MiB`, `swaps=0`, `cache_hits=12/12`, and warm wall-time/group-skew advisories.
- `2026-05-26`: final full validation passed from committed state with `scripts/run_all_tests.sh`. The lane exited 0 and wrote `target/validation_lane_reports/pr.latest.json`; it recorded `wall_time=3138.75s`, `max_rss=525.2MiB`, `swaps=0`, `cache_hits=0/19`, hardening `variants=28` with `blocking_failures=0`, and warm wall-time/group-skew advisories.

Formatter showcase before/after evidence:

```diff
--- demos/formatter_showcase/main.sifr.input
+++ target/formatter_showcase_m7/main.sifr
@@
-def normalize_scores( mut own scores:list[int],bonus:int)->list[int]:
+def normalize_scores(own mut scores: list[int], bonus: int) -> list[int]:
@@
-    if len(scores)>3:
-        scores[0]=scores[0]+bonus
+    if len(scores) > 3:
+        scores[0] = scores[0] + bonus
@@
-def main()->None:
-    values=[1,2,3]
-    updated=normalize_scores(values,4)
+def main() -> None:
+    values = [1, 2, 3]
+    updated = normalize_scores(values, 4)
```

## PR Log

- Milestone 1 `formatter_contract_manifests_and_ast_inventory`: <https://github.com/sifr-lang/sifr/pull/2175>
- Milestone 2 `ruff_fork_sifr_formatter_ast_completion`: Ruff fork PR <https://github.com/sifr-lang/ruff/pull/2>; superproject consumption PR <https://github.com/sifr-lang/sifr/pull/2176>
- Milestone 3 `sifr_format_ruff_backed_core`: <https://github.com/sifr-lang/sifr/pull/2177>
- Milestone 4 wave 1 `formatter_cli_surface`: <https://github.com/sifr-lang/sifr/pull/2178>
- Milestone 4 wave 2 `formatter_cli_config_selection_cache`: <https://github.com/sifr-lang/sifr/pull/2179>
- Milestone 5 `formatter_lsp_editor_parity`: VS Code PR <https://github.com/sifr-lang/sifr-vscode/pull/2>; editor integrations PR <https://github.com/sifr-lang/editor-integrations/pull/1>; superproject PR <https://github.com/sifr-lang/sifr/pull/2180>
- Milestone 6 `formatter_corpus_guardrails_and_performance`: Ruff fork PR <https://github.com/sifr-lang/ruff/pull/3>; superproject PR <https://github.com/sifr-lang/sifr/pull/2181>
- Milestone 7 `formatter_docs_closeout_and_release_readiness`: editor integrations docs PR <https://github.com/sifr-lang/editor-integrations/pull/2>; editor integrations guardrail wording fix PR <https://github.com/sifr-lang/editor-integrations/pull/3>; superproject closeout PR <https://github.com/sifr-lang/sifr/pull/2182>
