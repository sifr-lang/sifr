# Optional/None Closure Follow-up: Wave-9d Residual Canonicalization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

After wave-9c, a residual set still fails on `int | None` arithmetic/subtraction where accumulation logic depends on indexed container reads and helper results (`max(...)`, dict/list lookups) without explicit local proofs.

## Scope (batch-4)

- `0300_longest_increasing_subsequence`
- `0525_contiguous_array`
- `0554_brick_wall`
- `1343_number_of_sub_arrays_of_size_k_and_average_greater_than_or_equal_to_threshold`

## Root Cause

- DP/sliding-window accumulation code still relies on direct indexed reads and Optional-returning helper paths in arithmetic
- fixtures assume non-empty and present-key/index properties that are not encoded in explicit Sifr-safe forms

## Canonicalization Strategy

- preserve expected outputs and asymptotic complexity class per fixture
- rewrite to iterator-first and explicit accumulator/dict-default forms that avoid Optional arithmetic
- keep this wave fixture-only; no compiler rule changes
- keep scope strictly limited to listed fixtures

## Validation Matrix

- targeted `cargo run -q -p sifr -- check` and `-- run` for all scoped fixtures
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun artifact and status delta vs wave-9c
