

I have completed my review of the working tree changes for Phase 36 milestone 36.2 (Diagnostics, Rules, Suppressions, Exclusions, and Formatting Foundation).

## SATISFIED

The implementation is sound. No blocking findings. The milestone contract is met.

---

## Review Summary

### Scope Reviewed

Working tree changes only (staged + unstaged), across:
- Workspace additions: `crates/sifr_format/`, `crates/sifr_lint/`
- CLI integration: `crates/sifr/src/main.rs` (Fmt + Lint commands)
- Diagnostics: `crates/sifr_diagnostics/src/codes.rs` (FMT/LINT families)
- Generated docs: `docs/errors/SIFR-FMT-0001.md`, `docs/errors/SIFR-LINT-0001..4.md`
- Internal docs: `internal_docs/diagnostic_codes.md`
- Validation scripts: `verification/tooling/check_formatter_contract.py`, `verification/tooling/check_rule_suppression_contract.py`
- Test wiring: `crates/sifr_driver/src/tests/diagnostics.rs` (LINT labels)
- Script wiring: `scripts/run_all_tests.sh`

### What Passes

**Architecture correctness**
- `sifr_format` and `sifr_lint` are proper workspace members with clean dependencies: `sifr_format` depends only on `sifr_syntax` + `sifr_diagnostics`; `sifr_lint` depends only on `sifr_diagnostics`. No forbidden paths.
- Both crates are conservative by design — formatter normalizes line endings and trims trailing whitespace outside string tokens; lint surfaces suppression diagnostics and a single `trailing-whitespace` rule. This matches the phase contract's "conservative foundation" requirement.
- CLI commands wire through `run_with_panic_boundary` correctly, with proper `Result`/error propagation and panic-to-internal-diagnostic conversion.

**Formatter contract** (`sifr_format`)
- Parser round-trip is enforced: `format_source` re-parses the formatted output and returns an error if it doesn't parse.
- String-token protection: tokens with `kind.as_str() == "String"` are collected and passed to `trim_line_len_outside_protected`, which skips whitespace removal for protected ranges.
- `format_range` correctly returns the full-range `TextEdit` when the input range matches the full source.
- Idempotence is verified in the unit test (`formatter_is_idempotent_and_preserves_string_trailing_space`).

**Lint contract** (`sifr_lint`)
- `parse_suppressions` correctly handles the `# sifr: ignore[rule-id]` syntax, including comma-separated multi-rule suppression and the no-blanket-suppression rule (empty rule list after `sifr: ignore` produces `SIFR-LINT-0003`).
- `mark_suppressed` correctly tracks line + rule matching — suppression only applies to the rule on the same line.
- Unknown suppression (`SIFR-LINT-0001`) and unused suppression (`SIFR-LINT-0002`) are reported correctly.
- The `LintOptions::explicit_target` flag ensures exclusions never affect explicitly passed files (`should_exclude` short-circuits for `options.explicit_target && path.is_file()`).

**Diagnostic codes**
- `FMT` family has one code: `SIFR-FMT-0001` (Error). Registered in `DIAGNOSTIC_FAMILIES`, `DIAGNOSTIC_REGISTRY`, and `ACTIVE_DIAGNOSTIC_CODES`. Registry test `registry_skeleton_is_internally_consistent` enforces family/resolution parity.
- `LINT` family has four codes: `SIFR-LINT-0001..4` (all Warning). Same complete registration.
- `diagnostic_label_for_code_str` correctly maps `SIFR-FMT-*` → `"format error"` and `SIFR-LINT-*` → `"lint warning"`. The test `test_diagnostic_labels_are_derived_from_diagnostic_codes` covers FMT and LINT cases.
- Generated docs (`docs/errors/SIFR-*.md`) are consistent with registry entries (matching codes, severities, owners, templates).

**Validation scripts**
- `check_formatter_contract.py` positive path: builds, runs `fmt --check` on unformatted source (expects exit 1 with `SIFR-FMT-0001`), runs `fmt` to fix, verifies string contents preserved, runs `fmt --check` again (expects exit 0), verifies idempotence. Correct.
- `check_formatter_contract.py` self-test: invalid syntax file correctly produces `SIFR-PARSE-*` error (not silently formatted). Correct.
- `check_rule_suppression_contract.py` positive path: unknown rule + trailing whitespace correctly reported. Explicit suppression correctly applied. Correct.
- `check_rule_suppression_contract.py` self-test: blanket suppression correctly produces `SIFR-LINT-0003`. Correct.

**Script wiring**
- `run_all_tests.sh` runs all m36.2 contract checks alongside m36.1 checks under the "Developer Tooling Checks" step. Both positive and self-test invocations are included. Correct.

### Residual Risks (Non-blocking)

1. **Formatter is intentionally limited**: The contract explicitly calls for "conservative, syntax-validated edits" at this milestone. Full formatting (indentation, alignment, import sorting) lands in m36.4/m36.5. No gap here.

2. **Lint has one rule**: `trailing-whitespace` is the only implemented policy rule. The infrastructure (rule metadata, suppression parsing, unknown/unused diagnostics) is complete and extensible. Future rules add to `RULES` and `lint_source`. No gap here.

3. **No LSP integration yet**: `textDocument/formatting` / `textDocument/rangeFormatting` handlers land in m36.5. The formatter is designed to integrate with the LSP session model but the bridge is not yet built. This is by design per milestone sequencing.

4. **No `sifr_analysis` integration yet**: The formatter/lint are standalone crates. They will be called through `AnalysisHost` in m36.3+. No split-brain risk since neither crate depends on HIR or type-check semantics.

### Milestone Contract Status

| Requirement | Status |
|---|---|
| Concrete `sifr_format` and `sifr_lint` workspace crates | Done |
| `sifr fmt [--check] <path>` CLI | Done |
| `sifr lint <path>` CLI | Done |
| Sifr-owned FMT and LINT diagnostic families + generated docs | Done |
| Conservative syntax-validated formatter foundation over `sifr_syntax` | Done |
| Policy metadata, explicit suppressions, unknown/unused/blanket suppression diagnostics | Done |
| `check_formatter_contract.py` and `check_rule_suppression_contract.py` with negative self-tests | Done |
| m36.2 checks wired into `scripts/run_all_tests.sh` | Done |
| Local validation (`scripts/run_all_tests.sh --profile quick`) | PASS |
| Diagnostic/code/docs/schema guardrails | PASS |

All DoD criteria are satisfied. The implementation is ready for PR.
