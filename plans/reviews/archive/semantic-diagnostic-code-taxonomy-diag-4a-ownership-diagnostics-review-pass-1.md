# `milestone_diag_4a` slice 2b.15 — ownership HIR diagnostics review (pass 1)

Branch: `codex/semantic-diagnostics-diag-4a-ownership-diagnostics`.
Scope reviewed: migrate ownership HIR diagnostics from raw `ctx.error` (which fell through to the `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge) to active codes:

- `SIFR-OWN-0001` — use after move
- `SIFR-OWN-0002` — same-call borrow conflict (double-mut, mut-after-immut, immut-after-mut)
- `SIFR-OWN-0003` — borrowed-parameter return/store escape
- `SIFR-OWN-0004` — moved across `while` / `for` loop body

Plus extracting `crates/sifr_hir/src/lower/ownership_diagnostics.rs`, re-keying existing fail fixtures, adding two new mixed-borrow fixtures, and tightening unit assertions.

Validation reported by author: `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, focused HIR unit tests, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr -- --skip test_e2e_pass`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=121.13s`). I did not re-run the suites; I reviewed sources and the diff.

## Verdict

Approve with minor follow-ups. Taxonomy mapping, call-site coverage, fixture re-keying, and helper extraction are all on point and consistent with the rest of slice 2. Two registry-hygiene nits and a small unit-coverage gap; none block landing.

## What looks correct

- **All four codes are wired to the right call sites.**
  - `SIFR-OWN-0001` at the only `is_moved` read in [crates/sifr_hir/src/lower/expressions.rs:225-226](crates/sifr_hir/src/lower/expressions.rs:225).
  - `SIFR-OWN-0002` at the three same-call borrow-conflict branches in `lower_call` ([crates/sifr_hir/src/lower/expressions.rs:1827-1839](crates/sifr_hir/src/lower/expressions.rs:1827)). Conditions and ordering are unchanged; only the message-emission path was swapped.
  - `SIFR-OWN-0003` at the two escape paths (`lower_ann_assign` store at [statements.rs:1175](crates/sifr_hir/src/lower/statements.rs:1175) and `lower_return` at [statements.rs:1637](crates/sifr_hir/src/lower/statements.rs:1637)).
  - `SIFR-OWN-0004` at both loop forms (`lower_while` [statements.rs:2010](crates/sifr_hir/src/lower/statements.rs:2010) and `lower_for` [statements.rs:2165](crates/sifr_hir/src/lower/statements.rs:2165)).
- **No remaining raw ownership diagnostics.** A repo-wide search for "use of moved", "cannot borrow", "borrowed parameter", "moved inside loop" finds matches only in `ownership_diagnostics.rs` and the `sifr_diagnostics` registry (semantic templates). No call-sites slipped past the migration.
- **Helper extraction is appropriate.** `ownership_diagnostics.rs` is 63 lines, scope-private (`pub(super)`), and pure: each function is `(ctx, name[, func_name]) -> ctx.error_with_code(...)` with the canonical message. Extraction also paid back enough lines in `statements.rs` to keep it under the 2200-line guardrail (currently 2186 / 2200 — ~14 lines of headroom).
- **Messages are byte-identical to the previous output.** Each helper just rewraps the same `format!(...)` previously inlined at the call site. Behavior diff is purely "same message, now with a code attached" — no risk of UI regressions for tooling that grepped on the message text.
- **Re-keyed fixtures match the new codes.** All seven previously-using-`SIFR-TYPE-0001` ownership fixtures now assert the correct `SIFR-OWN-*` code. `missing_keyword_only_arg.sifr` was re-keyed to `SIFR-OWN-0003` because its actual error is the borrowed-parameter return escape — that's the correct code and the original keying was just incidentally going through the phase fallback.
- **New fixtures plug a real coverage gap.** `mut_borrow_after_immutable_borrow.sifr` and `immutable_borrow_after_mut_borrow.sifr` exercise the two `SIFR-OWN-0002` branches that previously had no e2e fixture (only `double_mut_borrow.sifr` did, which only hit the mut/mut path).
- **Unit assertions are now structured for the migrated codes that have unit tests.** `test_use_after_move` ([expressions_tests.rs:92-94](crates/sifr_hir/src/lower/expressions_tests.rs:92)) and the two escape tests ([own_mut_semantics_tests.rs:38-55](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs:38)) check both `message` and `code == Some(DiagnosticCode::OWN_*)`. The added `lower_errors` helper to keep `lower_error_messages` while exposing full errors is a clean refactor.

## Findings

### F1. Registry summary for `SIFR-OWN-0002` understates its real scope (registry-hygiene nit, low)

The registry entry calls this code "Double mutable borrow." with template `cannot borrow {binding} as mutable more than once`:

```text
crates/sifr_diagnostics/src/codes.rs:838-848
active_entry!(
    "SIFR-OWN-0002",
    "OWN",
    "Double mutable borrow.",
    Severity::Error,
    "crates/sifr/tests/e2e/fail/double_mut_borrow.sifr",
    "cannot borrow {binding} as mutable more than once",
    ...
)
```

But `OWN_DOUBLE_MUTABLE_BORROW` is now also emitted for "mutable after immutable" and "immutable after mutable" in the same call (per slice scope and the helper module). The registry summary and message template only describe the mut/mut case, which leaves the other two variants implicit and under-documented for downstream tooling that consumes the registry.

Two reasonable directions, either is fine:

- Cheap: broaden the summary/template to "Same-call borrow conflict." / "borrow conflict on {binding} in same call" — keeps one code, but documents what it covers.
- Slightly more work: rename the constant to `OWN_BORROW_CONFLICT` (the code stays `SIFR-OWN-0002`) so the symbol matches the broader semantic. The current name "double mutable borrow" reads as a strict subset.

Not blocking; flag for the next registry-hygiene pass (this is the same flavor of gap that slice 2b.6 cleaned up for `SIFR-TYPE-0009`).

### F2. Unit-test coverage for the new codes is partial (low)

`SIFR-OWN-0001` and `SIFR-OWN-0003` now have structured-code unit assertions. `SIFR-OWN-0002` and `SIFR-OWN-0004` are exercised only via e2e fixtures. The slice description says "tightens focused unit assertions for structured code" — that's true for the two it touched, but I would have expected at least one focused HIR unit test per code, mirroring what `expressions_tests.rs::test_use_after_move` does. e2e fail-fixture coverage is real, but those tests run far slower than HIR unit tests and don't isolate regressions to the lowering layer when the structured payload changes.

Suggested additions (cheap, can be folded into this slice or deferred):

- One `expressions_tests.rs` test for double-mut and one for each mixed direction, asserting `code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)`.
- One unit test in (or alongside) `own_mut_semantics_tests.rs` for the loop-move case, covering at least one `for` and ideally one `while`. The e2e fixture only covers `for`; both code paths emit through `ownership_diagnostics::moved_across_loop`, so the helper is exercised, but the `lower_while` branch in `statements.rs:2010` has no test that pins the structured code.

### F3. e2e fixture coverage of `SIFR-OWN-0004` only exercises `for` (low)

`use_after_move_loop.sifr` uses `for i in range(3)`. The diagnostic also fires from `lower_while` ([statements.rs:2010](crates/sifr_hir/src/lower/statements.rs:2010)) and the test surface for that path is empty (no e2e fixture, no unit test). A small `while`-form fixture would close this — out of scope for this slice's stated goal but worth a follow-up ticket.

### F4. Module-placement asymmetry vs. other domains (informational, no action required)

Other domains in slice 2b emit codes inline at the call site (RESULT, TYPE, NAME, IMPORT — no `<domain>_diagnostics.rs` helper). Ownership is the first to extract a per-domain diagnostic helper module. The motivation here is concrete: `statements.rs` is at 2186/2200 lines, so further inlining of `ctx.error_with_code(DiagnosticCode::OWN_*, format!(...))` blocks across four call sites would push it over the guardrail. The asymmetry is therefore justified, but it's worth recording as a precedent the team is comfortable with — a future hygiene pass might extract analogous helpers for the other domains rather than only ownership.

The new `ownership_diagnostics.rs` and the pre-existing `lower/diagnostics.rs` (which is actually a class-shape detector, not a diagnostic emitter) sit in the same directory with similar names. Mildly confusing on grep. If the helper convention spreads, consider naming successors `<domain>_diagnostic_codes.rs` or grouping under `lower/diagnostics/{ownership,result,...}.rs`. Out of scope.

### F5. Out-of-scope mutability/ownership call sites still flow through the phase fallback (informational)

These `ctx.error(format!(...))` call sites are semantically ownership/mutability concerns and currently still rely on the `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge:

- `cannot mutate through immutable parameter ...` — [crates/sifr_hir/src/lower/mutating_methods.rs:21](crates/sifr_hir/src/lower/mutating_methods.rs:21).
- `cannot reassign immutable parameter ...` — [statements.rs:1551](crates/sifr_hir/src/lower/statements.rs:1551), [tuple_unpack.rs:109](crates/sifr_hir/src/lower/tuple_unpack.rs:109), [aug_assign_lowering.rs:305](crates/sifr_hir/src/lower/aug_assign_lowering.rs:305).

These are *correctly* not part of slice 2b.15's stated scope (the slice scope explicitly enumerates four codes), and the existing tests in `own_mut_semantics_tests.rs` for these cases use the `lower_error_messages` helper rather than the new structured-code path, which is consistent. Calling out so they get scheduled into a future SIFR-OWN-* slice (likely a new code, e.g. `SIFR-OWN-0005`/`-0006`) before the `SIFR-TYPE-0001` bridge is finally retired — otherwise these will silently lose their codes when the bridge is deleted.

### F6. Tracker entry minor (informational)

`issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50` correctly opens slice 2b.15 with `[ ]` and "PR: pending". When this review is addressed and the slice merges, the standard "Claude implementation review for `milestone_diag_4a` slice 2b.15 completed ..." row needs to be appended (matching the format used at lines 67/66/etc.). No action now; just the usual close-out step.

## Behavior regressions

None observed. The changes are message-preserving and add a structured `code` field to diagnostics that previously lacked one (going from `code: None` → `code: Some(SIFR-OWN-*)`, with the `CompilePhase::TypeCheck => SIFR-TYPE-0001` mapping stripping away once the bridge is removed). Existing assertions like `errors.iter().any(|e| e.message.contains("moved value"))` continue to match. The four explicit fixture re-keys + two new fixtures all use the message-`contains` form, so they're robust to wording polish later.

## Suggested next steps

1. Address F1 (broaden `SIFR-OWN-0002` registry summary or rename the constant). Cheap, single-file.
2. Add focused HIR unit tests for `OWN_DOUBLE_MUTABLE_BORROW` (three variants) and `OWN_MOVED_ACROSS_LOOP` (`for` + `while`). One test each, structured-code asserts. (F2/F3.)
3. File a follow-up ticket for the immutable-parameter mutate/reassign diagnostics to land in a subsequent `SIFR-OWN-*` slice before the TYPE-0001 bridge is retired (F5).
4. On merge, append the slice 2b.15 review-completed row to the tracker (F6).

Items 1–2 are reasonable to fold into this slice; 3 and 4 are deferrable.
