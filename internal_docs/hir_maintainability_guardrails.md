# Lowering Maintainability Guardrails

This document defines anti-regrowth guardrails for the producer-side lowering decomposition work.

## File Boundaries

The canonical lowering and intrinsic-registry layout is:

- `crates/sifr_lowering/src/lower/mod.rs`
- `crates/sifr_lowering/src/lower/imports.rs`
- `crates/sifr_lowering/src/lower/diagnostics.rs`
- `crates/sifr_lowering/src/lower/classes.rs`
- `crates/sifr_lowering/src/lower/typing_and_functions.rs`
- `crates/sifr_lowering/src/lower/statements.rs`
- `crates/sifr_lowering/src/lower/expressions.rs`
- `crates/sifr_lowering/src/stdlib/mod.rs`
- `crates/sifr_lowering/src/stdlib/io_json.rs`
- `crates/sifr_lowering/src/stdlib/math_test.rs`
- `crates/sifr_lowering/src/stdlib/collections_bytes_time.rs`
- `crates/sifr_lowering/src/stdlib/sys_fs.rs`
- `crates/sifr_lowering/src/stdlib/crypto_regex_uuid.rs`
- `crates/sifr_lowering/src/stdlib/platform_misc.rs`

Monolithic files are explicitly banned:

- `crates/sifr_lowering/src/lower.rs`
- `crates/sifr_lowering/src/stdlib.rs`

Guardrail enforcement command:

- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`

`run_all_tests.sh` runs this check before unit/e2e validation.
The CI workflow `.github/workflows/local-first-validation.yml` runs `run_all_tests.sh`, so the guardrail is enforced in CI as well.

## Review Checklist

Use this checklist for every PR that changes lowering or intrinsic registry logic.

- [ ] Lowering logic is placed in the correct file (imports/diagnostics/classes/typing_and_functions/statements/expressions).
- [ ] Shared lowering helper extraction was considered before adding duplicate logic.
- [ ] Unified file-size guardrail passes locally (`python3 scripts/check_file_size_guardrails.py`).
- [ ] New lowering behavior includes at least one positive-path and one negative-path validation update.
- [ ] Guardrail script still passes locally (`python3 scripts/check_hir_maintainability_guardrails.py`).
