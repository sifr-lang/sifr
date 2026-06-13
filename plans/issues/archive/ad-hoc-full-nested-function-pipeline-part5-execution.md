# Part 5 Execution: Regression Corpus, Demos, and Full-Corpus Closure Evidence

Status: completed on 2026-03-15

- Demo: `demos/ad_hoc_nested_function_part5_demo.sifr`
- Pass fixture: `crates/sifr/tests/e2e/pass/nested_function_recursive_subsets_enumeration.sifr`
- Fail fixtures:
  - `crates/sifr/tests/e2e/fail/nested_function_capture_mutates_immutable_param.sifr`
  - `crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr`
- PR: `#1146`

## Closure Basis

- Added a phase-owned closure demo that exercises the supported nested-helper surface end to end: forward local callables, recursive local helpers, recursive captured collections, and supported non-recursive `nonlocal` rebinding.
- Added a permanent subsets/backtracking pass regression so the nested-helper corpus is locked with compiler-owned evidence instead of relying on the incorrect assertion order in `audits/leetcode/0078_subsets.sifr`.
- Re-ran the watched nested-helper audit set and recorded which outcomes are now supported passes, which are explicit ownership or mutability boundaries, and which remaining failures are outside the nested-function root cause closed by this phase.
- The phase-owned part 5 demo is now the authoritative closure artifact for the nested-function pipeline; the legacy `demos/milestone_nested_functions_demo.sifr` still runs, but it is no longer the only runnable milestone evidence.

## Watched Corpus Closure Evidence

Supported nested-helper cases:

- `cargo run -q -p sifr -- check audits/leetcode/0039_combination_sum.sifr` -> pass
- `cargo run -q -p sifr -- run audits/leetcode/0039_combination_sum.sifr` -> pass
- `cargo run -q -p sifr -- check audits/leetcode/0078_subsets.sifr` -> pass
- `cargo run -q -p sifr -- run demos/ad_hoc_nested_function_part5_demo.sifr` -> pass

Explicit language-boundary cases:

- `cargo run -q -p sifr -- check audits/leetcode/0090_subsets_ii.sifr`
  - `cannot mutate through immutable parameter \`nums\``
  - `cannot reassign immutable parameter \`i\``
- `cargo run -q -p sifr -- check audits/leetcode/0912_sort_an_array.sifr`
  - explicit immutable-parameter mutation diagnostics remain the supported ownership boundary for nested helpers mutating an outer immutable parameter

Residual non-nested blockers in the watched set:

- `cargo run -q -p sifr -- check audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr`
  - still fails on dict/string indexing and `Any` fallout unrelated to the nested-helper pipeline
- `cargo run -q -p sifr -- check audits/leetcode/0050_powx_n.sifr`
  - still fails on float/int comparison semantics unrelated to the nested-helper pipeline

## Validation

Targeted validation:

- `cargo run -q -p sifr -- run demos/ad_hoc_nested_function_part5_demo.sifr`
- `cargo run -q -p sifr -- run demos/milestone_nested_functions_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/nested_function_recursive_subsets_enumeration.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/nested_function_recursive_collection_backtracking.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/nested_function_capture_mutates_immutable_param.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr`
- watched audit reruns recorded above

Authoritative local gates:

- `cargo fmt --check`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
