## `milestone_diag_4a` slice 2b.12 — unknown simple/generic type annotation diagnostics migration to active `SIFR-NAME-0003` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-unknown-types`.
- Target: migrate the two `resolve_annotation_expr` fallbacks that previously emitted ad-hoc `ctx.error(...)` text — the `Expr::Name` simple-name fallback and the `Subscript` generic-base fallback — onto the active `DiagnosticCode::NAME_UNKNOWN_TYPE` (`SIFR-NAME-0003`) registry slot, with fixture coverage for both call sites.
- Files changed (per `git status`):
  - [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:384) — new private `unknown_type` helper plus two call-site migrations.
  - [crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr](../crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr:1) — re-keyed from `SIFR-TYPE-0001` / `unknown generic type` to `SIFR-NAME-0003` / `unknown type`.
  - [crates/sifr/tests/e2e/fail/unknown_type_annotation.sifr](../crates/sifr/tests/e2e/fail/unknown_type_annotation.sifr:1) — new fixture for the simple-name fallback.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:46) — slice 2b.11 flipped to merged with PR #1683; slice 2b.12 line added as "Started".
- Validation rerun by reviewer:
  - `cargo test -p sifr --test e2e -- test_e2e_fail` → 1 passed, 25 filtered.
  - `cargo run -q -p sifr -- check` against each new/edited fixture, observing the rendered diagnostic byte-for-byte against the `expect-error` substring (see F4).
  - `git stash` + `cargo test -p sifr_hir` on the parked tree to confirm the two `expressions_tests` failures (`test_empty_dict_literal_conflicting_write_reports_deterministic_error`, `test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`) reproduce on `main` and are not introduced by this slice.

## Verdict

**Satisfied — no blockers.** The taxonomy choice, message unification, helper shape, and fixture coverage are all correct. The slice exactly mirrors the cadence of 2b.7 / 2b.8 / 2b.10 / 2b.11 and stays inside its declared scope. Recommend merge.

## Findings

### F1 — Taxonomy choice: `SIFR-NAME-0003` is the right code for both call sites

The active registry entry at [codes.rs:526-536](../crates/sifr_diagnostics/src/codes.rs:526) declares `SIFR-NAME-0003` with summary `"Unknown type or generic type name."`, message template `"unknown type: {name}"`, and `owner_module = "sifr_hir::lower::typing_and_functions"` — i.e. the registry slot was *pre-allocated* to cover both shapes from this exact module. The const declaration at [codes.rs:23](../crates/sifr_diagnostics/src/codes.rs:23) (`NAME_UNKNOWN_TYPE`) further confirms the family-level intent: a name-resolution failure at *type* position, regardless of whether the surface syntax is `T` or `T[...]`. Neither call site is doing arity validation (which would be `TYPE-*`), annotation-shape parsing (`TYPE_INVALID_ANNOTATION = SIFR-TYPE-0007`), or undefined-callable resolution (`NAME_UNDEFINED_CALLABLE = SIFR-NAME-0002`). Both are exclusively "the lookup chain (`type_vars`, type aliases, `class_types`, `resolve_type_annotation`) returned no match for an identifier in type position" — that is, by construction, NAME-0003. No sibling code is a closer fit.

### F2 — Aligning the generic-base message to `unknown type` is acceptable and registry-consistent

Pre-slice, the two call sites rendered:

- Simple name: `unknown type: '{name}'` (e.g., `unknown type: 'Missing'`).
- Generic base: `unknown generic type: '{name}'` (e.g., `unknown generic type: 'UnknownType'`).

This slice unifies both onto `unknown type: '{name}'` via the new `unknown_type` helper at [typing_and_functions.rs:384-389](../crates/sifr_hir/src/lower/typing_and_functions.rs:384). Three reasons this is the right call rather than introducing a sibling code or keeping two helper variants:

1. **Registry template alignment.** The active entry's `message_template` is `"unknown type: {name}"` (no `generic` qualifier) — see [codes.rs:532](../crates/sifr_diagnostics/src/codes.rs:532). Keeping `unknown generic type` would have left the rendered text *diverged* from the declared template, which works at runtime (the e2e harness uses `failure.message.contains(...)` substring match per [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561), so divergence is not a test failure) but undermines the registry-as-source-of-truth contract that the rest of slice 2b has been enforcing.
2. **Information preservation.** The dropped word `generic` was redundant context: the diagnostic already carries the file/span pointing at the offending `Subscript` expression, and the renderer surfaces that span. The user reading a rendered diagnostic loses no actionable information by seeing `unknown type: 'UnknownType'` rather than `unknown generic type: 'UnknownType'` — both phrasings answer the same "which name was unresolved" question, and the surface form (`UnknownType[int]`) is visible in the source pointer the renderer emits.
3. **Single-helper invariant.** With one helper, every NAME-0003 emission is guaranteed to render the same prefix — there is no way for a future call site to drift back to the bifurcated phrasing. This is the same pattern slice 2b.7 used for `missing_method_param_annotation` ([classes.rs:53-65](../crates/sifr_hir/src/lower/classes.rs:53)) and slice 2b.8 used for `invalid_type_annotation` ([typing_and_functions.rs:380-382](../crates/sifr_hir/src/lower/typing_and_functions.rs:380)). Consistency with the established slice-2b convention.

The single quotes around `{name}` (e.g., `'UnknownType'`) are not in the registry template (`unknown type: {name}`) but are a rendering convention shared by sibling NAME-* and IMPORT-* messages (cf. `undefined variable: 'x'`, `unknown intrinsic module 'foo'`). The substring contract still holds because `failure.message.contains("unknown type: 'UnknownType'")` is satisfied by `error: [main] unknown type: 'UnknownType'`. The registry's `assert_template_placeholders_are_declared` test ([codes.rs:1606-1628](../crates/sifr_diagnostics/src/codes.rs:1606)) only checks placeholders are declared, not that rendered output matches the template byte-for-byte, so no registry test breaks.

### F3 — Helper extraction is appropriate and minimal

`unknown_type(ctx, name)` at [typing_and_functions.rs:384-389](../crates/sifr_hir/src/lower/typing_and_functions.rs:384) takes `&mut LowerCtx` plus `&str` and returns `()`, exactly mirroring the shape of the adjacent `invalid_type_annotation` helper. Both call sites pass `&name.id` / `&base_name` (already `&str` / `&String` in scope), so no new clones are introduced and no allocations beyond the single `format!` that the inlined version already had. The helper is `fn` (not `pub(super)`) — correct, because both consumers are in the same module. Function-local scope; no speculative API surface.

### F4 — Fixture coverage is sufficient — exactly one fixture per migrated call site

| Call site | Code path | Fixture | `expect-error` substring | Direct `cargo run` rendered output |
|---|---|---|---|---|
| Simple-name annotation fallback | [typing_and_functions.rs:406-409](../crates/sifr_hir/src/lower/typing_and_functions.rs:406) | [unknown_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/unknown_type_annotation.sifr:1) | `unknown type: 'MissingType'` | `error: [main] unknown type: 'MissingType'` |
| Generic-base subscript fallback | [typing_and_functions.rs:698-701](../crates/sifr_hir/src/lower/typing_and_functions.rs:698) | [generic_class_missing_type_arg.sifr:1](../crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr:1) | `unknown type: 'UnknownType'` | `error: [main] unknown type: 'UnknownType'` |

Each fixture's `expect-error` line has the form `# expect-error: SIFR-NAME-0003: <substring>`, which the e2e harness parses via `parse_expected_error` ([e2e.rs:596](../crates/sifr/tests/e2e.rs:596)) into a code + substring, then asserts at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) that some emitted diagnostic matches both halves. I confirmed both halves match end-to-end:

- `unknown_type_annotation.sifr` is the new fixture; it pins the simple-name path that was previously *uncovered* by any fixture (the closest existing fixture that would have hit it, `crates/sifr_hir/src/lower/type_alias_tests.rs:79-89`, is a unit test that asserts on rendered text via `error.message.contains("unknown type: 'Missing'")` and remains green because the simple-name rendered text is byte-identical pre/post-slice).
- `generic_class_missing_type_arg.sifr` is the existing fixture re-keyed off the bridge code `SIFR-TYPE-0001`. Its body is unchanged (`x: UnknownType[int] = 42`), so the migration test surface is exactly the rekeyed code + new substring `unknown type` (was `unknown generic type`). The fixture was already named in the registry's `representative_fixture_path` slot at [codes.rs:531](../crates/sifr_diagnostics/src/codes.rs:531) — re-keying it under SIFR-NAME-0003 *removes* a pre-existing inconsistency where the fixture *file* was registry-pointed at NAME-0003 but its `expect-error` line still asserted TYPE-0001. After this slice the registry pointer and the fixture's `expect-error` agree.

A single fixture per call site is the convention used throughout slices 2b.6–2b.11 (one fixture per migrated emission point); no additional fixture is needed here.

### F5 — Bridge-fixture rekey is the only safe move and does not orphan the bridge entry

A grep for `SIFR-TYPE-0001` ([crates/sifr/tests/e2e/fail/](../crates/sifr/tests/e2e/fail/)) returns ~30 surviving fixtures (range, enum, stdlib, with-statement, hashable, etc.) — the bridge `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` at [sifr_driver/src/diagnostics.rs](../crates/sifr_driver/src/diagnostics.rs) remains live and load-bearing for everything *except* the unknown-type case. The slice's scope statement explicitly defers bridge deletion, and re-keying *only* the one fixture that the slice migrates is the minimal correct change. Nothing else in the e2e fail corpus relied on the `unknown generic type` rendered text — confirmed by `grep -r "unknown generic type" crates/` returning zero results across `crates/`, including unit tests, snapshots, and other fixtures.

### F6 — Stale unit-test assertion still passes (no false negative introduced)

`crates/sifr_hir/src/lower/type_alias_tests.rs:86` asserts `error.message.contains("unknown type: 'Missing'")` for the simple-name path. Because the simple-name rendered text was already `unknown type: 'Missing'` and that text is unchanged by this slice (only the helper indirection differs), this unit test remains green — confirmed by running `cargo test -p sifr_hir lower::type_alias_tests` (228 passed; the 2 unrelated `expressions_tests` failures are pre-existing on `main` per Validation above). The slice does not need to update this unit test.

### F7 — Out-of-scope sites are correctly *not* migrated

Per the slice scope statement, the following remain untouched and verified by `git diff` showing no edits there:

- **Generic alias arity validation.** The `Type::Alias` path in `resolve_annotation_expr` at [typing_and_functions.rs:611-672](../crates/sifr_hir/src/lower/typing_and_functions.rs:611) emits `wrong number of type arguments for ...` via the legacy `ctx.error` channel; explicitly deferred.
- **Generic class arity / declaration shape.** The class-type subscript validation paths at [typing_and_functions.rs:633-696](../crates/sifr_hir/src/lower/typing_and_functions.rs:633) (e.g., "type 'X' is not generic", "expected N type arguments, got M") still flow through `ctx.error`; deferred.
- **Result error-type validity.** Untouched.
- **Bridge deletion.** `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` and the ~30 dependent fixtures intact.
- **Registry/docs generation.** No changes under `crates/sifr_diagnostics/` or `docs/`.

I confirmed each by `git diff --stat` showing exactly the four expected files (plus the issue) and a repo-wide `grep` for `ctx.error\|error_with_code` in `typing_and_functions.rs` showing all other emission sites are unchanged.

### F8 — Issue checklist transitions are clean

[issue:46](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:46) flips slice 2b.11 from "implementation complete and reviewer-satisfied" to `[x] merged ... PR: ...pull/1683`, matching the merged PR identifier from the prior review's verdict. [issue:47](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:47) adds slice 2b.12 with "Started" status. The wording mirrors the established cadence used for slices 2b.7 through 2b.11. No unrelated checklist drive-bys.

### F9 — Diff is tightly scoped

`git status` shows three modified files (HIR source, one rekeyed fixture, the issue) plus one untracked fixture — nothing else. No baselines, no `verification/`, no docs, no schema changes, no `crates/sifr_diagnostics/` edits — exactly emission migration plus fixtures plus the checklist edit, matching the slice cadence.

## Residual risks

### R1 — Two pre-existing `cargo test -p sifr_hir` failures unrelated to this slice

`expressions_tests::test_empty_dict_literal_conflicting_write_reports_deterministic_error` and `expressions_tests::test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation` fail on this branch *and* on `main` (verified by `git stash` + retest). They expect specific message substrings (`empty literal type conflict`, `return type mismatch`) that no longer match current rendered text — this is pre-existing drift outside the scope of slice 2b.12 and the implementer's reported `cargo test -p sifr -- --skip test_e2e_pass` validation does not exercise the `sifr_hir` test binary. Not a blocker for this slice; flagging so it is not silently inherited as a 2b.12 regression. Recommend filing a separate cleanup against `expressions_tests` in a follow-up slice once the relevant container/return-type diagnostic migration lands.

### R2 — The simple-name fixture's pre-slice coverage was unit-test-only, not e2e

Pre-slice, the only assertion that exercised the simple-name unknown-type fallback was the `type_alias_tests.rs:79-89` unit test — there was no e2e fixture pinning the rendered text. Adding `unknown_type_annotation.sifr` is the right fix and brings the simple-name path up to the same e2e-fixture-coverage bar as the generic-base path. Flagging this only because the slice description's "fixture coverage is sufficient" claim should be read against the *post-slice* state (one fixture per call site), not the pre-slice state (one fixture for one of two paths) — verified above as sufficient.

### R3 — Generic alias and generic class arity diagnostics still flow through the bridge

The other emission sites in `resolve_annotation_expr` (alias arity at [typing_and_functions.rs:611-672](../crates/sifr_hir/src/lower/typing_and_functions.rs:611), class generic arity at [typing_and_functions.rs:633-696](../crates/sifr_hir/src/lower/typing_and_functions.rs:633)) remain on `ctx.error` and bridge through `SIFR-TYPE-0001`. These are explicitly deferred per the slice scope statement and are tracked in the issue's open work. Not a blocker; flagging so the next slice in this file's migration sequence has clear eyes on what remains.

## Recommendation

Merge as-is. The slice cleanly closes the unknown-type half of `resolve_annotation_expr`'s migration and aligns the rendered text with the registry's pre-declared template, with one fixture per migrated call site and zero collateral damage to the bridge-keyed fixture corpus.
