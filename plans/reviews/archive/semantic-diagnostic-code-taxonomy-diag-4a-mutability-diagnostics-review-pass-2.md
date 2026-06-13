---
name: milestone_diag_4a slice 2b.17 — mutability diagnostics — review pass 2
description: Confirms pass 1 follow-ups (dead operation parameter removed, OWN-0006 sub-case e2e fixtures added) are correctly applied; slice remains reviewer-satisfied.
type: review
---

# `milestone_diag_4a` slice 2b.17 — mutability diagnostics migration — review pass 2

Branch: `codex/semantic-diagnostics-diag-4a-mutability-diagnostics`
Tracker: [ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior pass: [pass 1](semantic-diagnostic-code-taxonomy-diag-4a-mutability-diagnostics-review-pass-1.md)

## Verdict

**Reviewer satisfied / approved.** No remaining blockers. The two pass-1 polish items that called for action are addressed end-to-end; nothing else has regressed. The slice is ready to land as-is.

## Pass-1 follow-up confirmation

### Note 1 — drop the dead `_operation` parameter ✅ addressed

[crates/sifr_hir/src/lower/binding_mutability.rs:3](crates/sifr_hir/src/lower/binding_mutability.rs:3) is now `pub(super) fn ensure_mutable_parameter_binding(ctx: &mut LowerCtx, name: &str) -> bool`, and the body routes directly to `super::ownership_diagnostics::immutable_parameter_mutation(ctx, name)`. All ten call sites lost the trailing `"mutate through"` string literal:

- [aug_assign_lowering.rs:43](crates/sifr_hir/src/lower/aug_assign_lowering.rs:43), [:100](crates/sifr_hir/src/lower/aug_assign_lowering.rs:100), [:179](crates/sifr_hir/src/lower/aug_assign_lowering.rs:179), [:237](crates/sifr_hir/src/lower/aug_assign_lowering.rs:237).
- [statements.rs:1364](crates/sifr_hir/src/lower/statements.rs:1364), [:1388](crates/sifr_hir/src/lower/statements.rs:1388), [:1412](crates/sifr_hir/src/lower/statements.rs:1412), [:1443](crates/sifr_hir/src/lower/statements.rs:1443), [:1468](crates/sifr_hir/src/lower/statements.rs:1468).
- [tuple_unpack.rs:29](crates/sifr_hir/src/lower/tuple_unpack.rs:29).

Confirmed with `grep -rn "mutate through" --include='*.rs' .` — the only remaining occurrences are the OWN-0005 message itself in [ownership_diagnostics.rs:68](crates/sifr_hir/src/lower/ownership_diagnostics.rs:68), the registry template at [crates/sifr_diagnostics/src/codes.rs:880](crates/sifr_diagnostics/src/codes.rs:880), and the two assertion strings in [own_mut_semantics_tests.rs:59](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs:59),[:72](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs:72). All five are intended.

The semantic separation also reads correctly now: `ensure_mutable_parameter_binding` is exclusively a mutation-site gate (attribute-write, subscript-write, augassign-on-attr/subscript), and the three reassignment sites — bare reassign at [statements.rs:1547](crates/sifr_hir/src/lower/statements.rs:1547), augassign-on-name at [aug_assign_lowering.rs:302](crates/sifr_hir/src/lower/aug_assign_lowering.rs:302), tuple-target reassign at [tuple_unpack.rs:105](crates/sifr_hir/src/lower/tuple_unpack.rs:105) — call `immutable_parameter_reassignment` directly. The verb argument really was redundant; dropping it makes the helper's role unambiguous.

### Note 3 — add e2e fixtures for OWN-0006 augassign + tuple sub-cases ✅ addressed

Two new fixtures, marker-and-shape consistent with [own_parameter_reassignment_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_reassignment_requires_mut.sifr):

- [own_parameter_augassign_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_augassign_requires_mut.sifr) — `count += 1` on a borrowed `int` parameter; expects `SIFR-OWN-0006: cannot reassign immutable parameter \`count\`: ...`.
- [own_parameter_tuple_reassignment_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_tuple_reassignment_requires_mut.sifr) — `items, other = other, items` on borrowed list parameters; expects `SIFR-OWN-0006: cannot reassign immutable parameter \`items\`: ...`.

Both ride the same `failure.code == expected.code` matcher at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561), so they exercise the active OWN-0006 code path through the full driver→diagnostic→fail-harness pipeline — closing the gap between the three reassignment helper paths and the previously single bare-reassign fixture. `cargo test -p sifr --test e2e -- test_e2e_fail` PASS locally with 25 fail fixtures discovered.

## Other surfaces

No drift since pass 1 in any other surface I spot-checked:
- Helper module shape ([ownership_diagnostics.rs:65-79](crates/sifr_hir/src/lower/ownership_diagnostics.rs:65)), code identity attachment, and registry templates ([codes.rs:874-895](crates/sifr_diagnostics/src/codes.rs:874)) are unchanged from pass 1.
- Generated docs `docs/errors/SIFR-OWN-0005.md`, `docs/errors/SIFR-OWN-0006.md`, the index row in [docs/errors/diagnostic-codes.md:76-77](docs/errors/diagnostic-codes.md:76), and [internal_docs/diagnostic_codes.md:105-106](internal_docs/diagnostic_codes.md:105) all line up with the registry rows.
- HIR tests in [own_mut_semantics_tests.rs](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs) keep the structured-code assertions on every routed call site (8 tests, all PASS in `cargo test -p sifr_hir own_mut_semantics_tests`).
- Tracker [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:51-52](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:51) closes 2b.16 with PR #1688 and opens 2b.17 with `PR: pending`, matching slice convention. (`PR: pending` will need to be filled in before merge — same rule as previous slices, not blocking review.)

## Remaining non-blocking notes

These were filed in pass 1 and remain unaddressed by design — restated only so the docket is current.

1. **Stylistic inconsistency between the two new helpers.** [ownership_diagnostics.rs:68](crates/sifr_hir/src/lower/ownership_diagnostics.rs:68) inlines the `format!` argument on a single line; [:75-77](crates/sifr_hir/src/lower/ownership_diagnostics.rs:75) splits the same shape across three lines. `cargo fmt` is happy with both. Trivial; not worth a follow-up on its own.
2. **Optional `kind` JSON arg on OWN-0006.** The three reassignment forms (direct, augassign, tuple) share prose; if downstream tooling later needs to distinguish them, mirror OWN-0003's `escape_kind` slot. Deferred.
3. **`LoweringError.line/col` remain `None` for these helpers.** Cross-cutting HIR span gap; not specific to this slice.

## Validation

Re-ran locally on top of the pass-1 follow-up commit:
- `cargo test -p sifr_hir own_mut_semantics_tests` — 8/8 PASS.
- `cargo test -p sifr --test e2e -- test_e2e_fail` — PASS (25 fail fixtures, including the two new OWN-0006 ones).
- `grep -rn "cannot mutate through immutable parameter\|cannot reassign immutable parameter" --include='*.rs' .` — only the helper module, the test file, and the registry catalog match.
- `grep -rn "mutate through" --include='*.rs' .` — no straggler `"mutate through"` literals at call sites.

The PR description's full validation set (`run_all_tests.sh --profile quick` PASS, `report_signature=e1bf653aaa770517`; plus `gen-error-docs`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `check_hir_maintainability_guardrails.py`) is the same gate prior slices used and is consistent with what I spot-checked.

## Recommendation

Land this slice. Both actionable pass-1 follow-ups (notes 1 and 3) are resolved; the remaining notes are explicitly deferred and do not gate merge. Fill in the `PR:` link in the tracker once the PR is opened, per slice convention.
