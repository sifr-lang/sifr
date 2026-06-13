# Validation-lane alias cleanup review (pass 1)

Scope: uncommitted diff renaming `quick|pr|full|stress` aliases away and leaving only `create-pr|merge|nightly|release`.

## Summary

The Python/shell code paths are internally consistent and the cleanup also fixes a latent rejection-check bug in two scripts (see "Positive observations"). However, the cleanup is **incomplete in two ways**: (1) several still-active documentation pages, including the authoritative `AGENTS.md` pre-PR command and `internal_docs/architecture.md`, still tell users to invoke legacy profile names that the resolver now rejects; (2) the CI workflow swap from `--profile quick` to `--profile release` in the dedicated determinism job is a large, unguarded validation-cost regression on PRs.

## Blocking findings

### 1. `AGENTS.md:41` instructs the now-rejected `--profile quick`
The project's own `AGENTS.md` (loaded via `CLAUDE.md`) declares the local-validation gate as:
```
scripts/run_all_tests.sh --profile quick      # Fast signal — use for PRs
```
After this cleanup, `validation_lane.py resolve_profile` rejects `quick` with exit code 2 (`unsupported profile: quick (supported: create-pr, merge, nightly, release)`). The mandatory pre-PR command in the project's instructions will fail. Update the line to `--profile create-pr` (and consider also adding the `--profile merge` callout that already exists in `scripts/run_all_tests.sh` usage text).

### 2. `internal_docs/architecture.md:1226-1231` documents the removed profiles, including a "Legacy alias for" pointer that no longer applies
```
./scripts/run_all_tests.sh --profile quick   # Fast local-first profile
./scripts/run_all_tests.sh --profile pr      # Authoritative merge gate
./scripts/run_all_tests.sh --profile nightly # Broad nightly validation lane
./scripts/run_all_tests.sh --profile release # Highest-confidence local qualification lane
./scripts/run_all_tests.sh --profile full    # Legacy alias for `pr`
./scripts/run_all_tests.sh --profile stress  # Legacy alias for `release`
```
Three of the four documented commands will now exit 2. Update to `create-pr`/`merge`/`nightly`/`release` and drop the alias rows entirely — they're explicitly the cleanup target per the request.

### 3. `internal_docs/verification/baseline_governance.md:8,10` calls a now-invalid profile of `run_verification_hardening.py`
```
- python3 scripts/run_verification_hardening.py --profile full
- python3 scripts/run_verification_hardening.py --profile full --bless
```
`scripts/run_verification_hardening/core.py:40` now restricts `--profile` to `("create-pr", "merge", "nightly", "release")`, so `--profile full` will be rejected by argparse with `error: argument --profile: invalid choice: 'full'`. Update to `--profile nightly` or `--profile release`, whichever matches the "baseline bless" intent.

### 4. `internal_docs/verification/fuzz_property_policy.md:12` has the same problem
```
- python3 scripts/run_verification_hardening.py --suite property --suite fuzz-smoke --profile full
```
Same root cause as #3.

### 5. CI determinism job: `--profile quick` → `--profile release` is a large unguarded PR-time regression
`.github/workflows/local-first-validation.yml:56`:
```
- run: bash scripts/check_e2e_report_determinism.sh --profile release
```
Previously the workflow invoked `--profile quick`. In the old code path, `check_e2e_report_determinism.sh` ran `validation_lane.py canonical-profile --profile quick`, which returned `create-pr` *before* the script's rejection check `if PROFILE == "quick"`. That comparison was therefore always false, so the determinism CI silently ran on the (fast) create-pr lane. The cleanup correctly fixes the dead check (see Positive observations), but the workflow now passes `release` — the heaviest lane (`warm_wall_time_target_minutes: 45`, `cold_wall_time_target_minutes: 60`), executed twice by `check_e2e_report_determinism.sh`.

Two concrete consequences:

- The `deterministic-report-signature` job has no `if: github.event_name != 'pull_request'` guard, unlike the `local-first-profiles` matrix. So this slow release-lane run executes on **every PR**, while the matrix itself restricts PRs to just the `create-pr` profile. Net effect: PRs that used to run a fast determinism check now run the heaviest lane twice.
- For push/manual events, the `local-first-profiles` matrix already runs `profile=release`, and the release lane's `extra_checks` in `verification/validation_lanes/manifest.json:158-160` already includes `e2e_report_determinism`. The dedicated `deterministic-report-signature` job is therefore duplicative on push/manual.

Recommendations: pick one of
- Pass `--profile merge` to the dedicated job (cheaper signal still outside create-pr), and add an `if: github.event_name == 'pull_request'` guard.
- Delete the dedicated determinism job entirely and rely on the `local-first-profiles` matrix entry for `release` to exercise it (on push/manual), with a `merge`-lane determinism check at PR time if PR-time determinism coverage is desired.

This is a validation-risk item, not a correctness bug, but it materially changes CI behavior and should be explicitly confirmed as intended.

## Minor / cleanup findings

### 6. Other live planning docs still reference legacy profile names in normative language
These won't run automatically but read as authoritative checklist items in active phase work:

- `internal_docs/diagnostic_emission_inventory.md:16` — `--profile quick`
- `internal_docs/tooling_verification.md:130-131` — `--profile quick`, `--profile pr`
- `internal_docs/performance_budgets.md:36-38` — `--profile quick`, `--profile pr`
- `internal_docs/generated_code_quality.md:31` — `--profile pr`
- `internal_docs/phases/29_verification_hardening.md:262-278` — listed as the canonical closeout commands using `quick`, `full`, `stress`
- `internal_docs/phases/32_async_ecosystem.md:1174` — `--profile quick`
- `internal_docs/phases/34_generated_code_quality_and_production_readiness.md` — multiple `--profile pr`, `--profile quick`
- `internal_docs/phases/35_performance_benchmarking_and_budgets.md` — multiple `--profile pr`, `--profile quick`
- `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md` — multiple `--profile pr`, `--profile quick`

`internal_docs/typescript_go_architecture_transfer_m*.md`, `reviews/archive/**`, and `issues/archive/**` are historical run records of *past* invocations and are fine to leave verbatim — but consider whether `internal_docs/typescript_go_architecture_transfer_*.md` should be moved/marked as such, since it sits alongside live docs.

### 7. Stale review-file scaffold
The untracked `reviews/gate-speed-alias-cleanup-review-1.md` was a 0-byte file before this report was written. If you keep this review under that name, you can drop the empty-file check; otherwise scrub the file path before committing.

### 8. Dead `requested_profile`/`resolved_profile` distinction (cosmetic, optional)
`scripts/validation_lane.py emit_summary` (lines 165-166) still prints both `requested_profile=` and `resolved_profile=`, and `scripts/validation_lane_report.py summarize` still emits `"profile"` and `"requested_profile"` separately (line 333-335). Without aliases these always agree. Not a bug — flagged only if you'd prefer to keep the surface tight. (Same for `RESOLVED_PROFILE=` shell export, but that one's actively consumed by `scripts/run_all_tests.sh:82`, so keep it.)

### 9. Issue file consistency check
`issues/ad-hoc-pr-gate-speed-and-validation-lane-rebalancing.md` updated the lane-names header correctly. Re-reading the updated body, line 24's "create-pr e2e, cold ... in **full lane** ..." now reads oddly (the parent table entry refers to "create-pr" yet the note says "full lane"). Likely a missed substitution — should be `... in create-pr lane, ...` or rephrased.

## Positive observations

- The cleanup also fixes a latent bug in `scripts/check_e2e_report_determinism.sh` and `scripts/check_e2e_sequential_parallel_equivalence.sh`: the prior `if [[ "${PROFILE}" == "quick" ]]` checks ran *after* canonicalization (so they never matched). The new `if [[ "${PROFILE}" == "create-pr" ]]` check actually fires.
- `scripts/run_e2e_pass.sh:71-75` `set_profile_defaults` already had a `*)` rejection arm; combined with the new lane-set-only `resolve_profile`, removed aliases now fail fast cleanly.
- Manifest renames (`quick_e2e_manifest.json` → `create_pr_e2e_manifest.json`, `pr_e2e_manifest.json` → `merge_e2e_manifest.json`) are consistent with the in-file `"lane"` fields and the manifest-level pointers in `verification/validation_lanes/manifest.json`. No stale references in the active code paths.
- `scripts/run_verification_hardening/main_flow.py` correctly removes the no-op `canonicalize_profile(args.profile)` line; the `--profile` choices in `core.py:40` now match the four canonical lanes exactly.
- `scripts/run_all_tests.sh:436,441` pass `--profile "${PROFILE}"` (a canonical lane name) to the `check_e2e_*` scripts, which will now resolve correctly without alias plumbing.

## Conclusion

There are **blocking issues** before this cleanup meets the user's request: documentation gates that still tell users to run rejected legacy profiles (`AGENTS.md`, `internal_docs/architecture.md`, two `internal_docs/verification/*.md` policy docs), plus the CI determinism job's silent shift from create-pr-equivalent to the heaviest lane on every PR. The code-side cleanup itself is consistent and even fixes a latent bug — but the request was for full removal including docs/CI consistency, and that part is still missing.
