# `milestone_diag_4a` slice 2b.15 — ownership HIR diagnostics review (pass 2)

Branch: `codex/semantic-diagnostics-diag-4a-ownership-diagnostics`.
Prior review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md).

Scope of this pass: confirm that the pass-1 follow-ups F1, F2, and F3 have landed cleanly, and re-check that nothing else in the slice regressed since.

I re-ran the targeted ownership unit tests and the e2e fail suite against the working tree:

- `cargo test -p sifr_hir -- test_use_after_move test_double_mutable_borrow_has_ownership_code test_mutable_after_immutable_borrow_has_ownership_code test_immutable_after_mutable_borrow_has_ownership_code test_for_loop_move_has_ownership_code test_while_loop_move_has_ownership_code test_mut_borrow_parameter_cannot_escape_via_return test_mut_borrow_parameter_cannot_escape_via_local_binding` → 8 passed.
- `cargo test -p sifr --test e2e -- test_e2e_fail` → pass.
- `python3 scripts/check_hir_maintainability_guardrails.py` → PASS.

I did not re-run `scripts/run_all_tests.sh --profile quick` myself; the author's reported `report_signature=e1bf653aaa770517, wall_time=131.20s` is consistent with this and the surrounding diag_4a slices.

## Verdict

Reviewer satisfied / approved. F1, F2, and F3 are addressed; F4 was informational; F5 was explicitly out of scope and stays deferred to a future `SIFR-OWN-*` slice; F6 (tracker close-out row) is the standard post-merge step.

## Pass-1 follow-ups status

### F1 — registry summary for `SIFR-OWN-0002` (addressed)

The pass-1 review flagged that the registry described `SIFR-OWN-0002` only as "Double mutable borrow." with template `cannot borrow {binding} as mutable more than once`, even though the code now also fires for mut-after-immut and immut-after-mut.

The author took the cheap option: broadened the registry copy without renaming the constant. Confirmed in [crates/sifr_diagnostics/src/codes.rs:838-848](crates/sifr_diagnostics/src/codes.rs:838):

```text
"SIFR-OWN-0002",
"OWN",
"Same-call borrow conflict.",
Severity::Error,
"crates/sifr/tests/e2e/fail/double_mut_borrow.sifr",
"borrow conflict for {binding} in the same call",
...
```

Generated docs were regenerated and committed:

- [docs/errors/SIFR-OWN-0002.md](docs/errors/SIFR-OWN-0002.md) now reads "Same-call borrow conflict." with the broadened message template.
- [docs/errors/diagnostic-codes.md:73](docs/errors/diagnostic-codes.md:73) one-liner updated to "Same-call borrow conflict.".
- [internal_docs/diagnostic_codes.md:102](internal_docs/diagnostic_codes.md:102) registry-table row updated with the new template.

The Rust constant remains `OWN_DOUBLE_MUTABLE_BORROW`. The pass-1 review listed both options ("broaden summary" and "rename constant") as acceptable; the symbol↔summary asymmetry that remains is mild — the symbol reads as a strict subset of what the code now covers — but is not blocking. If a future hygiene pass touches OWN, renaming the constant to `OWN_BORROW_CONFLICT` (code stays `SIFR-OWN-0002`, no on-disk rotation) is still on the table; flagging only.

### F2 — focused HIR unit coverage for `SIFR-OWN-0002` (addressed)

Pass-1 noted only `SIFR-OWN-0001` and `SIFR-OWN-0003` had structured-code unit assertions; `SIFR-OWN-0002` and `SIFR-OWN-0004` were e2e-only. Three new tests in [crates/sifr_hir/src/lower/expressions_tests.rs:97-137](crates/sifr_hir/src/lower/expressions_tests.rs:97) close that gap, one per same-call borrow-conflict variant:

- `test_double_mutable_borrow_has_ownership_code` — mut/mut conflict.
- `test_mutable_after_immutable_borrow_has_ownership_code` — mut after immut conflict.
- `test_immutable_after_mutable_borrow_has_ownership_code` — immut after mut conflict.

Each test asserts both the wording (`cannot borrow 'items' as ...`) and `e.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)`, so a future change to either the message helper or the code constant gets pinned by isolated lowering tests and not just the slower e2e fixture path. The wording substrings each test checks are unique to that variant, so a regression that swaps two helpers would surface as a test failure, not silent overlap.

### F3 — `while` coverage for `SIFR-OWN-0004` (addressed)

Pass-1 noted the e2e fixture for `SIFR-OWN-0004` only exercised `for`, leaving `lower_while`'s `ownership_diagnostics::moved_across_loop` call site untested. Two new tests in [crates/sifr_hir/src/lower/expressions_tests.rs:139-163](crates/sifr_hir/src/lower/expressions_tests.rs:139) — `test_for_loop_move_has_ownership_code` and `test_while_loop_move_has_ownership_code` — pin both branches and assert `OWN_MOVED_ACROSS_LOOP`. The `while` test re-creates the same shape with an explicit counter, which is the right minimal exercise; both branches go through the same helper so wiring is uniform.

### F4 — module-placement asymmetry (informational, no action)

[crates/sifr_hir/src/lower/ownership_diagnostics.rs](crates/sifr_hir/src/lower/ownership_diagnostics.rs) is unchanged in pass 2 — still 63 lines, scope-private (`pub(super)`), seven thin emitters that each call `ctx.error_with_code(DiagnosticCode::OWN_*, format!(...))`. The pre-existing motivation (statements.rs at 2186/2200 against the guardrail) still holds. No new asymmetry introduced.

### F5 — out-of-scope mutability/parameter reassign call sites (deferred, intentional)

The slice description and the author confirm that "cannot mutate through immutable parameter ..." and "cannot reassign immutable parameter ..." (in `mutating_methods.rs`, `statements.rs`, `tuple_unpack.rs`, `aug_assign_lowering.rs`) are intentionally not part of slice 2b.15 and will be picked up in a later mutability/parameter-reassign code slice. The relevant tests in [crates/sifr_hir/src/lower/own_mut_semantics_tests.rs:58-89](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs:58) still use the `lower_error_messages` helper (no structured-code assertion), which is the right call until those sites get their own `SIFR-OWN-*` codes. This stays on the docket so the diagnostics don't silently lose codes when the `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge is finally retired.

### F6 — tracker close-out row (post-merge step)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50) still has the in-progress `[ ] ... PR: pending` line; once this slice merges, the standard `[x] agent implementation review for milestone_diag_4a slice 2b.15 completed ...` row needs to be appended (matching the format used for slices 2b.13/2b.14). No action required from this review — flagging only.

## Re-confirmed from pass 1

I re-checked the parts that were already correct in pass 1 to make sure nothing regressed:

- All four codes still wired to the same call sites: [expressions.rs:226](crates/sifr_hir/src/lower/expressions.rs:226), [expressions.rs:1827-1838](crates/sifr_hir/src/lower/expressions.rs:1827), [statements.rs:1176](crates/sifr_hir/src/lower/statements.rs:1176), [statements.rs:1638](crates/sifr_hir/src/lower/statements.rs:1638), [statements.rs:2010](crates/sifr_hir/src/lower/statements.rs:2010), [statements.rs:2165](crates/sifr_hir/src/lower/statements.rs:2165).
- A repo-wide `grep` for "moved value", "cannot borrow", "borrowed parameter", "moved inside loop" against `crates/sifr_hir/src/` finds substantive matches only in [ownership_diagnostics.rs](crates/sifr_hir/src/lower/ownership_diagnostics.rs) and the test files; the three remaining hits in `statements.rs` are inline comments only.
- All seven re-keyed and two new fail fixtures still assert the correct `SIFR-OWN-*` codes (verified by reading each fixture's `expect-error:` header).
- `[crates/sifr_hir/src/lower/ownership_diagnostics.rs](crates/sifr_hir/src/lower/ownership_diagnostics.rs)` is byte-identical-emission to the previous inline `format!` blocks; behavior diff remains "same message + structured code".

## Findings (none blocking)

None. F1/F2/F3 are addressed; the remaining pass-1 follow-ups (F4, F5, F6) are informational or post-merge by design.

One micro-observation, not blocking and not new for this pass: the symbol/summary asymmetry on `SIFR-OWN-0002` (constant `OWN_DOUBLE_MUTABLE_BORROW` vs. summary "Same-call borrow conflict.") is a small reading-friction point only. If renaming the constant is folded into a future registry-hygiene sweep, no on-disk code rotation is required — the constant rename is a Rust-side refactor only.

## Suggested next steps

1. Land the slice and append the standard close-out row to [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50) (F6).
2. Schedule a follow-up SIFR-OWN-* slice (likely `SIFR-OWN-0005`/`-0006`) for the immutable-parameter mutate/reassign diagnostics in `mutating_methods.rs` / `statements.rs` / `tuple_unpack.rs` / `aug_assign_lowering.rs` before the `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge is retired (F5).
3. Optional: in a future registry-hygiene sweep, rename `OWN_DOUBLE_MUTABLE_BORROW` → `OWN_BORROW_CONFLICT` (code stays `SIFR-OWN-0002`, on-disk taxonomy unchanged) so the symbol matches the broadened summary.
