# `milestone_diag_4a` slice 2b.32 — Builtin `sorted()` / `range()` missing-required-argument diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-builtin-missing-arg-diagnostics`.

## Scope under review

- Mark slice 2b.31 merged in the issue tracker after [sifr-lang/sifr#1703](https://github.com/sifr-lang/sifr/pull/1703) and add the in-progress entry for slice 2b.32 ([issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:66-67](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:66)).
- Migrate the two remaining ad-hoc missing-required-argument emissions for builtins from raw `ctx.error(...)` to `ctx.error_with_code(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT, ...)`:
  - [`lower_range_call`](crates/sifr_hir/src/lower/builtin_calls.rs:845) — post-keyword-loop missing-`stop` guard.
  - `sorted` block in [`lower_call`](crates/sifr_hir/src/lower/expressions.rs:1216) — `(None, None)` arm of the iterable-resolution `match`.
- Add focused HIR unit coverage for both migrated builtins as a single combined test [`test_sorted_and_range_missing_required_argument_have_call_code`](crates/sifr_hir/src/lower/expressions_tests.rs:271), asserting exact message text AND `DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT` for each case.
- Add an e2e fail fixture [crates/sifr/tests/e2e/fail/range_missing_required_argument.sifr](crates/sifr/tests/e2e/fail/range_missing_required_argument.sifr) for the `range()` missing-`stop` surface.
- No registry / generated-docs change is intended: `SIFR-CALL-0004` already has the active template `{callable} missing required argument '{argument}'` ([codes.rs:822-832](crates/sifr_diagnostics/src/codes.rs:822)) with representative fixture `missing_required_argument.sifr` set up by slice 2b.29.

This slice closes follow-up B from the slice 2b.30 / 2b.31 series, narrowing the residual uncoded "missing required argument" emissions in `sifr_hir::lower` down to the only remaining authority on that family — `method_call_args::missing_argument_error` ([method_call_args.rs:288-294](crates/sifr_hir/src/lower/method_call_args.rs:288)) — which already emits with `CALL_MISSING_REQUIRED_ARGUMENT` and is therefore not in scope.

## Verdict

**Approved — reviewer-satisfied for PR.** Implementation is correct, scope is minimal, message text is byte-for-byte preserved across both migrated sites, the HIR unit and e2e fixture coverage line up cleanly on a single `SIFR-CALL-0004` code+template, and the decision to skip a registry / generated-docs refresh is correct (the registry entry, owner, severity, declared/dedupe args, message template, and representative fixture are all unchanged from slice 2b.29). I traced both migrated sites end-to-end through the e2e harness contract at [crates/sifr/tests/e2e.rs:2541-2581](crates/sifr/tests/e2e.rs:2541) and through `error_with_code`'s `LoweringError` shape at [crates/sifr_hir/src/lower/mod.rs:237-244](crates/sifr_hir/src/lower/mod.rs:237) and confirmed the existing `failure.code == expected.code && failure.message.contains(expected.message_contains)` assertion path matches. No correctness, regression, message/code-drift, missing-coverage, or scope-creep blockers were found. A handful of strictly out-of-scope follow-ups in the same `sorted()` block (the `takes at most 1 positional argument` arity guard, the duplicate-keyword guards, the unpacked-kwargs guard, and the iterable element-type / key callable / reverse-bool checks) are documented at the bottom; none block this PR.

## What I checked

### 1. HIR call-site migration — `range()` missing-`stop` guard
[crates/sifr_hir/src/lower/builtin_calls.rs:845-851](crates/sifr_hir/src/lower/builtin_calls.rs:845)

- The `let Some(stop_raw) = stop_expr else { ... };` guard previously emitted via bare `ctx.error("range() missing required argument 'stop'".to_string())`. The migration converts it to `ctx.error_with_code(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT, "range() missing required argument 'stop'".to_string())` followed by the unchanged `return None;`.
- Wording is byte-for-byte identical (`"range() missing required argument 'stop'"`), matching the registry template `{callable} missing required argument '{argument}'` ([codes.rs:828](crates/sifr_diagnostics/src/codes.rs:828)) under the same `{callable}` → `<name>()` and `{argument}` → `<arg>` substitution convention used by `display() missing required argument 'verbose'` in the slice 2b.29 representative fixture and by the method-call helper at [method_call_args.rs:289-292](crates/sifr_hir/src/lower/method_call_args.rs:289).
- The `return None` short-circuit is preserved unchanged, so the existing post-guard lowering of `start_expr` / `stop_raw` / `step_expr` is not reached when `stop` is missing — no risk of cascade diagnostics from later `lower_expr` / `Type::Int` checks.
- The neighboring `start` / `stop` / `step` duplicate-keyword arms ([builtin_calls.rs:807-833](crates/sifr_hir/src/lower/builtin_calls.rs:807)) and the post-loop unexpected-keyword default arm ([builtin_calls.rs:835-841](crates/sifr_hir/src/lower/builtin_calls.rs:835)) already emit with `CALL_DUPLICATE_ARGUMENT` (slice 2b.26) and `CALL_UNEXPECTED_KEYWORD` (slice 2b.31) respectively. After this migration the `lower_range_call` keyword-handling region is fully coded for the canonical CALL family except for the unpacked-kwargs guard at [builtin_calls.rs:801](crates/sifr_hir/src/lower/builtin_calls.rs:801), which is intentionally out of scope (different diagnostic family — "unsupported feature").
- `DiagnosticCode` is already imported at [builtin_calls.rs:2](crates/sifr_hir/src/lower/builtin_calls.rs:2). No new imports needed.
- Behavior trace for the e2e fixture input `list(range())`:
  1. Outer `list(range())` → `func_name == "list"` at [expressions.rs:625-626](crates/sifr_hir/src/lower/expressions.rs:625) → `lower_list_constructor_call` at [builtin_calls.rs:121-137](crates/sifr_hir/src/lower/builtin_calls.rs:121) → `lower_single_optional_iterable_arg` at [builtin_calls.rs:95-119](crates/sifr_hir/src/lower/builtin_calls.rs:95) → `args.len() == 1` arm calls `lower_expr(&call.arguments.args[0], ctx)?`.
  2. Inner `range()` → `func_name == "range"` at [expressions.rs:654-656](crates/sifr_hir/src/lower/expressions.rs:654) → `lower_range_call` → keyword loop iterates zero keywords → `stop_expr.is_none()` → migrated `error_with_code(CALL_MISSING_REQUIRED_ARGUMENT, "range() missing required argument 'stop'")` and `return None`.
  3. The inner `?` propagates `None` through `lower_single_optional_iterable_arg`, then through `lower_list_constructor_call`, then through `lower_call`, then through the outer assignment-RHS lowering in `def main()`. No further diagnostic is emitted along this propagation chain — the surrounding annotated-assignment failure path is the same exercised by [`test_failed_annotated_assignment_rhs_still_seeds_followup_binding`](crates/sifr_hir/src/lower/expressions_tests.rs:62), which already validates that a failed initializer does not cascade to undefined-name or synthetic missing-return errors.

### 2. HIR call-site migration — `sorted()` missing-`iterable` guard
[crates/sifr_hir/src/lower/expressions.rs:1216-1222](crates/sifr_hir/src/lower/expressions.rs:1216)

- The `(None, None)` arm of the `iterable` resolution `match` (which fires when both the positional `args.first()` and the `iterable` keyword are absent) previously emitted via bare `ctx.error("sorted() missing required argument 'iterable'".to_string())`. The migration converts it to `ctx.error_with_code(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT, "sorted() missing required argument 'iterable'".to_string())` followed by the unchanged `return None;`.
- Wording is byte-for-byte identical, matching the same template/substitution discussed in §1.
- Diagnostic-ordering trace under the new HIR test input `sorted()`:
  1. `func_name == "sorted"` at [expressions.rs:1155](crates/sifr_hir/src/lower/expressions.rs:1155) → `args.len() > 1` guard at [expressions.rs:1156-1159](crates/sifr_hir/src/lower/expressions.rs:1156) is false (`args.len() == 0`).
  2. The `for keyword in &call.arguments.keywords` loop at [expressions.rs:1163-1208](crates/sifr_hir/src/lower/expressions.rs:1163) iterates zero items.
  3. The `match (call.arguments.args.first(), iterable_keyword)` at [expressions.rs:1209-1223](crates/sifr_hir/src/lower/expressions.rs:1209) hits `(None, None)` → migrated `error_with_code(CALL_MISSING_REQUIRED_ARGUMENT, "sorted() missing required argument 'iterable'")` → `return None`.
  4. Subsequent `callable_builtin_element_type` / key / reverse / list-output-type code is not reached. No cascade.
- The remaining sibling diagnostics in the same `sorted` block (arity guard at [expressions.rs:1157](crates/sifr_hir/src/lower/expressions.rs:1157), unpacked-kwargs guard at [expressions.rs:1165](crates/sifr_hir/src/lower/expressions.rs:1165), three duplicate-keyword guards at [expressions.rs:1171,1182,1192](crates/sifr_hir/src/lower/expressions.rs:1171), positional+keyword duplicate at [expressions.rs:1211](crates/sifr_hir/src/lower/expressions.rs:1211), iterable element-type guard at [expressions.rs:1225](crates/sifr_hir/src/lower/expressions.rs:1225), key callable / arity / reverse-type guards at [expressions.rs:1243,1248,1258](crates/sifr_hir/src/lower/expressions.rs:1243)) remain uncoded. All of these are intentionally out of scope for slice 2b.32 (which is narrowly scoped to *missing-required-argument* migrations) and map to other CALL-family or non-CALL codes that will be picked up by separate follow-ups (see §6 below).
- `DiagnosticCode` is already imported at [expressions.rs:56](crates/sifr_hir/src/lower/expressions.rs:56) and used at the immediately-adjacent `sorted()` unexpected-keyword arm at [expressions.rs:1201-1206](crates/sifr_hir/src/lower/expressions.rs:1201). No new imports needed.

### 3. Why no registry / generated-docs change is correct
[crates/sifr_diagnostics/src/codes.rs:822-832](crates/sifr_diagnostics/src/codes.rs:822), [docs/errors/SIFR-CALL-0004.md](docs/errors/SIFR-CALL-0004.md)

- The registry entry for `SIFR-CALL-0004` already declares: status `active` (via `active_entry!`), family `CALL`, severity `Error`, owner `sifr_hir::lower`, message template `{callable} missing required argument '{argument}'`, declared args `[arg!("callable"), arg!("argument")]`, dedupe args `["callable", "argument"]`, representative fixture `crates/sifr/tests/e2e/fail/missing_required_argument.sifr` — all set up by slice 2b.29.
- Both new runtime emissions (`range() missing required argument 'stop'` and `sorted() missing required argument 'iterable'`) substitute cleanly into that template under the same `{callable}` → `<name>()` and `{argument}` → `<arg>` convention that `display() missing required argument 'verbose'` in `missing_required_argument.sifr` already exercises. The existing representative fixture remains accurate and broadly applicable; no retargeting is warranted, and adding the new fixture as a *second* representative would duplicate registry semantics that one fixture already covers.
- [docs/errors/SIFR-CALL-0004.md](docs/errors/SIFR-CALL-0004.md) is auto-generated from the registry and reflects the slice 2b.29 state. Since no registry field changes, no doc regen is required. This mirrors slice 2b.30's and slice 2b.31's decisions to skip schema/doc-sync gates: each is also a code-only emit-path migration with no registry diff.
- The `gen-error-docs` / `check_diagnostic_docs_sync.py` / `check_diagnostic_schema_sync.py` chain is therefore a no-op for this slice. The local validation set actually run (`cargo fmt`, `python3 scripts/check_hir_maintainability_guardrails.py`, the targeted HIR test, and `cargo test -p sifr --test e2e -- test_e2e_fail`) is exactly the right set for this scope — broader than minimal but narrower than full-rebuild.

### 4. HIR unit test coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:270-287](crates/sifr_hir/src/lower/expressions_tests.rs:270)

- `test_sorted_and_range_missing_required_argument_have_call_code` follows the *exact same combined-test pattern* introduced by slice 2b.31's [`test_range_and_enumerate_unexpected_keywords_have_call_code`](crates/sifr_hir/src/lower/expressions_tests.rs:1542) — two sibling builtin assertions sharing one test fn, each block doing `lower_source(...)` → `assert!(result.is_err())` → `assert!(errors.iter().any(|error| error.message == ... && error.code == Some(...)))`. This is consistent with the immediately-preceding slice's convention; not a divergence.
- Both assertions correctly use *exact-string* equality on `error.message` (`==` rather than `.contains(...)`) and `Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)` equality on `error.code`. This is the strict shape — message-text drift or accidental code change in the lowering would fail the test loudly. Strict assertions match slice 2b.31's tightening pattern.
- The `sorted()` source `def main():\n    values: list[int] = sorted()\n` and the `range()` source `def main():\n    values: list[int] = list(range())\n` both produce a single coded `CALL_MISSING_REQUIRED_ARGUMENT` diagnostic per §1/§2 traces; the `errors.iter().any(...)` shape would still pass even if a hypothetical future cascade introduced extra errors, but the tests' real load-bearing claim — that the migrated emission carries the code — is independent of cascade behavior. Robust.
- The annotated-assignment LHS `values: list[int] = ...` is intentional: it makes the failed RHS path realistic (matching how user code writes these calls) and exercises the same failed-initializer regression behavior already locked by [`test_failed_annotated_assignment_rhs_still_seeds_followup_binding`](crates/sifr_hir/src/lower/expressions_tests.rs:62).
- The test correctly imports `DiagnosticCode` via the existing module-top `use sifr_diagnostics::DiagnosticCode;` at [expressions_tests.rs:2](crates/sifr_hir/src/lower/expressions_tests.rs:2). No new imports needed.

### 5. E2E fixture
[crates/sifr/tests/e2e/fail/range_missing_required_argument.sifr](crates/sifr/tests/e2e/fail/range_missing_required_argument.sifr)

- Fixture body: `# expect-error: SIFR-CALL-0004: range() missing required argument 'stop'` followed by `def main():\n    values: list[int] = list(range())\n    print(len(values))`.
- Trace through the e2e harness contract at [crates/sifr/tests/e2e.rs:2541-2581](crates/sifr/tests/e2e.rs:2541):
  - `extract_expect_errors` at [e2e.rs:419-424](crates/sifr/tests/e2e.rs:419) returns `["SIFR-CALL-0004: range() missing required argument 'stop'"]`.
  - `parse_expected_error` at [e2e.rs:596-627](crates/sifr/tests/e2e.rs:596) splits on the first `:`/whitespace, normalizes `SIFR-CALL-0004` as `code`, and yields `message_contains = "range() missing required argument 'stop'"`.
  - `compile_source` returns `Err(errors)` because the inner `range()` lowering fails per §1's trace; the lowering's `LoweringError { code: Some(CALL_MISSING_REQUIRED_ARGUMENT), message: "range() missing required argument 'stop'", ... }` flows through to a `CompileFailure` whose `code == "SIFR-CALL-0004"` and whose `message.contains("range() missing required argument 'stop'")`.
  - The harness `errors.iter().any(|failure| failure.code == expected.code && match expected.message_contains.as_ref() { Some(message) => failure.message.contains(message), None => true })` at [e2e.rs:2561-2567](crates/sifr/tests/e2e.rs:2561) matches.
- The fixture omits the optional `# Reference: <wave>` header. This is consistent with the immediately-preceding slice 2b.31 fixture [crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr](crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr) which also omits it, and with ~136 / 232 sibling fixtures in the same directory. Not a defect.
- The fixture deliberately wraps `range()` inside `list(...)` and binds with `values: list[int] = ...` followed by `print(len(values))`. Two reasons this shape is correct:
  - `range(...)` returns the `Type::Range` HIR shape ([builtin_calls.rs:888-893](crates/sifr_hir/src/lower/builtin_calls.rs:888)), which on a successful lowering would not satisfy `list[int]`. Using `list(range(...))` would be the canonical *successful* shape — so on failure (the case under test) it produces a realistic call-site that mirrors how user code typically writes `range(...)`. This is consistent with `range_duplicate_stop_keyword.sifr` ([range_duplicate_stop_keyword.sifr:5](crates/sifr/tests/e2e/fail/range_duplicate_stop_keyword.sifr:5)) which also uses `print(list(range(10, stop=20)))`.
  - `print(len(values))` after the assignment is dead code on the failure path (never reached because lowering fails), but it makes the fixture lower-readable as a complete, intentional test program; this matches the convention used by `missing_required_argument.sifr`'s `print(display("Alice"))` trailer.
- The fixture is picked up automatically by `read_dir_file_paths_sorted(fail_dir)` at [e2e.rs:2539](crates/sifr/tests/e2e.rs:2539); no manifest registration is needed.
- The user reports `cargo test -p sifr --test e2e -- test_e2e_fail` passes locally — consistent with the trace above.

### 6. Issue tracker update
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:66-67](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:66)

- Slice 2b.31 was correctly flipped from `[ ] ... implementation complete and reviewer-satisfied` to `[x] ... merged ... PR: https://github.com/sifr-lang/sifr/pull/1703`. PR URL resolution matches the slice 2b.31 review's "PR landed as #1703" expectation.
- Slice 2b.32 is added as `[ ] ... in progress: builtin sorted() and range() missing required argument diagnostics migration to active SIFR-CALL-0004 with fixture and HIR coverage. PR: pending.` — wording follows the established slice-tracker pattern from 2b.30 / 2b.31. The "PR: pending." placeholder is correct and will be flipped on merge per the established workflow.
- The slice description accurately captures all three deliverables (the two HIR migrations, the HIR test, and the e2e fixture). No drift between the tracker entry and the actual diff.

### 7. Residual scope check — no other ad-hoc missing-required-argument diagnostics remain
[crates/sifr_hir/src/](crates/sifr_hir/src/), [crates/sifr_codegen/src/](crates/sifr_codegen/src/)

- Repo-wide `grep -rn "missing required" crates/sifr_hir/src/ crates/sifr_codegen/src/` returns exactly three production-code occurrences:
  1. [crates/sifr_hir/src/lower/builtin_calls.rs:848](crates/sifr_hir/src/lower/builtin_calls.rs:848) — migrated by this slice.
  2. [crates/sifr_hir/src/lower/expressions.rs:1219](crates/sifr_hir/src/lower/expressions.rs:1219) — migrated by this slice.
  3. [crates/sifr_hir/src/lower/method_call_args.rs:289-292](crates/sifr_hir/src/lower/method_call_args.rs:289) — already coded with `CALL_MISSING_REQUIRED_ARGUMENT` by an earlier slice; out of scope.
- After this slice lands, every "missing required argument" emission in the lowering is structured (`code = Some(CALL_MISSING_REQUIRED_ARGUMENT)`). The CALL_MISSING_REQUIRED_ARGUMENT family is therefore *fully migrated* in `sifr_hir::lower`, and the slice's claim of closing this follow-up is accurate.
- I also verified no neighboring `ctx.error("...required...")` / `ctx.error("...missing...")` patterns slipped through under different wording: a follow-up grep of `crates/sifr_hir/src/lower/` for `ctx.error(` lines containing `missing` or `required` returns only the migrated lines, with no lingering uncoded sites.

### 8. Local validation envelope
- The user reports the following passed locally before review request:
  - `cargo fmt` — confirmed by my own `cargo fmt --check` re-run (clean exit, no output).
  - `python3 scripts/check_hir_maintainability_guardrails.py` — confirmed by my own re-run (`HIR maintainability guardrails: PASS`).
  - `cargo test -p sifr_hir test_sorted_and_range_missing_required_argument_have_call_code` — targeted HIR test for the new coverage in §4.
  - `cargo test -p sifr --test e2e -- test_e2e_fail` — e2e harness for the new fixture in §5 (and a regression check across all 232 fixtures, since the harness iterates the entire directory).
- This validation envelope is appropriately tight for a code-only emit-path migration with no registry change, and matches the envelopes used for slices 2b.30 and 2b.31. The full `scripts/run_all_tests.sh --profile quick` is not strictly required for this scope but is recommended pre-merge per AGENTS.md gating; the user has not yet reported running it for this slice. **Recommendation: run `scripts/run_all_tests.sh --profile quick` before opening the PR**, consistent with the gating convention used in slices 2b.7 / 2b.16 / 2b.24 etc. This is not a blocker but should be done.

## Findings

### Blocking
None.

### Suggestions (non-blocking, optional for this slice)

1. **No e2e fixture for `sorted()` missing-`iterable`.** The slice scope explicitly includes only the `range()` fixture; the `sorted()` case is covered by HIR coverage only. This is consistent with the slice's stated scope ("add an e2e fail fixture for range() missing required stop") and with how slices 2b.30/2b.31 handled multi-builtin migrations (one representative e2e fixture per slice plus broader HIR coverage). For full parity with `range_missing_required_argument.sifr`, a `sorted_missing_required_argument.sifr` fixture could be added in a future slice if the project standard is "one fixture per migrated builtin site"; if the standard is "one representative per code", the existing `missing_required_argument.sifr` plus the new `range_missing_required_argument.sifr` plus HIR coverage is already enough. Either reading is defensible. **No action required for this PR.**

2. **`sorted()` block has remaining uncoded diagnostics.** Strictly out of scope for slice 2b.32, but worth queuing as future follow-ups in `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`:
   - [expressions.rs:1156-1159](crates/sifr_hir/src/lower/expressions.rs:1156) — `"sorted() takes at most 1 positional argument"` → likely `CALL_WRONG_POSITIONAL_COUNT` (`SIFR-CALL-0001`), but message text would need to be *re-templated* to `{callable} takes exactly {expected_count} argument(s), got {actual_count}` per the registry template at [codes.rs:786-797](crates/sifr_diagnostics/src/codes.rs:786). This is wording drift, not just a code attach, so it is correctly carved out.
   - [expressions.rs:1163-1167](crates/sifr_hir/src/lower/expressions.rs:1163) — `"sorted() does not support unpacked keyword arguments"` → "unsupported feature" / mirror of `range()`'s unpacked-kwargs guard at [builtin_calls.rs:801](crates/sifr_hir/src/lower/builtin_calls.rs:801). Both are correctly carved out for the same reason (different diagnostic family).
   - [expressions.rs:1170-1175](crates/sifr_hir/src/lower/expressions.rs:1170), [1180-1186](crates/sifr_hir/src/lower/expressions.rs:1180), [1189-1196](crates/sifr_hir/src/lower/expressions.rs:1189), [1209-1212](crates/sifr_hir/src/lower/expressions.rs:1209) — four duplicate-argument variants for `iterable` / `key` / `reverse`. These are all `CALL_DUPLICATE_ARGUMENT` (`SIFR-CALL-0003`) candidates, and the wording across the four sites is *not* uniform (`"got multiple values for keyword argument 'iterable'"` vs. `"got multiple values for argument 'iterable'"`). A clean follow-up would unify to the registry template `{callable} got multiple values for argument '{argument}'`.
   - [expressions.rs:1224-1230](crates/sifr_hir/src/lower/expressions.rs:1224), [1242-1245](crates/sifr_hir/src/lower/expressions.rs:1242), [1246-1251](crates/sifr_hir/src/lower/expressions.rs:1246), [1257-1264](crates/sifr_hir/src/lower/expressions.rs:1257) — type-shape and callable-shape diagnostics; not CALL family, would need new codes.

3. **The combined-test naming convention diverges slightly from single-builtin tests.** `test_sorted_and_range_missing_required_argument_have_call_code` is fine and consistent with slice 2b.31's `test_range_and_enumerate_unexpected_keywords_have_call_code`. If the project later prefers per-builtin tests for searchability, splitting into `test_sorted_missing_required_argument_has_call_code` and `test_range_missing_required_argument_has_call_code` would be a trivial, reviewer-friendly cleanup. **No action required for this PR.**

4. **Pre-merge full-quick validation.** Per §8, recommend running `scripts/run_all_tests.sh --profile quick` once before opening the PR, consistent with the gating convention used in earlier slices, even though the targeted local validation already gives high confidence that nothing else regresses for this code-only migration.

### Out of scope (correctly carved out by this slice)
- All sibling `sorted()` / `range()` ad-hoc diagnostics enumerated in suggestion 2 above.
- The `range()` unpacked-kwargs guard at [builtin_calls.rs:801](crates/sifr_hir/src/lower/builtin_calls.rs:801).
- The `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge deletion (deferred per [tracker line 68](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:68)).

## Summary

Slice 2b.32 is a textbook small, reviewable, single-purpose emit-path migration that fully closes the *missing-required-argument* family for builtins in `sifr_hir::lower`. Implementation, message text, code attachment, HIR test, e2e fixture, and tracker update are all correctly scoped, byte-for-byte consistent with the canonical template, and aligned with the immediately-preceding slices 2b.30 / 2b.31. **Approve and proceed to PR**, optionally running `scripts/run_all_tests.sh --profile quick` once before opening per the suggestion above.
