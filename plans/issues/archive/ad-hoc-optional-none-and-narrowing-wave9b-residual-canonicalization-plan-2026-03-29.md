# Optional/None Closure Follow-up: Wave-9b Residual Canonicalization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

After wave-9a, several stock/array fixtures still rely on direct indexed reads from possibly-empty inputs, producing Optional arithmetic/assignment diagnostics under Sifr’s explicit safety model.

## Scope (batch-2)

- `0122_best_time_to_buy_and_sell_stock_ii`
- `0152_maximum_product_subarray`
- `0169_majority_element`
- `1800_maximum_ascending_subarray_sum`

## Canonicalization Strategy

- preserve algorithmic behavior and outputs
- replace index-heavy scans with iterator-first or explicit first-element seeding patterns
- keep total-return behavior explicit
- avoid compiler semantics changes; this wave is fixture-surface normalization only

## Validation Matrix

- targeted `cargo run -q -p sifr -- check` and `-- run` for all scoped fixtures
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun artifact and status delta vs wave-9a
