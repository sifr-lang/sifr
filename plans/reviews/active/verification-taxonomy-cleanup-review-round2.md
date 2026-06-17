I reviewed the excerpt against the stated prior finding and for new issues.

## Previous finding: resolved

The negative self-test is now exercised by default. `main()` calls `run_self_test(quiet=True)` before the repo scan and returns early on a non-zero status, so the self-test's bad-fixture assertions (`bad_label`, `bad_file`, mixed file) run on every invocation, not just under `--self-test`. The finding does not remain.

The allowlist change is also sound: `validate_text` now strips allowed spans in-place (`pattern.sub("", checked_line)`) and then scans the residue, instead of discarding the whole line. The `mixed_allowed_and_bad.rs` fixture meaningfully guards against regression to skip-whole-line behavior — its filename doesn't match `FILENAME_PATTERNS`, so the only way `"mixed_allowed_and_bad.rs"` reaches `rendered` is if the bad `"Milestone 99"` survives allow-span stripping and is flagged by the text path. Good.

I also confirmed the deliberate string-splitting (`"Phase" + " 99 closeout"`, `"milestone" + "_99_tests.rs"`) prevents the guard's own source from tripping the scan when it lives under an active root — intentional and correct.

## One non-blocking observation

The self-test asserts that bad content **is** present in `rendered`, but never asserts that allowed/good content is **absent**. If the allow patterns regressed into over-matching (false positives), the self-test would still pass — the `good` and mixed fixtures' allowed spans aren't checked for absence. In practice an over-strict regression would surface as failures during the real `ACTIVE_ROOTS` scan rather than passing silently, so this is a coverage gap in the self-test, not a correctness hole. Worth a follow-up assertion (e.g. assert the allowed substrings do not appear in `rendered`), but not blocking.

No blocking findings remain.
