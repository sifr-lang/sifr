# Phase 28 Execution Checklist (Decimal Types and Exact Numeric Semantics)

Status: in_progress (started 2026-03-07)
Owner: phase_28 execution loop
Reference phase docs:
- `.cursor/plans/main/phases/28_decimal_type_and_exact_numeric_semantics.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 28 To-Do Plan

### Part 1: milestone_28_1 Type-System, Parser, and HIR Integration
- [ ] Add `decimal` and `bigdecimal` as first-class types through parser -> HIR -> type checker -> codegen
- [ ] Add constructor typing/lowering for `Decimal(...)` and `BigDecimal(...)` with semantic validation
- [ ] Enforce mixed numeric policy (`float` with decimal types forbidden, `decimal` with `bigdecimal` forbidden without explicit conversion)
- [ ] Add demo: `demos/m28_1_type_system_parser_hir_integration_demo/main.sifr`
- [ ] Add negative cases for constructor and mixed arithmetic constraints
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_28_2 Deterministic Arithmetic and Context Semantics
- [ ] Implement arithmetic/comparison lowering for decimal and bigdecimal operators
- [ ] Implement required methods: `quantize`, `sqrt`, `round`, `abs`, `is_zero`, `is_finite`
- [ ] Add deterministic formatting contract coverage
- [ ] Add demo: `demos/m28_2_deterministic_arithmetic_and_context_demo/main.sifr`
- [ ] Add negative cases
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_28_3 Conversion and Boundary Contracts
- [ ] Implement explicit conversion contracts (`int`/`bigint`/`str`/cross-decimal)
- [ ] Enforce explicit ban on `float -> decimal|bigdecimal` paths
- [ ] Add conversion boundary tests and deterministic behavior checks
- [ ] Add demo: `demos/m28_3_conversion_and_boundary_contracts_demo/main.sifr`
- [ ] Add negative cases
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_28_4 Decimal Diagnostics Contract
- [ ] Reserve and enforce diagnostics `E2501-E2508`
- [ ] Add precise, stable decimal diagnostics for constructors/mixing/conversion/context issues
- [ ] Add regression locks for stable diagnostic content
- [ ] Add demo: `demos/m28_4_decimal_diagnostics_contract_demo/main.sifr`
- [ ] Add negative cases
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_28_5 Verification Corpus and Determinism Gates
- [ ] Expand pass/fail decimal corpus with deterministic coverage
- [ ] Add repeated-run determinism checks and negative seeded cases
- [ ] Add end-to-end milestone demo using both decimal types
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_28_1 Type-System, Parser, and HIR Integration
status: done (2026-03-07, PR pending)

- [x] Decimal/bigdecimal types added across type system and codegen mapping
- [x] Constructor validation and mixed-numeric policy enforced
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m28_1_type_system_parser_hir_integration_demo/main.sifr` -> prints `m28_1 type-system/parser/HIR integration demo`, `14.50`, `7.25`.
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/decimal_type_system_basic.sifr` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m28_1_type_system_parser_hir_integration_demo/negative_cases/forbidden_float_constructor/main.sifr` -> exits `1` with `[E2505]` decimal constructor diagnostic.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/decimal_float_mixed_arithmetic.sifr` -> exits `1` with `[E2503]`.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/decimal_bigdecimal_mixed_arithmetic.sifr` -> exits `1` with `[E2504]`.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/decimal_constructor_float.sifr` -> exits `1` with `[E2505]`.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/bigdecimal_constructor_non_literal_string.sifr` -> exits `1` with `[E2502]`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (404 pass e2e fixtures, 0 failures).

## PR Log
- Part 1: pending
- Part 2: pending
- Part 3: pending
- Part 4: pending
- Part 5: pending

## External Review Passes
- Reviewer pass 1 prompt output: pending
- Reviewer pass 1 remediation PR: pending
- Reviewer pass 2 prompt output: pending
- Reviewer pass 2 remediation PR: pending
