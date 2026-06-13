# Review pass 2 — `milestone_diag_8` slice 1: return-completeness migration to `SIFR-FLOW-0004`

## Summary

Pass-2 spot check after the single non-blocking nit from pass 1 was addressed. The nit was the fully qualified `sifr_diagnostics::DiagnosticCode::FLOW_MISSING_RETURN_VALUE` path in [crates/sifr_hir/src/lower/expressions_tests.rs:713](crates/sifr_hir/src/lower/expressions_tests.rs:713) being inconsistent with the file's existing `use sifr_diagnostics::DiagnosticCode;` import and the bare-form convention used by neighboring assertions.

**Result: still satisfied.** The nit fix is correct, scoped, and behavior-preserving. No other in-tree changes since pass 1. Ready for PR.

## Verification of the nit fix

[crates/sifr_hir/src/lower/expressions_tests.rs:706-715](crates/sifr_hir/src/lower/expressions_tests.rs:706) now reads:

```
fn test_non_none_return_annotation_requires_exhaustive_returns() {
    let result = lower_source("def f(flag: bool) -> int:\n    if flag:\n        return 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("must return a value of type 'int' on all control-flow paths")
            && e.code == Some(DiagnosticCode::FLOW_MISSING_RETURN_VALUE)
    }));
}
```

- Bare form `DiagnosticCode::FLOW_MISSING_RETURN_VALUE` resolves to the same constant as the previous fully qualified path because [crates/sifr_hir/src/lower/expressions_tests.rs:2](crates/sifr_hir/src/lower/expressions_tests.rs:2) already has `use sifr_diagnostics::DiagnosticCode;`. No semantic change.
- Style now matches the rest of the file's code-pinning assertions (e.g. [crates/sifr_hir/src/lower/expressions_tests.rs:93](crates/sifr_hir/src/lower/expressions_tests.rs:93), [:107](crates/sifr_hir/src/lower/expressions_tests.rs:107), [:121](crates/sifr_hir/src/lower/expressions_tests.rs:121), [:134](crates/sifr_hir/src/lower/expressions_tests.rs:134)).
- Pinning logic itself (message substring AND `e.code == Some(...)`) is unchanged from pass 1; this slice still strengthens the regression beyond the prior message-only assertion.

## Scope check vs pass 1

Confirmed via `git diff` that no other files have changed since pass 1:

- `crates/sifr_diagnostics/src/codes.rs` — same +13/−0 (constant, registry entry, active-list inclusion) as pass 1.
- `crates/sifr_hir/src/lower/flow_diagnostics.rs` — same +9/−0 (`missing_return_value` helper) as pass 1.
- `crates/sifr_hir/src/lower/typing_and_functions.rs` — same +6/−5 (call-site migration of the `Ok(false)` branch) as pass 1.
- `docs/errors/SIFR-FLOW-0004.md` — generator-shaped page, unchanged from pass 1.
- `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md`, `internal_docs/diagnostic_emission_inventory.md` — same single-row insertions as pass 1.
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` — slice-1 in-progress bullet at line 87, unchanged from pass 1.
- `crates/sifr/tests/e2e/fail/missing_return_value.sifr` — unchanged: `# expect-error: SIFR-FLOW-0004`, the smallest fixture exercising the migrated `Ok(false)` branch.

The pass-1 review artifact (`reviews/semantic-diagnostic-code-taxonomy-diag-8-return-completeness-review-pass-1.md`) is now present as expected.

## Re-checked invariants

The nit fix is purely cosmetic so I did not re-derive every pass-1 invariant; I confirmed the load-bearing ones still hold:

- **Constant ↔ registry ↔ active-list parity.** All three references to `SIFR-FLOW-0004` / `FLOW_MISSING_RETURN_VALUE` in [codes.rs](crates/sifr_diagnostics/src/codes.rs) remain mutually consistent (id, severity, family, owner, fixture, template, args).
- **Helper ↔ template parity.** The format string in `missing_return_value` ([flow_diagnostics.rs:23-30](crates/sifr_hir/src/lower/flow_diagnostics.rs:23)) still produces text byte-identical to the registry `message_template` after `{function}` and `{return_type}` substitution.
- **Call-site scope.** Only the `Ok(false)` branch in `lower_function` ([typing_and_functions.rs:830-849](crates/sifr_hir/src/lower/typing_and_functions.rs:830)) is migrated; the `Ok(true)` no-op and the `Err(_) => ctx.warn(...)` panic-boundary advisory are correctly left alone.
- **No fallback.** The helper unconditionally carries `DiagnosticCode::FLOW_MISSING_RETURN_VALUE`; no parallel uncoded emission, no feature gate, no compatibility shim.
- **Fixture marker.** `# expect-error: SIFR-FLOW-0004` remains the canonical unqualified form accepted by the e2e harness.

## Validation cross-check

The user reported `cargo fmt --check` and `scripts/run_all_tests.sh --profile quick` running or already passed. Given that the only change since pass 1 is the bare-vs-qualified path (compile-equivalent to the same constant), that is a sufficient gate. The `quick` profile re-runs the registry tests, the HIR regression that pins both the message and the code, and the full e2e fail corpus including the new fixture — all of which already passed at pass 1, and none of which can be perturbed by the nit fix. No additional commands recommended for this pass.

## Findings

### Blockers

None.

### Non-blocking nits

None remaining. The pass-1 nit was addressed exactly as suggested.

### Observations (no action required)

- The previously noted untracked `verification/leetcode/full_corpus_failure_taxonomy_*.json` snapshots still contain pre-coded "type error: function '…' must return a value of type '…'" strings. They remain out of scope for this slice; if a later slice commits them, they should be re-keyed to `SIFR-FLOW-0004`.
- After PR merge, the slice-1 issue-tracker bullet on [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:87](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:87) should flip to `[x] … implementation complete and reviewer-satisfied: … PR: …` and a paired `[x] Claude implementation review for milestone_diag_8 slice 1 …` line should be appended (mirroring the slice-5 pair pattern at lines 85–86), with the `report_signature`/`wall_time` from the final `scripts/run_all_tests.sh --profile quick` run recorded.

## Verdict

**Satisfied — clear to PR.** The pass-1 nit was applied correctly and behavior-preservingly. No new diff, no scope creep, no regressions in the load-bearing pass-1 invariants. Re-opening any already-approved item is not warranted.
