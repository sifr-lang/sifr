# Optional/None Closure Follow-up: Wave-9c Residual Canonicalization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

After wave-9b, a residual DP/index cluster still fails with `int | None` arithmetic and return mismatches because fixture logic depends on implicit index safety and non-empty seeds.

## Scope (batch-3)

- `0063_unique_paths_ii`
- `0119_pascal_triangle_ii`
- `0120_triangle`
- `0135_candy`

## Root Cause

- indexed DP reads (`dp[i]`, `dp[i+1]`, `list[0]`) flow into arithmetic without explicit local non-`None` proof in these fixture shapes
- fixtures assume problem constraints imply non-empty/index-safe access, but those assumptions are not encoded in a Sifr-explicit form

## Canonicalization Strategy

- preserve algorithmic outputs and complexity class per fixture
- replace Optional-contaminated indexed arithmetic with iterator-first or explicit accumulator forms
- keep all optional boundaries explicit; no compiler-rule changes in this wave
- keep rewrites local to fixtures in scope

## Validation Matrix

- targeted `cargo run -q -p sifr -- check` and `-- run` for all scoped fixtures
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun artifact and status delta vs wave-9b
