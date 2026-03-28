# Ad Hoc Recursive Type Feature: Part 3 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_type_representation_in_the_type_system`
PR: `#1124`

## Goal

Preserve recursive references in the internal type system instead of collapsing them during alias lookup or generic substitution.

This slice needs to make recursive aliases and recursive generic aliases survive annotation resolution with enough structure for later HIR and codegen work.

## Root Cause

After part 2, recursive alias names resolved deterministically and invalid cycles were rejected correctly, but the internal type representation still lost critical information:

- plain alias lookup flattened directly to the alias body,
- recursive references inside alias bodies degraded to `Unknown`,
- and generic recursive aliases lost their specialized type arguments on recursive edges.

That meant the compiler could accept some recursive aliases syntactically while still erasing the recursive structure the later pipeline needs.

## Implementation

- Extend the internal alias representation so aliases can carry both:
  - their nominal alias identity,
  - and any specialized type arguments for generic alias applications.
- Predeclare aliases as symbolic alias references rather than raw `Unknown`.
- Keep alias wrappers through annotation resolution and generic substitution instead of flattening eagerly.
- Preserve symbolic self-references inside recursive alias bodies without infinite expansion.
- Tighten generic alias arity checking so recursive generic aliases cannot silently instantiate with the wrong number of type arguments.
- Add regression coverage for:
  - recursive alias annotations preserving symbolic self-references,
  - recursive generic aliases preserving concrete type arguments through recursive edges,
  - and generic alias arity errors.

## Validation

Targeted validation:

- `cargo test -p sifr_hir type_alias -- --nocapture`
- `cargo test -p sifr_codegen --lib --no-run`
- `target/debug/sifr run demos/ad_hoc_recursive_type_part3_demo.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/recursive_generic_type_alias_representation.sifr`
- `target/debug/sifr check crates/sifr/tests/e2e/fail/recursive_generic_type_alias_wrong_arity.sifr`
- `target/debug/sifr check` on inline probes for `Node[int]` recursive specialization and `Json` symbolic self-reference preservation

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr/tests/e2e/pass/recursive_generic_type_alias_representation.sifr`
- `crates/sifr/tests/e2e/fail/recursive_generic_type_alias_wrong_arity.sifr`
- `crates/sifr_hir/src/lower/type_alias_tests.rs`
- `demos/ad_hoc_recursive_type_part3_demo.sifr`

## Closure Decision

Part 3 is complete because recursive aliases now survive internal type resolution as structured alias references instead of degrading to raw `Unknown` placeholders, and recursive generic aliases keep their specialized type arguments on recursive edges.

This slice intentionally stops at type representation. Remaining work is still deferred to the later planned parts:

- part 4: recursive HIR expression behavior and tree-surface attribute usage,
- part 5: recursive Rust lowering and codegen closure,
- part 6: final regression matrix and Phase 31 handoff.
