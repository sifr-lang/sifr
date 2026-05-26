

---

## M4 Wave 2 Review

### Blockers

**None.** All six review criteria pass.

### Detailed Findings

**1. CLI surface and config behavior:** 
- All M4 CLI flags present and functional: `--config`, `--isolated`, `--exclude`, `--respect-gitignore`/`--no-respect-gitignore`, `--force-exclude`/`--no-force-exclude`, `--no-cache`, `--cache-dir`
- `--config` accepts both file paths and `KEY=VALUE` overrides (verified: `target-version` rejection, unknown key rejection, cycle detection, `--isolated` bypass, `line-length=40` override all work correctly)
- `target-version` and `extension` absent from CLI and rejected in config with deterministic `FMT_FORMATTING_DRIFT` diagnostics (verified)
- Default target is `.` when no paths provided (verified via `collect_sifr_files`)
- `--isolated` correctly bypasses config discovery while still accepting `--config` overrides (verified)

**2. Config discovery and precedence:**
- `discover_sifr_toml()` walks cwd ancestors from current directory (verified: works from any subdirectory)
- `extend` loading with BTreeSet cycle detection (verified: self-reference produces deterministic error)
- Precedence order: CLI overrides → explicit `--config` files/overrides → discovered `sifr.toml` → defaults
- Unknown config keys and Python-only options produce deterministic diagnostics (verified)

**3. File selection behavior:**
- `select_formatter_files()` respects exclude patterns, gitignore patterns, explicit-target vs directory distinction, and `force_exclude` flag
- `.gitignore` parsing strips empty lines and comments (verified)
- Explicit file targets bypass excludes unless `--force-exclude` is set (verified)
- Default excluded directories: `.git`, `target`, `.venv`, `venv`, `node_modules`, `sifr_output`

**4. Cache behavior:**
- Cache directory creation on first use (verified: `b62affc358e3c05d` entry created)
- Cache hit skips formatting in write mode (verified: second run prints "formatted" but doesn't format)
- `formatter_cache_key` covers: path, source content, `final_newline`, `line_length`, `preview` — consistent with `FormatOptions` fields used in `sifr_format`
- `--no-cache` bypasses both reads and writes (verified)

**5. Formatter routing:**
- All formatting routes through `sifr_format::format_source` → `format_sifr_module_source` → Ruff formatter. No parallel formatter.
- `check_path_with_options`, `format_path_with_options`, `format_range`, `format_source` all use the same `ruff_options()` conversion path.

**6. File sizes and safety:**
- `formatter_config.rs`: 232 lines (new file, well under 900)
- `formatter_cli.rs`: 65 lines (new file, well under 900)
- `check_and_package_commands.rs`: 687 lines (below 900)
- `cli_model_and_entrypoint.rs`: 885 lines (below 900)
- No panics in user paths; `run_with_panic_boundary` wraps formatter execution
- Diff output to stdout, success messages to stderr — matches Sifr conventions

**7. Tracker completeness:**
- Validation log records wave 2 targeted validation with all smoke checks
- Checklist item `CLI and config parity completed` is still unchecked — needs manual update to `[x]` before merge
- PR log has wave 1 PR link; M4 wave 2 (closure PR) not yet created

### Non-Blocking Observations

1. **Cache key missing Sifr version:** `formatter_cache_key` does not include `SIFR_BUILD_VERSION` or `third_party/ruff` commit hash. The cache is functional but version drift across rebuilds won't invalidate stale entries. This is a future optimization, not a blocker.

2. **Config schema partial:** `apply_format_table` handles `line-length`, `preview`, `exclude`, `respect-gitignore`, `force-exclude`, `no-cache`, `cache-dir`, `target-version`, `extension`, and unknown keys. The full contract schema (`indent-width`, `indent-style`, `quote-style`, `line-ending`, `skip-magic-trailing-comma`, `docstring-code-format`, `docstring-code-line-length`) is listed in the phase contract but not yet wired in `sifr_format::FormatOptions` — those options would need to be added to both `FormatOptions` and the config parser. This is future work beyond M4 scope.

3. **`respect_gitignore` default value:** The clap field changed from `default_value_t = true` to implicit `false` default (lines 38-40 in `formatter_cli.rs`), which is overridden by `apply_cli_overrides` only when the flag is provided. This means `--respect-gitignore` and `--no-respect-gitignore` both override the config, which is correct behavior — the clap default should be absent so CLI never overrides config when neither flag is given. Verified correct.

---

**M4 is approved to merge.** M4 wave 2 satisfies the M4 rows for `sifr.toml` discovery, explicit `--config`, `extend` loading with cycle rejection, `--isolated`, unknown key diagnostics, Python-only option rejection, exclude/gitignore/force-exclude behavior, and cache directory creation. All validation passed. M5 may begin after the checklist item is updated and the closure PR is opened.
