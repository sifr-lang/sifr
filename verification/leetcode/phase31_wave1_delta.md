# Phase 31 Remediation Wave 1 Delta

- Baseline seed status counts (2026-03-11): `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
- Wave 1 seed status counts (2026-03-11): `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
- Net change:
  - `PASS`: `+3`
  - `CHECK_ERROR`: `-1`
  - `RUN_ERROR`: `-2`

## Fixed Seed Cases

- `0069_sqrtx`: `RUN_ERROR -> PASS`
  - Root cause: tuple destructuring emitted immutable Rust bindings for locals that are later reassigned.
  - Fix: tuple destructuring patterns now emit `mut` for bindings detected in the mutated-var set.
- `0151_reverse_words_in_a_string`: `RUN_ERROR -> PASS`
  - Root cause: reassigned borrowed string parameters were emitted as direct writes to `&String` parameters.
  - Fix: mutated borrowed parameters are shadowed into owned mutable locals before body lowering.
- `2235_add_two_integers`: `CHECK_ERROR -> PASS`
  - Root cause: builtin call lowering claimed `sum(...)` before normal symbol resolution could see a user-defined function named `sum`.
  - Fix: builtin call lowering now yields to local/function bindings before applying builtin-specific lowering rules.

## Regression Coverage

- Unit:
  - `cargo test -p sifr_codegen lowers_simple_tuple_unpack_stmt_with_mutated_bindings -- --nocapture`
  - `cargo test -p sifr_hir test_user_defined_sum_shadows_builtin`
- E2E pass fixtures:
  - `crates/sifr/tests/e2e/pass/tuple_unpack_reassignment.sifr`
  - `crates/sifr/tests/e2e/pass/borrowed_param_shadowing.sifr`
  - `crates/sifr/tests/e2e/pass/function_shadowing_builtin_sum.sifr`
- E2E fail fixture:
  - `crates/sifr/tests/e2e/fail/builtin_sum_wrong_arity.sifr`

## Runner Determinism Check

- Command repeated twice:
  - `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/phase31_seed_corpus.json --output verification/leetcode/phase31_seed_results_wave1.json`
- Result:
  - normalized outputs are identical after excluding per-stage `duration_ms` fields and the run-specific temp/output paths
