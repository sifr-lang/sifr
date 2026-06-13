# Review pass 1 — `milestone_diag_8` slice 3: bare `raise` migration to existing `SIFR-RESULT-0003`

## Summary

Reviewed the uncommitted implementation that migrates the bare-`raise` lowering diagnostic from raw `ctx.error(...)` (uncoded) transport to the pre-existing `SIFR-RESULT-0003` (`RESULT_INVALID_RAISE`) code, factors the three RESULT-0003 emission sites (string-literal raise, non-Error-type raise, bare raise) through a new domain helper module `result_diagnostics.rs`, adds a HIR unit test for the bare-raise code, and adds an e2e fail fixture `error_raise_bare.sifr`. Inventory and issue tracker are updated to reflect the broader fixture set on `SIFR-RESULT-0003` and the in-progress slice entry.

**Result: satisfied.** No blockers. Two minor advisory observations recorded under "Residual risks". Implementation is correct, in-scope, internally consistent with the slice‑1 / slice‑2 helper-module pattern, message text for the two refactored sites is byte-identical to the prior emission, and no fallback or compatibility path was introduced.

## Files inspected

- [crates/sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs:63) — module declaration for `result_diagnostics` and the `#[cfg(test)] result_diagnostics_tests` companion
- [crates/sifr_hir/src/lower/statements.rs:246-266](../crates/sifr_hir/src/lower/statements.rs:246) — three migrated emission sites under `Stmt::Raise`
- [crates/sifr_hir/src/lower/result_diagnostics.rs](../crates/sifr_hir/src/lower/result_diagnostics.rs:1) — new helper module (string / non-Error / bare wrappers around the shared private `invalid_raise(ctx, msg)`)
- [crates/sifr_hir/src/lower/result_diagnostics_tests.rs](../crates/sifr_hir/src/lower/result_diagnostics_tests.rs:1) — new HIR unit test for bare-raise code attachment
- [crates/sifr/tests/e2e/fail/error_raise_bare.sifr](../crates/sifr/tests/e2e/fail/error_raise_bare.sifr:1) — new e2e fail fixture (canonical unqualified `# expect-error: SIFR-RESULT-0003`)
- [crates/sifr/tests/e2e/fail/error_raise_str.sifr](../crates/sifr/tests/e2e/fail/error_raise_str.sifr:1), [crates/sifr/tests/e2e/fail/error_raise_non_error.sifr](../crates/sifr/tests/e2e/fail/error_raise_non_error.sifr:1) — pre-existing fixtures verified unchanged on disk and still asserting `SIFR-RESULT-0003`
- [internal_docs/diagnostic_emission_inventory.md:338](../internal_docs/diagnostic_emission_inventory.md:338) — fixture list for `SIFR-RESULT-0003` expanded to all three sites
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:91](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:91) — `milestone_diag_8` slice 3 in-progress entry
- [crates/sifr_diagnostics/src/codes.rs:99](../crates/sifr_diagnostics/src/codes.rs:99), [:1095-1105](../crates/sifr_diagnostics/src/codes.rs:1095), [:1400](../crates/sifr_diagnostics/src/codes.rs:1400) — pre-existing `RESULT_INVALID_RAISE` constant, registry entry, and active-list inclusion (re-used, not modified)
- [docs/errors/SIFR-RESULT-0003.md](../docs/errors/SIFR-RESULT-0003.md) — pre-existing generated page (re-used, not modified)
- [crates/sifr_hir/src/lower/flow_diagnostics.rs](../crates/sifr_hir/src/lower/flow_diagnostics.rs:1) — sibling helper module pattern compared for style/visibility consistency
- [crates/sifr/tests/e2e.rs:611-640](../crates/sifr/tests/e2e.rs:611) — `# expect-error:` parser harness that consumes the new fixture

## Correctness

- **Bare-raise migration is a pure transport upgrade.** [statements.rs:262-265](../crates/sifr_hir/src/lower/statements.rs:262) replaces the previous `ctx.error("bare 'raise' without an expression is not supported".to_string())` (which produced `LoweringError { code: None, … }` per [mod.rs:225-232](../crates/sifr_hir/src/lower/mod.rs:225)) with `super::result_diagnostics::invalid_bare_raise(ctx)`. The helper at [result_diagnostics.rs:18-23](../crates/sifr_hir/src/lower/result_diagnostics.rs:18) emits the byte-identical message string `"bare 'raise' without an expression is not supported"` through the shared `invalid_raise(ctx, msg)` private at [result_diagnostics.rs:25-27](../crates/sifr_hir/src/lower/result_diagnostics.rs:25), which calls `ctx.error_with_code(DiagnosticCode::RESULT_INVALID_RAISE, message)`. Net effect: message text preserved verbatim, `code` field flips `None` → `Some(RESULT_INVALID_RAISE)`, return value (`None`) and control flow are unchanged.
- **String-raise refactor preserves emission verbatim.** Pre-migration ([git diff:statements.rs:248-251](../crates/sifr_hir/src/lower/statements.rs:248)) emitted `error_with_code(DiagnosticCode::RESULT_INVALID_RAISE, "raise requires an Error class instance — \`raise \"message\"\` is not allowed, use e.g. \`raise ValueError(\"message\")\`")` inline. Post-migration ([statements.rs:249-252](../crates/sifr_hir/src/lower/statements.rs:249)) calls `super::result_diagnostics::invalid_raise_string(ctx)`. The helper at [result_diagnostics.rs:4-9](../crates/sifr_hir/src/lower/result_diagnostics.rs:4) holds the byte-identical literal. The matched-arm guard `matches!(exc.as_ref(), Expr::StringLiteral(_) | Expr::FString(_))` ([statements.rs:249](../crates/sifr_hir/src/lower/statements.rs:249)) and `return None` are unchanged.
- **Non-Error-raise refactor preserves emission verbatim.** Pre-migration ([git diff:statements.rs:256-263](../crates/sifr_hir/src/lower/statements.rs:256)) emitted `error_with_code(RESULT_INVALID_RAISE, format!("raise requires an Error class instance — `{ty_name}` is not an Error class"))`. Post-migration ([statements.rs:256-259](../crates/sifr_hir/src/lower/statements.rs:256)) calls `super::result_diagnostics::invalid_raise_non_error(ctx, ty_name.as_str())`. The helper at [result_diagnostics.rs:11-16](../crates/sifr_hir/src/lower/result_diagnostics.rs:11) re-binds the parameter as `type_name: &str`, then `format!("raise requires an Error class instance — `{type_name}` is not an Error class")`. The two parameter names (`ty_name`, `type_name`) are local-only — the rendered message text is byte-identical. The owned `String` returned by `format_type_name(raised_ty)` ([statements.rs:257](../crates/sifr_hir/src/lower/statements.rs:257)) is bound to `ty_name` so `.as_str()` is valid for the duration of the call. `return None` preserved.
- **Helper module shape matches sibling convention.** [result_diagnostics.rs:1-27](../crates/sifr_hir/src/lower/result_diagnostics.rs:1) imports `super::LowerCtx` and `sifr_diagnostics::DiagnosticCode`, exposes three `pub(super) fn`-shaped wrappers, and centralises the code attachment in a private `fn invalid_raise(ctx, message)`. This mirrors the [flow_diagnostics.rs](../crates/sifr_hir/src/lower/flow_diagnostics.rs:1) pattern (single helper file per family, `pub(super)` visibility, `&mut LowerCtx` first arg) introduced by slices 1–2 of `milestone_diag_8`. Centralising the `error_with_code(RESULT_INVALID_RAISE, …)` call in one place eliminates the risk of future drift between the three callers' code attachments.
- **Module wiring is minimal and correct.** [mod.rs:66-68](../crates/sifr_hir/src/lower/mod.rs:66) declares `mod result_diagnostics;` (always-on, since the helpers are reachable from the always-on `Stmt::Raise` arm) and `#[cfg(test)] mod result_diagnostics_tests;` (test-only). Position in the alphabetical block (`protocol_diagnostics` → `result_diagnostics` → `scope_helpers`) is correct.
- **Caller path style matches siblings.** `super::result_diagnostics::invalid_*(ctx, …)` at the call sites mirrors the existing `super::flow_diagnostics::break_outside_loop(ctx)` ([statements.rs:208](../crates/sifr_hir/src/lower/statements.rs:208)), `super::flow_diagnostics::continue_outside_loop(ctx)` ([statements.rs:215](../crates/sifr_hir/src/lower/statements.rs:215)), and `super::ownership_diagnostics::immutable_parameter_reassignment(ctx, &name)` ([statements.rs:1540](../crates/sifr_hir/src/lower/statements.rs:1540)) usages — fully-qualified at each site, no top-of-file `use` shortcut. Consistent.
- **`DiagnosticCode` import in `statements.rs` is still required.** [statements.rs:48](../crates/sifr_hir/src/lower/statements.rs:48) `use sifr_diagnostics::DiagnosticCode;` remains live: greps confirm five other call sites (`RESULT_UNUSED_VALUE`, three `TYPE_MISMATCH`, `TYPE_UNPACK_SHAPE_MISMATCH`) still reference it directly. No dead-import follow-up needed.

## Test coverage

- **HIR unit test.** [result_diagnostics_tests.rs:10-21](../crates/sifr_hir/src/lower/result_diagnostics_tests.rs:10) `bare_raise_has_result_invalid_raise_code` lowers `def main():\n    raise\n` through `lower_module(parsed.suite())`, expects `Err`, and asserts `errors.iter().any(|e| e.code == Some(DiagnosticCode::RESULT_INVALID_RAISE))`. The `.any(…)` predicate is the right choice: it pins the structured-code attachment for the bare-raise emission without over-constraining downstream cascading errors. The format-string used in the assertion's failure message (`"got {errors:?}"`) is structurally informative.
- **Test plumbing matches existing convention.** `lower_source` helper signature `Result<(), Vec<LoweringError>>` (test discards the success payload) matches the lightweight pattern from sibling tests; the heavier-weight `Result<HirModule, …>` variant in [expressions_tests.rs:6-9](../crates/sifr_hir/src/lower/expressions_tests.rs:6) is appropriate when assertions read the module, which is not needed here.
- **e2e fixture is minimal and on-purpose.** [error_raise_bare.sifr](../crates/sifr/tests/e2e/fail/error_raise_bare.sifr:1):
  ```
  # expect-error: SIFR-RESULT-0003

  def main():
      raise
  ```
  The `def main()` wrapper is the smallest function shape that satisfies the e2e harness's "must have entry point" requirement; the `raise` body is the single statement that exercises the migrated path (the `raise_stmt.exc.is_none()` branch at [statements.rs:247-265](../crates/sifr_hir/src/lower/statements.rs:247)). The `# expect-error: SIFR-RESULT-0003` marker is the canonical unqualified form per [`parse_expect_error_line`](../crates/sifr/tests/e2e.rs:611) — bare canonical code, no message substring, registry state must be Active (it is per [codes.rs:1400](../crates/sifr_diagnostics/src/codes.rs:1400)).
- **No regression for the two refactored callers.** The pre-existing fixtures [error_raise_str.sifr](../crates/sifr/tests/e2e/fail/error_raise_str.sifr:1) and [error_raise_non_error.sifr](../crates/sifr/tests/e2e/fail/error_raise_non_error.sifr:1) still assert `# expect-error: SIFR-RESULT-0003` and the `cargo test -p sifr --test e2e test_e2e_fail -- error_raise --nocapture` cone covers them — the user reported `cargo test … error_raise_bare` passed; the broader `scripts/run_all_tests.sh --profile quick` run with `report_signature=e1bf653aaa770517` covers the full e2e fail directory.
- **Coverage gap (advisory, not blocking).** The new `result_diagnostics_tests.rs` adds a unit test only for the *new* code attachment (bare raise). The two refactored sites (`invalid_raise_string`, `invalid_raise_non_error`) rely entirely on their pre-existing e2e fixtures for regression coverage; there is no HIR-level unit assertion that the helper-routed string/non-Error paths still attach `RESULT_INVALID_RAISE`. Given the helpers funnel through the same private `invalid_raise(…)`, the structured-code attachment for all three is mechanically guaranteed by inspection — but a symmetric unit pair (one test per helper, asserting `.any(|e| e.code == Some(RESULT_INVALID_RAISE))` for `raise "x"` and `raise 1`) would be cheap (~20 lines) and make the helper module self-asserting. Recording as advisory; the e2e fixtures are sufficient to catch regression.

## Family choice / taxonomy fit

`SIFR-RESULT-0003` is the right reuse for bare raise. Three reinforcing signals:

1. **Sifr semantics.** Per [AGENTS.md](../AGENTS.md), Sifr enforces `Result`/`Option` over exceptions. The compiler's `raise` form is the surface that emits a Result-error value; "bare `raise`" therefore means "no error value supplied to the result-emission machinery", which is structurally a result-emission validity failure, not a control-flow / parser failure.
2. **Existing sibling emissions.** `RESULT_INVALID_RAISE` already covers two failure modes of the same statement form: string-literal-instead-of-Error (`raise "x"`) and non-Error-typed expression (`raise 1`). Bare raise (`raise`) is the third invalid `raise`-statement shape — same construct, same family, same code is the parsimonious choice.
3. **No existing alternative.** There is no `FLOW`-family or `RESULT`-family code that better fits "bare raise". `FLOW-0001..0005` cover unreachable, break/continue outside loop, nonlocal, missing return, and condition type. None of these is closer to `Stmt::Raise` validity than `RESULT-0003`.

The slice deliberately avoids minting a new code (e.g. `SIFR-RESULT-0004 — bare-raise`), which is consistent with the milestone_diag_4a slice 2b.14 review's earlier reasoning that the three `RESULT-0003` shapes share one user-facing concept ("invalid `raise`").

## Registry / docs / inventory consistency

- **No registry edits.** [codes.rs:99](../crates/sifr_diagnostics/src/codes.rs:99) `RESULT_INVALID_RAISE`, [codes.rs:1095-1105](../crates/sifr_diagnostics/src/codes.rs:1095) registry entry, [codes.rs:1400](../crates/sifr_diagnostics/src/codes.rs:1400) active-list entry, and [docs/errors/SIFR-RESULT-0003.md](../docs/errors/SIFR-RESULT-0003.md) generated page are all unchanged from milestone_diag_4a slice 2b.14. This is the correct hygiene for a slice that is only adding a third caller to a registered code — no `gen-error-docs` regeneration is required (the user did not run it; the page contents are still the registry-derived shape, which has not changed).
- **Inventory updated correctly.** [internal_docs/diagnostic_emission_inventory.md:338](../internal_docs/diagnostic_emission_inventory.md:338) extends the `SIFR-RESULT-0003` row's fixture cell from a single `error_raise_str.sifr` to a comma-separated triple `error_raise_bare.sifr, error_raise_non_error.sifr, error_raise_str.sifr`. Order is alphabetical, all three paths exist on disk, table column count is preserved (4 columns: code / description / source / fixture). The description ("invalid `raise` expression") and source ("statement lowering") cells remain unchanged and continue to read accurately for the now-three-fixture set.
- **Issue tracker entry.** [issues:91](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:91) adds `[ ] milestone_diag_8 slice 3 in progress: migrate bare \`raise\` diagnostics from raw \`ctx.error(...)\` transport to the existing \`SIFR-RESULT-0003\` invalid-raise helper path and add e2e fixture coverage.` This wording matches the slice‑1 / slice‑2 in-progress template (slices [:87](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:87) and [:89](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:89) at the same review stage), the unchecked-box state is correct for "in progress, awaiting PR/review", and the slice does not flip any earlier checklist line.

## Scope

`git diff --name-only HEAD` plus the untracked-files list:

| Path | Status | In scope? |
|---|---|---|
| [crates/sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs) | modified | ✅ module wiring |
| [crates/sifr_hir/src/lower/statements.rs](../crates/sifr_hir/src/lower/statements.rs) | modified | ✅ migrated three emission sites |
| [crates/sifr_hir/src/lower/result_diagnostics.rs](../crates/sifr_hir/src/lower/result_diagnostics.rs) | new | ✅ helper module |
| [crates/sifr_hir/src/lower/result_diagnostics_tests.rs](../crates/sifr_hir/src/lower/result_diagnostics_tests.rs) | new | ✅ HIR unit test |
| [crates/sifr/tests/e2e/fail/error_raise_bare.sifr](../crates/sifr/tests/e2e/fail/error_raise_bare.sifr) | new | ✅ e2e fixture |
| [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md) | modified | ✅ fixture list |
| [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) | modified | ✅ slice tracker |

The other untracked items in `git status` (`issues/ad-hoc-signature-invalid-fixture-adaptation-*.md`, `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`, `package.json`, `package-lock.json`, `reviews/ownership-mutability-boundary-root-cause-review-pass{1,2}.md`, `verification/leetcode/`) are pre-existing artefacts from other branches / streams and are explicitly not part of this slice's diff. They should not block this review.

**Scope-creep check.** Refactoring the two existing inline `error_with_code` calls (string raise, non-Error raise) into helper wrappers when the slice's headline is "migrate bare raise" is a borderline scope question. I judge it acceptable because (a) the helper module already had to be created to host `invalid_bare_raise` (no pre-existing `result_diagnostics.rs`), and orphaning the bare-raise helper next to two siblings that *don't* go through it would be the worse outcome; (b) it matches the slice‑1 (`flow_diagnostics::missing_return_value`) and slice‑2 (`flow_diagnostics::invalid_condition_type`) shape, where each `milestone_diag_8` slice introduces or extends the family helper module rather than dropping a single one-off wrapper; (c) the message text is byte-identical for the two refactored sites, so there is no behavioral regression risk. I do not consider this scope creep.

**No fallback path.** All three call sites now route through `error_with_code` (which sets `code = Some(…)`); none of the previous uncoded `ctx.error(…)` paths remains for `Stmt::Raise`. There is no compatibility shim, no dual emission, no feature flag.

## Local validation reported by user (cross-checked against slice norms)

| Step | Notes |
|---|---|
| `cargo fmt --check` | Required gate; reported clean. |
| `git diff --check` | Whitespace gate; reported clean. |
| `cargo test -p sifr_hir bare_raise_has_result_invalid_raise_code -- --nocapture` | New unit test passes. |
| `cargo test -p sifr --test e2e test_e2e_fail -- error_raise_bare --nocapture` | New e2e fixture passes. |
| `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/error_raise_bare.sifr` | "expected nonzero user diagnostic" — confirms CLI path emits the diagnostic and exits non-zero. |
| `cargo clippy -p sifr_hir --no-deps -- -D warnings` | Required gate; reported clean. |
| `scripts/run_all_tests.sh --profile quick` | `report_signature=e1bf653aaa770517`, `wall_time=804.70s`. Signature matches the merged report signature recorded for slice 1 ([issues:88](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:88)) and slice 2 ([issues:90](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:90)). The warm wall-time / group-skew advisories are the same advisory class previously accepted on slices 1 and 2, not new failures. |

The reported gate set matches the milestone_diag_8 slice norm. I did not re-run the gates myself (this is a no-modification review), but the steps reported are sufficient and the signature equality is a strong consistency signal.

## Residual risks (advisory)

### R1 — Registry message template `"invalid raise expression of type {actual}"` does not match any of the three emission texts (pre-existing, extended by this slice)

[codes.rs:1101](../crates/sifr_diagnostics/src/codes.rs:1101) declares `RESULT_INVALID_RAISE`'s `message_template` as `"invalid raise expression of type {actual}"`. None of the three actual emissions matches this template:

- string raise: `"raise requires an Error class instance — `raise "message"` is not allowed, use e.g. `raise ValueError("message")`"`
- non-Error raise: `"raise requires an Error class instance — `{type_name}` is not an Error class"`
- bare raise: `"bare 'raise' without an expression is not supported"`

The template's `{actual}` placeholder is *especially* a poor fit for bare raise, which has no "type" at all. This is a **pre-existing inconsistency** (the template was set during registry population before the helper-module migration; the prior milestone_diag_4a slice 2b.14 review also did not flag it because the registry template is purely doc-shape and is not load-bearing for runtime emission — `error_with_code` ignores the template and passes the explicit `message: String` through verbatim per [mod.rs:234-241](../crates/sifr_hir/src/lower/mod.rs:234)). This slice extends the divergence (one new emission text) rather than introducing it.

**Recommendation (out-of-scope for this slice; track separately):** when a future slice does a `gen-error-docs` regeneration, also revisit the template to either (a) generalise it (e.g. `"invalid raise: {detail}"` with a single `detail` arg) or (b) split into per-shape codes. Either is a follow-up, not a slice-3 fix.

### R2 — Bare-raise wording diverges in tone from the two sibling RESULT-0003 emissions

The two pre-existing emissions read `"raise requires an Error class instance — …"`. The new bare-raise emission reads `"bare 'raise' without an expression is not supported"` — phrased as a "not supported" passive observation rather than as a "raise requires …" prescriptive guidance. This is the verbatim pre-migration text (the slice does not re-word it, only re-codes it), so there is no regression. However, future work that aligns RESULT-0003 family wording could consider rewording to e.g. `"raise requires an Error class instance — bare \`raise\` is not allowed, use e.g. \`raise ValueError(\"…\")\`"` for parallelism with the string-raise emission. **Out-of-scope advisory; no action required for this slice.**

### R3 — The two refactored sites have no HIR-level unit test (only e2e fixture coverage)

See "Coverage gap" under "Test coverage" above. Adding `string_raise_has_result_invalid_raise_code` and `non_error_raise_has_result_invalid_raise_code` companions in `result_diagnostics_tests.rs` would symmetrise the helper module's test surface. Cheap, optional. **Advisory, not blocking** — the e2e fixtures are an authoritative regression net.

## Verdict

**Satisfied.** No blockers. No required nits. Proceed to PR. The advisory items R1, R2, R3 are explicitly out-of-scope for this slice and should be deferred to follow-up work or a future RESULT-family hygiene pass.
