# Ad Hoc Recursive Type Feature: Part 1 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_symbol_predeclaration_and_alias_order_resolution`
PR: `#1122`

## Goal

Close the declaration-order root cause for recursive-type part 1 by making type-alias symbol resolution deterministic instead of source-order dependent.

This slice is intentionally limited to symbol registration and alias-body ordering:

- forward alias-to-alias references in the same module,
- recursive alias names resolving at the symbol level instead of erroring as unknown names,
- and preserving unresolved-name diagnostics outside the predeclared alias set.

It does not attempt to close:

- recursive alias well-formedness validation,
- recursive type preservation through the full type system,
- recursive class boxing/codegen,
- or the remaining tree-surface LeetCode closure work.

## Root Cause

Type aliases were still resolved in one source-order pass. That meant:

- `type A = B` failed when `B` was declared later,
- recursive aliases errored as `unknown type` before later parts could validate them properly,
- and any later compiler work had to inherit order-sensitive alias behavior.

The fix for this slice is to treat alias names as symbols first and alias bodies second.

## Implementation

- Added a dedicated `type_aliases` lowering module in `crates/sifr_hir/src/lower/`.
- Added a predeclaration pass for all top-level type-alias symbols before body resolution.
- Added alias dependency graph analysis plus SCC-based resolution so acyclic aliases are resolved in dependency order instead of source order.
- Kept recursive SCCs symbol-resolvable at this stage by predeclaring their names, deferring well-formedness enforcement to part 2.
- Reworked module lowering into a two-stage class pass:
  - collect full class shapes before final alias resolution so aliases like `Shape = Circle | Square` see real class fields,
  - refresh classes after alias resolution so class annotations that depend on later aliases see final alias shapes.
- Moved the new lowering tests into a dedicated test module to keep HIR maintainability guardrails green.

## Validation

Targeted validation:

- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_forward_type_alias_resolves_independent_of_declaration_order`
- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_recursive_type_alias_name_resolves_without_unknown_type_error`
- `target/debug/deps/sifr_hir-0607dee8cc39383e --exact lower::type_alias_tests::test_unresolved_type_alias_dependency_still_errors`
- `target/debug/sifr run demos/ad_hoc_recursive_type_part1_demo.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/recursive_type_alias_symbol_predeclaration.sifr`
- `target/debug/sifr check crates/sifr/tests/e2e/fail/type_alias_missing_dependency.sifr`
- `target/debug/sifr emit crates/sifr/tests/e2e/pass/discriminated_union.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr_hir/src/lower/type_aliases.rs`
- `crates/sifr_hir/src/lower/type_alias_tests.rs`
- `crates/sifr/tests/e2e/pass/recursive_type_alias_symbol_predeclaration.sifr`
- `crates/sifr/tests/e2e/fail/type_alias_missing_dependency.sifr`
- `demos/ad_hoc_recursive_type_part1_demo.sifr`

## Closure Decision

Part 1 is complete because type-alias symbols now resolve deterministically at the name-resolution layer, recursive alias names no longer fail as unknown types, and unresolved names outside the supported model still report deterministic errors.

Remaining work is intentionally deferred to later recursive-type parts:

- part 2: recursive well-formedness and diagnostics,
- part 3: recursive type representation beyond placeholder symbol resolution,
- parts 4-5: recursive expression/codegen behavior,
- part 6: corpus closure and final regression lock.
