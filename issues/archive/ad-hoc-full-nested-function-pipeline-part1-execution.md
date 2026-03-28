# Ad Hoc Nested Function Pipeline: Part 1 Execution

Status: complete
Started: 2026-03-14
Completed: 2026-03-14
Part: `milestone_nested_1`
PR: `#1139`

## Goal

Close the first structural gap in the nested-function pipeline by predeclaring nested helper symbols inside the enclosing function body and treating those symbols as typed local callables during lowering instead of only as late-added entries in `ctx.functions`.

This slice is intentionally limited to:

- deterministic nested symbol predeclaration within the current block,
- typed callable registration for nested helper names,
- forward local helper resolution during HIR lowering,
- and preserving explicit unresolved-name diagnostics for truly missing helpers.

It does not attempt to close:

- usage-driven parameter or return inference,
- recursive local-helper inference,
- capture mutation or `nonlocal`-style state updates,
- or broader codegen/diagnostic boundary cleanup.

## Root Cause

Nested helpers were still registered only when the lowering pass reached the `def` statement itself. That created two structural holes:

- earlier statements in the same enclosing body could not resolve the helper name as a local callable value,
- and direct local calls before the nested `def` fell through the same late-registration gap.

The fix for this slice is to treat nested helper names as local symbols up front, with a typed callable representation, before body lowering begins.

## Implementation

- Added `register_local_function_symbol` plus callable-type conversion in `crates/sifr_hir/src/lower/typing_and_functions.rs`.
- Added a block-entry nested-function predeclaration pass in `crates/sifr_hir/src/lower/statements.rs`.
- Reworked nested-function lowering to consume the predeclared signature instead of depending on statement-order registration.
- Added dedicated HIR regression tests for:
  - forward higher-order use of a nested helper,
  - forward direct calls to a nested helper,
  - and explicit unresolved-name diagnostics when no helper exists.
- Added `demos/ad_hoc_nested_function_part1_demo.sifr` to prove the milestone-owned callable-registration surface.
- Recorded the phase-wide entry baseline and execution checklist in `issues/ad-hoc-full-nested-function-pipeline.md`.

## Validation

Targeted validation:

- `cargo test -p sifr_hir nested_function_tests:: -- --nocapture`
- `cargo run -q -p sifr -- check /tmp/nested-predecl-<generated>.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr`
- `cargo run -q -p sifr -- run demos/ad_hoc_nested_function_part1_demo.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Closure Decision

Part 1 is complete because nested helpers now enter lowering as deterministic local callable symbols, forward local helper references no longer fail due to late registration, and missing helpers still fail through the explicit unresolved-name path.

The remaining phase work is intentionally deferred to later parts:

- part 2: usage-driven inference and recursive local-helper typing,
- part 3: capture typing and `nonlocal`-style mutation,
- part 4: codegen and unsupported-shape boundaries,
- part 5: regression/corpus/demo closure.
