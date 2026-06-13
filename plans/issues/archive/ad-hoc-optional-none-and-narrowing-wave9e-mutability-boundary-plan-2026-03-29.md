# Optional/None Closure Follow-up: Wave-9e Mutability Boundary Canonicalization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

After wave-9d, a high-frequency residual cluster still fails because fixtures mutate list parameters without explicit `mut` boundary declarations, and one fixture also relies on unsupported tuple-swap assignment syntax.

## Scope (batch-5)

- `0026_remove_duplicates_from_sorted_array`
- `0027_remove_element`
- `0080_remove_duplicates_from_sorted_array_ii`
- `0448_find_all_numbers_disappeared_in_an_array`

## Root Cause

- mutating parameter-backed containers without `mut` declarations violates Sifr ownership/mutability contracts
- fixture-level syntax still includes tuple swap assignment unsupported in this lowering lane

## Canonicalization Strategy

- preserve algorithm behavior and in-place semantics
- make mutability boundaries explicit at function signatures (`mut nums: list[int]`)
- rewrite tuple swap shape into explicit temporary assignment flow
- keep this wave fixture-only; no compiler semantics changes

## Validation Matrix

- targeted `cargo run -q -p sifr -- check` and `-- run` for all scoped fixtures
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun artifact and status delta vs wave-9d
