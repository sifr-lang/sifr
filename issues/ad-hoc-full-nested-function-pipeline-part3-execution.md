# Ad Hoc Nested Function Pipeline: Part 3 Execution

Status: complete
Started: 2026-03-14
Completed: 2026-03-14
Part: `milestone_nested_3`
PR: `#1143`

## Goal

Close the capture-typing and explicit captured-state-update gap for nested helpers by supporting deterministic typed captures plus supported `nonlocal`-style rebinding patterns, while turning unsupported capture-mutation shapes into explicit diagnostics instead of parser/lowering gaps or production-path panics.

This slice is intentionally limited to:

- preserving typed captured locals inside nested helper bodies,
- supporting non-recursive `nonlocal` rebinding for supported local helpers,
- marking captured mutation so supported non-recursive helpers lower to valid mutable Rust bindings,
- and rejecting unsupported recursive / tuple-unpack captured-state updates explicitly.

It does not attempt to close:

- recursive captured-state lowering/codegen,
- broad nested-function production-path cleanup,
- or corpus cases whose remaining blockers are outside nested capture semantics.

## Root Cause

After part 2, nested helper reads could infer and type-check, but captured-state updates still fell through partial systems:

- `nonlocal` parsed but did not lower,
- simple assignment/augmented assignment still used lexical lookup rules that could silently mutate outer bindings,
- enclosing mutable-state tracking did not account for capture writes inside nested helpers,
- and unsupported recursive/nonlocal shapes still surfaced as generic unsupported statements or downstream production-path failures.

The fix for this slice establishes one explicit capture-update boundary:

- collect and validate `nonlocal` declarations at function entry,
- distinguish current-function locals from enclosing-function bindings,
- lower supported non-recursive capture rebinding through stable typed bindings,
- and reject unsupported recursive / tuple-unpack nonlocal updates explicitly before codegen.

## Implementation

- Added function-scope metadata for nested lowering in `crates/sifr_hir/src/lower/function_scopes.rs`.
- Added dedicated nonlocal/capture helpers in `crates/sifr_hir/src/lower/nonlocal_support.rs`.
- Split tuple-unpack assignment lowering into `crates/sifr_hir/src/lower/tuple_unpack.rs` to stay within HIR maintainability guardrails.
- Implemented `Stmt::Nonlocal` validation and deterministic enclosing-scope binding resolution.
- Tightened assignment semantics so function-local rebinding only reaches enclosing state when explicitly declared `nonlocal`.
- Added explicit diagnostics for:
  - recursive nested helpers mutating captured state with `nonlocal`,
  - tuple unpacking that tries to rebind captured state with `nonlocal`,
  - and augmented assignment to captured variables without `nonlocal`.
- Extended codegen mutation analysis so enclosing locals mutated by nested helpers are emitted as mutable, and supported non-recursive nested helpers lower as mutable closure bindings when needed.
- Added milestone-owned runnable/fail fixtures:
  - `crates/sifr/tests/e2e/pass/nested_function_nonlocal_accumulator.sifr`
  - `crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr`
- Added `demos/ad_hoc_nested_function_part3_demo.sifr` for the supported runnable milestone demo.

## Validation

Targeted validation:

- `cargo test -p sifr_hir test_nonlocal_nested_helper_rebinds_enclosing_name -- --nocapture`
- `cargo run -q -p sifr -- run demos/ad_hoc_nested_function_part3_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/nested_function_nonlocal_accumulator.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0052_n_queens_ii.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0673_number_of_longest_increasing_subsequence.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Reclassification Results

- `0052_n_queens_ii` no longer fails on an unsupported parser/lowering path; it now fails explicitly as `recursive nested function 'backtrack' cannot mutate captured state with \`nonlocal\` yet`.
- `0673_number_of_longest_increasing_subsequence` no longer degrades through tuple/nonlocal ambiguity; it now fails explicitly as `tuple unpacking cannot rebind captured state with \`nonlocal\` yet`.
- Supported non-recursive `nonlocal` helpers now run end to end without falling back to unresolved outer rebinding or invalid immutable Rust bindings.

## Closure Decision

Part 3 is complete because typed captures and supported non-recursive `nonlocal` updates now lower and run deterministically, while unsupported recursive / tuple-unpack captured-state updates fail explicitly before codegen instead of falling through to generic unsupported-statement errors or production-path panics.

Remaining work is intentionally deferred:

- part 4: full nested-helper production-path codegen and unsupported-shape boundaries,
- part 5: regression/corpus/demo closure.
