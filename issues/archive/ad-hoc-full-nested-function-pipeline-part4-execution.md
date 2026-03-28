# Part 4 Execution: Codegen, Diagnostics, and Unsupported-Shape Boundaries

Status: completed on 2026-03-15

- Demo: `demos/ad_hoc_nested_function_part4_demo.sifr`
- Pass fixture: `crates/sifr/tests/e2e/pass/nested_function_recursive_collection_backtracking.sifr`
- Fail fixture: `crates/sifr/tests/e2e/fail/nested_function_capture_mutates_immutable_param.sifr`
- PR: `#1145`

## Closure Basis

- Structured production codegen now lowers recursive nested helpers with real Rust local functions instead of panicking once the helper body leaves the simple-stmt path.
- Recursive capture lowering now carries deterministic capture order plus explicit borrow or mut-borrow conventions, so outer collection state is emitted as valid Rust references rather than owned placeholder values.
- Nested-helper usage now feeds outer empty-collection binding hints back into lowering, eliminating the supported-path `list[Any]` fallback that previously surfaced as `Vec<Box<dyn Any>>` codegen mismatches.
- Early-return sequence guards now cover method-style `.len()` checks and `or`-combined false exits, which unblocks supported recursive backtracking helpers such as `combination_sum`.
- Immutable-parameter capture mutation now stops at explicit diagnostics instead of collapsing into downstream `Any` or Rust borrow-check noise.

## Positive Evidence

- `cargo run -q -p sifr -- run demos/ad_hoc_nested_function_part4_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/nested_function_recursive_collection_backtracking.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0078_subsets.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0039_combination_sum.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0039_combination_sum.sifr`

## Unsupported-Boundary Evidence

- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/nested_function_capture_mutates_immutable_param.sifr`
  - `type error: cannot mutate through immutable parameter 'nums': add \`mut\` to the parameter declaration`
- `cargo run -q -p sifr -- check audits/leetcode/0912_sort_an_array.sifr`
  - explicit immutable-parameter capture mutation diagnostics remain the supported boundary for nested helpers that try to mutate an outer immutable parameter.

## Validation

- `cargo fmt --check`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
