# `milestone_diag_4a` slice 2b.16 — flow diagnostics migration — review pass 1

Branch: `codex/semantic-diagnostics-diag-4a-flow-diagnostics`
Tracker: [ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Predecessor: slice 2b.15 (ownership diagnostics) — [#1687](https://github.com/sifr-lang/sifr/pull/1687)

## Verdict

**Reviewer satisfied / approved.** The slice is correct, scoped tightly, and well-tested. No blockers. A handful of non-blocking polish notes below — none of them gate this PR.

## Scope check

Stated scope:
- Migrate `'break'` / `'continue'` outside-of-loop and the invalid-nonlocal / nested-function family to active SIFR-FLOW codes.
- Add `crates/sifr_hir/src/lower/flow_diagnostics.rs` and route HIR call sites through it.
- Re-key three existing fail fixtures.
- Add structured-code assertions to existing HIR tests; add three new HIR tests for FLOW-0003 sub-cases.
- Mark 2b.15 merged and open 2b.16 in tracker.
- Explicitly defer SIFR-FLOW-0901 (unreachable warning) — warnings still flow as `Vec<String>`.

Observed scope matches stated scope. The slice does not touch the diagnostic registry (`crates/sifr_diagnostics/src/codes.rs`); FLOW-0001/0002/0003/0901 entries were already populated in the earlier registry-population slice, so this PR only wires emission sites and re-keys fixtures. That is the right factoring.

## Correctness review

### Routing — clean

Verified by `grep` that no raw-string emission of any flow diagnostic remains in HIR or downstream crates:

- [crates/sifr_hir/src/lower/statements.rs:206](crates/sifr_hir/src/lower/statements.rs:206) — `break` → `flow_diagnostics::break_outside_loop`
- [crates/sifr_hir/src/lower/statements.rs:213](crates/sifr_hir/src/lower/statements.rs:213) — `continue` → `flow_diagnostics::continue_outside_loop`
- [crates/sifr_hir/src/lower/statements.rs:595](crates/sifr_hir/src/lower/statements.rs:595) — recursive nested helper → `flow_diagnostics::recursive_nonlocal_nested_function`
- [crates/sifr_hir/src/lower/nonlocal_support.rs:60](crates/sifr_hir/src/lower/nonlocal_support.rs:60), [:67](crates/sifr_hir/src/lower/nonlocal_support.rs:67), [:71](crates/sifr_hir/src/lower/nonlocal_support.rs:71) — three nonlocal-resolution failure modes routed through helpers.
- [crates/sifr_hir/src/lower/aug_assign_lowering.rs:289](crates/sifr_hir/src/lower/aug_assign_lowering.rs:289) — captured augassign without `nonlocal` → helper.
- [crates/sifr_hir/src/lower/tuple_unpack.rs:90](crates/sifr_hir/src/lower/tuple_unpack.rs:90) — tuple-unpack rebind → helper.

Cross-crate sweep: `grep -rn "outside of loop\|recursive nested function\|nonlocal name\|tuple unpacking cannot rebind\|captured variable.*nonlocal\|nonlocal declaration requires" crates --include="*.rs"` only matches `flow_diagnostics.rs`, `expressions_tests.rs`, `nested_function_tests.rs`, `codes.rs`, and a single non-error `.expect("recursive nested function lowered")` panic guard in `crates/sifr_codegen/src/lower_stmt.rs:9284`. No stragglers.

### Code identity — correctly attached

`flow_diagnostics.rs` calls `LowerCtx::error_with_code(...)`, which sets `LoweringError.code = Some(_)` ([crates/sifr_hir/src/lower/mod.rs:230](crates/sifr_hir/src/lower/mod.rs:230)). That field is forwarded faithfully through `lowering_error_to_compile_error` ([crates/sifr_hir/src/lower/../../../sifr_driver/src/frontend/module_lowering.rs:47](crates/sifr_driver/src/frontend/module_lowering.rs:47)) into `CompileError::with_code` and out via `compile_errors_to_diagnostics`. The e2e fail harness ([crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561)) matches on `failure.code == expected.code`, so the re-keyed fixtures exercise the active code path end-to-end and would fail loudly if any helper accidentally fell back to `ctx.error(...)`.

### Helper module shape

[crates/sifr_hir/src/lower/flow_diagnostics.rs](crates/sifr_hir/src/lower/flow_diagnostics.rs) is small and consistent:

- Public surface is `pub(super)` only, so it cannot leak outside `lower::*` — appropriate.
- `invalid_nonlocal` is an internal seam shared by all five FLOW-0003 sub-cases and only used through the named helpers — readers don't have to memorize codes at the call site.
- Messages are byte-for-byte preserved relative to pre-migration text, so existing tests, baselines, and the unit tests at [expressions_tests.rs:1694](crates/sifr_hir/src/lower/expressions_tests.rs:1694) and [nested_function_tests.rs:130](crates/sifr_hir/src/lower/nested_function_tests.rs:130) continue to assert on stable strings.

### Test coverage

HIR-level structured-code assertions are present for every routed call site:

- `test_break_outside_loop` and `test_continue_outside_loop` — code + message.
- `test_nonlocal_tuple_unpack_fails_explicitly` — FLOW_INVALID_NONLOCAL.
- `test_augassign_to_capture_requires_nonlocal` — FLOW_INVALID_NONLOCAL.
- `test_recursive_nonlocal_nested_helper_fails_explicitly` — FLOW_INVALID_NONLOCAL.
- `test_top_level_nonlocal_requires_enclosing_binding_code` — FLOW_INVALID_NONLOCAL (new).
- `test_unresolved_nonlocal_has_flow_code` — FLOW_INVALID_NONLOCAL (new).
- `test_nonlocal_current_binding_conflict_has_flow_code` — FLOW_INVALID_NONLOCAL (new).

E2E coverage re-keys the three existing fixtures. That keeps the harness honest end-to-end on the most common surface (`break`, `continue`, recursive nonlocal helper). See follow-up note below on the remaining FLOW-0003 sub-cases.

### Tracker

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50-51](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50) cleanly closes 2b.15 and opens 2b.16 with the right naming pattern. PR link is `pending` per convention.

### Out-of-scope handling

The slice description correctly identifies SIFR-FLOW-0901 (unreachable statement warning) as a non-goal. Confirmed at [crates/sifr_hir/src/lower/statements.rs:127](crates/sifr_hir/src/lower/statements.rs:127): `ctx.warn(...)` is still string-based and `LowerCtx.warnings: Vec<String>`. Migrating this requires structured warning transport (mirroring the `LoweringError { code, ... }` work for errors), which deserves its own slice. Punting it out of an error-code-migration slice is the right call.

## Non-blocking notes

These are observations for future polish; none gate this PR.

1. **FLOW-0001 / FLOW-0002 catalog templates miss the surrounding quotes.** The runtime emits `'break' outside of loop` (with single quotes around the keyword), but the registry templates at [crates/sifr_diagnostics/src/codes.rs:877](crates/sifr_diagnostics/src/codes.rs:877) and [:888](crates/sifr_diagnostics/src/codes.rs:888) read `break outside of loop` / `continue outside of loop`. The user-facing `docs/errors/SIFR-FLOW-0001.md` therefore advertises a slightly different message than what users see. Cosmetic; the templates were set by the earlier registry-population slice. Worth syncing in a docs-only follow-up.

2. **FLOW-0003 catalog template doesn't reflect any actual emission.** [crates/sifr_diagnostics/src/codes.rs:899](crates/sifr_diagnostics/src/codes.rs:899) advertises `invalid nested function flow: {reason}` with a `reason` arg, but none of the six `flow_diagnostics` helpers produce that prefix and no emission site populates a structured `reason` arg — all helpers pass fully-formatted strings through `error_with_code`. Two reasonable follow-ups, both out of scope here:
   - Replace the template with one of the canonical real messages (e.g., the `nonlocal name '{name}' does not resolve...` shape), drop the unused `reason` arg, and re-run `gen-error-docs`. This keeps FLOW-0003 as one umbrella code.
   - Or split FLOW-0003 into per-cause codes (e.g., FLOW-0004 missing-enclosing, FLOW-0005 captured-augassign, etc.) so each row has a faithful template — heavier and probably overkill for the current emission volume.

3. **E2E coverage of FLOW-0003 sub-cases is limited to the recursive-helper fixture.** Five of six FLOW-0003 emission paths (top-level nonlocal, unresolved nonlocal, current-binding conflict, tuple-unpack rebind, captured augassign) are exercised only through HIR unit tests, which means a regression in the driver→diagnostic pipeline could escape the e2e harness. The slice scope explicitly limits itself to re-keying existing fixtures, so this is consistent with the stated scope. Adding one fail fixture per sub-case is a small follow-up and would round out the coverage matrix without affecting this slice's correctness.

4. **`LoweringError.line/col` remain `None` for flow helpers.** Same as the existing pre-migration code path, so this is not a regression — but worth noting that span attachment for HIR-emitted diagnostics is a separate gap that several slices have left unaddressed.

## Validation re-confirmation

The PR description lists the validation set; locally I spot-checked:
- Helper visibility (`pub(super)`) and the wiring in [mod.rs:27](crates/sifr_hir/src/lower/mod.rs:27).
- Cross-crate absence of stale flow-error strings.
- Catalog rows present and active for SIFR-FLOW-0001, -0002, -0003, -0901 ([crates/sifr_diagnostics/src/codes.rs:64-67](crates/sifr_diagnostics/src/codes.rs:64) and [:1341-1344](crates/sifr_diagnostics/src/codes.rs:1341)).
- e2e fail-harness flow (code-equality match at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561)).

`scripts/run_all_tests.sh --profile quick` PASS with `report_signature=e1bf653aaa770517` (per PR description) — including `check_diagnostic_docs_sync.py` and `check_diagnostic_schema_sync.py`, which would have caught any registry/docs drift introduced by this slice.

## Recommendation

Land this slice as-is. Open a small docs-sync follow-up for the FLOW-0001/-0002 quoting and the FLOW-0003 template drift (note 1 + note 2). Optionally schedule the per-sub-case fail fixtures (note 3) before the SIFR-TYPE-0001 bridge is removed in a later 2b.x slice.
