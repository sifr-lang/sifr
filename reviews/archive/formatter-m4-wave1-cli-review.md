

## M4 Wave 1 Review

### Blockers

**None.** Wave 1 passes all six review criteria.

### Detailed Findings

**1. CLI surface and guardrails:** `formatter_cli.rs` is 65 lines (new file, well under the cap). `check_and_package_commands.rs` grew from ~409 to 576 lines — stays within the 900-line hand-maintained limit. Existing commands unchanged.

**2. Formatter routing:** All formatting routes through `sifr_format::FormatOptions` → Ruff `PyFormatOptions` via `ruff_options()` in `sifr_format/src/lib.rs:214-242`. No parallel formatter anywhere in the path.

**3. Wave-1 safety:**
- `fmt_stdin` at `check_and_package_commands.rs:460-487` correctly routes stdin when `--stdin-filename` is provided **or** when paths are empty and stdin is non-tty.
- `parse_byte_range` at line 518-532 rejects malformed input with a clear `FMT_FORMATTING_DRIFT` diagnostic. Invalid ranges (non-UTF-8 boundaries, start > end) route to `invalid_range_diagnostic` through `validate_range`.
- `--check` / `--diff` mutual exclusion via clap is correct.
- Diff output goes to stdout, success messages to stderr — matches existing Sifr conventions.

**4. Line length and preview conversion:** `format_options_from_args` at line 489-495 correctly maps:
- `line_length`: falls back to 88 when absent; `ruff_options` converts to `LineWidth::try_from()` which rejects out-of-range.
- `preview`: `args.preview && !args.no_preview` — evaluates to `false` when neither flag is provided (both default to `false` in clap), matching the phase's "stable mode default" requirement.

**5. Tracker completeness:** The checklist item `CLI and config parity completed` is unchecked. The execution log at line 405 only records wave-1 targeted validation. Config, excludes, gitignore, and cache behavior are correctly **not** marked complete — those are wave-2 scope per the wave-1 scope description.

**6. Merge readiness:** The `--respect-gitignore`, `--no-respect-gitignore`, `--force-exclude`, `--no-force-exclude`, `--exclude`, `--no-cache`, and `--cache-dir` flags are parsed and available on `FmtArgs`, but the actual file-selection behavior is deferred to wave 2. This is by design per the scope description and is not a blocker. Wave 1 focuses on the CLI arg surface and routing through the Ruff-backed formatter core.

---

**M4 wave 1 is approved to merge. M4 wave 2 may begin.** Wave 2 scope (config discovery, excludes, gitignore, cache wiring) is clearly bounded and correctly deferred.
