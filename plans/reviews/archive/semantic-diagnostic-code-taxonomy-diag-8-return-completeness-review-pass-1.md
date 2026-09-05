# Review pass 1 — `milestone_diag_8` slice 1: return-completeness migration to `SIFR-FLOW-0004`

## Summary

Reviewed the uncommitted implementation that migrates the function return-completeness diagnostic from raw `ctx.error(...)` transport to a dedicated `SIFR-FLOW-0004` code with a domain helper, registry entry, generated docs, internal index entry, inventory entry, HIR regression assertion, and e2e fail fixture, plus the in-progress entry on the phase issue tracker.

**Result: satisfied.** No blockers. One minor stylistic nit. Implementation is correct, in-scope, internally consistent, and introduces no fallback or compatibility paths.

## Files inspected

- [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs) — constant, registry entry, active-list inclusion
- [crates/sifr_hir/src/lower/flow_diagnostics.rs](crates/sifr_hir/src/lower/flow_diagnostics.rs) — new `missing_return_value` helper
- [crates/sifr_hir/src/lower/typing_and_functions.rs](crates/sifr_hir/src/lower/typing_and_functions.rs:830) — migrated call site
- [crates/sifr_hir/src/lower/expressions_tests.rs:706](crates/sifr_hir/src/lower/expressions_tests.rs:706) — strengthened HIR regression test
- [crates/sifr/tests/e2e/fail/missing_return_value.sifr](crates/sifr/tests/e2e/fail/missing_return_value.sifr) — new e2e fail fixture
- [docs/errors/SIFR-FLOW-0004.md](docs/errors/SIFR-FLOW-0004.md) — generated per-code page
- [docs/errors/diagnostic-codes.md](docs/errors/diagnostic-codes.md) — public index updated
- [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md) — internal index updated
- [internal_docs/diagnostic_emission_inventory.md](internal_docs/diagnostic_emission_inventory.md) — inventory updated
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:87](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:87) — slice-1 in-progress entry

## Correctness

- **Constant** (codes.rs:74): `FLOW_MISSING_RETURN_VALUE` declared as `Self::new("SIFR-FLOW-0004", Severity::Error)`. Family/severity match the registry entry and the docs page.
- **Registry entry** (codes.rs:913–923): family `FLOW`, summary `"Function may finish without returning a required value."`, error severity, owner `sifr_hir::lower::typing_and_functions` (this is the actual emitting module per typing_and_functions.rs:830), representative fixture path matches the new fixture, message template uses `{function}` and `{return_type}` placeholders, declared args list both as `MessageAndJson`, dedupe args list both. Inserted between `SIFR-FLOW-0003` and `SIFR-FLOW-0901` so the registry stays lexicographically ordered.
- **Active list** (codes.rs:1372): `FLOW_MISSING_RETURN_VALUE` inserted between `FLOW_INVALID_NONLOCAL` and `FLOW_UNREACHABLE_STATEMENT`, matching declaration order.
- **Helper** (flow_diagnostics.rs:23–30): plain wrapper around `ctx.error_with_code(DiagnosticCode::FLOW_MISSING_RETURN_VALUE, …)`. Format string substitutes `function_name` and `return_type` into wording that is byte-identical to the registry `message_template` after placeholder substitution. Helper name and signature mirror the existing FLOW helpers (`break_outside_loop`, `continue_outside_loop`, `recursive_nonlocal_nested_function`).
- **Call site** (typing_and_functions.rs:830–849): only the `Ok(false)` branch (the actual user-facing missing-return diagnostic) is migrated to the coded helper. The `Ok(true)` no-op and the `Err(_)` `ctx.warn(...)` invalid-CFG path are correctly left untouched — that warn path is not a user-facing semantic diagnostic but an internal panic-boundary advisory, so it stays out of the FLOW migration. No fallback path was introduced; the helper carries the active code unconditionally.
- **HIR regression** (expressions_tests.rs:710–714): assertion now requires both the message substring and `e.code == Some(...FLOW_MISSING_RETURN_VALUE)`. This pins the structured identity rather than only the human text. The two adjacent negative-cascade tests (lines 52–58, 76–82) remain message-substring only because they assert the *absence* of a missing-return diagnostic; converting them to code checks is unnecessary and out of slice scope.

## Registry / docs consistency

- `docs/errors/SIFR-FLOW-0004.md` is the generator-shaped page (correct generator banner, all fields populated, no manual editing).
- The public index row (`docs/errors/diagnostic-codes.md:83`) and internal index row (`internal_docs/diagnostic_codes.md:107`) match the registry: id, family, state Active, severity Error, docs path, fixture path, owner module, message template, declared/dedupe args, no severity override, `fix_all_eligible=false`.
- The inventory row (`internal_docs/diagnostic_emission_inventory.md:323`) maps `SIFR-FLOW-0004` to "missing return value on some control-flow paths / function return-completeness validation / new fixture path", and lands in the correct family-grouped section between `SIFR-FLOW-0003` and `SIFR-MATCH-0001`.
- The registry placeholder validator (`codes.rs:1633`, executed by the package tests) requires both `{function}` and `{return_type}` in the template to be declared as `MessageAndJson` args; both are. Dedupe-arg subset check (`codes.rs:1624`) holds for both.

## Fixture validity

- `crates/sifr/tests/e2e/fail/missing_return_value.sifr` is minimal and on-purpose:
  ```
  # expect-error: SIFR-FLOW-0004

  def choose(flag: bool) -> int:
      if flag:
          return 1
  ```
  This is the smallest function shape that exercises the migrated path: a non-`None` return annotation, no `yield`, and a CFG where `always_exits()` returns `false`. The `# expect-error: SIFR-FLOW-0004` marker is the canonical unqualified form per `validate_expected_error_code` (e2e.rs:757) and `parse_expect_error_line` (e2e.rs:611). The fixture's representative-fixture path string in the registry exactly matches this on-disk path.
- Independent re-run of `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/missing_return_value.sifr` reproduced the user's stated outcome: exit 1 with `function 'choose' must return a value of type 'int' on all control-flow paths`. The leading `type error:` label is the family-default human prefix for non-PARSE/CODEGEN/BUILD/WORKSPACE codes from `diagnostic_label_for_code_str` (sifr_driver/src/diagnostics.rs:130–138); that's pre-existing FLOW-family behavior and not a regression introduced here.

## Fallback / compatibility check

No fallback or compatibility shim is introduced. Specifically:

- The helper does not gate the code on any feature flag, optional context, or "legacy text" branch.
- The migrated call site directly replaces `ctx.error(...)` with the coded helper; there is no parallel codeless emission and no `code.unwrap_or(...)` style downgrade.
- The catch-unwind invalid-CFG branch (`Err(_) => ctx.warn(...)`) is *not* a user-facing semantic-diagnostic fallback — it is the existing panic-boundary advisory and its behavior is identical before and after this slice.
- The `LoweringError`-via-`ctx.error` legacy transport remains accessible for other (out-of-scope) call sites, but no new uncoded emission of return-completeness text is added; the previous one is removed in the same change.

## Issue tracker placement

The new bullet on line 87 ("`milestone_diag_8` slice 1 in progress: …") is added immediately after the `milestone_diag_7` slice 5 reviewer-satisfied bullet, which matches the chronological ordering of "Execution Status". The wording correctly identifies this as the return-completeness migration targeting `SIFR-FLOW-0004`. Once the PR lands and review concludes, the slice 1 bullet should be flipped to `[x] … implementation complete and reviewer-satisfied: … PR: …`, and a separate `[x] agent implementation review for milestone_diag_8 slice 1 …` bullet should be appended, mirroring slice 5's pair pattern (lines 85–86). That follow-up is not a blocker for this review.

## Validation cross-check

The user reported the following local validation as passing:

- `cargo fmt --check`
- `cargo run -q -p sifr_diagnostics --bin gen-error-docs`
- `cargo test -p sifr_diagnostics`
- `cargo test -p sifr_hir test_non_none_return_annotation_requires_exhaustive_returns -- --nocapture`
- `cargo test -p sifr --test e2e test_e2e_fail -- missing_return_value --nocapture` (242 fail tests; ok)
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/missing_return_value.sifr` (exit 1, expected text)

I independently re-ran the CLI `check` invocation and reproduced exit 1 with the expected diagnostic text. The other commands were not re-run since the implementation is small, in-scope, and the listed validations cover the relevant gates: registry tests (template/declared-args/dedupe-args/markdown-safety), generated docs drift, the HIR regression that pins both the message and the code, and the full e2e fail corpus including the new fixture.

CLAUDE.md notes that `scripts/run_all_tests.sh --profile quick` is the authoritative pre-PR gate. That isn't listed in the validation summary and would be the recommended next step before opening the PR. Not a review blocker, but worth running so the slice's "merged + reviewer-satisfied" entry can record the standard `report_signature`/`wall_time` fingerprint that all other recently-merged slice entries record.

## Findings

### Blockers

None.

### Non-blocking nits

1. **Stylistic — fully qualified path in test assertion.** [crates/sifr_hir/src/lower/expressions_tests.rs:713](crates/sifr_hir/src/lower/expressions_tests.rs:713) writes `e.code == Some(sifr_diagnostics::DiagnosticCode::FLOW_MISSING_RETURN_VALUE)` even though the file already imports `use sifr_diagnostics::DiagnosticCode;` at line 2. Neighboring tests (lines 93, 107, 121, 134, …) consistently use the bare `DiagnosticCode::OWN_USE_AFTER_MOVE` short form. Switching to `DiagnosticCode::FLOW_MISSING_RETURN_VALUE` matches local convention; pure cosmetics, not a correctness issue.

### Observations (no action required for this slice)

- Untracked `verification/leetcode/full_corpus_failure_taxonomy_20260402_live.json` contains stale `"type error: function '…' must return a value of type '…' on all control-flow paths"` snapshots. Those files are listed as untracked in `git status` and are out of scope for this slice. If they are committed in a later slice, they may benefit from being re-keyed to `SIFR-FLOW-0004` to reflect the new structured identity, but the snapshot text itself remains accurate.
- The e2e fail-harness `match_compile_failure_expectations` (e2e.rs:870) only enforces that every expected diagnostic exists; it does not fail on extras. The new fixture is small enough that in practice only `SIFR-FLOW-0004` fires, so the unqualified `# expect-error` is appropriate. No change recommended here.

## Verdict

**Satisfied.** The slice is correctly scoped, registry/docs are internally consistent, the fixture exercises exactly the migrated path, the HIR regression now pins both the message and the structured code, and no fallback or compatibility path was introduced. The single nit is cosmetic and can be addressed inline before the PR is opened or left as-is.
