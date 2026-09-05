## Review: `milestone_40_0` — Architecture and Gate Lock

Scope inspected: full working-tree diff (37 modified files), all 30 untracked additions, the Phase 40 plan, `AGENTS.md`, and `plans/issues/active/phase-40-stable-channel-ga-execution.md`.

The governance core is genuinely strong: `governance/*` validators are strict, key-exact, fail-closed, and reject the zero digest; `load_json_strict` rejects duplicate keys and non-canonical bytes; the schema-epoch guard plus `selftest.py` mutation battery cover most of the DoD's negative list; the atomic v2 cutover reached the Rust CLI, receipt schema, dispatcher, metadata generator, workflow, and fixtures with no reader/negotiation left behind. The findings below are what stands between that and approval.

---

### Blocking / actionable

**1. The `release` profile does not select `rust_interop:stable-candidate`, and `release_report.py` requires it — so `--release-report-out` can never produce a report.**
`verification/profiles/release.json` selects `rust_interop: ["matrix","tiers","compatibility-matrix","stale-drafts"]`, but `verification/areas/distribution_release/governance/release_report.py:32-43` lists `stable-candidate` in `REQUIRED_SUITES`. `validate_profile` (`release_report.py:148-154`) therefore fails every generated report with `missing required rust_interop suite(s): stable-candidate`. The milestone DoD ("The `release` profile visibly executes the Rust-interop step and all four structural suites plus `stable-candidate`") and the scope item "Register the upstream stable-candidate validator in the Rust-interop manifest as the `stable-candidate` suite" are both unmet — `verification/areas/rust_interop/manifest.json` is untouched.

This is the upstream prerequisite you asked me to scope rather than implement. **Precise integration once it lands on `origin/main`:**
- add the upstream validator as a `stable-candidate` suite in `verification/areas/rust_interop/manifest.json` (upstream's job; verify it merged);
- append `"stable-candidate"` to `selected_areas[rust_interop].suites` in `verification/profiles/release.json` — that single edit flows through `ProfileRunner.run_rust_interop_checks` (`profile_runner.py:471-491`, which derives suites from `selected_areas` and passes them to `validate_area_result`) and satisfies both `REQUIRED_SUITES` checks;
- validate the initial `stable_support_claims.json` against the Phase 39 matrix and tick the two corresponding boxes in the execution issue.

No other code change is required. What *is* required now: the milestone cannot be reported as done while its central artifact path is inert. Either land the integration or explicitly record in the issue that 40.0 closes with a known-blocked DoD item.

**2. `profile_runner.py` is exactly 899 lines, reached by deleting blank lines rather than splitting.**
The diff removes the blank line after the module docstring (`profile_runner.py:1-2`) and the one before `class ProfileRunnerError` (`:26-27`) — leaving PEP8-invalid spacing and zero headroom under the 900-line cap. Milestone scope is explicit: "Preserve the 900-line source cap; **split profile-step execution by responsibility** before adding the inherited Rust and documentation steps if the combined file approaches it." Extract the step-execution methods into a sibling module and restore the blank lines.

**3. The preview release-index compare-and-swap cannot fail, and nothing re-verifies before replacement.**
`.github/workflows/preview-release.yml:194-215` fetches the index to `${current_metadata}`, computes `expected_generation`/`expected_sha256` **from that same file**, then passes both plus `--current "${current_metadata}"` to `update-preview-index`. `release_governance.py:207-211` compares the file against digests derived from itself — the check is structurally incapable of failing. The actual mutation (`gh release upload channels ... --clobber`, `preview-release.yml:306`) happens three steps later with no re-fetch. Identical pattern in `scripts/distribution/create_new_version.sh:359-387`. Plan lines 149-152 and 310-314 require the mutating workflow to *reacquire and verify* generation and digest inside the concurrency group and immediately before replacement. Today the only real serialization is the `sifr-release-index` group; the digest machinery is decorative. Re-fetch and re-verify in the publish step, or drop the parameters so the guarantee isn't overstated.

**4. Immutable release provenance may name a commit that did not build the artifacts.**
`preview-release.yml:91-94` checks out the moving `base_ref` in `build-artifacts`; `:135-138` checks out `base_ref` again in `publish-release`; `:200` records `source_commit="$(git rev-parse HEAD)"` from that second resolution into the governed release record. If `main` advances between jobs, the write-once index binds the wrong commit. Plan lines 276-278: "resolves a requested ref to one commit before building … A moving branch name is never recorded as release provenance." Resolve to a SHA in the `validate` job and use it for every checkout and for `build-release-record`.

**5. Evidence-custody base-ref resolution fails open.**
`governance/evidence_custody.py:81-108`: if `merge-base origin/main HEAD` and `rev-parse HEAD^` both fail (shallow clone, detached first commit, missing remote), `merge_base` is `""`, the committed-range diff is skipped entirely, and `changed_paths()` degrades to working-tree + untracked files only. A commit mixing compiler source with release evidence then passes the custody suite. The suite is the DoD's source/evidence separation gate — it must fail closed when it cannot establish a base.

**6. The canonical preview mutator lost the only channel-downgrade guard in the repo.**
`release_governance.py:202-226` rejects re-adding an existing version but happily points `beta` at a *lower* new version; `validate_release_index` enforces class and active-status but not ordering. The deleted `scripts/distribution/bootstrap_channel_metadata.py` and its case `channel_metadata_bootstrap_from_github_releases.sh` were the only place enforcing "refusing to downgrade beta channel from X to Y", and both are removed here with no replacement. A backward channel move breaks every client (`sifr self update` treats it as a downgrade requiring `--force`). Add a monotonicity check to `update_preview_index`.

**7. The milestone's central new capability has no end-to-end validation.**
Only `_release_report_precondition_self_test` (`selftest.py`, three rejection cases) exercises `--release-report-out`. Nothing tests `write_release_profile_report`, `build_steps`, or `collect_critical_results` producing a schema-valid report — and per finding 1 it currently cannot. Note also that `validate_release_profile_report` re-runs the clean-tree check *after* the lane executed (`release_report.py:107-116`); any untracked, non-ignored byproduct of the release lane makes the report permanently unobtainable, and nothing proves the release profile is clean in that sense. Add a fixture-level test that drives `write_release_profile_report` against synthesized lane logs and area results. `milestone_40_1` binds and hashes these exact bytes; the path must be proven before it does.

**8. `create_new_version.sh` prefers a local site-checkout index over the canonical one, and adds a network fetch.**
`create_new_version.sh:359-366` uses `${INSTALL_ROOT}/channels.json` when present and otherwise `curl`s the governance asset. The plan requires deleting local `channels.json` shadow paths precisely "so schema-v1 residue cannot override the canonical fetched index" (line 632-634), and requires local tooling to be plan/dry-run only and deterministic. Deriving the CAS baseline from the site checkout — which may lag the governance release — is the weakest possible source for that value. Also note the milestone demo must run "without network access"; this puts `curl` on a local `real_run` path.

**9. Milestone-scope creep: `rc` removal from non-JSON runtime and workflow surfaces is `milestone_40_2` work.**
Plan lines 578-585 assign "installer `APP_CHANNEL` derivation, dispatcher exact-pin parsing, and `preview-release.yml` inputs, plus their tests and docs" to 40.2, and state that only "the schema, receipt, CLI, self-update-plan, and fixture removals" occur in 40.0. This diff removes `rc` from `generate_dispatchers.sh:80-86`, `generate_version_installer.sh:70,224-227`, `build_preview_artifacts.sh:82`, `trigger_preview_release.sh:93,158`, `create_new_version.sh:117,174`, and `preview-release.yml:59`. Not a correctness defect, but it is out of the exact milestone scope you asked me to hold the line on; either revert those surfaces to 40.2 or record an approved scope amendment in the execution issue.

---

### Non-blocking suggestions

- `generate_dispatchers.sh:123-131`: `validate_preview_release_index` uses `grep -Eq` over raw metadata text, so `"ga_status":"preview"` or a positive `"generation"` occurring anywhere satisfies it, and the resolved version's release record is never checked for `status: active`. Defensible for generated POSIX-sh dispatchers with the structural validator in `validate_self_update_metadata.sh` behind it, but flag it for replacement in 40.2.
- `release_index.schema.json:40` and `stable_release_plan.schema.json:159` allow the all-zero digest; `release_profile_report.schema.json:117-122` excludes it. The Python validators reject it everywhere — align the schemas for defense in depth.
- `release_evidence.py:209-213` and `:247-251` hardcode the `developer_tooling:full → editor-release` expansion in the producer, and `release_report.py` requires it in the validator — a closed loop. The `editor-release:*` labels also appear twice in the report (under `full` and under the synthesized `editor-release` entry), which reads oddly against "case evidence exactly once". `governance/selftest.py:364-380` does independently assert `FULL_SUITES.count("editor-release") == 1`, which is the useful half.
- `selftest.py:_documentation_profile_self_test` executes the real documentation area via `uv` with a hardcoded `ProfileRunner("release")` inside *every* profile's hardening self-tests. On the release lane the docs area runs twice and the report's `documentation-release-results.json` digest is whichever invocation ran last; on other lanes it silently creates release-named evidence. Prefer asserting on the already-produced result file.
- `demos/milestone_40_0_demo.sh:20` imports its plan from `governance.selftest.valid_plan()`. Using the mutation-test fixture as the demo's product input couples the demo to test code; a checked-in fixture JSON would be cleaner.
- `plans/reviews/active/phase-40-milestone-40-0-agent-review-pass-1.md` is a zero-byte file.
- `plans/releases/stable_gate_inventory.json:10,18` set `target_milestone` `40.5` for the preview workflow and trigger, but the plan assigns the preview-publication refactor to 40.2.
- `run_all_tests.sh:51-54`: `--release-report-out` with a missing value dies on `shift 2` under `set -euo pipefail` with no diagnostic.
- `check_structure.py` doesn't verify that an `active` inventory check's `suite` exists in `manifest.json` (it holds today only by coincidence).
- `self_update_metadata.rs`: a stable release record inside a preview index is rejected via `PreviewVersion::parse`, surfacing the misleading "stable-looking versions are disabled" message rather than a governed-index diagnostic.
- The execution issue has every `milestone_40_0` checkbox unchecked and no commands, evidence, review rounds, or PR link recorded, while the plan's Validation Contract requires each milestone to record them.

---

### Verdict

**CHANGES REQUESTED** — nine blocking/actionable findings, chiefly: the release report is unproducible because the profile omits `rust_interop:stable-candidate` while the validator mandates it (upstream integration point identified above); the preview index CAS is tautological with no re-verification before replacement; release provenance can bind a commit that did not build the artifacts; evidence custody fails open on base-ref resolution; the channel-downgrade guard was deleted without replacement; and the 900-line cap was met by stripping blank lines instead of the required responsibility split.
