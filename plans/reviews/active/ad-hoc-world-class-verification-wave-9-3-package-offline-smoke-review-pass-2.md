# Pass 2 Review — Wave 9.3 Package Offline Smoke

## Blockers
**None.** All four cleanups from pass 1 landed correctly and validate cleanly.

## Cleanup verification

1. **Duplicate removed from `offline-integration`** — `manifest.json` now lists `offline-integration` with only `demo-corpus-lockfile-determinism`; the lock-graph case lives exclusively in `offline-merge-smoke`. Profile flow: merge runs `offline-merge-smoke` (unconditional path skips the loop entry), nightly/release run `offline-merge-smoke` plus `offline-integration`. No double-execution. (`verification/areas/package_management/manifest.json:25–57`, `verification/runner/sifr_verify/profile_runner.py:269–275`)

2. **Explicit profile-runner logs** — Both `Running offline package merge smoke` and `Running package-management suite {suite}` print in the right places. (`profile_runner.py:269,274`)

3. **`required_packages` → `expected_packages_present` rename** — Data file uses the new key for all five entries; checker reads it in `run_demo_corpus_check` at line 190 and the failure-message label matches (`...expected_packages_present`). (`data/offline_demo_lockfile_digests.json`, `tools/check_offline_package_merge_smoke.py:190–195`)

4. **Workspace dependency-order determinism assertion** — Added at `tools/check_offline_package_merge_smoke.py:107–114`. Reverses `workspace.dependencies` and asserts the resulting lock equals the canonical lock. Complements the existing reversed-registry assertion.

## Validation
- Direct, `--self-test`, and `--demo-corpus` runs each print `offline package merge smoke ok` (exit 0).
- All five corpus lockfiles listed in `offline_demo_lockfile_digests.json` exist on disk.
- `run_case` now centralises the command→args map in `COMMAND_ARGS` and fails closed on unknown commands; `failed_cases`/`total_variants` math is correct under the 1-variant-per-case schema.

## Non-blocking observations (no action required)
- `offline-merge-smoke` is both listed in `merge.json` *and* run unconditionally by `profile_runner` (the loop skips it). The selected-area entry is currently documentary — harmless, but if you ever want a single source of truth, you could drop the unconditional `offline-merge-smoke` invocation and lift it into every profile via `selected_areas`. Leaving as-is is the safer choice since it guarantees the merge smoke runs on every profile regardless of profile edits.
- `parse_lock_packages` relies on `name`/`version` being adjacent lines in Cargo.lock; this is true for `cargo`'s canonical formatter but is technically format-dependent. A future Cargo bump could invalidate it. Not worth pre-empting.

## Verdict
**No further review round needed.** The pass-1 cleanups are correctly applied, the checker remains deterministic and fails-closed, profile wiring is consistent with the policy doc, and all advertised validation passes. Ready to land.
