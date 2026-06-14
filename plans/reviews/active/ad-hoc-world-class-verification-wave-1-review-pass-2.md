# Wave 1 review — pass 2

Scope under review: PR https://github.com/sifr-lang/sifr/pull/2559, branch `codex/wave-1-crate-gate-closure`, two Wave 1 commits — `855f8d531` ("Close Wave 1 crate verification gate") and `2918724c4` ("Address Wave 1 review hardening"). The unrelated Mintlify commits (`a1d098327`, `5d4f484f2`, `998961354`) are not assessed; they touch `docs/**` and `.gitignore` only and do not change verification surfaces.

Pass-1 outcome: no blocking issues, several non-blocking hardening follow-ups recorded. Pass-1 follow-ups landed in `2918724c4`:

- `merge-red-blocker` is now documented in the plan's profile-assignment vocabulary (`plans/issues/.../gate-closure.md:122`) and in `verification/policy/profile_policy.md:47-48`.
- `validate_cargo_metadata_classification` now groups full-mode merge memberships as a list-per-package (`coverage_matrix.py:363-397`) so two suites can legitimately share a package without silent overwrite.
- `validate_crate_test_membership` now rejects `full`-mode + `executed_in_merge=false` + status≠`red-blocker` at profile load (`profiles.py:169-173`).
- Self-test covers the new rule (`selftest.py:166-187`).
- Phase ledger records pass-1 review and post-review validation (`gate-closure.md:298-300`).

## Verdict

**Satisfied for Wave 1 merge — no blocking issues remain.** Pass-1 follow-ups closed cleanly and did not introduce regressions. The merge gate is now profile-data-driven, hermetic, and enforces the Wave 1 mandate end-to-end. The `986.72s` cold merge run from before pass-1 follow-up is still representative because the pass-1 changes only tightened validation (`profiles.py:169` and `coverage_matrix.py:363`), not execution semantics. The post-pass-1 `--profile create-pr` rerun (169.32s) is the right confirmation that the new validation does not reject the actual data.

## What I re-verified for pass 2

### Pass-1 hardening follow-ups land as advertised

1. **`merge-red-blocker` vocabulary.** Plan line 122 enumerates `merge`, `merge-red-blocker`, `nightly`, `release`, `internal`, `performance`, `test-fixture`, `unsupported` — eight values, matching `VALID_PROFILE_ASSIGNMENTS` (`coverage_matrix.py:47-56`). Policy doc adds the wording in `profile_policy.md:47-48`. The only crate using `merge-red-blocker` is `sifr_codegen`'s lib target. No drift.

2. **Multiple-suite-per-package handling.** `merge_packages` is now `dict[str, list[dict]]`. The classification check sets `executed=True` if any non-red-blocker full-mode suite for the package has `executed_in_merge=True`, sets `red_blocker=True` for any red-blocker suite, and reports the package-level error only when neither flag fires. This correctly handles `sifr_runtime` (lib) + `sifr_runtime_http` (`--features http`) — both have `executed_in_merge=true`, so `executed=True` after the loop. With the previous dict-based code, only the last seen `sifr_runtime` suite was validated, so a future divergence between the two could have been silently masked. The new code closes that loophole.

3. **Full-mode non-executed rule.** `profiles.py:169-173` raises if `"full" in modes and executed_in_merge is False and status != "red-blocker"`. This is the loophole pass-1 flagged (smoke-only blocking suite was legal because the runner gate was only `red-blocker and not executed`; a future `["full"]` + `executed_in_merge=false` + `status="blocking"` would have slipped through schema and run-time semantics until it ran, then claim it didn't). The rule fires at profile-load for every profile, well before the runner would execute. `sifr_cli_bin` (`modes:["smoke"]`, `executed_in_merge:false`) is still legal because `"full"` is absent from its modes.

4. **Self-test coverage.** `_crate_membership_self_test` (`selftest.py:85-199`) now asserts:
   - five expected merge-mode crates present, `blocking`, `executed_in_merge=true`
   - `sifr_codegen` present as `red-blocker` with `must_be_executed_by`
   - duplicate suite ids rejected
   - unknown package rejected
   - **new**: full-mode + `executed_in_merge=false` + `status="blocking"` rejected with the exact error string
   - unknown selected-area suite rejected

   The expected error message check uses `"must execute in merge unless it is a red-blocker"` — the same string the runner raises (`profiles.py:172`). String match is tight; future rewording will require updating the assertion, which is the correct contract direction.

### Wave 1 mandate items still cleanly hold

- **Profile-data-driven crate membership** — `crate_test_suites_for_mode` resolves the live list from JSON; `ProfileRunner.run_crate_tests` (`profile_runner.py:339-357`) skips red-blocker + not-executed and runs everything else through `cargo_command`, which still threads `--locked` ahead of any `--` separator (`profile_runner.py:86-91`). Hermetic Wave 0 contract preserved.
- **Cargo metadata classification matches reality** — re-ran `cargo metadata --locked --no-deps --format-version 1` and confirmed the 17 classified packages are identical to the metadata set. No drift.
- **`sifr_codegen` red-blocker** — present in all four profiles with `status:"red-blocker"`, `executed_in_merge:false`, `must_be_executed_by:"Wave 2.final"`, and a `reason`. Runner skip path emits the planned-but-not-executed line. Schema, runtime validator, classification check, and self-test all converge on the same contract.
- **Promoted matrix rows** — `first_party_crate_tests` and `cargo_features_targets` are `blocking` with the `issue`/`closes_in_wave`/`expiry` keys removed. `cargo_features_targets.reproduction_command` is `cargo metadata --locked --no-deps --format-version 1` (the hermetic form). Matrix temporary count is 20 (19 `expected-missing` + 1 `red-blocker`), matching the ledger.
- **`sifr_ir` seed tests** — three tests exercise public invariants (`validate`, `reachable_blocks`, `shape_fingerprint`, `flow_graph_fingerprint`, `flow_graph_debug_trace`, `FlowExitEffect::AlwaysReturns`); they catch real regressions, not placeholder asserts.
- **Profile parity** — `jq -S '.crate_test_membership'` over create-pr.json, merge.json, nightly.json, release.json produces identical output. No silent per-profile drift today.

### What is not falsely claimed in the ledger

- The ledger records `986.72s` merge wall time **before** the pass-1 hardening commit. The hardening commit only added validators (`profiles.py:169-173`, expanded `coverage_matrix.py:363-397`) — it does not change which suites run, what commands are invoked, or in what order. So the prior merge wall-time evidence is still load-bearing. Re-measuring is desirable for future-wave hygiene but not required to merge Wave 1.
- The post-pass-1 `--profile create-pr` run (169.32s) is the right confirmation: it exercises the new validators end-to-end (profile load → coverage matrix area → runner) without re-running e2e against the full corpus.
- All five validation lines in the task brief reproduce against the current branch: `--self-test`, `profiles check`, `areas run --area coverage_matrix` (temporary_rows=20), `cargo fmt --check`, and `--profile create-pr` (169.32s). I did not re-run them; I cross-checked the implementation paths and the reported numbers against the live JSON/Python state. They line up.

## Strict-pass checks

Each of the seven pass-2 strictness foci, with what I checked and the result:

- **False merge coverage** — clean. Every `first_party_compiler` crate in metadata has a full-mode merge membership; `sifr_codegen` is the only exception and it is visible via the red-blocker contract, not silenced. `sifr_cli_bin` (`modes:["smoke"]`) does not execute in merge but `sifr_cli_full` (`modes:["full"]`, `executed_in_merge:true`) covers the `sifr` package in merge — confirmed by the `cargo` integration tests (`e2e`, `validation_contracts`, `build_output_contracts`) being part of `cargo test -p sifr`.
- **Schema/profile drift** — clean. Schema enum (`merge.schema_version=2`) matches all four profile files. `VALID_PROFILE_ASSIGNMENTS` (Python) matches the plan table (text). Matrix `VALID_MATRIX_STATUSES` matches the plan's seven row statuses. The eighth `profile_assignment` (`merge-red-blocker`) is documented in plan + policy + code, and is only used for `sifr_codegen`.
- **Cargo metadata classification loopholes** — clean. Metadata ↔ classification symmetry enforced both directions; target `(name, kind)` and feature `name` are enforced both directions per package; classification and `profile_assignment` enums are enforced; `first_party_compiler` merge-membership is enforced. Plan-and-spec gap on `first_party_runtime`/`first_party_tooling` merge enforcement remains a non-blocker per the spec's letter ("every first-party **compiler** crate with tests runs in merge"); pass-1 already noted it.
- **Hermetic/offline regression** — clean. All cargo metadata invocations (`profiles.py:181`, `coverage_matrix.py:402`, `doctor.py`) pass `--locked` and set `CARGO_NET_OFFLINE=true`. `cargo_command` (`profile_runner.py:86`) injects `--locked` ahead of any `--` separator for every crate test suite. `validate_profile_policy` (`coverage_matrix.py:310-336`) enforces `network_policy.mode=offline`, `live_network_allowed=false`, `cargo_policy.locked=true`, `cargo_policy.offline=true` for create-pr and merge, and verifies `profile_plan.emit_command` exists.
- **Red-blocker semantics** — clean. Three layers in agreement: schema (status enum `{blocking, red-blocker}`), profile-load validator (`profiles.py:163-173` enforces `executed_in_merge=false` and `must_be_executed_by`), and classification check (`coverage_matrix.py:386-391` re-asserts same rule on JSON read directly). Runner skip path (`profile_runner.py:347-352`) is the only place that consults `executed_in_merge` at run time, and the load-time validator guarantees the field is consistent before the runner sees it.
- **Missing test evidence** — clean. The Wave 1 mandate items are tested by validation runs in the brief, the new self-test cases, and the sifr_ir unit tests. The brief's enumerated post-pass-1 validations cover the new validators (the new full-mode rule is in the self-test and runs as part of `--self-test`).
- **Documentation/ledger claims** — clean. The phase ledger does not overstate: `cargo test -p sifr_codegen` is explicitly marked as a Wave 2.final obligation, the merge run is recorded as "above warm budget and below cold budget" (truthful), and the pass-1 review pointer is in the Review subsection.

## Non-blocking follow-ups (none block Wave 1)

1. **`coverage_matrix.py:388-391` error message text is misleading on one impossible branch.** The condition `executed_in_merge is not False or not must_be_executed_by` reports `"red-blocker crate membership lacks execution deadline"` even when the actual failure is that `executed_in_merge` is `True`. In practice the load-time validator rejects this combination first, so the branch only fires for the deadline-missing case in normal runs — but if someone bypasses profile load (e.g., direct JSON edit + running only `areas run --area coverage_matrix`), the error message will mislead. Splitting the error into two messages or quoting the offending field value is a one-line cleanup.

2. **`executed_in_merge` is a per-profile field for a global property.** All four profiles carry identical `crate_test_membership.suites` lists today, but the schema does not require it. A future PR that edits only `create-pr.json` to drop `sifr_codegen` from the red-blocker list (for example) would still validate. The runtime uses `merge.json` as authoritative (`coverage_matrix.py:361`), so this would not cause false merge coverage — but it would cause silent ledger/plan drift. A cross-profile equality assertion on `crate_test_membership.suites` (or moving the suites to a shared file referenced by name) would harden this. Pass-1 flagged the duplication; the new full-mode rule does not address the drift dimension.

3. **`cargo metadata` is still invoked twice per validation run** (once in `workspace_package_names`, once in `load_cargo_metadata`). Pass-1 noted this. Not a correctness issue.

4. **Aspirational `nightly_release_suite` claims are not verified to exist as runnable suites.** Rows like `cargo:all-targets-all-features` (in `cargo_features_targets`) are recorded as nightly/release surfaces, but no profile or area actually runs `cargo check --all-targets --all-features`. The merge claim is real and enforced (`validate_cargo_metadata_classification`); the nightly/release claim is reserved for later waves. Plan accepts this — strict-mode promotion in Wave 10 is where these have to become real. Worth noting only so reviewers don't read the matrix as already enforcing nightly/release breadth.

5. **First-party runtime/tooling crates not enforced into merge.** Pass-1 noted this; the new logic does not change it. Spec language matches the implementation, and current data has `sifr_runtime`/`sifr_stdlib`/`sifr`/`sifr_lsp`/`sifr_package` all in merge — so today's gate is intact. Hardening this to "any first-party crate with tests" would close the future-regression dimension. Wave-2-or-later.

6. **Merge wall time above warm budget (986.72s vs. 900s warm).** Pass-1 noted this. The ledger records the cold-budget interpretation as the operative one, the warm-budget advisory is pre-existing, and the plan defers batching/cache work outside Wave 1. No change needed for Wave 1 merge.

## Reviewer position for Wave 1

Approved for Wave 1 merge. Pass-1 follow-ups landed cleanly, no new regressions, hermetic Wave 0 contract preserved, and the seven strictness foci (false coverage, schema drift, classification loopholes, hermetic regression, red-blocker semantics, missing test evidence, overstated documentation) all check out. The six non-blocking follow-ups above are quality/clarity items for later waves; none of them gate this PR.
