# Ad Hoc Recursive Type Feature: Part 2 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_well_formedness_validation`
PR: `#1123`

## Goal

Define and enforce the first explicit well-formedness rule for recursive type aliases:

- allow recursive alias SCCs only when every recursive cycle crosses a valid indirection boundary,
- reject naked recursive alias cycles deterministically,
- and keep accepted recursive aliases available for later type-representation and codegen work.

This slice is intentionally alias-focused. It does not attempt to close recursive class boxing or full recursive-type runtime behavior.

## Root Cause

After part 1, recursive alias names resolved deterministically, but the compiler still accepted all recursive alias SCCs equally. That meant infinite-size aliases such as `type Bad = Bad` were indistinguishable from well-founded container recursion such as `type Json = list[Json] | dict[str, Json] | None`.

The missing rule was structural: a recursive SCC is only well-formed when every recursive cycle crosses an indirection boundary.

## Implementation

- Extended alias dependency analysis to record whether each alias-to-alias edge crosses a valid boundary.
- Defined valid alias boundaries for this slice as heap-owning container forms:
  - `list[...]`
  - `dict[..., ...]`
  - `set[...]`
- Added recursive SCC validation that rejects only cycles that can loop entirely through non-boundary edges.
- Kept order-independent alias resolution from part 1 while downgrading invalid recursive aliases to `Unknown` after emitting the deterministic diagnostic, preventing follow-on crashes or accidental acceptance.
- Added regression coverage for:
  - accepted container-recursive aliases,
  - accepted mixed forward/mutual alias cycles with a container boundary,
  - rejected naked recursion,
  - rejected recursive generic tuple aliases.

## Validation

Targeted validation:

- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_mutual_recursive_alias_accepts_cycle_with_container_boundary`
- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_naked_recursive_alias_is_rejected`
- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_recursive_generic_tuple_alias_is_rejected`
- `target/debug/sifr run demos/ad_hoc_recursive_type_part2_demo.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/recursive_type_alias_well_formed.sifr`
- `target/debug/sifr check crates/sifr/tests/e2e/fail/recursive_type_alias_missing_boundary.sifr`
- `target/debug/sifr check crates/sifr/tests/e2e/pass/recursive_type_alias_symbol_predeclaration.sifr`
- `target/debug/sifr check` on an inline generic negative probe for `type AlsoBad[T] = tuple[AlsoBad[T], T]`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr/tests/e2e/pass/recursive_type_alias_well_formed.sifr`
- `crates/sifr/tests/e2e/fail/recursive_type_alias_missing_boundary.sifr`
- `crates/sifr_hir/src/lower/type_alias_tests.rs`
- `demos/ad_hoc_recursive_type_part2_demo.sifr`

## Closure Decision

Part 2 is complete because recursive alias validation now distinguishes well-founded container recursion from infinite-size alias cycles with deterministic diagnostics.

Remaining recursive work is intentionally deferred:

- part 3: preserve recursive references through the internal type representation,
- part 4: recursive HIR expression behavior and tree-surface attribute usage,
- part 5: recursive Rust lowering/boxing,
- part 6: final regression matrix and Phase 31 handoff.
