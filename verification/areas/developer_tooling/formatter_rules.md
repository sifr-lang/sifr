# Formatter Rules Reference

This file is the durable verification-owned reference for formatter capability,
CLI parity, and AST coverage manifests. The formatter manifest checker verifies
that each checked-in manifest row remains represented here.

## Capabilities

| Capability | Requirement |
| --- | --- |
| Whole-file formatting | `sifr_format` calls Ruff formatter library APIs over Sifr AST/source and returns complete formatted source. |
| Recursive path formatting | `sifr fmt <path>` discovers `.sifr` files through Sifr project/path rules while preserving Ruff explicit-target behavior. |
| Check mode | `sifr fmt --check` reports drift without writes and exits nonzero on drift. |
| Diff mode | Ruff diff behavior is CLI-layer, not formatter-library-layer. Sifr must generate unified diffs from original/formatted text in its CLI wrapper without shelling out to Ruff. |
| Stdin formatting | Sifr CLI owns stdin/stdout and optional filename context, then calls the same in-process formatter API. |
| Single-file CLI range formatting | Add a Ruff-compatible `--range` option for one resolved file only, using the same position grammar and formatter range API. |
| Formatter cache | Implement Ruff-style formatter cache behavior with cache keys covering source metadata, formatter options, Sifr version, and Sifr Ruff fork revision. |
| `--respect-gitignore` / `--no-respect-gitignore` | Match Ruff's file-selection behavior for Sifr paths. |
| `--force-exclude` / `--no-force-exclude` | Match Ruff's distinction between explicit roots and forced exclusions. |
| CLI `--exclude` | Match Ruff comma-delimited file-pattern override semantics for format file selection. |
| CLI `--line-length` | Match Ruff formatter override semantics and precedence. |
| CLI `--pvalidation` / `--no-pvalidation` | Match Ruff pvalidation toggle behavior while keeping stable style default. |
| CLI `--target-version` | Do not expose Python target-version semantics. Sifr has no formatter syntax-version flag in the current rules; a future language-edition rules may add one explicitly. |
| CLI `--extension` | Sifr formatter source kind is `.sifr` only in the current rules; multiple source-kind mapping requires a later product decision. |
| Config discovery | Sifr canonical config is `sifr.toml`; Ruff config files are migration inputs only under the precedence rules below. |
| Exclude/include and VCS ignores | Sifr file discovery must support formatter include/exclude settings, `.gitignore`, and explicit target overrides. |
| Line length | Map directly to Ruff `PyFormatOptions` line width. |
| Indent width | Map directly to Ruff indent width. |
| Indent style | Map directly to Ruff indent style. |
| Quote style | Map directly where Sifr string syntax uses Ruff-compatible string tokens. |
| Line ending | Map directly to Ruff line-ending behavior. |
| Magic trailing comma behavior | Map directly for Sifr calls, collections, signatures, and type constructs that use Ruff AST layouts. |
| Docstring code formatting | Support Ruff-recognized docstring code forms by formatting Sifr snippets with the Sifr parser/formatter. The option remains disabled by default. |
| Docstring code line length | Same disposition as docstring code formatting. |
| Formatter pvalidation mode | Expose explicit Sifr pvalidation flag/config that maps to Ruff pvalidation mode. Stable mode remains default. |
| `# fmt: off` and `# fmt: on` | Apply at statement level, matching Ruff/Black semantics. |
| `# fmt: skip` | Apply to preceding statement, case header, decorator, or other Ruff-supported syntactic boundary adapted to Sifr AST. |
| `# yapf: disable` and `# yapf: enable` | Treat as aliases for `fmt: off/on` at the same statement-level boundaries Ruff documents. |
| Editor document formatting | LSP `textDocument/formatting` routes through `sifr_lsp -> sifr_analysis -> sifr_format` and is advertised only when `sifr.format.enable` is true. |
| Editor range formatting | LSP `textDocument/rangeFormatting` routes through the same formatter context; no independent LSP/editor formatter. |
| Editor setup and format-on-save | Neovim, Zed, Helix, Emacs, and VS Code assets/docs must expose formatting through `sifr lsp --stdio` and standard editor LSP formatting hooks. |
| VS Code document formatting provider | The extension uses the Sifr LSP client document formatting provider and format-on-save settings; extension TypeScript owns no formatter. |
| Formatter ecosystem checks | Add Sifr corpus checks inspired by Ruff ecosystem checks: idempotence, parser roundtrip, no panics, invalid-source diagnostics, comments/pragmas, config matrix, and performance budgets. |
| Notebook formatting | Sifr has no notebook product surface in the current rules. A later product rules must add this explicitly if needed. |
| Import sorting | Ruff treats import sorting as lint/fix behavior, not formatting. Sifr format must not reorder imports. |

## CLI Parity

| Ruff surface | Sifr spelling | Required fixture |
| --- | --- | --- |
| ruff format [FILES]... | sifr fmt [FILES]... | fmt_cli_default_dot_and_multi_path |
| no positional files defaults to `.` | same | fmt_cli_default_dot_and_multi_path |
| `--check` | `--check` | fmt_cli_check_exit_status_and_changed_listing |
| `--diff` | `--diff` | fmt_cli_diff_exit_status_and_unified_diff |
| `--no-cache` | `--no-cache` | fmt_cli_cache_flags |
| `--cache-dir <path>` | `--cache-dir <path>` | fmt_cli_cache_flags |
| `--respect-gitignore` | `--respect-gitignore` | fmt_cli_gitignore_flags |
| `--no-respect-gitignore` | `--no-respect-gitignore` | fmt_cli_gitignore_flags |
| `--exclude <pattern[,pattern...]>` | `--exclude <pattern[,pattern...]>` | fmt_cli_exclude_and_force_exclude |
| `--force-exclude` | `--force-exclude` | fmt_cli_exclude_and_force_exclude |
| `--no-force-exclude` | `--no-force-exclude` | fmt_cli_exclude_and_force_exclude |
| `--line-length <n>` | `--line-length <n>` | fmt_cli_line_length_override |
| `--stdin-filename <path>` | `--stdin-filename <path>` | fmt_cli_stdin_filename |
| stdin without files | stdin without files | fmt_cli_stdin_default_context |
| `--extension <ext:language>` | none | fmt_cli_extension_rejected_or_absent |
| `--target-version <version>` | none | fmt_cli_target_version_absent |
| `--pvalidation` | `--pvalidation` | fmt_cli_pvalidation_flags |
| `--no-pvalidation` | `--no-pvalidation` | fmt_cli_pvalidation_flags |
| `--range <range>` | `--range <range>` | fmt_cli_single_file_range |
| global `--config <file-or-override>` | `--config <file-or-override>` or validated Sifr equivalent | fmt_cli_config_override_precedence |
| global `--isolated` | `--isolated` or validated Sifr equivalent | fmt_cli_isolated_ignores_config |
| global logging flags used by formatter summaries | Sifr diagnostic/logging equivalent | fmt_cli_summary_and_error_streams |

## AST Coverage

| Row | Syntax | Required behavior |
| --- | --- | --- |
| param_default_borrow | default borrow parameter | no modifier before parameter name |
| param_mut | `mut name: T` parameter | preserve canonical `mut name: T` |
| param_own | `own name: T` parameter | preserve canonical `own name: T` |
| param_own_mut | `own mut name: T` parameter | preserve canonical `own mut name: T` |
| param_mut_own_tolerant | tolerant `mut own name: T` parameter | canonicalize to `own mut name: T` in the Ruff fork |
| sifr_type_annotations | Sifr type annotation extensions | format all current Sifr type annotation syntax through AST-aware Ruff rules |
| sifr_generics | Sifr generic definitions and applications | format current generic syntax without wrapper post-processing |
| match_case | match/case syntax | format match statements, case headers, guards, patterns, comments, and pragmas through Ruff AST formatting |
| ownership_aware_collections | ownership-aware collection syntax | format all current ownership-aware collection forms with parser-roundtrip and idempotence coverage |
| formatter_pragmas | `# fmt` and `# yapf` formatter pragmas | match Ruff statement-level suppression semantics for Sifr syntax boundaries |
| docstring_code_snippets | Sifr snippets inside docstrings | format Ruff-recognized docstring code forms when docstring code formatting is enabled |
