## `milestone_diag_4a` slice 2b.11 — enum/protocol/newtype method missing param annotations migration to active `SIFR-TYPE-0004` — review pass 2

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-class-member-annotations`.
- Target: confirm closure of the single residual finding from [pass 1](semantic-diagnostic-code-taxonomy-diag-4a-class-member-annotations-review-pass-1.md) — R2, "Newtype fixture's call-site triggers an unrelated pre-existing dispatch error" — and reconfirm the pass-1 verdict (satisfied / no blocking findings).
- Diff since pass 1: a single line change in [crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr:1). `main()` was `user_id.add(2)`; it is now `print(user_id)`. No implementation code touched, no other fixture or doc edits, no checklist drive-bys.
- Validation rerun by reviewer: `cargo test -p sifr --test e2e -- test_e2e_fail` (1 passed, 25 filtered), and direct `cargo run -p sifr -- check` on each of the three new fixtures to confirm the rendered diagnostics now match the F3/R1 expectations exactly (see F1 below).

## Findings

### F1 — R2 is closed; the newtype fixture now emits exactly the missing-annotation diagnostic

`cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr` now produces a single line:

```
type error: [main] parameter 'amount' in UserId.add is missing a type annotation
```

The pre-existing `type error: [main] type 'int' has no method 'add'` dispatch error from pass 1 R2 is gone, because `main()` no longer invokes `user_id.add(2)`. The fixture still exercises the migrated [classes.rs:765-770](../crates/sifr_hir/src/lower/classes.rs:765) emission site at `UserId.add`'s declaration time — the diagnostic is independent of whether `add` is called, since it fires during `lower_class`'s parameter-iteration pass irrespective of body callers. So the signal-strengthening fix preserves coverage of the migrated code path. The fixture's `expect-error` substring (`parameter 'amount' in UserId.add is missing a type annotation`) is unchanged and remains a verbatim slice of the one rendered line, so the e2e harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) still matches on both halves (code AND substring) cleanly.

The R1 single-emission asymmetry from pass 1 (newtype 1×, enum/protocol 2×) is unchanged and as expected — newtypes have no `collect_class_type` body iteration per pass-1 F5, so `lower_class` is the only emission point.

### F2 — Enum and protocol fixtures untouched and still match pass-1 behavior

Confirmed by repeating the pass-1 R1 spot-check via `cargo run -p sifr -- check`:

- `enum_method_missing_type_annotation.sifr` → still emits `parameter 'other' in Direction.same is missing a type annotation` 2× (dual-pass, deferred per pass-1 R1).
- `protocol_method_missing_type_annotation.sifr` → still emits `parameter 'value' in Sink.accept is missing a type annotation` 2× (same).

Behavior identical to pass 1; no regressions.

### F3 — Diff is the minimum surface area to close R2

`git diff HEAD --stat` against pass-1 HEAD shows only the newtype fixture's `main()` body changed. The `# expect-error` line is byte-identical to pass 1, no extra fixtures were added, the migrated helper at [classes.rs:53-65](../crates/sifr_hir/src/lower/classes.rs:53) is untouched, the four migrated emission sites at [classes.rs:282-287](../crates/sifr_hir/src/lower/classes.rs:282), [classes.rs:322-327](../crates/sifr_hir/src/lower/classes.rs:322), [classes.rs:434-438](../crates/sifr_hir/src/lower/classes.rs:434), [classes.rs:483-487](../crates/sifr_hir/src/lower/classes.rs:483), and [classes.rs:765-770](../crates/sifr_hir/src/lower/classes.rs:765) are unchanged, and the issue checklist edit from pass 1 is unchanged. This is the exact minimal change pass 1's R2 mitigation suggestion outlined ("the simplest fix is to drop the `user_id.add(2)` call from `main()` (or replace it with `print(user_id)`)").

### F4 — `print(user_id)` is a valid newtype usage and exercises the wrapper

`UserId(int)` newtype's `print` path goes through the wrapper's `Display`/`Debug` impl rather than recursively through the inner `int`, so the call is well-formed and does not trigger the orthogonal newtype dispatch bug surfaced in pass-1 R2. `main()` therefore type-checks cleanly aside from the intended missing-annotation diagnostic on `UserId.add`'s declaration. The fixture's intent — pin the migrated code path's diagnostic without ambient noise — is now fully realized.

## Residual risks

### R1' — Pass-1 R1 unchanged: enum and protocol diagnostics still fire twice per param

Pre-existing dual-pass duplication via [mod.rs:595](../crates/sifr_hir/src/lower/mod.rs:595) and [mod.rs:605](../crates/sifr_hir/src/lower/mod.rs:605); deferred per pass-1 R1. No change in this pass.

### R2' — Pass-1 R3 unchanged: enum/protocol/newtype `__init__` parameters still silent

Same as pass-1 R3. Special-class `__init__` is intentionally skipped at [classes.rs:273-275](../crates/sifr_hir/src/lower/classes.rs:273) (enum), [classes.rs:313-315](../crates/sifr_hir/src/lower/classes.rs:313) (protocol), and via early return at [classes.rs:234](../crates/sifr_hir/src/lower/classes.rs:234) (newtype). Functionally inert for these shapes; deferred. No change in this pass.

### R3' — Pass-1 R4 unchanged: vararg / keyword-only class-method params remain uncovered

Same structural gap as pass-1 R4 — `func.parameters.vararg` / `kwonlyargs` are not iterated at any of the migrated sites, matching the pre-existing slice-2b.7 class-member behavior. Deferred. No change in this pass.

### R4' — Pass-1 R5 unchanged: no registry-level binding for the three new fixtures

Same as pass-1 R5. The three new fixtures are protected only by the e2e harness; the registry's representative-fixture map at [codes.rs:597-602](../crates/sifr_diagnostics/src/codes.rs:597) still pins `missing_type_annotation.sifr` as the sole representative for `SIFR-TYPE-0004`. Deferred to `milestone_diag_11`. No change in this pass.

### R5' — Pass-1 R6 unchanged: helper remains private

Same as pass-1 R6. `missing_method_param_annotation` remains `fn` (not `pub(super)`/`pub(crate)`); top-level / nested missing-annotation emissions in [typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:316) and [nested_function_inference.rs](../crates/sifr_hir/src/lower/nested_function_inference.rs:440) cannot reuse it. Distinct format strings make direct reuse a non-trivial refactor anyway. Milestone-scoped, not a blocker. No change in this pass.

(Pass-1 R2 — newtype fixture call-site noise — is closed by F1 above and not carried forward.)

## Verdict

**Satisfied / no blocking findings.** Pass 2 closes the only residual finding from pass 1 (R2, newtype fixture dispatch noise) with a single-line fixture edit that swaps `user_id.add(2)` for `print(user_id)`, leaving the migrated code path exercised exactly as before but eliminating the secondary unrelated diagnostic. No implementation code, helper signature, or other fixture changed. The local validation set the implementer reports (`report_signature=e1bf653aaa770517`, `wall_time=60.09s`, plus the targeted e2e/fmt/HIR-guardrails/diagnostic-transport/clippy/unit-test rerun) is the established gate, and my own re-runs (`cargo test -p sifr --test e2e -- test_e2e_fail` plus per-fixture `cargo run -p sifr -- check`) confirm both the signal-cleanup intent and the unchanged behavior of the enum/protocol fixtures. All five carried-forward residuals (R1', R2', R3', R4', R5') are pre-existing structural or milestone-scoped concerns explicitly out of slice 2b.11's scope. Slice 2b.11 is reviewer-satisfied and ready to ship.
