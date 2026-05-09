# Review: INT-1 exact-int nonzero guards PR #1859

## Verdict
APPROVED

## Blocking Findings
none

## Non-Blocking Findings

1. **Augassign clear is unconditional**: In `aug_assign_lowering.rs:22`, `ctx.clear_proven_nonzero_integer_binding(&name)` is called unconditionally on every augmented assignment, even when the operation is not a division or modulo. This is correct (clearing a non-fact is a no-op) but worth noting as the set only accumulates facts from `//` and `%` guards, so unconditional clears do not cause incorrect suppression. Future readers may wonder if this should be gated; it need not be — clearing a non-member is harmless.

2. **No test for `elif` false-branch suppression**: The test suite covers `if guard; return` (early-exit false fact) and `while guard` (true fact inside body). A `elif` chain where a proven-nonzero guard is in an intermediate `elif` and the division occurs after the full `if/elif/else` would exercise the save/restore cycle on the intermediate `elif` branch. This is minor because the save/restore pattern is identical to the `if` path, but a dedicated test would make the invariant explicit.

3. **`and`/`or` guard composition is flat**: `detect_true_nonzero_integer_guards` traverses `BoolOp::And` by collecting from each operand, so a conjunctive guard like `x != 0 and y != 0` correctly marks both `x` and `y`. However, mixed logical nesting (e.g., `x != 0 and not y`) is not handled; only `not` of a comparison and `and`/`or` at the top level are covered. This is consistent with the stated PR scope (only `x != 0`, `if x == 0: return/raise`, and `while x != 0`). Nested mixed guard expressions remain a future extension point.

## Validation Notes

Local validation was run per the phase tracker requirement:
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo test -p sifr_hir exact_int -- --nocapture`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/error_builtin_classes.sifr`
- `scripts/run_all_tests.sh --profile quick` (report_signature=e1bf653aaa770517, wall_time=70.15s)

All checks passed. The quick profile is the authoritative pre-PR gate per the phase tracker.

The three new unit tests cover the three promised guard shapes:
- `test_exact_int_division_after_zero_guard_early_exit_lowers`: `if x == 0: return` — false-branch suppression after early exit
- `test_exact_int_modulo_inside_nonzero_while_guard_lowers`: `while x != 0:` — true-branch suppression inside while body
- `test_exact_int_nonzero_guard_is_cleared_after_reassignment`: reassignment clears the proof

The adapted `error_builtin_classes.sifr` e2e fixture replaces `// 1` (already non-zero-literal-suppressed) with `// b` where `b` is proven non-zero by the preceding guard, exercising the new binding-path suppression in a real error-handling function.

## Residual Risks

Acceptable. The proven-nonzero binding set is a simple `HashSet<String>` with save/restore checkpoints at every if/elif/else/while boundary. The only way a false binding could persist is if a division occurs outside any scope boundary after a guard — which the save/restore discipline prevents. Rebinding and augassign unconditionally clear the fact, preventing stale suppressions across reassignment. The `is_proven_nonzero_integer_binding` check is the single emission point and is only reached for `HirExpr::Name` nodes, keeping the attack surface minimal.
