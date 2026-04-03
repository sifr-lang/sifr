# Ad Hoc Optional/None Closure: Wave-R3e Check-Residual Canonicalization Plan (2026-03-30)

## Scope

Residual check-stage fixtures after wave-R3d:

- `0054_spiral_matrix`
- `0763_partition_labels`

## Reviewer-Gated Decision

Use canonical fixture rewrites for this wave instead of compiler broadening.

Rationale:

- `0054` depends on matrix row-shape assumptions that are not explicitly modeled in the current Sifr type surface.
- `0763` reads from a map across disjoint loops where key-presence is semantically true for LeetCode constraints, but not currently encoded as a general sound flow fact in HIR.
- Forcing compiler acceptance here risks unsound Optional narrowing and violating Sifr safety principles.

Reviewer artifact:

- `reviews/ad-hoc-optional-none-wave-r3e-review-pass1.md`

## Planned Changes

1. `0763_partition_labels`:
- canonicalize map read to explicit total form (`dict.get(key, default)`), avoiding Optional dict-index leakage in `max(...)`.

2. `0054_spiral_matrix`:
- canonicalize matrix row extraction into explicit Optional-safe local bindings before slicing/iteration.
- avoid direct nested indexed append paths that leak `int | None` into result accumulation.

## Success Criteria

- `cargo run -q -p sifr -- check audits/leetcode/0054_spiral_matrix.sifr` passes.
- `cargo run -q -p sifr -- run audits/leetcode/0054_spiral_matrix.sifr` passes.
- `cargo run -q -p sifr -- check audits/leetcode/0763_partition_labels.sifr` passes.
- `cargo run -q -p sifr -- run audits/leetcode/0763_partition_labels.sifr` passes.
- `scripts/run_all_tests.sh --profile quick` passes.
