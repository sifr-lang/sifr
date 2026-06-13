---
title: Match diagnostics slice — pass 2 review
slice: milestone_diag_4a 2b.18
branch: codex/semantic-diagnostics-diag-4a-match-diagnostics
---

# Review — `milestone_diag_4a` slice 2b.18 (match diagnostics) — pass 2

Branch: `codex/semantic-diagnostics-diag-4a-match-diagnostics`
Scope: re-audit the slice after the pass-1 hygiene finding (F1) was addressed.
The fix updated the registry message template and declared/dedupe args for
`SIFR-MATCH-0001` to the representative enum non-exhaustive shape, and the
generated docs were regenerated to match. Pass-1 verdict was already
"ready for PR"; this pass confirms whether the F1 fix is clean and whether
any new blockers have appeared.

Review style: read-only audit, no files modified.

## TL;DR

The F1 fix is correct and minimal: registry entry for `SIFR-MATCH-0001` now
declares the enum non-exhaustive shape, the doc page and the
`internal_docs/diagnostic_codes.md` table are regenerated consistently, and
no source/test code was touched. All pass-1 findings other than F1 are
either nit-level or discussion-only and remain fine to defer. No new
blockers introduced by the fix; no behavioral regressions.

**Recommendation: ready to commit and open as a PR.**

## What was reviewed in this pass

Diff vs. `origin/main` (working tree, no commits yet on the branch):

- New helpers + tests:
  - [crates/sifr_hir/src/lower/match_diagnostics.rs](crates/sifr_hir/src/lower/match_diagnostics.rs)
  - [crates/sifr_hir/src/lower/match_diagnostics_tests.rs](crates/sifr_hir/src/lower/match_diagnostics_tests.rs)
- Module wiring:
  - [crates/sifr_hir/src/lower/mod.rs:42](crates/sifr_hir/src/lower/mod.rs:42) — `mod match_diagnostics;` + `#[cfg(test)] mod match_diagnostics_tests;`
- Call-site migration in `lower_match` / `lower_pattern`:
  - [crates/sifr_hir/src/lower/statements.rs:664](crates/sifr_hir/src/lower/statements.rs:664), [statements.rs:803](crates/sifr_hir/src/lower/statements.rs:803), [statements.rs:841](crates/sifr_hir/src/lower/statements.rs:841), [statements.rs:861](crates/sifr_hir/src/lower/statements.rs:861), [statements.rs:967](crates/sifr_hir/src/lower/statements.rs:967)
- Registry + docs alignment (the F1 follow-up):
  - [crates/sifr_diagnostics/src/codes.rs:946](crates/sifr_diagnostics/src/codes.rs:946), [codes.rs:957](crates/sifr_diagnostics/src/codes.rs:957), [codes.rs:968](crates/sifr_diagnostics/src/codes.rs:968)
  - [docs/errors/SIFR-MATCH-0001.md](docs/errors/SIFR-MATCH-0001.md), [docs/errors/SIFR-MATCH-0002.md](docs/errors/SIFR-MATCH-0002.md), [docs/errors/SIFR-MATCH-0003.md](docs/errors/SIFR-MATCH-0003.md)
  - [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md)
- Fixture re-keys:
  - [crates/sifr/tests/e2e/fail/enum_match_non_exhaustive.sifr](crates/sifr/tests/e2e/fail/enum_match_non_exhaustive.sifr), [match_invalid_field_name.sifr](crates/sifr/tests/e2e/fail/match_invalid_field_name.sifr), [match_non_exhaustive_literal.sifr](crates/sifr/tests/e2e/fail/match_non_exhaustive_literal.sifr), [match_non_exhaustive_optional.sifr](crates/sifr/tests/e2e/fail/match_non_exhaustive_optional.sifr), [match_non_exhaustive_union.sifr](crates/sifr/tests/e2e/fail/match_non_exhaustive_union.sifr), [match_type_mismatch_guard.sifr](crates/sifr/tests/e2e/fail/match_type_mismatch_guard.sifr)
- Tracker entry:
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53)

## F1 fix audit (registry + docs alignment for `SIFR-MATCH-0001`)

The fix is in the right places and only in those places:

- `crates/sifr_diagnostics/src/codes.rs` (registry) for all three
  `SIFR-MATCH-*` codes, plus `SIFR-MATCH-0002` / `SIFR-MATCH-0003` got their
  templates polished to exactly match emitted-message shapes (modulo
  surrounding single-quotes around values, which is normal across the
  registry — see pass 1).
- `docs/errors/SIFR-MATCH-000{1,2,3}.md` were regenerated with the new
  templates and arg lists. Each doc page reflects the registry one-for-one,
  which is what `gen-error-docs` produces.
- `internal_docs/diagnostic_codes.md` table rows for the three codes mirror
  the registry exactly.
- No source code, helper, test, or fixture was touched as part of F1 — the
  fix is doc/registry-only, which is the right scope.

Specifically for `SIFR-MATCH-0001`, the new registry shape is:

| Field | Value |
|---|---|
| Template | `non-exhaustive match: enum {enum_name} has uncovered variants: {uncovered}` |
| Declared args | `enum_name (message+json)`, `uncovered (message+json)` |
| Dedupe args | `enum_name`, `uncovered` |
| Representative fixture | `crates/sifr/tests/e2e/fail/enum_match_non_exhaustive.sifr` |

This is pass-1's Option 1 ("pick the most representative shape") executed
with the *enum* form. Pass 1 had floated the union form as the most likely
representative, but the enum form is equally defensible — the rep fixture
on the registry row is the enum case, so template-vs-fixture alignment is
now self-consistent. Either choice resolves F1; this one is fine.

`SIFR-MATCH-0002` template went from `match guard must be bool, got
{actual}` → `match guard must be a bool expression, got {actual}`, matching
the helper-emitted text verbatim (modulo value quoting). `SIFR-MATCH-0003`
went from `class pattern field {field} does not exist on {class_name}` →
`class {class_name} has no field {field}`, again matching the helper.

`docs/errors/SIFR-MATCH-0001.md` previously cited the legacy template; the
update is faithful and complete (no other parts of the doc reference the old
template).

## Carry-over from pass 1

Pass 1 surface area was mostly already clean. Re-checked here:

- **F2 — one code, three shapes for `SIFR-MATCH-0001`.** Still discussion-
  only. Two message shapes (union, literal) emit under this code without
  matching the registry's declared `enum_name`/`uncovered` argument
  vocabulary. This is a known precedent shared with `SIFR-OWN-0001` and is
  out of scope for this slice. Worth flagging again for the structured-args
  milestone but not blocking.
- **F3 — pre-joined `&str` for `uncovered`.** Unchanged. Same justification.
- **F4 — call-site style (`use super::...;` vs full path).** Unchanged.
  Both styles already coexist in `statements.rs`.
- **F5 — guard-test fragility (`n + 1`).** Unchanged. Mitigated by the
  parallel e2e fixture.
- **F6 — tracker entry.** The tracker now lists slice 2b.17 as merged and
  slice 2b.18 as in-progress with `PR: pending` (
  [issues/...md:53](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53) ).
  PR URL still needs to be filled in once the PR opens; same as pass 1, no
  action now.

## New observations introduced by the F1 fix

None of the following are blockers; they are documented here only so the
next pass (or a future structured-args slice) doesn't have to re-derive them.

### O1. Registry-declared args for `SIFR-MATCH-0001` cover only one of three emitted shapes

The registry now declares `enum_name` + `uncovered`. Two of the three
helpers under this code emit a different vocabulary:

- `non_exhaustive_union` emits `subject_type` + `uncovered`
  ([match_diagnostics.rs:12](crates/sifr_hir/src/lower/match_diagnostics.rs:12)).
- `non_exhaustive_literal` emits `subject_type` only and no `uncovered`
  ([match_diagnostics.rs:30](crates/sifr_hir/src/lower/match_diagnostics.rs:30)).

This is the same observation as pass-1 F2, but now lives explicitly in the
registry. Today the args are documentation-only (template strings, not
runtime format strings), so this is harmless. When structured args land,
this code will need either sub-codes or a `kind` discriminator + a wider
arg vocabulary. Not actionable for slice 2b.18.

### O2. `SIFR-MATCH-0003` registry declares `field`/`class_name` but the helper also passes `available_fields`

The registry's declared/dedupe args are `field`, `class_name`. The actual
emitted message includes a third value, `available_fields`, formatted into
the trailing `— available fields: x, y` clause
([match_diagnostics.rs:39](crates/sifr_hir/src/lower/match_diagnostics.rs:39)).
The doc-only template explicitly does not surface this, which is consistent
with how `SIFR-MATCH-0002` documents only `actual` despite the helper
quoting the value. Fine for now; flagging only because structured-args work
will likely promote `available_fields` to a first-class arg.

### O3. Helper signature parameter `field_name` vs registry/template arg `field`

Cosmetic. The helper takes `field_name: &str`
([match_diagnostics.rs:42](crates/sifr_hir/src/lower/match_diagnostics.rs:42))
while the registry/template uses `field`. Helper parameter names are
internal-only and don't propagate to the registry, so no functional impact.
If the next round prefers strict parity, renaming the helper parameter to
`field` is a one-line touch-up. Not blocking.

## Correctness, regressions, and safety re-check

- No new `unwrap` / `expect` / `panic!` in non-test code
  (`match_diagnostics.rs` is pure formatting + `ctx.error_with_code`).
- Migration completeness: a fresh grep for the original message strings
  (`match guard must be a bool expression`, `non-exhaustive match: type`,
  `non-exhaustive match: enum`, `cannot be fully covered by literal
  patterns`, `class '` ... `has no field '`) finds them only in the new
  helpers and the new unit-test file. No raw `ctx.error` survivor for the
  five migrated diagnostics.
- The two raw `ctx.error` calls remaining in `lower_pattern`
  ("class pattern class name must be a simple name" at
  [statements.rs:947](crates/sifr_hir/src/lower/statements.rs:947) and
  "tuple pattern requires subject of tuple type, got '...'" at
  [statements.rs:996](crates/sifr_hir/src/lower/statements.rs:996)) remain
  out of scope for this slice — same situation as pass 1. They have no
  allocated `SIFR-MATCH-*` code yet.
- All e2e fixtures use substring matching; the migrated `expect-error`
  prefixes (`SIFR-MATCH-0001/2/3`) align with what the helpers emit.
- Test wiring: the new module is correctly gated under `#[cfg(test)]` at
  [mod.rs:43](crates/sifr_hir/src/lower/mod.rs:43); release builds are
  unaffected.
- Repo-wide check that no other match-shape fixtures still use
  `SIFR-TYPE-0001` (`grep -n SIFR-TYPE-0001 crates/sifr/tests/e2e/fail/*.sifr`)
  returns only `map_callable_arity_mismatch.sifr` and
  `stdlib_test_assert_eq_type_mismatch.sifr` — neither is a `match`
  statement diagnostic, so the legacy bucket is fully drained for this
  domain.
- Workspace clippy / fmt / docs-sync / schema-sync gates already passed
  per the user-supplied evidence; the F1 fix's diff doesn't touch anything
  those gates couldn't catch on a re-run.

## Validation evidence (as reported by the user)

After the F1 fix:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs` — passed.
- `cargo fmt --check` — passed.
- `python3 scripts/check_diagnostic_docs_sync.py` — passed.
- `python3 scripts/check_diagnostic_schema_sync.py` — passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` — passed.
- `cargo test -p sifr_hir match_diagnostics_tests` — passed.
- `cargo test -p sifr_diagnostics` — passed.
- `cargo test -p sifr --test e2e -- test_e2e_fail` — passed.

Pre-F1 broader sweep (pass-1 evidence):

- `cargo test -p sifr -- --skip test_e2e_pass` — passed.
- `cargo clippy --workspace -- -D warnings` — passed.
- `scripts/run_all_tests.sh --profile quick` — report_signature
  `e1bf653aaa770517`, wall_time 141.16s.

The F1 follow-up changed only the registry literal entries and the
generated docs/internal table, which the docs-sync and schema-sync gates
cover end-to-end. I did not re-run any of these locally; the change set
is small and consistent with what those gates already cover.

## Verdict

The F1 hygiene finding is resolved cleanly, no new blockers were introduced,
and no pass-1 finding above nit/discussion severity remains. The slice is
**ready to commit and open as a PR**.

## Suggested action items (in priority order)

1. Open the PR; once GitHub assigns it a URL, flip `PR: pending` →
   the PR URL on
   [issues/...md:53](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53).
2. (Optional, future slice) Decide whether `SIFR-MATCH-0001`'s three
   message shapes split into sub-codes or stay unified with a `kind` arg
   (pass-1 F2 / pass-2 O1); revisit helper signatures (pass-1 F3 / pass-2
   O2) when structured args land.
3. (Optional, cosmetic) Rename the helper parameter `field_name` →
   `field` in `invalid_class_pattern_field` for strict registry/helper
   parameter parity (pass-2 O3).
