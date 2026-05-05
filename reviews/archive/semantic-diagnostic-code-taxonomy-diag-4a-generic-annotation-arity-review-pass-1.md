## `milestone_diag_4a` slice 2b.13 — generic type alias and class annotation arity/surface diagnostics migration to active `SIFR-TYPE-0007` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-generic-annotation-arity`.
- Target: migrate the three remaining raw `ctx.error(format!(...))` call sites in [`resolve_annotation_expr`](../crates/sifr_hir/src/lower/typing_and_functions.rs:391) — generic type alias arity, generic class non-generic-subscript, and generic class arity — onto the active `DiagnosticCode::TYPE_INVALID_ANNOTATION` (`SIFR-TYPE-0007`) channel via the existing `invalid_type_annotation` helper, with one e2e fail fixture per call site and unit-test coverage in `expressions_tests.rs` / `type_alias_tests.rs` asserting both rendered substring and structured `code`.
- Files changed (per `git status`):
  - [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:601) — three call-site migrations from `ctx.error` to `invalid_type_annotation`.
  - [crates/sifr_hir/src/lower/expressions_tests.rs](../crates/sifr_hir/src/lower/expressions_tests.rs:1758) — added `DiagnosticCode` import and tightened two assertions to also pin `e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)`.
  - [crates/sifr_hir/src/lower/type_alias_tests.rs](../crates/sifr_hir/src/lower/type_alias_tests.rs:179) — added `DiagnosticCode` import and tightened one assertion the same way.
  - [crates/sifr/tests/e2e/fail/generic_type_alias_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/generic_type_alias_wrong_arity.sifr:1) — new fixture pinning the alias arity emission.
  - [crates/sifr/tests/e2e/fail/generic_class_non_generic_subscript.sifr](../crates/sifr/tests/e2e/fail/generic_class_non_generic_subscript.sifr:1) — new fixture pinning the non-generic-class subscript emission.
  - [crates/sifr/tests/e2e/fail/generic_class_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/generic_class_wrong_arity.sifr:1) — new fixture pinning the class arity emission.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:47) — slice 2b.12 flipped to merged with PR #1684; slice 2b.13 line added as "in progress / PR: pending".
- Validation rerun by reviewer:
  - `cargo fmt --check` → passed (no diff).
  - `cargo run -q -p sifr -- check` against each new fixture, observing the rendered diagnostic text matches the `expect-error` substring byte-for-byte (see F4).
  - `cargo run -q -p sifr -- check` against pre-existing [recursive_generic_type_alias_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/recursive_generic_type_alias_wrong_arity.sifr:1) confirms the migrated path is also exercised there with text `generic type alias 'Node' expects 1 type argument(s), got 2`.

## Verdict

**Satisfied — no blockers.** The taxonomy choice is correct, the helper reuse is clean, the diff is tightly scoped, and the three new fixtures cover one call site each. The slice closes the last three "annotation arity/surface" emissions that slice 2b.12 had explicitly deferred (per F7 of [reviews/semantic-diagnostic-code-taxonomy-diag-4a-unknown-types-review-pass-1.md](../reviews/semantic-diagnostic-code-taxonomy-diag-4a-unknown-types-review-pass-1.md:71)). Recommend merge. Two non-blocking observations under Residual risks.

## Findings

### F1 — Taxonomy choice: `SIFR-TYPE-0007` is the right code for all three call sites

The active registry entry at [codes.rs:625-635](../crates/sifr_diagnostics/src/codes.rs:625) declares `SIFR-TYPE-0007` with summary `"Invalid type annotation shape."`, message template `"invalid type annotation for {annotation_kind}"`, and `owner_module = "sifr_hir::lower::typing_and_functions"` — i.e. the registry slot is pre-allocated to cover annotation-shape diagnostics in this exact module. All three migrated emissions are *type-annotation surface* errors at the `Subscript` form:

- "generic type alias `'X'` expects N type argument(s), got M" — the user wrote `Pair[int, str]` for an alias declared with `[T]`. The annotation is structurally invalid; the underlying alias name *did* resolve.
- "class `'X'` does not declare type parameters; use `class X[T]: ...`" — the user wrote `LegacyBox[int]` for a non-generic class. Again, the name resolved, but the subscript form is invalid for the resolved entity.
- "generic class `'X'` expects N type argument(s), got M" — the user wrote `Pair[int, str]` for a class declared with `[T]`. Same family.

None of these are name-resolution failures (which would be `NAME_UNKNOWN_TYPE = SIFR-NAME-0003`, owned by the new `unknown_type` helper at [typing_and_functions.rs:384-389](../crates/sifr_hir/src/lower/typing_and_functions.rs:384) — the unresolved-base case is already handled at [typing_and_functions.rs:707-710](../crates/sifr_hir/src/lower/typing_and_functions.rs:707) and [406-409](../crates/sifr_hir/src/lower/typing_and_functions.rs:406)). They are not protocol-bound failures (`PROTO-*`), nor missing-annotation (`TYPE-0004`), nor type-mismatch (`TYPE-0002`). They are exactly the same family as the existing TYPE-0007 sites in this same `match` arm — `dict`/`tuple`/`Result`/`Callable` arity and shape — at [typing_and_functions.rs:444, 460, 470, 505, 525, 549, 562, 581, 715](../crates/sifr_hir/src/lower/typing_and_functions.rs:444). Same code path, same registry slot, same helper. No sibling code is closer.

### F2 — Helper reuse is correct; no new helper introduced

The slice does *not* add a new helper — it reuses the pre-existing `invalid_type_annotation` at [typing_and_functions.rs:380-382](../crates/sifr_hir/src/lower/typing_and_functions.rs:380), which was introduced by slice 2b.8 (commit `3b23ff52`). All three migrated call sites pass `ctx` plus a `format!(...)` String; no clones or allocations are added beyond what the pre-migration `ctx.error(format!(...))` already had. The helper signature `fn invalid_type_annotation(ctx: &mut LowerCtx, message: impl Into<String>)` accepts both `&'static str` and `String`, so the new `format!`-built call arguments compose cleanly. Module visibility (`fn`, not `pub(super)`) remains correct — both consumers are in the same module.

### F3 — All three migrated call sites match the same syntactic conversion shape

Each migration is a mechanical `ctx.error(format!(...))` → `invalid_type_annotation(ctx, format!(...))` swap, with no message text changes:

- Alias arity at [typing_and_functions.rs:601-611](../crates/sifr_hir/src/lower/typing_and_functions.rs:601): identical message body.
- Class non-generic subscript at [typing_and_functions.rs:645-652](../crates/sifr_hir/src/lower/typing_and_functions.rs:645): identical.
- Class arity at [typing_and_functions.rs:654-664](../crates/sifr_hir/src/lower/typing_and_functions.rs:654): identical.

I diffed each against the pre-migration text from `git diff HEAD -- crates/sifr_hir/src/lower/typing_and_functions.rs` and confirmed byte-equivalence of the rendered substrings. Returning `Type::Any` after each emission is preserved, so downstream typing semantics are unchanged. No behavior regression in pass-suite or transport-layer test surfaces.

### F4 — Fixture coverage is sufficient — one fixture per migrated call site

| Call site | Code path | Fixture | `expect-error` substring | Direct `cargo run` rendered output |
|---|---|---|---|---|
| Generic type alias arity | [typing_and_functions.rs:601-611](../crates/sifr_hir/src/lower/typing_and_functions.rs:601) | [generic_type_alias_wrong_arity.sifr:1](../crates/sifr/tests/e2e/fail/generic_type_alias_wrong_arity.sifr:1) | `generic type alias 'Pair' expects 1 type argument(s), got 2` | `type error: [main] generic type alias 'Pair' expects 1 type argument(s), got 2` |
| Class non-generic subscript | [typing_and_functions.rs:645-652](../crates/sifr_hir/src/lower/typing_and_functions.rs:645) | [generic_class_non_generic_subscript.sifr:1](../crates/sifr/tests/e2e/fail/generic_class_non_generic_subscript.sifr:1) | ``class 'LegacyBox' does not declare type parameters; use `class LegacyBox[T]: ...` `` | ``type error: [main] class 'LegacyBox' does not declare type parameters; use `class LegacyBox[T]: ...` `` |
| Generic class arity | [typing_and_functions.rs:654-664](../crates/sifr_hir/src/lower/typing_and_functions.rs:654) | [generic_class_wrong_arity.sifr:1](../crates/sifr/tests/e2e/fail/generic_class_wrong_arity.sifr:1) | `generic class 'Pair' expects 1 type argument(s), got 2` | `type error: [main] generic class 'Pair' expects 1 type argument(s), got 2` |

Each fixture's `expect-error` line uses the form `# expect-error: SIFR-TYPE-0007: <substring>`, which the e2e harness parses via `parse_expected_error` ([e2e.rs:596](../crates/sifr/tests/e2e.rs:596)) into a code + substring pair, then asserts at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) that some emitted diagnostic matches both halves. I confirmed all three pairs match end-to-end with direct `cargo run -q -p sifr -- check` invocations against each fixture (rendered output column above).

A single fixture per call site is the convention used throughout slices 2b.6–2b.12. No additional e2e fixture is required.

### F5 — Unit-test assertions correctly tighten to dual-condition (message + code)

The three pre-existing unit tests that already asserted the rendered messages are now extended to also pin the structured code:

- [expressions_tests.rs:1758-1768](../crates/sifr_hir/src/lower/expressions_tests.rs:1758) — `test_generic_class_subscript_requires_declared_type_params` now requires `e.message.contains("does not declare type parameters") && e.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)`.
- [expressions_tests.rs:1770-1781](../crates/sifr_hir/src/lower/expressions_tests.rs:1770) — `test_generic_class_subscript_arity_mismatch_errors` same dual condition with substring `"expects 1 type argument(s), got 2"`.
- [type_alias_tests.rs:179-189](../crates/sifr_hir/src/lower/type_alias_tests.rs:179) — `test_generic_type_alias_wrong_arity_still_errors` same dual condition with full equality on `"generic type alias 'Pair' expects 1 type argument(s), got 2"`.

These dual-condition assertions are exactly the diagnostic-transport contract the slice cadence has been enforcing since slice 2a (`error.code: Option<DiagnosticCode>` is populated by `error_with_code`, `None` by raw `error`). The two new `use sifr_diagnostics::DiagnosticCode;` imports are the minimum surface needed; no other test-file edits required.

### F6 — Diff is tightly scoped

`git diff --stat HEAD` shows exactly four modified files (the HIR source, the two unit-test files, and the issue tracker) plus three untracked fixture files — nothing else. No `crates/sifr_diagnostics/` edits, no `docs/errors/` edits, no schema updates, no renderer changes, no driver-bridge edits. The slice does not touch the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge (intentionally deferred per the standing scope statement) and does not perturb the registry's representative-fixture pointer for SIFR-TYPE-0007 (which still points at [invalid_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr:1) — correct, since one representative fixture per code is the registry contract, and the new fixtures supplement rather than replace).

### F7 — Issue-tracker cadence is consistent

[issue:47](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:47) flips slice 2b.12 from "in progress" to `[x] merged ... PR: ...pull/1684`, matching the merged PR identifier from the prior pass-1 review's verdict. [issue:48](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:48) adds slice 2b.13 with the "in progress" + "PR: pending" wording, identical phrasing to the corresponding entries for slices 2b.7–2b.12 at this same review stage. No unrelated checklist drive-bys.

### F8 — Out-of-scope sites in the same file are correctly *not* migrated

Per the slice scope (generic type alias and class annotation arity/surface), the following raw `ctx.error` call sites in [typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs) remain untouched and verified by `git diff` showing no edits there:

- **Function default-argument expression validation** at [typing_and_functions.rs:247, 261](../crates/sifr_hir/src/lower/typing_and_functions.rs:247) — unrelated domain (default expression lowering, not annotation surface).
- **Result `[T, E]` invalid error type** at [typing_and_functions.rs:516-520](../crates/sifr_hir/src/lower/typing_and_functions.rs:516) — has its own pre-allocated code `RESULT_INVALID_ERROR_TYPE = SIFR-RESULT-0002` ([codes.rs:89](../crates/sifr_diagnostics/src/codes.rs:89)) and is a future slice (R1).
- **Function exhaustive-return** at [typing_and_functions.rs:826](../crates/sifr_hir/src/lower/typing_and_functions.rs:826) — control-flow domain, separate slice.
- **Return-type inference callback** at [typing_and_functions.rs:847](../crates/sifr_hir/src/lower/typing_and_functions.rs:847) — type-system inference plumbing, not annotation surface.

I confirmed by `grep -rn "ctx.error\b" crates/sifr_hir/src/lower/typing_and_functions.rs | grep -v error_types | grep -v error_hierarchy | grep -v error_with_code` that exactly five raw `ctx.error` call sites remain in the file, all enumerated above. After this slice, every `ctx.error` call inside `resolve_annotation_expr` itself is either migrated (eight via `invalid_type_annotation`, two via `unknown_type`) or correctly retained for the Result-error-type case awaiting its own slice.

## Residual risks

### R1 — Result `[T, E]` invalid-error-type emission still flows through the bridge

Inside the same `resolve_annotation_expr` `match` arm, the Result-error-type validation at [typing_and_functions.rs:514-521](../crates/sifr_hir/src/lower/typing_and_functions.rs:514) still emits via raw `ctx.error(format!(...))`, currently bridged through `CompilePhase::TypeCheck => "SIFR-TYPE-0001"`. The pre-allocated `RESULT_INVALID_ERROR_TYPE = SIFR-RESULT-0002` at [codes.rs:89](../crates/sifr_diagnostics/src/codes.rs:89) is the natural target. This is *not* in the declared scope of slice 2b.13 ("generic type alias and generic class annotation arity/surface") — it is a Result-domain emission. Not a blocker; flagging so the next slice in this file's migration sequence has explicit eyes on this one remaining annotation-position emission.

### R2 — `recursive_generic_type_alias_wrong_arity.sifr` exercises the migrated alias-arity path but has no `expect-error` line to pin SIFR-TYPE-0007

Pre-existing fixture [recursive_generic_type_alias_wrong_arity.sifr:1](../crates/sifr/tests/e2e/fail/recursive_generic_type_alias_wrong_arity.sifr:1) (added in an earlier alias-recursion slice, no `expect-error`) traverses the same alias-arity code path with a recursive alias body. Direct `cargo run -q -p sifr -- check` confirms it now emits `type error: [main] generic type alias 'Node' expects 1 type argument(s), got 2` with code `SIFR-TYPE-0007` after this slice. The harness today (per [e2e.rs:2541-2587](../crates/sifr/tests/e2e.rs:2541)) only requires fixtures *with* `expect-error` lines to assert code+message; fixtures without those lines must merely fail to compile. So nothing breaks. But appending `# expect-error: SIFR-TYPE-0007: generic type alias 'Node' expects 1 type argument(s), got 2` to that file would lock in a free additional regression-anchor for the migrated path under the recursive-alias surface, mirroring the slice's three new fixtures. Not a blocker — the new `generic_type_alias_wrong_arity.sifr` already pins the arity check on a non-recursive alias — flagging only as a possible follow-up if the team wants the recursive variant explicitly anchored at the e2e layer.

### R3 — Two `class` fixtures emit a secondary unrelated diagnostic alongside the primary

Both [generic_class_non_generic_subscript.sifr](../crates/sifr/tests/e2e/fail/generic_class_non_generic_subscript.sifr:8) and [generic_class_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/generic_class_wrong_arity.sifr:8) end with `print(value.value)`, which on `Type::Any` (returned after the migrated arity diagnostic) trips the unrelated `attribute access '.value' is not supported as an expression; use as a method call` emission. The e2e harness uses `errors.iter().any(...)` so the primary diagnostic still matches, and the test passes. Fixtures intentionally surfacing a single primary diagnostic is the cadence convention (cf. [generic_type_alias_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/generic_type_alias_wrong_arity.sifr:1) which is clean), so a future cleanup could swap the trailing `print(value.value)` for `print("ok")` to keep the rendered diagnostic stream single-line. Cosmetic only — not a behavior gap, not a code-coverage gap.

## Recommendation

Merge as-is. The slice cleanly closes the generic-alias and generic-class arity/surface half of `resolve_annotation_expr`'s migration onto SIFR-TYPE-0007, with one fixture per call site, dual-condition unit-test assertions, and zero collateral damage to the bridge-keyed fixture corpus. The remaining annotation-position `ctx.error` (Result-error-type at line 516) is correctly out of scope and tracked as the next natural slice for this file.
