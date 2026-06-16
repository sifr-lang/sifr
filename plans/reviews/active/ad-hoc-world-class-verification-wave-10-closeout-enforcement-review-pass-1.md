Review complete. Two blockers below, followed by non-blocking suggestions.

## Blockers

### B1 — `negative-self-tests` is never selected by any profile, so closeout enforcement runs in ad-hoc mode only

`verification/areas/coverage_matrix/manifest.json:66-77` defines `negative-self-tests` as a **separate** suite from `closeout`. None of the four profiles add it to their selection — `verification/profiles/create-pr.json:63-72`, `verification/profiles/merge.json:63-72`, `verification/profiles/nightly.json:66-74`, `verification/profiles/release.json:65-74` all list only `["closeout"]` under the `coverage_matrix` area.

Yet the docs and the Wave 10 plan claim the negative self-tests gate at closeout:
- `internal_docs/architecture.md:1321` — "[the closeout suite] runs negative self-tests for the closeout enforcement claims"
- `verification/policy/profile_policy.md:36-41` — "[the closeout suite] runs strict mode with `SIFR_COVERAGE_MATRIX_STRICT=1`, the profile-assignment matrix check, and negative self-tests"
- `verification/policy/suite_taxonomy.md:9` — lists "negative self-tests" as a required artifact for `coverage_matrix`
- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1656` — "Add negative self-tests proving enforcement fails on..."

The validation command the author ran (`uv run … --area coverage_matrix --suite closeout --suite negative-self-tests`) explicitly added `--suite negative-self-tests`, masking the fact that profile runs (`scripts/run_all_tests.sh --profile merge`, etc.) do **not** trigger it. A regression in `coverage_matrix.py` that broke any closeout assertion would not be caught by the merge gate.

Fix: either move the `coverage_matrix_negative_self_tests` case into the existing `closeout` suite (single suite, three cases), or list `"negative-self-tests"` alongside `"closeout"` in each profile's `coverage_matrix` `selected_areas.suites`.

### B2 — `profile_assignment_mismatch` / `ci_plan_omits_local_merge_suite` self-tests are self-affirming and do not exercise production code

`verification/areas/coverage_matrix/checks/coverage_matrix_self_test.py:232-245`:
```python
profile_assignment_matrix.validate_expected_tokens(
    "parser_acceptance_rejection", "merge",
    ["core_language:syntax_parser_lexer_matrix"],
    {"core_language": {"syntax_parser_lexer_matrix"}},
    errors,
)
actual = {"core_language:phase24_hir_analysis"}
token = "core_language:syntax_parser_lexer_matrix"
if token not in actual:
    errors.append(f"parser_acceptance_rejection: merge omits required suite {token}")
return errors
```

The `validate_expected_tokens` call passes a token that is present in `area_suites` and therefore produces zero errors (that function only flags unknown areas/suites — see `profile_assignment_matrix.py:122-139`). The test then **manually appends** the `"omits required suite"` string and verifies its own appended string. The actual production loop at `profile_assignment_matrix.py:55-59`:

```python
for token in expected:
    if is_area_suite_token(token) and token not in actual:
        errors.append(f"{surface_id}: {profile} omits required suite {token}")
```

is never invoked from either self-test. If that loop is deleted, both `"profile assignment mismatch"` and `"CI plan omitting local merge suite"` self-tests still pass green. The same self-test entry also misrepresents itself as exercising local-vs-CI plan equivalence (the test name says "CI plan omitting local merge suite") even though no plan comparison happens — the test never touches `sifr_verify profiles compare-plans` or any plan JSON.

Fix: factor `profile_assignment_matrix.py` so the per-row "token present in actual selected areas" check is a callable function (e.g. `validate_row_membership(surface_id, profile, expected, actual, errors)`), call it from the self-test with `actual` set to omit a required token, and assert the production-emitted `"omits required suite"` string fires from real code. If local-vs-CI plan equivalence is genuinely intended to have a negative self-test, build one against `compare-plans` instead of reusing the assignment check.

## Non-blocking observations

- **Duplicate test cases.** `coverage_matrix_self_test.py:53` `("zero-test temporary status", expired_tests_none, "expiry has passed")` is a verbatim repeat of line 40 `("expired tests:none", expired_tests_none, …)` — same function, same expected substring. Same pattern for `ci_plan_omits_local_merge_suite` ↔ `profile_assignment_mismatch`. Inflates the `cases=17` count without adding distinct coverage.
- **Owner-registry coverage is shallow.** `coverage_matrix.py:269-272` only checks that each area manifest's `owner` is *some* known id; it does not verify that a given area maps to its expected owner. Swapping `owner` between two areas slips through. Consider an expected-owner table or invariant per area.
- **No inverse coverage check on `profile_assignment_matrix.json`.** `profile_assignment_matrix.py` validates that listed tokens land in the corresponding profile, but it never verifies that every `merge_suite` / `nightly_release_suite` token in `compiler_surface_matrix.json` has a covering row, nor that every stable surface_id has a row. The matrix uses aggregated labels (e.g. `diagnostics`, `runtime_platform`, `performance`) rather than 1:1 surface ids, so adding a new stable surface to `compiler_surface_matrix.json` without updating `profile_assignment_matrix.json` would not fail. Add a final pass that asserts surface coverage.
- **`regression_surface` in `shipped_guarantees.json` is unvalidated.** `coverage_matrix.py:178` deliberately skips `regression_surface` because the values (`regression_fixedbugs`, `regression_crashes`) are not surface_ids. As a result, an empty or typo'd `regression_surface` would still pass `require_string` but not validate against any registry. Consider validating against the regression suite tokens.
- **`ALLOWED_WAVES = {"1"…"9"}` (`coverage_matrix.py:60`) hard-stops at 9.** Fine for closeout since strict mode rejects all temporary statuses; flag for future iteration if any Wave 10+ staging is ever needed.
- **`advisory` suite (`coverage_matrix/manifest.json:33-45`) is essentially redundant with `closeout`** (closeout is `advisory` plus strict checks). With closeout selected by every profile, `advisory` is dead weight unless documented otherwise.
- **`unpinned_merge_corpus` is mis-named.** It tests `pinned_corpus: {required: true, revision: "local"}` (checksum missing). The function name suggests "no pin at all"; the actual test is "pin required but checksum absent." Rename for clarity.
- **`coverage_matrix_consistency` in the `advisory` suite uses `entry: ".../coverage_matrix.py"` with no `SIFR_COVERAGE_MATRIX_STRICT` env flag**, while `coverage_matrix_closeout` flips strict via a wrapper. Both check files live next to each other — minor maintainability watchpoint if a future change ever inverts the default.

Validation already passing (per task context) covers `py_compile`, `jq empty`, the closeout/negative-self-tests area run, the `compare-plans` round-trip with a `merge.local.json` copied to `merge.ci.json`, file-size guardrails, and `git diff --check`. None of those reach the two blockers above — B1 because the profile JSON shape is valid even with the suite missing, B2 because the fake self-test currently passes by appending its own error string.
