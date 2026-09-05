## Review — Phase 40 / milestone_40_1 (uncommitted working tree)

Scope reviewed: diff vs `origin/main` (27 modified, 9 added files) against `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` §`milestone_40_1`, `plans/issues/active/phase-40-stable-channel-ga-execution.md`, and `AGENTS.md`.

The shape is right: read-only qualification workflow, exact-SHA identity gates, immutable 30-day artifact custody, a non-mutating `plan-stable-release` command whose output must land outside the checkout, and a workflow-contract case that mechanically pins permissions/matrix/upload immutability. Naming is capability-based throughout (`build_release_artifacts.sh`, `artifact_stable_candidate_generation.sh`, `release_qualification_workflow_contract.sh`) — no phase/milestone identifiers. The `rc` and `stable` boundaries in `generate_version_installer.sh` and `self_update_receipt.rs` stay inside 40.1's remit.

The blocking problems are in *validation depth*: the planner — the milestone's central deliverable — has no test coverage at all, its determinism test is vacuous, and its inputs are not constrained tightly enough to keep it fail-closed.

---

### P1 — Blocking

**1. `test_plan_digest_sensitivity` proves nothing about the planner.**
`verification/areas/distribution_release/governance/qualification_selftest.py:195-215` builds `release_plan()` (a static fixture dict), mutates a field, and asserts the SHA-256 of the two different byte strings differs. That is a test of `hashlib`, not of plan materialization. The DoD items *"A fixture-backed dry run produces a byte-deterministic, schema-complete plan for identical inputs"* and *"Changing any commit, submodule, lockfile, version, target, artifact, sysroot, installer, or Rust-claim input owned by this milestone changes the fixture plan digest"* are therefore unproven. The test must run `materialize_stable_plan` (or `plan-stable-release`) twice over a fixture evidence tree, compare bytes, then perturb each *input artifact* and require the run to either fail closed or produce a different digest.

**2. `materialize_stable_plan` is entirely untested.**
The only planner symbols imported by any test are `resolve_source_once`, `stable_claim_ids`, `validate_target_report` (`qualification_selftest.py:14`). Unexercised: `validate_source_identity`, `verify_transported_artifacts`, `bind_target_reports`, `bind_aggregate_artifacts`, `validate_aggregate_checksums`, the release-report↔plan cross-binding (`planner.py:74-91`), the qualification-index↔plan provenance check (`planner.py:93-112`), and the outside-the-repo output guard (`scripts/distribution/release_governance.py:254-260`). Consequently none of the milestone's required negative cases exist: stale report, cross-target artifact, expired artifact, missing target evidence, a qualification run for another source commit, version/digest drift. `resolve_source_once(REPO_ROOT, "main")` (`qualification_selftest.py:188`) only reaches `require_commit`, so the "floating ref" case tests an argument regex, not ref resolution.

**3. Planner raises raw `KeyError` on artifact ids the validator never constrains.**
`planner.py:249` (`artifact_paths[f"qualification-report-{target}"]`), `:276`, `:286`, `:289`, `:292`, `:294` index by canonical id, but `validate_qualification_artifact_index` imposes no id-set requirement — only the pattern `^[a-z0-9][a-z0-9_.-]+$`. Verified locally:

```
ACCEPTED: renamed report ids pass validation (planner then KeyErrors)
```

`release_governance.py:164` catches only `GovernanceError`, so this surfaces as an unhandled traceback rather than a governed diagnostic — the exact class of defect passes 3–5 of the 40.0 review eliminated. Either require the exact id set in `artifact_index.py` or resolve ids through a governed lookup.

**4. `workflow_artifact_name` is a path-traversal hole in the custody boundary.**
`verification/areas/distribution_release/schemas/qualification_artifact_index.schema.json:41` declares `{"type": "string", "minLength": 1}` and `artifact_index.py:143-186` checks only prefix (and suffix for target artifacts). `name` correctly forbids `/` and `..` (`artifact_index.py:191-192`), the directory component does not. Verified:

```
ACCEPTED: traversal in workflow_artifact_name passes validation
```

`planner.py:227-231` then joins it under `--artifact-root`, so a validated index can make the planner hash a file outside the transported artifact root and bind it as candidate evidence. Apply the same `^[^/]+$`-style constraint to `workflow_artifact_name` in both schema and validator.

**5. Report coverage check is `<` where it must be exact and per-kind.**
`artifact_index.py:199`: `if report_count < len(TARGETS) + 1`. Extra, unattributable `report` artifacts pass, and nothing distinguishes the four per-target reports from the editor report. Verified:

```
ACCEPTED: extra unattributed report artifact passes
```

Require exactly one `qualification-report-<target>` per governed target plus exactly one `editor-qualification-report`.

**6. `installer_version` and `receipt_channel` in the target report are echoes, not evidence.**
`scripts/distribution/qualify_stable_target.py:185-187` sets `sifr_version`/`installer_version` to the `--version` argument and `receipt_channel` to the literal `"stable"`. The receipt itself is hand-constructed at `:133-149` rather than produced by the immutable installer, and the aggregate installer is built in a different job (`assemble`) that the matrix job never installs. So the milestone's *"Verify binary version, sysroot manifest, archive contents, artifact digest, installer digest, target, and release-plan agreement"* holds for the binary version (`:120`), sysroot manifest, and archive digest — but `installer_version` is never compared to the generated installer's `APP_VERSION`, and `planner.py:256-270` "verifies" plan agreement against these self-attested constants. Either install through the generated installer in the matrix job, or bind the installer's embedded `APP_VERSION`/`APP_CHANNEL` in the assemble stage and cross-check it there.

---

### P2 — Should fix before merge

**7. Unrequested fallback in editor qualification.**
`.github/workflows/release-qualification.yml:180-183`: `package.get("sifrCompilerCompatibility", "not-yet-materialized")`. AGENTS.md forbids fallback paths absent an explicit request; the placeholder becomes signed candidate evidence, and `planner.py:303-311` only checks equality against whatever the plan says, so the placeholder can propagate into a plan unnoticed. Fail closed on a missing compatibility range.

**8. Editor VSIX selection is nondeterministic and its submodule SHA is unchecked.**
`release-qualification.yml:162` copies `dist/*.vsix` (glob) and `:170` takes `next(...glob("*.vsix"))` — arbitrary selection if `dist/` holds more than one, and `collect_qualification_artifacts.py:240-246` derives the *expected* VSIX name from what was observed, so it cannot detect the wrong one. `:175` uses `os.popen(...)` without checking exit status: a failed `git rev-parse` silently records `submodule_commit: ""`, which nothing validates. Assert exactly one VSIX and use a checked `subprocess.run`.

**9. Candidate builds bypass the governed release build path.**
`release-qualification.yml:97` runs bare `cargo build --locked --release`, then packages via `build_release_artifacts.sh --binary` — the path whose own help text says *"intended for local validation fixtures"* (`build_release_artifacts.sh:23`). The production `--cargo-build` path (`:283`) is what applies `release_rustflags` path remapping, and it is skipped, so candidate `binary_sha256` embeds runner-local checkout/cargo-home paths and diverges from the reproducible recipe documented in `internal_docs/distribution_pipeline.md:202-208`. Separately, `--cargo-build` at `:283` still omits `--locked`, contradicting *"Build on each supported target with locked dependencies."*

**10. `assemble` inputs are timing-dependent.**
`release-qualification.yml:202-217`: the job needs only `build`, but downloads `sifr-stable-candidate-<v>-<sha>-*` with `merge-multiple: true`, so whether the `editor` artifact lands in `target-artifacts` depends on job scheduling. `checksums.txt` is unaffected today only because it globs `*.tar.gz`. Narrow the pattern to the four target artifacts.

**11. Repository custody is asserted, not verified.**
`collect_qualification_artifacts.py:154` hardcodes `"repository": "sifr-lang/sifr"`, which `artifact_index.py` then validates as a constant. A fork or mirror run emits an index falsely attributing custody to the canonical repository. `workflow_run.head_sha` is carried in the run metadata but never compared, and `--run-attempt` is taken on trust. Pass and check `GITHUB_REPOSITORY`.

---

### P3 — Tracking, scope, and residue

**12. Execution issue not updated.** `plans/issues/active/phase-40-stable-channel-ga-execution.md:82-88` — all five milestone_40_1 boxes unchecked, no PR link, no positive/negative evidence, no command list, no review record (`plans/reviews/active/phase-40-milestone-40-1-agent-review-pass-1.md` is a zero-byte file). AGENTS.md "Required workflow" and the phase Validation Contract both mandate this per milestone.

**13. Rust integration boundary is undocumented and compensated by fallbacks.** `stable-candidate` is absent from `verification/areas/rust_interop/manifest.json`, absent from all four profiles, and `verification/areas/rust_interop/data/stable_support_claims.json` does not exist. Deferring while `certification_0` is open is reasonable and I am not asking for Rust work here — but nothing in the diff records the deferral, and `planner.py:384-403` compensates with shape-guessing over the unlanded contract (`claims` → `advertised_claims` → `claim_ids`, then `id` → `claim_id` → `row_id`). That is three fallback layers over a contract that does not exist yet. Record the deferral in the execution issue and pin one exact shape (or fail closed until the artifact lands).

**14. Milestone demo not implemented.** The required demo — *"A local planner run consumes fixture-backed docs and VSIX evidence plus a newly built host artifact, emits a schema-complete unapproved plan, installs the artifact in an isolated directory, and runs `sifr --version`, `sifr check`, and `sifr self version`"* — has no implementation. `demos/stable_release_governance_demo.sh` is unchanged from 40.0 and calls only `generate-release-plan`; `artifact_stable_candidate_generation.sh` covers install but with a mock binary and no `self version`.

**15. `rc` residue and its inventory disposition.** `build_release_artifacts.sh:83` and `generate_version_installer.sh:70` still accept `X.Y.Z-rc.N`. Removal is 40.2 scope, but `plans/releases/stable_gate_inventory.json:33` now *records* rc acceptance as current behavior while the `immutable-installer-generator` disposition (`:41`) dropped its rc-rejection statement — the gate loses its owner. Restore an explicit rc disposition.

**16. Dead branch in generated stable installer.** `generate_version_installer.sh:132-135` expands `${VERSION}` at generation time, so a stable installer ships a literal `case "0.1.0" in *-*) APP_CHANNEL="0.1.0"; ... esac` — a statically unreachable branch in generated release code. Resolve the channel at generation time and emit `APP_CHANNEL="stable"` directly.

---

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `sifr_verify areas run --area distribution_release --suite qualification` | pass (4 self-tests, 1 variant) |
| `sifr_verify areas run --area distribution_release --suite representative` | pass (41 variants, 0 failures) |
| `sifr_verify areas run --area distribution_release --suite evidence-custody` | pass |
| `bash .../cases/release_qualification_workflow_contract.sh` | PASS |
| `bash .../cases/artifact_stable_candidate_generation.sh` | PASS |
| Adversarial probe of `validate_qualification_artifact_index` (renamed report ids / extra report / `..` in `workflow_artifact_name`) | all three **accepted** — findings 3, 4, 5 |
| File-size guardrail (manual `wc -l`) | largest touched file `governance/selftest.py` = 862 lines; all under 900 |
| Grep for planner call sites | only `release_governance.py:264`; no test path |

Not run: `scripts/run_all_tests.sh --profile create-pr`/`merge`, `cargo test -p sifr` (the `self_update_receipt.rs` change is reviewed by inspection only), and the GitHub-hosted matrix jobs.

---

**CHANGES_REQUESTED**
