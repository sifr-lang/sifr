---
name: milestone_diag_4a slice 2b.4 review pass 2
description: Pass-2 review of HIR reassignment type-mismatch + tuple-unpack shape migration after closing the pass-1 coverage gap with two additional e2e fail fixtures.
---

# Review — `milestone_diag_4a` slice 2b.4 pass 2 (HIR reassignment type-mismatch + tuple-unpack shape migration)

Branch: `codex/semantic-diagnostics-diag-4a-assignment-shape`
Phase issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Pass 1 of record: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-assignment-shape-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-assignment-shape-review-pass-1.md)
Scope under review (unchanged from pass 1): migrate HIR reassignment-shape `type mismatch` emissions to active `SIFR-TYPE-0002` and tuple-unpack shape diagnostics to active `SIFR-TYPE-0009`, preserving human message text and adding representative e2e fail fixtures. Pass-2 increment: two additional fixtures targeted at the coverage-gap residual flagged in pass 1.

## Findings

### Pass-1 coverage gap (residual #3) is closed

Pass 1 flagged that two of the four migrated emission sites had no representative e2e fixture pinning them through the harness's joint `failure.code == expected.code` + substring contract ([crates/sifr/tests/e2e.rs:2561-2567](../crates/sifr/tests/e2e.rs:2561)). Both sites are now pinned:

- [crates/sifr/tests/e2e/fail/tuple_unpack_non_tuple_shape_mismatch.sifr:1](../crates/sifr/tests/e2e/fail/tuple_unpack_non_tuple_shape_mismatch.sifr:1) pins `SIFR-TYPE-0009: cannot unpack non-tuple type 'list[int]'`. The fixture body (`left, right = [1, 2]`) routes through `lower_tuple_unpack_assign` ([crates/sifr_hir/src/lower/tuple_unpack.rs:44](../crates/sifr_hir/src/lower/tuple_unpack.rs:44)); `value_ty` is `Type::List(int)` rather than `Type::Tuple`, so control falls into the `else` branch at [tuple_unpack.rs:75-81](../crates/sifr_hir/src/lower/tuple_unpack.rs:75) which is the migrated `error_with_code(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH, …)` site. I confirmed via `cargo run -q -p sifr -- check` that the produced text is exactly `cannot unpack non-tuple type 'list[int]'`.
- [crates/sifr/tests/e2e/fail/tuple_unpack_reassignment_type_mismatch.sifr:1](../crates/sifr/tests/e2e/fail/tuple_unpack_reassignment_type_mismatch.sifr:1) pins `SIFR-TYPE-0002: type mismatch: cannot assign 'str' to variable 'left' of type 'int'`. The fixture body declares `left = 1` first, then `left, label = ("not an int", "name")` — `targets.len() == elems.len() == 2` clears the count gate at `tuple_unpack.rs:62-72`, the value is a 2-tuple so it clears the non-tuple gate at `tuple_unpack.rs:75-81`, and the per-element loop reaches the rebind branch at [tuple_unpack.rs:115-125](../crates/sifr_hir/src/lower/tuple_unpack.rs:115) where `reconcile_optional_reassignment` rejects `str → int` and the migrated `error_with_code(DiagnosticCode::TYPE_MISMATCH, …)` fires for `left`. I verified the rendered text matches the pin.

All four migrated sites (`statements.rs:1551`, `tuple_unpack.rs:62-72`, `tuple_unpack.rs:75-81`, `tuple_unpack.rs:115-125`) now have at least one e2e fail fixture exercising them through the harness pipeline. Future regressions on any of the four would be caught by the fail-corpus gate.

### Routing verification

Both new fixtures were exercised end-to-end:

- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/tuple_unpack_non_tuple_shape_mismatch.sifr` emits `cannot unpack non-tuple type 'list[int]'` (and a downstream `undefined variable: 'left'` from the unrelated `print(left + right)` line). The harness only asserts that *some* failure matches the pinned `code+substring`, not that there are no other failures, so the cascading `undefined variable` does not weaken the assertion. The `SIFR-TYPE-0009` code is produced uniquely by the non-tuple branch in this fixture (the `[1, 2]` literal cannot itself emit `SIFR-TYPE-0009`).
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/tuple_unpack_reassignment_type_mismatch.sifr` emits exactly `type mismatch: cannot assign 'str' to variable 'left' of type 'int'` and nothing else, confirming the fixture lands precisely on the migrated per-element reassignment site without cascading noise.
- The full `cargo test -p sifr --test e2e -- --skip test_e2e_pass` reports `183 fail tests completed` (179 pre-slice + 4 fixtures from this slice) and passes.

### Pass-2 diff is appropriately bounded

The pass-2 increment is purely additive: two new fixture files, zero changes to compiler source, zero changes to other tests, zero changes to the issue tracker beyond pass 1's bullet flips. No incidental refactors, no movement on out-of-scope surfaces. The four out-of-scope surfaces called out in pass 1 (for-loop tuple destructuring, star-unpack list-shape, `tuple()` constructor / CALL-family / annotation-shape, and `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge deletion) remain on the bridge, which is what the slice's stated scope requires.

### Pass-1 residual risks 1, 2, and 4 still apply but are out of scope

- Residual #1 (for-loop tuple-unpack-shape siblings at [statements.rs:2105](../crates/sifr_hir/src/lower/statements.rs:2105) and [statements.rs:2118](../crates/sifr_hir/src/lower/statements.rs:2118)) and residual #2 (star-unpack list-shape at [tuple_unpack.rs:169](../crates/sifr_hir/src/lower/tuple_unpack.rs:169)) remain on the legacy bridge. No e2e fixture pins either, so this slice cannot regress them; both should be picked up by the next slice's framing before the bridge is finally deleted.
- Residual #4 (registry `message_template` for `SIFR-TYPE-0009` is `cannot unpack {actual_count} value(s) into {expected_count} target(s)` at [crates/sifr_diagnostics/src/codes.rs:653](../crates/sifr_diagnostics/src/codes.rs:653) while emitted text remains `tuple unpacking: expected {targets} values, got {elems}` and `cannot unpack non-tuple type '{ty}'`) is unchanged. Reconciling abstract template vs. rendered text belongs to a registry-hygiene slice when compact grouping / recovery limits land.

## Informational note (not a finding)

Running `test_e2e_fail --nocapture` prints two `internal compiler error: invalid control-flow graph: branch terminator in block 2 is incomplete (1 target(s))` stack traces to stderr from [crates/sifr_hir/src/cfg.rs:540](../crates/sifr_hir/src/cfg.rs:540). The test still passes (the panics are caught and converted into compile failures by the harness; no fixture asserts a panic). I verified by moving the four new fixtures aside and re-running `cargo test -p sifr --test e2e -- test_e2e_fail`: the panic count stays at exactly two with the new fixtures removed, so this is pre-existing and unrelated to slice 2b.4. Mentioned only so the next slice's reviewer doesn't mistake it for a regression.

## Verdict

Satisfied / no blocking findings. The pass-1 coverage-gap residual is closed: both previously-uncovered migrated emission sites (`tuple_unpack.rs:75-81` non-tuple unpack and `tuple_unpack.rs:115-125` per-element reassignment) are now pinned by representative e2e fail fixtures using the harness's joint code+substring contract. All four migrated sites exercise active codes (`SIFR-TYPE-0002` / `SIFR-TYPE-0009`) end-to-end through `LowerCtx::error_with_code`, the diff stays tight, and explicitly out-of-scope surfaces remain unperturbed. Pass-1 residual risks #1, #2, and #4 carry forward unchanged for follow-up slices to pick up before bridge deletion.
