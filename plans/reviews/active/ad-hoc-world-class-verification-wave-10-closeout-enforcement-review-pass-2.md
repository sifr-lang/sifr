I have enough to report. The runtime validation passes, but I want to verify my reading of one self-test before reporting.

Tracing `ci_plan_omits_local_merge_suite` (verification/areas/coverage_matrix/checks/coverage_matrix_self_test.py:230-259) against the matcher in `main()` at line 60 (`any(expected in error for error in errors)`):

- Expected substring: `"profile plan mismatch"` (line 53).
- Success branch (result == 1, line 258): appends `"profile plan mismatch detected"` → contains the substring.
- Failure branch (result != 1, line 256): appends `"profile plan mismatch self-test did not fail"` → also contains the substring.

If a regression made `compare_plans` return `0`, the failure-branch message would still match `"profile plan mismatch"` and the assertion would pass anyway. Confirmed by reading the code, not run-time simulated.

## Blockers

**B2 still partially open — `ci_plan_omits_local_merge_suite` assertion is tautological** (`verification/areas/coverage_matrix/checks/coverage_matrix_self_test.py:230-259`).

- The self-test now calls production `compare_plans(...)` — progress over pass 1.
- But the failure-branch sentinel string `"profile plan mismatch self-test did not fail"` (line 256) *contains* the expected substring `"profile plan mismatch"` (line 53), the same substring the success-branch sentinel `"profile plan mismatch detected"` (line 258) contains.
- Matcher at line 60 only checks substring membership, so the assertion succeeds in BOTH branches.
- Consequence: if production `compare_plans` regressed and returned `0` instead of `1` for a mismatched plan, this negative self-test would still report green. The captured `stdout`/`stderr` from `compare_plans` are populated (lines 251-254) but never asserted against, so the actual production output is not verified either.
- Fix options (pick one): (a) make the failure sentinel not contain "profile plan mismatch" (e.g. `"compare_plans did not report mismatch (returned …)"`), (b) assert against the captured `stderr.getvalue()` for the production diagnostic text, or (c) only append to `errors` in the success branch and let the outer matcher fail naturally on an empty list.

## Resolved from pass 1

- **B1 (negative self-tests unselected)** — Resolved. `coverage_matrix_negative_self_tests` is now a case under the `closeout` suite in `verification/areas/coverage_matrix/manifest.json:64-70`, and `closeout` is the suite all four profiles select for `coverage_matrix` per `profile_assignment_matrix.json:6-12`.
- **B2 for profile-assignment self-test** — Resolved. `profile_assignment_mismatch` (self-test L262-278) now calls the production `profile_assignment_matrix.validate_row_membership(...)` (defined at `profile_assignment_matrix.py:145-154`) and lets production code append to `errors`. `main()` in `profile_assignment_matrix.py:56-62` uses the same function, so production and self-test exercise one code path.

## Validation results

- `python3 -m py_compile …` → OK
- `jq empty …` → OK
- `uv run … areas run --area coverage_matrix --suite closeout --hardening-summary` → `verification ok: variants=3, failures=0`; all three cases (`coverage_matrix_closeout`, `profile_assignment_matrix`, `coverage_matrix_negative_self_tests`) pass with `strict=yes`, `rows=16`, `cases=16`.

Reviewer is **not** satisfied — B2 still has a tautology in the CI-plan self-test that masks production regressions.
