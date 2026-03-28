# Ad Hoc Nested Function Pipeline: Part 2 Execution

Status: complete
Started: 2026-03-14
Completed: 2026-03-14
Part: `milestone_nested_2`
PR: `#1141`

## Goal

Close the usage-driven inference gap for supported nested helpers by inferring nested parameter and return types from enclosing call sites and recursive body usage instead of requiring explicit annotations for every local helper.

This slice is intentionally limited to:

- supported nested parameter inference from direct calls and recursive calls,
- recursive local-helper return inference for deterministic numeric/helper patterns,
- explicit failure for conflicting call-site inference,
- and regression coverage for the supported recursive-helper surface.

It does not attempt to close:

- captured mutable-state semantics,
- `nonlocal`-style updates,
- codegen closure-shape cleanup for broader backtracking/capture families,
- or unrelated downstream gaps such as optional indexing or immutable-parameter mutation.

## Root Cause

After part 1, nested helpers were registered early enough to resolve, but any missing nested parameter annotations still collapsed to `Any`. Real corpus helpers therefore failed before the compiler could use the surrounding call graph or recursive body structure to discover stable local signatures.

The fix for this slice is a dedicated nested-function inference pass over the enclosing block:

- collect the local helper set,
- iterate call-site and body-usage constraints to a fixed point,
- finalize supported inferred signatures,
- and reject conflicting inference instead of silently degrading to `Any`.

## Implementation

- Added `crates/sifr_hir/src/lower/nested_function_inference.rs` to own nested-helper signature inference.
- Wired the inference pass into block-entry nested-function predeclaration before normal lowering.
- Added support for:
  - direct call-site parameter inference,
  - recursive call propagation,
  - integer/collection usage refinement inside nested bodies,
  - and recursive return-type inference for supported numeric helpers.
- Added deterministic conflicting-call-site rejection in HIR lowering tests.
- Added runnable recursive-inference fixtures:
  - `crates/sifr/tests/e2e/pass/nested_function_inference_recursive_int.sifr`
  - `crates/sifr/tests/e2e/pass/nested_function_inference_recursive_capture.sifr`
- Added `demos/ad_hoc_nested_function_part2_demo.sifr` for the milestone-owned runnable demo.

## Validation

Targeted validation:

- `cargo test -p sifr_hir nested_function_tests:: -- --nocapture`
- `target/debug/sifr run demos/ad_hoc_nested_function_part2_demo.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/nested_function_inference_recursive_int.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/nested_function_inference_recursive_capture.sifr`
- `target/debug/sifr check audits/leetcode/0078_subsets.sifr`
- `target/debug/sifr check audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr`
- `target/debug/sifr check audits/leetcode/0039_combination_sum.sifr`
- `target/debug/sifr check audits/leetcode/0050_powx_n.sifr`
- `target/debug/sifr check audits/leetcode/0090_subsets_ii.sifr`
- `target/debug/sifr check audits/leetcode/0912_sort_an_array.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Reclassification Results

- `0078_subsets` now checks cleanly.
- `0017_letter_combinations_of_a_phone_number` moved past missing nested annotations into downstream optional-indexing fallout.
- `0039_combination_sum` moved past missing index/accumulator annotations into downstream optional-indexing fallout.
- `0050_powx_n` moved past nested-annotation failures into an unrelated `float`/integer-literal comparison rule.
- `0090_subsets_ii` moved past nested index-parameter inference into remaining immutable-parameter / slicing fallout.
- `0912_sort_an_array` moved past nested annotation fallout into existing immutable-parameter mutation diagnostics.

## Closure Decision

Part 2 is complete because supported recursive nested helpers now infer deterministically, conflicting local-helper inference fails explicitly, and the watched corpus has moved off the original missing-annotation / `Any` root cause that this slice owned.

Remaining work is intentionally deferred:

- part 3: capture typing and `nonlocal`-style mutation,
- part 4: codegen and unsupported-shape boundaries,
- part 5: regression/corpus/demo closure.
