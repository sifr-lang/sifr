

All checks pass. Here is my structured review.

---

## Review: `codex/diag-11-guardrail-audit`

### 1. New guardrail scripts

**`check_diagnostic_code_coverage.py`** — Validates that every active `DiagnosticCode` constant has at least one non-test compiler source that uses it and a valid representative fixture file. The `strip_cfg_test_blocks` helper is correct (strips `#[cfg(test)]` blocks including nested content). The fixture-existence check is file-level only (splitting on `::` for `path::fn` test-function fixtures). No false-positive surface area detected.

**`check_diagnostic_baseline_hygiene.py`** — Scans all e2e fixtures, verification baselines, and docs for legacy `[Edddd]` pseudo-codes and forbidden `SIFR-TYPE-0001` catch-all usage. The regex `EXPECT_ERROR_RE` correctly requires full-line `expect-error` annotations with a canonical code. Clean and focused.

**`check_diagnostic_cancel_usage.py`** — Restricts `.cancel()` to `crates/sifr_diagnostics/src/model/mod.rs` only, which is the correct design: cancellation is an implementation detail of the diagnostic model, not something callers should invoke. The `strip_cfg_test_blocks` logic is shared with the coverage script and correct here too. All four existing `.cancel()` calls in the model file are compliant.

**Wiring** — All three scripts are inserted into `run_all_tests.sh` between the docs/schema sync checks and the verification hardening self-test, in appropriate alphabetical order. Correct placement.

### 2. Stale active code deletion: `SIFR-STDLIB-0002` and `SIFR-CODEGEN-0002`

**Evidence for "no non-test emit path":**

- `grep` across all `*.rs` compiler sources confirms zero hits for `STDLIB_ARGUMENT_TYPE_MISMATCH` or `CODEGEN_BACKEND_FAILURE` in non-test code.
- The review history confirms both codes were planned-anchor placeholders: `SIFR-CODEGEN-0002` was the non-panic codegen failure code, with `SIFR-INTERNAL-0001` as the panic-boundary counterpart; `SIFR-STDLIB-0002` was the stdlib-argument-type-mismatch code with no active lowering path.
- The `stdlib_wrong_type.sifr` fixture exists on disk but has no `expect-error` directive and no baseline referencing it, confirming it was never wired into a live error path.
- `INTERNAL-0001` and `INTERNAL-0002` are the correct pre-1.0 homes for panic-boundary and recovery-cap notes.

**Deletion is correct pre-1.0.** Stale codes that have never been emitted are dead weight in the registry, the docs index, and the `ACTIVE_DIAGNOSTIC_CODES` slice. Removing them before 1.0 is the right call.

### 3. Affected tests — `diagnostics.rs` and `main.rs`

The two removals from `test_diagnostic_labels_are_derived_from_diagnostic_codes` (`STDLIB_ARGUMENT_TYPE_MISMATCH` and `CODEGEN_BACKEND_FAILURE`) are correct: neither was ever a real emitted code, so neither had a meaningful label to test. The mechanical substitution in `test_compiler_diagnostics_preserve_order` (`CODEGEN_BACKEND_FAILURE` → `BUILD_MATERIALIZATION_FAILURE`) is also correct — the test is about ordering and formatting, not code identity, and `BUILD_MATERIALIZATION_FAILURE` is a genuine active code.

The `main.rs` test substitution (`SIFR-CODEGEN-0002` → `SIFR-INTERNAL-0002`) is consistent with the renamed constant. `SIFR-INTERNAL-0002` is a `Note`-severity code with message `consider adding a type annotation`, matching the test intent exactly.

### 4. Fixture repointing for `SIFR-TYPE-0901`, `SIFR-TYPE-0902`, `SIFR-FLOW-0901`

The e2e `.sifr` fixtures (e.g., `arithmetic_overflow_warning.sifr`, `reveal_type.sifr`, `unreachable_statement_warning.sifr`) have no baselines referencing them — confirmed by grepping all `.stderr`/`.stdout` files. Repointing the representative fixtures to unit-test functions in `single_file_frontend.rs` is appropriate: these diagnostics are surfaced as structured notes/warnings, and the unit tests provide a more stable and verifiable anchor than end-to-end fixture files.

### 5. Validation summary

| Check | Result |
|---|---|---|
| `check_diagnostic_code_coverage.py` | pass |
| `check_diagnostic_baseline_hygiene.py` | pass |
| `check_diagnostic_cancel_usage.py` | pass |
| `cargo test -p sifr_diagnostics` | pass |
| `cargo test -p sifr_driver tests::diagnostics` | pass (12/12) |
| `cargo test -p sifr test_compact_renderer_snapshot_multi_severity_group_order` | pass |
| `cargo clippy -p sifr_diagnostics -p sifr_driver -p sifr` | pass (zero warnings) |

### 6. Blocking findings

**None.** No false positives in the guardrail scripts, no false negatives (codes that should be flagged are not), no missing test coverage, and the deletion of the two stale active codes is correct pre-1.0.

**Reviewer satisfied.**
