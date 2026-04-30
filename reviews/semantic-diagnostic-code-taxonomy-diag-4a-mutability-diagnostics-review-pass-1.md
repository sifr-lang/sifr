# `milestone_diag_4a` slice 2b.17 — mutability diagnostics migration — review pass 1

Branch: `codex/semantic-diagnostics-diag-4a-mutability-diagnostics`
Tracker: [ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Predecessors: slice 2b.15 ownership ([#1687](https://github.com/sifr-lang/sifr/pull/1687)), slice 2b.16 flow ([#1688](https://github.com/sifr-lang/sifr/pull/1688))

## Verdict

**Reviewer satisfied / approved.** No blockers. The slice cleanly closes out the parameter-mutability strings flagged but deferred in slice 2b.15 review pass 2, behavior is preserved byte-for-byte, and the registry/docs/test surfaces are aligned. A few non-blocking polish notes below — none gate this PR.

## Scope check

Stated scope:
- Add `SIFR-OWN-0005` (immutable parameter mutated) and `SIFR-OWN-0006` (immutable parameter reassigned) as active codes, registry rows, and generated docs.
- Route the five existing raw `ctx.error(...)` emitters through new `ownership_diagnostics` helpers.
- Re-key two existing fail fixtures, add one new fail fixture for OWN-0006.
- Strengthen HIR unit tests with structured-code assertions and add new sub-case coverage (augassign + tuple reassign).
- Mark slice 2b.16 merged and open slice 2b.17 in tracker.

Observed scope matches stated scope. No drive-by changes.

## Correctness review

### Routing — clean and complete

`grep` for `is_parameter_binding() && !info.is_mutable_binding()` enumerates every immutable-parameter gate; all five have been routed:

- [crates/sifr_hir/src/lower/binding_mutability.rs:13](crates/sifr_hir/src/lower/binding_mutability.rs:13) — subscript / attribute mutation gate → `ownership_diagnostics::immutable_parameter_mutation`.
- [crates/sifr_hir/src/lower/mutating_methods.rs:21](crates/sifr_hir/src/lower/mutating_methods.rs:21) — list/dict/set mutating-method gate → `immutable_parameter_mutation`.
- [crates/sifr_hir/src/lower/statements.rs:1547](crates/sifr_hir/src/lower/statements.rs:1547) — bare reassignment → `immutable_parameter_reassignment`.
- [crates/sifr_hir/src/lower/aug_assign_lowering.rs:302](crates/sifr_hir/src/lower/aug_assign_lowering.rs:302) — augmented reassignment → `immutable_parameter_reassignment`.
- [crates/sifr_hir/src/lower/tuple_unpack.rs:105](crates/sifr_hir/src/lower/tuple_unpack.rs:105) — tuple-target reassignment → `immutable_parameter_reassignment`.

Cross-crate sweep `grep -rn "cannot mutate through immutable parameter\|cannot reassign immutable parameter" --include='*.rs'` returns only the helper module in `crates/sifr_hir`, the test file, and the registry catalog — no stragglers. The two raw-string sites highlighted in [reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md:84](reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md:84) are now both closed out.

### Code identity — correctly attached end-to-end

Both new helpers in [crates/sifr_hir/src/lower/ownership_diagnostics.rs:65-79](crates/sifr_hir/src/lower/ownership_diagnostics.rs:65) call `LowerCtx::error_with_code(...)`, which sets `LoweringError.code = Some(_)` ([crates/sifr_hir/src/lower/mod.rs:230](crates/sifr_hir/src/lower/mod.rs:230)). That field is forwarded faithfully through `lowering_error_to_compile_error` ([crates/sifr_driver/src/frontend/module_lowering.rs:47](crates/sifr_driver/src/frontend/module_lowering.rs:47)) into `CompileError::with_code(... CompilePhase::TypeCheck, code)`. The e2e fail harness at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561) matches on `failure.code == expected.code`, so the re-keyed and new fixtures exercise the active `SIFR-OWN-000{5,6}` code path end-to-end and would fail loudly if any helper accidentally fell back to `ctx.error(...)`.

### Behavior preservation — byte-for-byte

The five emitter sites previously produced two stable strings:

- `cannot mutate through immutable parameter ` + `name` + `: add `mut` to the parameter declaration`
- `cannot reassign immutable parameter ` + `name` + `: add `mut` to the parameter declaration`

The helpers reproduce both verbatim ([ownership_diagnostics.rs:68](crates/sifr_hir/src/lower/ownership_diagnostics.rs:68), [:76](crates/sifr_hir/src/lower/ownership_diagnostics.rs:76)). The pre-existing HIR tests continue to assert on the exact same strings (now also asserting on `code`), so any regression in either prose or code attachment surfaces deterministically.

### Helper module shape

[ownership_diagnostics.rs](crates/sifr_hir/src/lower/ownership_diagnostics.rs) stays consistent with the slice 2b.15 conventions:

- Both new helpers are `pub(super)` only — appropriate.
- Each takes `(ctx, name)` and emits a single structured error; no fan-out, no shared seam needed.
- They sit alongside the existing OWN-0001…0004 helpers in the same module, matching the file/family one-to-one mapping.

### Registry and generated docs

[crates/sifr_diagnostics/src/codes.rs:874-895](crates/sifr_diagnostics/src/codes.rs:874) adds the two `active_entry!` rows, and the constants list at [:1366-1367](crates/sifr_diagnostics/src/codes.rs:1366) is updated. Templates follow the established OWN convention (terse `cannot mutate through immutable parameter {binding}` / `cannot reassign immutable parameter {binding}`, no remediation suffix), matching how OWN-0003's `borrowed parameter {binding} escapes` template diverges from its concrete prose. `binding` is declared `message+json` and `binding` is the dedupe key — same shape as OWN-0001, OWN-0002, OWN-0004.

`docs/errors/SIFR-OWN-0005.md`, `docs/errors/SIFR-OWN-0006.md`, [docs/errors/diagnostic-codes.md:76-77](docs/errors/diagnostic-codes.md:76), and [internal_docs/diagnostic_codes.md:105-106](internal_docs/diagnostic_codes.md:105) all line up with the registry rows. The PR description confirms `check_diagnostic_docs_sync.py` and `check_diagnostic_schema_sync.py` pass, which is what would catch any per-row drift.

### Test coverage

HIR-level tests in [own_mut_semantics_tests.rs](crates/sifr_hir/src/lower/own_mut_semantics_tests.rs):

- `test_own_parameter_cannot_be_mutated_without_mut` — subscript path → OWN-0005 (asserts both message and code).
- `test_own_parameter_mutating_method_requires_mut` — mutating-method path → OWN-0005.
- `test_borrowed_parameter_cannot_be_reassigned_without_mut` — bare reassign → OWN-0006.
- `test_borrowed_parameter_cannot_be_augassigned_without_mut` (new) — `count += 1` → OWN-0006.
- `test_borrowed_parameter_cannot_be_tuple_reassigned_without_mut` (new) — `items, other = other, items` → OWN-0006.

Every routed call site has at least one HIR test that asserts `error.code == Some(...)`. The previously-generic `lower_error_messages` helper (which only inspected messages) is removed in favor of `lower_errors`, so the structured-code coverage is the explicit assertion path going forward. Good cleanup.

E2E coverage:

- [own_parameter_mutation_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_mutation_requires_mut.sifr) — re-keyed to OWN-0005 (subscript path).
- [own_parameter_method_mutation_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_method_mutation_requires_mut.sifr) — re-keyed to OWN-0005 (mutating-method path).
- [own_parameter_reassignment_requires_mut.sifr](crates/sifr/tests/e2e/fail/own_parameter_reassignment_requires_mut.sifr) — new, OWN-0006 (bare reassign).

The two OWN-0006 sub-cases (augassign, tuple-unpack) are HIR-only — see follow-up note 3 below.

### Tracker

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:51-52](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:51) closes 2b.16 with the merged PR link and opens 2b.17 with `PR: pending`, matching the established convention.

## Non-blocking notes

These are observations for future polish; none gate this PR.

1. **`ensure_mutable_parameter_binding` now has a dead `_operation` parameter.** [crates/sifr_hir/src/lower/binding_mutability.rs:6](crates/sifr_hir/src/lower/binding_mutability.rs:6) renames the third argument to `_operation` so the call sites can keep passing `"mutate through"`, but every one of the eleven callers (in `tuple_unpack.rs`, `statements.rs`, `aug_assign_lowering.rs`) now passes the same string and the helper ignores it. Drop the parameter and the matching `"mutate through"` literals in a small follow-up — the helper is purely a "this is a mutation site" gate now, so the verb argument no longer carries information. Not done here, presumably to keep the slice's diff narrow.

2. **Stylistic inconsistency between the two new helpers.** [ownership_diagnostics.rs:68](crates/sifr_hir/src/lower/ownership_diagnostics.rs:68) inlines the `format!` arg on a single line; [:75-77](crates/sifr_hir/src/lower/ownership_diagnostics.rs:75) splits the same shape across three lines. `cargo fmt` is happy with both, but the rest of the file (lines 5-63) splits long format strings consistently — the mutation helper is the outlier. Trivial.

3. **E2E coverage of OWN-0006 is limited to the bare-reassignment fixture.** Augassign-based reassignment ([aug_assign_lowering.rs:302](crates/sifr_hir/src/lower/aug_assign_lowering.rs:302)) and tuple-target reassignment ([tuple_unpack.rs:105](crates/sifr_hir/src/lower/tuple_unpack.rs:105)) are exercised only through HIR unit tests, so a regression in the driver→diagnostic pipeline that affects those paths could escape the e2e harness. Adding two small fail fixtures (e.g. `own_parameter_augassign_requires_mut.sifr`, `own_parameter_tuple_reassignment_requires_mut.sifr`) would round out the matrix without affecting this slice's correctness, and is consistent with the deferred-coverage note 3 in the flow-diagnostics review.

4. **Optional: a `kind` JSON arg for OWN-0006.** The three reassignment forms (direct, augmented, tuple) are emitted with the same prose. If downstream tooling ever needs to distinguish them, the natural place is a `json_arg!("kind")` slot on the registry row plus a kind enum threaded through the helper — mirroring OWN-0003's `escape_kind`. No demand today, so deferring is the right call; flagging it so the option stays on the docket.

5. **`LoweringError.line/col` remain `None` for these helpers.** Same as the pre-migration code path and every other HIR-emitted diagnostic — not a regression. Span attachment for HIR diagnostics is a cross-cutting gap that several slices have left unaddressed.

## Validation re-confirmation

The PR description lists the validation set; locally I spot-checked:
- Helper visibility (`pub(super)`) and the routing at all five emitter sites.
- Cross-crate absence of stale immutable-parameter strings.
- Catalog rows for SIFR-OWN-0005/0006 are `Active` and the constants list at [crates/sifr_diagnostics/src/codes.rs:1366-1367](crates/sifr_diagnostics/src/codes.rs:1366) lists them.
- Generated docs `docs/errors/SIFR-OWN-0005.md` and `docs/errors/SIFR-OWN-0006.md` match the registry rows.
- e2e fail-harness flow (`failure.code == expected.code` at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561)).

`scripts/run_all_tests.sh --profile quick` PASS with `report_signature=e1bf653aaa770517` (per PR description) — including `check_diagnostic_docs_sync.py` and `check_diagnostic_schema_sync.py`, which would have caught any registry/docs drift introduced by this slice.

## Recommendation

Land this slice as-is. Open a small cleanup follow-up for the dead `_operation` parameter (note 1) and, before the SIFR-TYPE-0001 bridge is removed in a later 2b.x slice, add the two missing OWN-0006 sub-case fail fixtures (note 3).
