

---

## Milestone 3 Review

**No blockers.** Milestone 3 is approved to merge. Milestone 4 may begin.

---

### Checklist Results

**1. format_source and format_range routing** ✅

`format_source` at `lib.rs:59` calls `format_sifr_module_source`. `format_range` at `lib.rs:97` calls `format_sifr_range` (aliased as `ruff_format_sifr_range`). Both route through the public Ruff fork Sifr formatter wrappers from `third_party/ruff/crates/ruff_python_formatter/src/lib.rs`. The wrappers were added in Milestone 2 (Ruff PR `sifr-lang/ruff#2`, commit `f9da466`).

**2. Old whitespace normalizer removed** ✅

The Phase 36 whitespace-only formatter path is completely removed:

- `normalize_source`, `trim_line_len_outside_protected`, `protected_contains`, `full_range` — all deleted
- No fallback path remains
- The 389-line old lib.rs is replaced with a 534-line Ruff-backed version (net +145 lines, accounting for Ruff error handling, range validation, and new tests)

**3. Test coverage** ✅

6 tests cover the required cases:

| Test | Covers |
|---|---|
| `formatter_is_ruff_backed_and_preserves_string_contents` | Idempotence, string contents, parser roundtrip |
| `formatter_canonicalizes_sifr_parameter_conventions` | `mut own` → `own mut`, Sifr param conventions |
| `range_formatting_returns_minimal_text_edit` | Range edits, byte-level correctness |
| `check_reports_formatting_drift` | Check mode, diagnostic code |
| `invalid_source_reports_sifr_diagnostic` | Parse errors → `SIFR-FMT-0001` |
| `unsupported_final_newline_option_reports_diagnostic` | Unsupported option → diagnostic |

**4. Ruff error conversion** ✅

`FormatModuleError` variants (`ParseError`, `FormatError`, `PrintError`) are converted into deterministic `RenderedDiagnostic` values at `lib.rs:246–280` with stable Sifr codes (`SIFR-FMT-0001`). Raw Ruff types are not exposed to callers.

**5. Cargo dependencies** ✅

- `ruff_formatter` and `ruff_python_formatter` added to workspace `Cargo.toml` under `third_party/ruff` path
- `sifr_format/Cargo.toml` uses `{ workspace = true }` for both
- Consistent with existing submodule/workspace strategy (no external crates.io additions)

**6. check_formatter_contract.py update** ✅

Self-test updated to check for `SIFR-FMT-0001` and `"formatter could not parse Sifr source"` — aligned with the new fail-closed Ruff-backed diagnostic surface. Both `run_positive()` and `run_self_test()` pass.

**7. Merging readiness** ✅

All changed files pass validation:
- `cargo test -p sifr_format` — 6/6 pass
- `python3 verification/tooling/check_formatter_contract.py` — PASS
- `python3 verification/tooling/check_formatter_contract.py --self-test` — PASS
- `git diff --check` — clean

Changed files: `Cargo.toml`, `crates/sifr_format/Cargo.toml`, `crates/sifr_format/src/lib.rs`, `verification/tooling/check_formatter_contract.py`, `issues/ad-hoc-production-grade-sifr-formatter-execution.md` (execution log update).

---

### Notes for Milestone 4

- `FormatOptions` currently only has `final_newline`; the locked public API in the phase contract specifies full `FormatOptions` with `line_length`, `indent_width`, `indent_style`, `quote_style`, `line_ending`, `magic_trailing_comma`, `preview`, `docstring_code_format`, `docstring_code_line_length`
- The CLI currently has only `--check`; the full locked CLI surface is in the CLI parity manifest
- Config discovery (`sifr.toml`, Ruff migration config, CLI overrides, extend) is Milestone 4 scope, not a Milestone 3 gap
