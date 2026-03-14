# Ad Hoc `own mut` Parameter Convention: Part 2 Execution

Status: complete locally
Started: 2026-03-14
Completed: 2026-03-14
Part: `borrow_checking_and_escape_semantics_for_four_parameter_modes`
PR: `#1132`

## Goal

Close the semantic gap left after part 1 by making borrow escape analysis and parameter mutability rules derive from the orthogonal ownership/mutability model instead of the old borrowed-by-default shortcuts.

This slice is intentionally limited to:

- preventing borrowed parameters, including `mut` borrowed parameters, from escaping by return or local storage,
- preventing immutable parameters from being reassigned or mutated through,
- proving `own mut` parameters can mutate and return successfully,
- and aligning the bundled stdlib and regression corpus with the stricter parameter-mutation contract.

It does not attempt to finish:

- canonical Rust parameter emission for `own mut`,
- or the final `1299` runtime closure wave.

## Root Cause

After part 1, the compiler could represent `own mut`, but the semantic checks still behaved as if parameter mutability and escape rules were mostly implicit:

- escape analysis only treated shared borrows as non-escaping,
- mutable borrowed parameters could still escape through `return` and local storage,
- and immutable parameters could still be reassigned or mutated because parameter binding mutability was not tracked structurally.

The fix for this slice was to make parameter bindings carry their binding kind plus mutability, and to derive both escape and mutation checks from those facts.

## Implementation

- Extended `Scope::VarInfo` with binding mutability and binding kind so parameter bindings are tracked structurally in the HIR lowering scope.
- Added dedicated parameter binding definition in scope construction so function parameters preserve `mut` vs immutable binding semantics.
- Tightened borrowed-parameter escape analysis so both shared and mutable borrowed move-type parameters are rejected for return/store escapes.
- Added immutable-parameter diagnostics for direct reassignment, augmented assignment, field assignment, and subscript-based mutation.
- Updated bundled stdlib sources in `lib/sifr/heapq.sifr` and `lib/sifr/fnmatch.sifr` to declare the parameters they reassign as `mut`.
- Updated existing fail-fixture expectations and the Phase 31 pass fixture to the new canonical borrow-escape/mutability semantics.
- Added dedicated HIR semantics tests, new pass/fail e2e fixtures, and the part 2 demo.

## Validation

Targeted validation:

- `cargo test -p sifr_hir own_mut_semantics_tests`
- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part2_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/own_mut_parameter_semantics.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/borrowed_mut_parameter_return_escape.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/own_parameter_mutation_requires_mut.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

Demo output:

- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part2_demo.sifr` -> prints `9`, `10`, `3`

## Coverage Added

- `crates/sifr_hir/src/lower/own_mut_semantics_tests.rs`
- `crates/sifr/tests/e2e/pass/own_mut_parameter_semantics.sifr`
- `crates/sifr/tests/e2e/fail/borrowed_mut_parameter_return_escape.sifr`
- `crates/sifr/tests/e2e/fail/own_parameter_mutation_requires_mut.sifr`
- `demos/ad_hoc_own_mut_parameter_convention_part2_demo.sifr`

## Closure Decision

Part 2 is locally complete because the compiler now enforces the four parameter modes consistently at the semantic layer: immutable parameters cannot be mutated or reassigned, borrowed parameters cannot escape by value, and `own mut` parameters can both mutate and return successfully.

The remaining work is intentionally deferred:

- part 3: emit canonical Rust `mut x: T` for owned mutable parameters and close the runtime/`1299` surface,
- part 4: phase closure docs and external review cycles.
