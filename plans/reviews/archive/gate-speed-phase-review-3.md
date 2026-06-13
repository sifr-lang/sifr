# Code Review — Gate Speed Phase, Round 3

## Prior blockers — verified clean

1. **Perf retry removed.** `scripts/run_all_tests.sh:245-284` `run_performance_budget_checks` has no retry loop; `verification/performance/run_benchmarks.py` and `check_budgets.py` contain no retry/attempt logic. Round-1 #1 resolved.

2. **Create-PR manifest ↔ policy doc alignment.** `internal_docs/validation_lane_policy.md:16-26` lists smoke-only bullets and explicitly excludes "broad project-mode matrices"; `verification/validation_lanes/manifest.json:17` `matrix_suites: []` matches. Round-1 #2 resolved.

3. **Phase evidence + acceptance ticks.** `issues/ad-hoc-pr-gate-speed-and-validation-lane-rebalancing.md:159-184` carries the before/after table the issue mandated, all milestone checkboxes are `[x]`. Round-1 #3 resolved.

4. **Resolver doc claim weakened.** `internal_docs/validation_lane_policy.md:12` now says "*primary* shell-facing resolver" and acknowledges helper scripts may keep standalone defaults with the same alias semantics. Round-1 #4 resolved.

5. **`exit` → `return` inside `timed_step` callees.** `scripts/run_all_tests.sh:346` (GCQ default arm) and `:384` (crate-test default arm) both use `return 2`; no other `exit` survives inside a `timed_step` callee. Round-1 #5 + Round-2 follow-up resolved.

6. **`runner.rs` temp cleanup.** `crates/sifr/tests/validation_contract_support/runner.rs:66-84` — work is wrapped in an inner closure, `let _ = std::fs::remove_dir_all(&tmp_dir);` runs unconditionally at line 73 *after* the closure result is captured, then `result?` propagates the error. Cleanup happens on both pass and fail paths. Round-1 #6 resolved.

7. **`timed_step` errexit preservation.** `scripts/run_all_tests.sh:108-112`:
   ```
   set +e
   (set -euo pipefail; "$@")
   status=$?
   set -e
   ```
   The subshell re-enables `errexit`/`pipefail` inside each bucket so mid-bucket failures still abort, the outer `set +e` keeps the wrapper alive long enough to emit `[sifr-lane-step] … status=fail`, and `return "${status}"` propagates. Round-2 blocker resolved.

8. **Unsupported-mode returns.** Both `scripts/run_all_tests.sh:343-346` (`generated-code quality mode`) and `:381-384` (`crate test mode`) write to stderr and `return 2`, which `timed_step` will surface as `status=fail` rather than terminating the shell silently. Verified.

## New surface — lane-aware `crate_tests=smoke/full`

- **Manifest** (`verification/validation_lanes/manifest.json:25,61,100,143`): only `create-pr` → `"crate_tests": "smoke"`; `merge`, `nightly`, `release` → `"full"`.
- **Resolver** (`scripts/validation_lane.py:134`): exported as `CRATE_TEST_MODE`.
- **Shell** (`scripts/run_all_tests.sh:375-388`): `smoke` runs only `cargo test -p sifr --bin sifr` (CLI unit tests). `full` runs `cargo test -p sifr -- --skip test_e2e_pass`. Common crates (`sifr_diagnostics`, `sifr_hir`, `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, `sifr_lsp`, `sifr_package`, `sifr_driver --lib`) run in both modes. Unknown mode → `return 2`.
- **Policy doc** (`internal_docs/validation_lane_policy.md:23,25`): create-PR includes "library crate unit tests, CLI unit tests, and representative e2e pass fixtures" and explicitly excludes "the slower `sifr` integration/e2e-support crate tests". Manifest, shell, and doc agree.

## Latest create-PR pass

`target/validation_lane_reports/quick.latest.json`: `profile=create-pr`, `real_seconds=74.82`, `within_warm_budget=true`, advisories=`[]`. All 13 lane_steps pass; bucket timings exactly match `issues/.../*:169-184`. Generated-code smoke at 18.11s (target ≤30s), diagnostic contracts at 7.97s (target ≤10s), crate_tests at 17.04s under the new smoke split.

## Pre-existing minors (not in this round's scope, flagging only)

- `scripts/run_distribution_validation.sh:32` still `exit "${status}"` inside the per-script loop — Round-1 #15 untouched. Distribution lane is "none" on create-PR so it doesn't affect the gate today.
- `scripts/run_verification_hardening/main_flow.py:218` writes the canonicalized profile (e.g. `"merge"` for `pr`) into `target/verification/hardening-results.json` with `schema_version: 1` — Round-1 #9 untouched. Internal artifact; document or bump when external consumers appear.
- `runner.rs:65` `temp_root()?` early-returns before the `[sifr-case-timing]` log on tmp-dir creation failure. Cleanup is moot (nothing was created), but the timing line is lost. Cosmetic.

## Verdict

**SATISFIED.** All eight named prior blockers are properly fixed, the new `crate_tests=smoke/full` split is consistent across manifest, resolver, shell, and policy doc, and the measured 74.82s warm pass with no advisories matches the documented evidence. The two minor pre-existing items above are appropriate follow-ups, not gate-blockers.
