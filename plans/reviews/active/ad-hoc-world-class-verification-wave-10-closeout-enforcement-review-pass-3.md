I read `verification/areas/coverage_matrix/checks/coverage_matrix_self_test.py` and inspected `ci_plan_omits_local_merge_suite` (lines 230–261) against the pass 2 blocker.

Verification of the substring-matching contract (expected = `"profile plan mismatch"`):

- Line 256 (result != 1): appends `"compare_plans did not report mismatch"` — does **not** contain `"profile plan mismatch"` (no `"plan mismatch"` substring with `"profile"` adjacent).
- Line 258 (stderr lacks diagnostic): appends `"compare_plans mismatch diagnostic missing"` — does **not** contain `"profile plan mismatch"`.
- Line 260 (success): appends `"profile plan mismatch detected"` — **does** contain `"profile plan mismatch"`.

This means main()'s substring check at line 60 now fails the self-test if either `compare_plans` returns the wrong exit code or fails to emit the diagnostic — the tautology is gone. The test only passes when `compare_plans` actually does its job.

The pass 2 blocker is resolved. Reviewer is satisfied.
