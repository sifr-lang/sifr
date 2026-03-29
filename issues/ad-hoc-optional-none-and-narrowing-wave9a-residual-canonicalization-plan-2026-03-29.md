# Optional/None Closure Follow-up: Wave-9a Residual Canonicalization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

After wave-8b, unresolved Optional failures still include fixtures that depend on unstated non-empty/index-safety preconditions from external problem constraints.

These fixtures are valid algorithmically but not explicit enough for Sifr’s static no-panic model.

## Root Cause

The fixture bodies encode implicit assumptions such as “input list is non-empty” or “index access is safe by problem contract” without proving them through Sifr-safe control flow.

## Scope (batch-1)

- `0062_unique_paths`
- `0121_best_time_to_buy_and_sell_stock`
- `0377_combination_sum_iv`
- `0540_single_element_in_a_sorted_array`

## Canonicalization Strategy

- preserve the algorithmic intent and output contract
- rewrite to explicit Sifr-safe flow:
  - avoid Optional-returning direct index reads without local proof
  - use iteration-driven formulations where practical
  - use explicit total-return paths
- no compiler-rule weakening, no hidden unwrap behavior

## Validation Matrix

- targeted `cargo run -q -p sifr -- check` and `-- run` for all scoped fixtures
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun artifact and status delta vs wave-8b
