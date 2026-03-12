# Phase 31 Follow-up: `m31_c_stdlib_module_parity` Slice 4

Status: complete
Started: 2026-03-12
Current slice: `m31_c_slice_4_private_heapq_max_compat`
PR: `#1112`

## Goal

Remove the remaining `heapq` module-surface blocker inside `stdlib.python_module_surface` by supporting the private max-heap helpers that current audit programs call:

- `heapq._heapify_max(...)`
- `heapq._heappop_max(...)`
- `heapq._heapreplace_max(...)`

This slice is scoped to the intentional CPython-compat surface needed by the corpus. It does not attempt to solve the deeper `Any`/annotation issues that remain once those helpers resolve.

## Targeted Cases

Primary seeded case:

- `1046` `last_stone_weight`

Broader parity probe:

- `2971` `find_polygon_with_the_largest_perimeter`

## Root-cause Hypothesis

- `sifr.heapq` implemented only the public min-heap helpers, so private max-heap audit calls had no backing stdlib implementation.
- Even if the helpers were added to `lib/sifr/heapq.sifr`, the stdlib export/signature pipeline stripped all leading-underscore callables, so compat import resolution would still fail.
- The real fix therefore needs both the stdlib algorithms and a narrow export policy that exposes only the intentional private compat surface instead of every underscored stdlib function.

## Implemented Root-cause Fixes

- Added pure-Sifr max-heap helpers to `sifr.heapq`:
  - `_sift_down_max`
  - `_sift_up_max`
  - `_heapify_max`
  - `_heappop_max`
  - `_heapreplace_max`
- Marked `_heapreplace_max`'s replacement item as `own` so storing it back into the heap matches Sifr's ownership/codegen rules.
- Added a shared export-policy helper in `sifr_driver` that allowlists only the intentional private `sifr.heapq` max-heap surface.
- Wired that allowlist through:
  - project export collection,
  - stdlib function/default export collection,
  - stdlib signature export collection used by codegen/compat imports.

## Regression Coverage

- `crates/sifr/tests/e2e/pass/phase31_heapq_max_private_compat.sifr`
- `crates/sifr_driver/src/tests/stdlib_exports.rs`
- Demo:
  - `demos/phase31_heapq_max_compat_demo.sifr`

## Validation Evidence

- `cargo build -q -p sifr`
- `cargo test -q -p sifr_driver stdlib_heapq_exports_allowlisted_private_max_heap_helpers`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_heapq_max_private_compat.sifr`
- `target/debug/sifr run demos/phase31_heapq_max_compat_demo.sifr`
- `target/debug/sifr check audits/leetcode/1046_last_stone_weight.sifr`
- `target/debug/sifr check audits/leetcode/2971_find_polygon_with_the_largest_perimeter.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31c_wave4_results.json --case 0003 --case 0007 --case 0127 --case 0217 --case 0502 --case 1046`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Artifacts:

- `verification/leetcode/phase31_m31c_wave4_results.json`

Targeted six-case status after this slice:

- `PASS=2`
- `CHECK_ERROR=3`
- `RUN_ERROR=1`

Observed movement:

- `1046` remains `CHECK_ERROR`, but the prior `undefined variable: 'heapq'` failure is gone. The remaining diagnostics are deeper type-flow issues from the unannotated duplicate solution:
  - `parameter 'stones' in function 'lastStoneWeight' is missing a type annotation`
  - `argument 1 ('data') of function '__compat_sifr_heapq__heapify_max': expected 'list[T]', got 'Any'`
  - `abs() argument must be numeric, got 'Any'`
- `2971` now resolves the private `heapq` max-heap helpers and is blocked only by a deeper optional arithmetic diagnostic:
  - `unsupported operand type(s) for *: 'int | None' and 'int'`

## Remaining Slice-local Follow-on

- `1046` has moved fully out of stdlib-surface failure and into Phase 32-style annotation / `Any` cleanup.
- `2971` is likewise blocked by deeper typing, not module parity.
- The remaining seeded failures in `m31_c` are now:
  - `0003` downstream codegen panic
  - `0127`, `0502`, `1046` downstream type/lowering issues

## Definition Of Done

- `heapq._heapify_max`, `heapq._heappop_max`, and `heapq._heapreplace_max` resolve through the stdlib compat path.
- The private-API export policy stays narrow and intentional instead of exposing every underscored stdlib helper.
- The seeded `1046` case moves past undefined-symbol failure and is reclassified to deeper non-stdlib work.
- A direct parity probe proves the max-heap helpers behave correctly end to end.
