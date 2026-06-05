# Validation-lane alias cleanup review (pass 2)

Scope: re-review of the uncommitted diff after remediation of pass-1 blockers. Verifying that the legacy profile aliases `quick|pr|full|stress` are fully removed as accepted profiles and from live/normative docs/checkers, while CI behavior remains reasonable for PR speed.

## Status of pass-1 blockers

1. **`AGENTS.md:41` rejected-profile instruction** — fixed. Now reads `scripts/run_all_tests.sh --profile create-pr` (fast) and the default `scripts/run_all_tests.sh` (merge gate). Pre-PR command resolves cleanly through `validation_lane.py resolve_profile`.
2. **`internal_docs/architecture.md:1226-1231` rejected profiles + alias rows** — fixed. The two alias rows are gone; the four canonical rows (`create-pr`, `merge`, `nightly`, `release`) remain. The trailing paragraph also drops the `quick`/`pr` phrasing in favor of `create-pr`/`merge`.
3. **`internal_docs/verification/baseline_governance.md:8,10` `--profile full`** — fixed. Both verify and bless commands now use `--profile merge`, which is a valid choice in `run_verification_hardening/core.py:40`.
4. **`internal_docs/verification/fuzz_property_policy.md:12` `--profile full`** — fixed. Now `--profile merge`.
5. **CI determinism job regression** — fixed. `.github/workflows/local-first-validation.yml:52-57` now (a) guards the dedicated `deterministic-report-signature` job with `if: github.event_name == 'pull_request'`, and (b) passes `--profile merge` instead of the heaviest lane. The `local-first-profiles` matrix is now `[create-pr, merge, release]` with PRs restricted to `create-pr`, so push/manual broader coverage (including determinism via `extra_checks` on the release lane) flows through the matrix. This matches the stated intent.

Minor pass-1 findings:

6. **Other live planning docs with legacy profile names** — fixed. Grep for `--profile (quick|pr|full|stress)` against `internal_docs/`, `scripts/`, `verification/`, `.github/`, and live `issues/` (excluding `issues/archive/`) returns no matches. Historical run records under `issues/archive/`, `reviews/archive/`, and the prior pass-1 review file legitimately retain past invocations for traceability and are correctly out of scope.
7. **Empty review-1 scaffold** — resolved. `reviews/gate-speed-alias-cleanup-review-1.md` now contains the pass-1 report content.
8. **Cosmetic `requested_profile`/`resolved_profile` redundancy** — left as-is, consistent with pass 1's "cosmetic, optional" note.
9. **Issue file "full lane" wording on line 24** — fixed. The entry now reads `57.34s in create-pr lane, 49.00s test body`. Surrounding language in Root Causes #1 and #4-5 was also harmonized (`primary authoritative merge-gate blocker`, `broad-lane process conditions`, `create-pr and merge gates`).

## Independent verification

Re-running the user's validation checklist:

- `bash -n` of the four shell scripts → `BASH_SYNTAX_OK`.
- `python3 -m py_compile` of the five Python files → `PY_COMPILE_OK`.
- JSON parse of `manifest.json`, `create_pr_e2e_manifest.json`, `merge_e2e_manifest.json` → `JSON_PARSE_OK`.
- `python3 verification/tooling/check_phase36_closeout.py --self-test` → `phase36 closeout self-test: PASS`.
- `python3 verification/tooling/check_phase36_closeout.py` → `phase36 closeout: PASS`.
- `python3 scripts/validation_lane.py summary --profile create-pr` → resolves to lane `create-pr`, 67 fixtures, smoke generated-code-quality/perf-budget/crate-tests.
- `python3 scripts/validation_lane.py profile --profile merge` → prints `merge`.
- `python3 scripts/validation_lane.py profile --profile quick` → stderr `unsupported profile: quick (supported: create-pr, merge, nightly, release)`, exit 2 (correctly rejected).
- `python3 scripts/run_verification_hardening.py --profile quick --suite diagnostics` → argparse error `invalid choice: 'quick' (choose from create-pr, merge, nightly, release)`, exit 2 (correctly rejected).
- `cargo fmt --check` → no diff.
- `python3 scripts/check_file_size_guardrails.py` → `PASS (2058 files, limit 900 lines)`.
- `git diff --check` → no whitespace issues.

The `--profile create-pr` wall-time data point cited by the user (114.58s, advisories=none, e2e cache_hits=18/18, manifest=`create_pr_e2e_manifest.json`) is consistent with the manifest's `create-pr` lane (67-fixture e2e manifest, smoke modes, 2/2/2 e2e jobs).

## Additional spot checks

- `scripts/validation_lane.py` no longer exposes a `canonical-profile` subcommand; the remaining subcommands (`profile`, `shell`, `summary`) only operate on the four manifest-defined lanes. `resolve_profile` requires exact membership in the lane set — no compatibility paths.
- `scripts/run_verification_hardening/core.py:40` restricts argparse `--profile` choices to exactly `("create-pr", "merge", "nightly", "release")`. `main_flow.py` no longer canonicalizes the arg.
- `verification/validation_lanes/manifest.json` declares exactly four lanes (`create-pr`, `merge`, `nightly`, `release`), with `create-pr` and `merge` pointing at the renamed manifests, and `release.extra_checks` carrying `e2e_report_determinism` + `e2e_sequential_parallel_equivalence` (so push/manual coverage flows through the matrix release lane as intended).
- `scripts/check_e2e_report_determinism.sh` and `scripts/check_e2e_sequential_parallel_equivalence.sh` retain their rejection arms; with `validation_lane.py profile --profile <X>` now only returning a canonical lane name (or exiting 2), the post-canonicalization comparison `PROFILE == "create-pr"` is no longer the dead check it used to be — it now meaningfully fires for the create-pr lane.
- The cleanup correctly leaves unrelated package-/import-/type-aliases untouched: the diff is scoped to validation-profile names plus dependent manifest renames and doc updates.

## Conclusion

No blocking issues. The cleanup meets the user's request: only `create-pr`, `merge`, `nightly`, and `release` are accepted; the legacy names `quick`, `pr`, `full`, and `stress` are rejected by both `validation_lane.py` and `run_verification_hardening` argparse; live normative docs (`AGENTS.md`, `internal_docs/architecture.md`, validation-lane policy, baseline governance, fuzz/property policy, phase docs, performance budgets, generated-code quality, tooling verification, diagnostic emission inventory, the active issue file) have been updated to the canonical names; the e2e manifests are renamed to `create_pr_e2e_manifest.json` / `merge_e2e_manifest.json` and referenced consistently; and the CI determinism job is restricted to pull requests on `--profile merge`, with push/manual determinism coverage flowing through the matrix release lane.
