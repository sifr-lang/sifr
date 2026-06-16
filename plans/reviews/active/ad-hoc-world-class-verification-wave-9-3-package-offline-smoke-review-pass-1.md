I've reviewed all the changed files plus the supporting fixtures, profile wiring, and policy docs. Findings below in severity order.

## Blockers
**None.** The merge smoke is fail-closed, repo-local, deterministic, and correctly wired into the merge/nightly/release split per the policy doc and phase target.

## Verifications passed during review

- **Coverage of the three required dimensions**
  - Offline registry fixture integrity: `validate_registry_sources` recomputes SHA-256 over the package tree (sorted `rglob`, NUL-separated to avoid prefix collisions) and compares to `checksum_sha256` in `registry.json`. Self-test mutates the checksum and confirms failure.
  - Deterministic lockfile generation: `build_lock` sorts packages by `package_id`, sorts dependencies in `parse_dependencies`, sorts graph edges, and emits a `graph_digest_sha256` over a canonically encoded payload. `validate_lockfile` enforces byte-equivalent JSON against the checked-in `Sifr.lock`.
  - Package graph behavior: `validate_graph_behavior` asserts the expected id set and edge set, and requires offline-source packages to carry checksums.

- **Fail-closed posture**
  - Self-test variant proves digest mutation, checksum mutation, and missing-dependency cases all trigger failures.
  - All paths are rejected for `://`, absolute, or `..` components in registry, workspace, and demo-corpus loaders.
  - Checksum format validated as lowercase hex 64 chars.
  - `FixtureError` surfaces missing offline package or cycle.

- **Profile wiring**
  - `verification/runner/sifr_verify/profile_runner.py:266-273` unconditionally runs `guardrails` then `offline-merge-smoke`, then loops over remaining `package_management` suites in the active profile (skipping the two already-run). This correctly executes:
    - merge → `offline-merge-smoke` only
    - nightly/release → `offline-merge-smoke` + `offline-integration`
  - Selected-areas entries in `merge.json`, `nightly.json`, `release.json` match the policy doc's allocation.

- **Merge/nightly boundary**
  - Merge's `offline-merge-smoke` suite is repo-local, byte-deterministic, no network.
  - Broader integration cases live only in `offline-integration` (nightly/release).
  - `verification/policy/package_management.md` explicitly states broader generated/expanded cases promote to merge only after 20 consecutive nightly green runs with no quarantine entries and no flaky retries.

- **Schema/report accounting**
  - Manifest still schema_version 1; runner converted from single-case `guardrails` special case to a generic `run_case` driven by `COMMAND_ARGS`. Variant accounting now sums `len(case_results)`; matches observed `variants=2` for `offline-merge-smoke`, `variants=2` for `offline-integration`, `variants=5` for the full area (guardrails + 2 smoke + 2 integration).
  - `zip(cases, case_results, strict=True)` correctly requires the result list and the manifest cases to line up.

- **Maintainability**
  - profile_runner.py 566 lines, area runner.py 176 lines, check_offline_package_merge_smoke.py 454 lines — all well under the 900-line cap.
  - Demo corpus lockfile digests exist on disk for every entry in `data/offline_demo_lockfile_digests.json` (sifr-demo-app, -http, -json, -test-support, -workspace).

## Non-blocking suggestions

1. **Determinism check varies only registry order.** `parse_dependencies` already sorts dependency tuples by `package_id` (line 390), and the workspace lockfile path also feeds through that sort, so the reverse-order check only proves that registry-dict iteration order doesn't affect output. Workspace dependency declaration order is already canonicalized at parse time, so the existing test cannot catch a regression in declaration-order handling. Consider also reversing `workspace.dependencies` for a second equality check, or moving the sort out of `parse_dependencies` into `build_lock` so the determinism property is exercised end-to-end.

2. **Duplicate execution in nightly/release.** `offline-integration` includes `offline-registry-lock-graph`, which is the same command as the merge smoke. Because `run_core_guardrails` always invokes `offline-merge-smoke`, nightly/release run that exact case twice (once in the always-on smoke, once in `offline-integration`). The check is fast so it's acceptable, but a tighter `offline-integration` suite containing only `demo-corpus-lockfile-determinism` would avoid the redundancy while keeping the policy boundary intact.

3. **Log line in `run_core_guardrails` is now narrower than what runs.** The `print("Running package-manager guardrails")` precedes guardrails + offline-merge-smoke + any other selected package_management suites. The label remains accurate for grep purposes but understates the scope. Optional tweak: print the suite list, or split into a separate `Running offline package merge smoke` print before the second `run_command`.

4. **Smoke is a contract test against a Python reproducer, not against `sifr_package`.** The verifier rebuilds the expected lock in Python and compares to the checked-in `Sifr.lock`. Real package-manager behavior is exercised separately by `cargo test -p sifr_package` (now in merge per Wave 1). The phase target ("offline registry smoke, lockfile determinism, and package graph behavior") is satisfied by this combination, but it's worth flagging that this single tool does not yet exercise the live resolver. A follow-up to drive the smoke through `cargo run -p sifr -- ...` (or the `sifr_package` API) against the offline fixture would close that gap — out of scope for Wave 9.3.

5. **`required_packages` is a subset check.** `parse_lock_packages` records every `name = "X"\nversion = "Y"` pair in each demo Cargo.lock; the verifier only asserts the declared subset is present. The SHA-256 over the full file is the strict gate, so this is fine, but the subset check reads as a tighter assertion than it is. Optional: rename to `expected_packages_present` or add an "exact" mode that also fails on unexpected packages.

## Recommendation

**No additional review round is needed.** Wave 9.3 is ready for PR/merge. The non-blocking items above can be threaded into a follow-up wave (or into Wave 10 closeout) if the owners want to harden the determinism check or trim the nightly duplication; none of them change correctness or the merge/nightly boundary.
